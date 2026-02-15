<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { tick, onMount, onDestroy } from 'svelte';
  import { 
    Send, X, Bot, User, Sparkles, FileCode, Loader2, Eraser, 
    Settings, Download, Upload, Sliders, Plus, MessageSquare, Trash2 
  } from 'lucide-svelte';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';
  import ModelSelector from '../ModelSelector.svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();

  let conversations = $state([]);
  let activeConversationId = $state(null);
  let messages = $state([]);
  let input = $state('');
  let isLoading = $state(false);
  let showSettings = $state(false);
  let showHistory = $state(false);
  let messagesContainer;
  let textarea;
  let unlistenHandlers = [];
  
  // Streaming states
  let streamingContent = $state('');
  let streamingReasoning = $state('');

  // Settings
  let temperature = $state(0.7);
  let maxTokens = $state(2000);
  let selectedModel = $state(configStore.settings.ai.selectedModel || 'gemini-1.5-pro');

  const SYSTEM_PROMPT = "You are Karsa, an autonomous coding agent. Use markdown.";

  onMount(async () => {
    await loadConversations();
    
    // Listeners specific to sidebar context if needed, or global
    // Here we use send_chat_completion_stream which emits global events
    // But we need to filter or handle them. 
    // Ideally, we should have instance-specific event channels. 
    // For now, let's assume single active stream global.
    
    const unlistenChunk = await listen('ai-stream-chunk', (event) => {
      const chunk = typeof event.payload === 'string' ? event.payload : event.payload?.chunk || '';
      streamingContent += chunk;
      scrollToBottom();
    });

    const unlistenReasoning = await listen('ai-stream-reasoning', (event) => {
      const reasoning = typeof event.payload === 'string' ? event.payload : '';
      streamingReasoning += reasoning;
    });

    const unlistenDone = await listen('ai-stream-done', async () => {
      if (streamingContent.trim() && isLoading) {
        await saveMessage('assistant', streamingContent); // reasoning not saved in simple schema yet
        const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        messages = [...messages, { 
          role: 'assistant', 
          content: streamingContent, 
          reasoning: streamingReasoning,
          timestamp 
        }];
      }
      streamingContent = '';
      streamingReasoning = '';
      isLoading = false;
      scrollToBottom();
    });
    
    unlistenHandlers.push(unlistenChunk, unlistenReasoning, unlistenDone);
  });

  onDestroy(() => {
    unlistenHandlers.forEach(fn => fn());
  });

  async function loadConversations() {
    try {
      conversations = await invoke('get_conversations', { mode: 'editor', limit: 20 });
      if (conversations.length > 0 && !activeConversationId) {
        await loadConversation(conversations[0].id);
      }
    } catch (e) {}
  }

  async function loadConversation(id) {
    activeConversationId = id;
    try {
      const msgs = await invoke('get_messages', { conversationId: id });
      messages = msgs.map(m => ({
        role: m.role,
        content: m.content,
        timestamp: new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      }));
      scrollToBottom();
    } catch (e) {}
  }

  async function createNewConversation() {
    try {
      const id = await invoke('create_conversation', {
        mode: 'editor',
        title: 'New Chat',
        contextPath: fsStore.activeFile?.path || null,
        model: selectedModel
      });
      activeConversationId = id;
      messages = [];
      await loadConversations();
    } catch (e) {}
  }

  async function saveMessage(role, content) {
    if (!activeConversationId) await createNewConversation();
    try {
      await invoke('add_message', { conversationId: activeConversationId, role, content });
    } catch (e) {}
  }

  async function sendMessage() {
    if (!input.trim() || isLoading) return;

    const userMessage = input.trim();
    input = '';
    isLoading = true;
    streamingContent = '';
    streamingReasoning = '';
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    await saveMessage('user', userMessage);
    
    await tick();
    scrollToBottom();

    try {
      const config = {
        provider: configStore.settings.ai.provider,
        api_key: configStore.settings.ai.apiKey,
        base_url: configStore.settings.ai.baseUrl || 'https://api.kilo.ai/api/gateway/chat/completions',
        model_name: selectedModel,
        custom_models: []
      };
      
      let contextContent = '';
      if (fsStore.activeFile) {
        contextContent = `\n\nFile: ${fsStore.activeFile.name}\n\`\`\`\n${fsStore.activeFileContent}\n\`\`\``;
      }

      const msgs = [
        { role: 'system', content: SYSTEM_PROMPT },
        ...messages.slice(-4).map(m => ({ role: m.role, content: m.content })),
        { role: 'user', content: `${userMessage}${contextContent}` }
      ];

      await invoke('send_chat_completion_stream', { messages: msgs, config });
    } catch (e) {
      messages = [...messages, { role: 'assistant', content: `Error: ${e}`, timestamp }];
      isLoading = false;
    }
  }

  function scrollToBottom() {
    if (messagesContainer) messagesContainer.scrollTop = messagesContainer.scrollHeight;
  }

  function handleKeydown(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }
  
  function autoResize(e) {
    const target = e.target;
    target.style.height = 'auto';
    target.style.height = `${Math.min(target.scrollHeight, 150)}px`;
  }
</script>

