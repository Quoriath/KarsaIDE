import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { fsStore } from './fileSystem.svelte.js';
import { configStore } from './config.svelte.js';

class ChatStore {
  conversations = $state([]);
  activeConversationId = $state(null);
  messages = $state([]);
  
  isLoading = $state(false);
  isStreaming = $state(false);
  isToolExecuting = $state(false);
  isThinkingComplete = $state(false);
  
  streamingContent = $state('');
  streamingReasoning = $state('');
  streamingSegments = $state([]);
  currentSegmentText = $state('');

  unlistenHandlers = [];

  constructor() {
    this.init();
  }

  async init() {
    this.unlistenHandlers.push(
      await listen('ai-stream-chunk', (event) => {
        const chunk = typeof event.payload === 'string' ? event.payload : event.payload?.chunk || '';
        this.currentSegmentText += chunk;
        this.streamingContent += chunk;
      }),
      await listen('ai-stream-reasoning', (event) => {
        const reasoning = typeof event.payload === 'string' ? event.payload : '';
        this.streamingReasoning += reasoning;
        
        // Clear previous thinking segment and add new one
        this.streamingSegments = this.streamingSegments.filter(s => s.type !== 'thinking');
        if (this.streamingReasoning) {
          this.streamingSegments.unshift({ type: 'thinking', content: this.streamingReasoning });
        }
      }),
      await listen('ai-reasoning-complete', () => {
        this.isThinkingComplete = true;
      }),
      await listen('ai-tool-call', (event) => {
        const toolCall = event.payload;
        if (this.currentSegmentText.trim()) {
          this.streamingSegments = [...this.streamingSegments, { type: 'text', content: this.currentSegmentText }];
          this.currentSegmentText = '';
        }
        this.streamingSegments = [...this.streamingSegments, {
          type: 'tool',
          name: toolCall.name,
          arguments: toolCall.arguments,
          executing: true,
          result: null,
          error: null
        }];
        this.isToolExecuting = true;
      }),
      await listen('ai-tool-result', (event) => {
        const payload = event.payload;
        this.streamingSegments = this.streamingSegments.map(seg => {
          if (seg.type === 'tool' && seg.executing && seg.name === payload.name) {
            return { ...seg, executing: false, result: payload.result, error: payload.error };
          }
          return seg;
        });
        this.isToolExecuting = false;
      }),
      await listen('ai-stream-done', async () => {
        if (this.isStreaming) {
          if (this.currentSegmentText.trim()) {
            this.streamingSegments = [...this.streamingSegments, { type: 'text', content: this.currentSegmentText }];
          }
          const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          const newMessage = { 
            role: 'assistant', 
            content: this.streamingContent, 
            segments: [...this.streamingSegments],
            timestamp 
          };
          this.messages = [...this.messages, newMessage];
          await this.saveMessage('assistant', this.streamingContent);
        }
        this.resetStreamingState();
      })
    );
  }

  resetStreamingState() {
    this.streamingContent = '';
    this.streamingReasoning = '';
    this.streamingSegments = [];
    this.currentSegmentText = '';
    this.isStreaming = false;
    this.isLoading = false;
    this.isToolExecuting = false;
    this.isThinkingComplete = false;
  }

  async loadConversations() {
    try {
      this.conversations = await invoke('get_conversations', { mode: 'all', limit: 50 });
    } catch (e) {
      console.error('Failed to load conversations:', e);
    }
  }

  async loadConversation(id) {
    this.activeConversationId = id;
    try {
      const msgs = await invoke('get_messages', { conversationId: id });
      this.messages = msgs.map(m => ({
        ...m,
        timestamp: new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      }));
    } catch (e) {
      console.error('Failed to load messages:', e);
    }
  }

  async createNewConversation(mode = 'unified') {
    try {
      const id = await invoke('create_conversation', {
        mode,
        title: 'New Chat',
        contextPath: fsStore.activeFile?.path || null,
        model: configStore.settings.ai.selectedModel
      });
      this.activeConversationId = id;
      this.messages = [];
      await this.loadConversations();
      return id;
    } catch (e) {
      console.error('Failed to create conversation:', e);
    }
  }

  async saveMessage(role, content) {
    if (!this.activeConversationId) return;
    try {
      await invoke('add_message', { conversationId: this.activeConversationId, role, content });
    } catch (e) {
      console.error('Failed to save message:', e);
    }
  }

  async sendMessage(input) {
    if (!input.trim() || this.isLoading) return;

    const userMessage = input.trim();
    this.isLoading = true;
    this.isStreaming = true;
    this.resetStreamingState();
    this.isStreaming = true;
    this.isLoading = true;
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    this.messages = [...this.messages, { role: 'user', content: userMessage, timestamp }];
    await this.saveMessage('user', userMessage);
    
    try {
      const cwd = fsStore.activeWorkspace || '';
      const mcpPrompt = await invoke('mcp_get_system_prompt', { mode: 'code', cwd: cwd || 'no-workspace' });
      
      let contextContent = '';
      if (fsStore.activeFile) {
        contextContent = `

[Current File: ${fsStore.activeFile.path}]
\`\`\`
${fsStore.activeFileContent}
\`\`\``;
      }

      const msgs = [
        { role: 'system', content: mcpPrompt },
        ...this.messages.slice(-10).map(m => ({ role: m.role, content: m.content })),
        { role: 'user', content: `${userMessage}${contextContent}` }
      ];

      const config = {
        provider: configStore.settings.ai.provider,
        api_key: configStore.settings.ai.apiKey,
        base_url: configStore.settings.ai.baseUrl,
        model_name: configStore.settings.ai.selectedModel,
        custom_models: []
      };

      await invoke('send_agent_message_stream', { messages: msgs, config, workspace: cwd || null });
    } catch (e) {
      this.messages = [...this.messages, { role: 'assistant', content: `Error: ${e}`, timestamp }];
      this.resetStreamingState();
    }
  }

  async stopGeneration() {
    await invoke('cancel_ai_stream').catch(() => {});
    this.resetStreamingState();
  }
}

export const chatStore = new ChatStore();
