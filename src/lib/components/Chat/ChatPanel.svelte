<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { tick, onMount } from 'svelte';
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
  let searchQuery = $state('');
  let messagesContainer;
  let textarea;
  
  // Settings
  let temperature = $state(0.7);
  let maxTokens = $state(2000);
  let selectedModel = $state(configStore.settings.ai.selectedModel || 'gemini-1.5-pro');

  const SYSTEM_PROMPT = `You are Karsa, an AI coding assistant with direct access to the codebase through MCP (Model Context Protocol) tools.

AVAILABLE TOOLS (use these to answer questions):
1. get_project_map() - See the entire project structure (folders & files)
2. query_codebase(query) - Search for specific files or code
3. get_codebase_stats() - Get project statistics (languages, file count, lines)
4. file_read(path) - Read file contents (max 1000 lines)
5. file_read_range(path, start, end) - Read specific lines (max 500 lines)
6. list_symbols(path) - List functions/classes in a file
7. search(pattern, path) - Search for text patterns

WORKFLOW:
1. When asked about project structure → use get_project_map()
2. When asked about specific files → use query_codebase() or search()
3. Before reading files → use get_file_info() to check size
4. For large files → use list_symbols() first, then file_read_range()

CORE RULES:
- ALWAYS use tools to verify information about the codebase
- Never guess about project structure - use get_project_map()
- Cite actual file paths and line numbers
- If uncertain, use tools to check
- Be direct and concise

CODE RESPONSES:
- Provide minimal, working code
- Show actual implementation from the codebase
- Reference real files and functions

AVOID:
- Saying "I can't see" when tools are available
- Hallucinating code that doesn't exist
- Guessing about project structure

Focus on accuracy using available tools.`;

  onMount(async () => {
    await loadConversations();
    
    // Listen for new messages from streaming
    await listen('chat-message', (event) => {
      const { content } = event.payload;
      if (messages.length > 0 && messages[messages.length - 1].role === 'assistant') {
        messages[messages.length - 1].content += content;
      }
    });
    
    // Listen for conversation updates (real-time sync from other views)
    await listen('conversation-updated', async (event) => {
      await loadConversations();
      // Only reload if it's NOT the active conversation (avoid duplicate)
      if (activeConversationId !== event.payload.id) {
        return;
      }
    });
  });

  async function loadConversations() {
    try {
      if (searchQuery.trim()) {
        conversations = await invoke('search_conversations', {
          query: searchQuery,
          mode: 'all'
        });
      } else {
        conversations = await invoke('get_conversations', {
          mode: 'all',
          limit: 50
        });
      }
      
      if (conversations.length > 0 && !activeConversationId) {
        await loadConversation(conversations[0].id);
      }
    } catch (e) {
      console.error('Failed to load conversations:', e);
    }
  }

  async function loadConversation(id) {
    try {
      activeConversationId = id;
      const msgs = await invoke('get_messages', { conversationId: id });
      messages = msgs.map(m => ({
        role: m.role,
        content: m.content,
        timestamp: new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      }));
      await tick();
      scrollToBottom();
    } catch (e) {
      console.error('Failed to load messages:', e);
    }
  }

  async function createNewConversation() {
    try {
      const title = "New Chat";  // Temporary title
      const id = await invoke('create_conversation', {
        mode: 'unified',
        title,
        contextPath: fsStore.activeFile?.path || null,
        model: selectedModel
      });
      
      activeConversationId = id;
      messages = [];
      await loadConversations();
    } catch (e) {
      console.error('Failed to create conversation:', e);
    }
  }
  
  async function generateAndUpdateTitle(firstMessage) {
    if (!activeConversationId) return;
    
    try {
      const config = {
        provider: configStore.settings.ai.provider,
        api_key: configStore.settings.ai.apiKey,
        base_url: configStore.settings.ai.baseUrl || 'https://api.kilo.ai/api/gateway/chat/completions',
        model_name: selectedModel,
        custom_models: []
      };
      
      const title = await invoke('generate_chat_title', {
        firstMessage,
        config
      });
      
      await invoke('update_conversation_title', {
        id: activeConversationId,
        title
      });
      
      await loadConversations();
    } catch (e) {
      console.error('Failed to generate title:', e);
    }
  }

  async function deleteConversation(id) {
    try {
      await invoke('delete_conversation', { id });
      if (activeConversationId === id) {
        activeConversationId = null;
        messages = [];
      }
      await loadConversations();
    } catch (e) {
      console.error('Failed to delete conversation:', e);
    }
  }

  async function saveMessage(role, content) {
    if (!activeConversationId) {
      await createNewConversation();
    }
    
    try {
      await invoke('add_message', {
        conversationId: activeConversationId,
        role,
        content
      });
    } catch (e) {
      console.error('Failed to save message:', e);
    }
  }

  async function sendMessage() {
    if (!input.trim() || isLoading) return;

    // Prevent double send
    isLoading = true;

    const userMessage = input.trim();
    const isFirstMessage = messages.length === 0;
    input = '';
    if (textarea) textarea.style.height = 'auto';
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    
    // Save to database
    await saveMessage('user', userMessage);
    
    // Generate title for first message
    if (isFirstMessage) {
      generateAndUpdateTitle(userMessage);
    }
    
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
      
      // Get MCP system prompt
      const mcpPrompt = await invoke('mcp_get_system_prompt');
      
      let contextContent = '';
      if (fsStore.activeFile) {
        contextContent = `\n\nCurrent file: ${fsStore.activeFile.name}\n\`\`\`\n${fsStore.activeFileContent}\n\`\`\``;
      }

      const msgs = [
        { role: 'system', content: SYSTEM_PROMPT + '\n\n' + mcpPrompt },
        ...messages.slice(-6).map(m => ({ role: m.role, content: m.content })),
        { role: 'user', content: `${userMessage}${contextContent}` }
      ];

      const response = await invoke('send_chat_completion', { messages: msgs, config });

      const aiTimestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      const aiMessage = { role: 'assistant', content: response, timestamp: aiTimestamp };
      messages = [...messages, aiMessage];
      
      // Save to database
      await saveMessage('assistant', response);
    } catch (e) {
      const errTimestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      const errorMsg = { role: 'assistant', content: `**Error**: ${e}`, timestamp: errTimestamp };
      messages = [...messages, errorMsg];
      await saveMessage('assistant', errorMsg.content);
    } finally {
      isLoading = false;
      await tick();
      scrollToBottom();
    }
  }

  function scrollToBottom() {
    if (messagesContainer) {
      messagesContainer.scrollTop = messagesContainer.scrollHeight;
    }
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
      <button 
        class={cn("p-1 hover:bg-muted rounded transition-colors", showHistory ? "text-primary bg-primary/10" : "text-muted-foreground")} 
        title="History" 
        onclick={() => showHistory = !showHistory}
      >
        <MessageSquare size={14} />
      </button>
      <button 
        class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors" 
        title="New Chat" 
        onclick={createNewConversation}
      >
        <Plus size={14} />
      </button>
      <button 
        class={cn("p-1 hover:bg-muted rounded transition-colors", showSettings ? "text-primary bg-primary/10" : "text-muted-foreground")} 
        title="Settings" 
        onclick={() => showSettings = !showSettings}
      >
        <Sliders size={14} />
      </button>
      <div class="w-[1px] h-3 bg-border mx-1"></div>
      <button onclick={onClose} class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors">
        <X size={14} />
      </button>
    </div>
  </div>

  <!-- History Sidebar -->
  {#if showHistory}
    <div class="border-b border-border bg-muted/5 max-h-64 overflow-y-auto">
      <!-- Mode Toggle -->
      <div class="px-2 pt-2 pb-1 space-y-2">
        <input
          type="text"
          bind:value={searchQuery}
          oninput={loadConversations}
          placeholder="Search chats..."
          class="w-full bg-background border border-border rounded px-2 py-1 text-xs focus:ring-1 focus:ring-primary outline-none"
        />
      </div>
      
      <div class="p-2 space-y-1">
        {#each conversations as conv}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class={cn(
              "w-full text-left px-3 py-2 rounded text-xs transition-colors flex items-center justify-between group cursor-pointer",
              activeConversationId === conv.id 
                ? "bg-primary/10 text-primary" 
                : "hover:bg-muted text-muted-foreground"
            )}
            onclick={() => loadConversation(conv.id)}
          >
            <div class="flex-1 truncate">
              <div class="flex items-center gap-2">
                <div class="font-medium truncate">{conv.title}</div>
                {#if showAllModes}
                  <span class={cn(
                    "text-[9px] px-1.5 py-0.5 rounded",
                    conv.mode === 'agent' ? "bg-purple-500/20 text-purple-400" : "bg-blue-500/20 text-blue-400"
                  )}>
                    {conv.mode}
                  </span>
                {/if}
              </div>
              <div class="text-[10px] opacity-70">{new Date(conv.updated_at * 1000).toLocaleDateString()}</div>
            </div>
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="opacity-0 group-hover:opacity-100 p-1 hover:bg-destructive/10 rounded cursor-pointer"
              onclick={(e) => {
                e.stopPropagation();
                deleteConversation(conv.id);
              }}
            >
              <Trash2 size={12} class="text-destructive" />
            </div>
          </div>
        {/each}
        
        {#if conversations.length === 0}
          <div class="text-center py-4 text-xs text-muted-foreground">
            No conversations yet
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Settings Panel (Collapsible) -->
  {#if showSettings}
    <div class="px-4 py-3 border-b border-border bg-muted/5 animate-in slide-in-from-top-2 duration-200">
       <div class="space-y-3">
          <ModelSelector 
            bind:selectedModel 
            apiKey={configStore.settings.ai.apiKey}
            onModelChange={(m) => configStore.updateAiConfig({ selectedModel: m.id })}
          />
          
          <div class="space-y-1">
             <div class="flex justify-between text-[10px] text-muted-foreground">
                <span>Temperature</span>
                <span>{temperature}</span>
             </div>
             <input type="range" min="0" max="1" step="0.1" bind:value={temperature} class="w-full h-1.5 bg-muted rounded-full accent-primary appearance-none cursor-pointer" />
          </div>
          <div class="space-y-1">
             <div class="flex justify-between text-[10px] text-muted-foreground">
                <span>Max Tokens</span>
                <span>{maxTokens}</span>
             </div>
             <input type="range" min="100" max="8000" step="100" bind:value={maxTokens} class="w-full h-1.5 bg-muted rounded-full accent-primary appearance-none cursor-pointer" />
          </div>
       </div>
    </div>
  {/if}

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-6 scroll-smooth" bind:this={messagesContainer}>
    {#if messages.length === 0}
      <div class="h-full flex flex-col items-center justify-center text-center p-4 space-y-4 opacity-60">
        <div class="w-12 h-12 bg-primary/10 rounded-xl flex items-center justify-center text-primary">
           <Bot size={24} />
        </div>
        <div>
          <h3 class="font-medium text-foreground text-sm">How can I help?</h3>
          <p class="text-xs text-muted-foreground mt-1">Ask me to generate code, debug, or explain concepts.</p>
        </div>
        <div class="flex flex-col gap-2 w-full">
           <button onclick={() => input = "Explain this code"} class="text-xs border border-border rounded px-3 py-2 hover:bg-muted/50 text-left transition-colors">Explain this code</button>
           <button onclick={() => input = "Find bugs"} class="text-xs border border-border rounded px-3 py-2 hover:bg-muted/50 text-left transition-colors">Find bugs</button>
        </div>
      </div>
    {/if}

    {#each messages as msg}
      <div class={cn("flex gap-3 max-w-full animate-in fade-in slide-in-from-bottom-2 duration-300", msg.role === 'user' ? "flex-row-reverse" : "")}>
        <!-- Avatar -->
        <div class={cn("w-8 h-8 rounded-lg flex items-center justify-center shrink-0 border border-border shadow-sm", 
          msg.role === 'user' 
            ? "bg-gradient-to-br from-primary to-blue-600 text-primary-foreground" 
            : "bg-card text-foreground")}>
          {#if msg.role === 'user'}
            <User size={14} />
          {:else}
            <Bot size={14} />
          {/if}
        </div>

        <div class={cn("flex flex-col max-w-[85%]", msg.role === 'user' ? "items-end" : "items-start")}>
          <div class={cn("rounded-2xl px-3 py-2.5 text-sm shadow-sm select-text", 
            msg.role === 'user' 
              ? "bg-primary text-primary-foreground rounded-tr-sm" 
              : "bg-muted/30 border border-border text-foreground rounded-tl-sm")}>
            
            {#if msg.role === 'assistant'}
              <MarkdownRenderer content={msg.content} />
            {:else}
              <div class="whitespace-pre-wrap break-words">{msg.content}</div>
            {/if}
          </div>
          
          <span class="text-[10px] text-muted-foreground opacity-50 px-1 mt-1">
            {msg.timestamp}
          </span>
        </div>
      </div>
    {/each}

    {#if isLoading}
      <div class="flex items-center gap-3 px-1">
        <div class="w-8 h-8 rounded-lg bg-card border border-border flex items-center justify-center shrink-0">
           <Bot size={14} class="text-muted-foreground" />
        </div>
        <div class="flex gap-1">
           <span class="w-1.5 h-1.5 bg-muted-foreground/40 rounded-full animate-bounce"></span>
           <span class="w-1.5 h-1.5 bg-muted-foreground/40 rounded-full animate-bounce delay-150"></span>
           <span class="w-1.5 h-1.5 bg-muted-foreground/40 rounded-full animate-bounce delay-300"></span>
        </div>
      </div>
    {/if}
  </div>

  <!-- Input Area -->
  <div class="p-3 bg-background border-t border-border shrink-0">
    <div class="relative flex items-end gap-2 bg-muted/30 border border-border rounded-xl p-1.5 focus-within:ring-1 focus-within:ring-primary/30 focus-within:border-primary/50 transition-all shadow-sm">
      <button class="p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground rounded-lg transition-colors shrink-0 mb-0.5" title="Upload Context">
         <Plus size={16} />
      </button>
      
      <textarea
        bind:this={textarea}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={autoResize}
        placeholder="Ask anything..."
        class="flex-1 bg-transparent border-0 resize-none max-h-[150px] min-h-[24px] py-1.5 px-1 focus:ring-0 text-sm placeholder:text-muted-foreground/50 scrollbar-hide"
        rows="1"
      ></textarea>
      
      {#if isLoading}
        <button 
          onclick={() => { isLoading = false; }}
          class="p-1.5 bg-destructive text-destructive-foreground rounded hover:bg-destructive/90 transition-colors shrink-0 mb-0.5"
          title="Stop generation"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <rect x="6" y="6" width="12" height="12" rx="2" />
          </svg>
        </button>
      {:else}
        <button 
          onclick={sendMessage}
          disabled={!input.trim()}
          class="p-1.5 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors shrink-0 mb-0.5"
        >
          <Send size={14} />
        </button>
      {/if}
    </div>
    <div class="flex items-center justify-between mt-2 px-1">
       {#if fsStore.activeFile}
          <div class="flex items-center gap-1.5 max-w-[70%]">
             <FileCode size={10} class="text-muted-foreground" />
             <span class="text-[10px] text-muted-foreground truncate">{fsStore.activeFile.name}</span>
          </div>
       {:else}
          <span></span>
       {/if}
       <span class="text-[9px] text-muted-foreground font-medium opacity-70">{selectedModel}</span>
    </div>
  </div>
</div>