<div class="h-full flex flex-col bg-background border-l border-border">
  <!-- Header -->
  <div class="h-10 min-h-[40px] px-3 border-b border-border flex items-center justify-between bg-muted/10 shrink-0">
    <div class="flex items-center gap-2 text-xs font-semibold text-muted-foreground uppercase tracking-wider">
       <Sparkles size={14} class="text-primary" />
       Assistant
    </div>
    
    <div class="flex items-center gap-1">
      <button class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors" title="New Chat" onclick={createNewConversation}>
        <Plus size={14} />
      </button>
      <button class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors" title="History" onclick={() => showHistory = !showHistory}>
        <MessageSquare size={14} />
      </button>
      <div class="w-[1px] h-3 bg-border mx-1"></div>
      <button onclick={onClose} class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors">
        <X size={14} />
      </button>
    </div>
  </div>

  {#if showHistory}
    <div class="border-b border-border bg-muted/5 max-h-48 overflow-y-auto">
       {#each conversations as conv}
          <button class="w-full text-left px-3 py-2 text-xs hover:bg-muted truncate" onclick={() => {loadConversation(conv.id); showHistory = false;}}>
             {conv.title}
          </button>
       {/each}
    </div>
  {/if}

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-6 scroll-smooth" bind:this={messagesContainer}>
    {#if messages.length === 0 && !isLoading && !streamingContent}
      <div class="h-full flex flex-col items-center justify-center text-center p-4 space-y-4 opacity-60">
        <Bot size={24} class="text-primary" />
        <p class="text-xs text-muted-foreground">Ask me anything about your code.</p>
      </div>
    {/if}

    {#each messages as msg}
      <div class={cn("flex gap-3 max-w-full animate-in fade-in slide-in-from-bottom-2 duration-300", msg.role === 'user' ? "flex-row-reverse" : "")}>
        <div class={cn("w-8 h-8 rounded-lg flex items-center justify-center shrink-0 border border-border shadow-sm", msg.role === 'user' ? "bg-primary text-primary-foreground" : "bg-card text-foreground")}>
          {#if msg.role === 'user'} <User size={14} /> {:else} <Bot size={14} /> {/if}
        </div>

        <div class={cn("flex flex-col max-w-[85%]", msg.role === 'user' ? "items-end" : "items-start")}>
          
          {#if msg.role === 'assistant' && msg.reasoning}
             <div class="mb-1 text-[10px] bg-muted/20 border border-border px-2 py-1 rounded text-muted-foreground w-full">
                Thinking Process...
             </div>
          {/if}

          <div class={cn("rounded-2xl px-3 py-2.5 text-sm shadow-sm", msg.role === 'user' ? "bg-primary text-primary-foreground rounded-tr-sm" : "bg-muted/30 border border-border text-foreground rounded-tl-sm")}>
            {#if msg.role === 'assistant'}
              <MarkdownRenderer content={msg.content} />
            {:else}
              <div class="whitespace-pre-wrap break-words">{msg.content}</div>
            {/if}
          </div>
        </div>
      </div>
    {/each}

    <!-- Streaming -->
    {#if isLoading || streamingContent}
       <div class="flex gap-3 animate-in fade-in">
          <div class="w-8 h-8 rounded-lg bg-card border border-border flex items-center justify-center shrink-0"><Bot size={14} /></div>
          <div class="flex flex-col max-w-[85%]">
             {#if streamingReasoning}
                <div class="mb-2 text-[10px] text-muted-foreground bg-muted/10 p-2 rounded border border-border/50 italic animate-pulse">
                   {streamingReasoning}
                </div>
             {/if}
             <div class="bg-card border border-border rounded-2xl rounded-tl-sm px-3 py-2.5 text-sm shadow-sm">
                <MarkdownRenderer content={streamingContent} />
                <span class="inline-block w-1.5 h-3.5 bg-primary animate-pulse ml-1 align-middle"></span>
             </div>
          </div>
       </div>
    {/if}
  </div>

  <!-- Input Area -->
  <div class="p-3 bg-background border-t border-border shrink-0">
    <div class="relative flex items-end gap-2 bg-muted/30 border border-border rounded-xl p-1.5 focus-within:ring-1 focus-within:ring-primary/30 transition-all shadow-sm">
      <textarea
        bind:this={textarea}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={autoResize}
        placeholder="Type a message..."
        class="flex-1 bg-transparent border-0 resize-none max-h-[150px] min-h-[24px] py-1.5 px-2 focus:ring-0 text-sm placeholder:text-muted-foreground/50 scrollbar-hide"
        rows="1"
      ></textarea>
      
      <button 
        onclick={sendMessage}
        disabled={isLoading || !input.trim()}
        class="p-1.5 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 transition-colors shrink-0 mb-0.5"
      >
        {#if isLoading} <Loader2 size={14} class="animate-spin" /> {:else} <Send size={14} /> {/if}
      </button>
    </div>
    {#if fsStore.activeFile}
      <div class="flex items-center gap-1.5 mt-2 px-1 text-[10px] text-muted-foreground">
         <FileCode size={10} /> <span class="truncate">{fsStore.activeFile.name}</span>
      </div>
    {/if}
  </div>
</div>
