<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { configStore } from '../stores/config.svelte.js';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { 
    Send, Bot, User, Sparkles, Loader2, Plus, Settings, Search, 
    MessageSquare, Trash2, ChevronDown, Copy, Check, Pencil, 
    RotateCcw, Square, MoreHorizontal, Download, Share2, FolderTree
  } from 'lucide-svelte';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import ModelSelector from './ModelSelector.svelte';
  import ThinkingBlock from './Chat/ThinkingBlock.svelte';
  import MCPToolCall from './MCPToolCall.svelte';
  import { cn } from '../utils.js';

  // --- STATE MANAGEMENT ---
  let conversations = $state([]); 
  let activeConversationId = $state(null);
  let messages = $state([]); 
  
  // Virtualization - load more messages for context
  let visibleMessages = $derived(messages.length > 100 ? messages.slice(messages.length - 100) : messages);
  
  let searchQuery = $state('');
  let input = $state('');
  let isLoading = $state(false);
  let showScrollButton = $state(false);
  
  // Streaming Buffers
  let streamingContent = $state(''); 
  let streamingReasoning = $state('');
  let streamingSegments = $state([]); // Array of {type: 'text'|'tool', content/name/args/result}
  let currentSegmentText = $state(''); // Accumulate text before tool
  
  // UI State
  let editingTitle = $state(false);
  let editingTitleValue = $state('');
  let copiedMessageId = $state(null);
  let showContextMenu = $state(null);
  let regenerateFromIndex = $state(null);
  
  let messagesContainer;
  let scrollAnchor;
  let inputTextarea;
  let unlistenHandlers = [];

  // Persistent Config
  let selectedModel = $state(configStore.settings.ai.selectedModel || 'gemini-1.5-pro');

  onMount(async () => {
    await loadConversations();
    if (conversations.length === 0) await createNewConversation();
    else await loadMessages(conversations[0].id);

    // Event Listeners
    const unlistenChunk = await listen('ai-stream-chunk', (event) => {
      const chunk = typeof event.payload === 'string' ? event.payload : '';
      currentSegmentText += chunk;
      streamingContent += chunk;
      scrollToBottom();
    });

    const unlistenReasoning = await listen('ai-stream-reasoning', (event) => {
      const reasoning = typeof event.payload === 'string' ? event.payload : '';
      streamingReasoning += reasoning;
      scrollToBottom();
    });

    const unlistenReasoningComplete = await listen('ai-reasoning-complete', (event) => {
      const reasoning = typeof event.payload === 'string' ? event.payload : '';
      streamingReasoning = reasoning;
      scrollToBottom();
    });

    const unlistenToolCall = await listen('ai-tool-call', (event) => {
      const toolCall = event.payload;
      
      // Save accumulated text as segment
      if (currentSegmentText.trim()) {
        streamingSegments = [...streamingSegments, {
          type: 'text',
          content: currentSegmentText
        }];
        currentSegmentText = '';
      }
      
      // Add tool segment
      streamingSegments = [...streamingSegments, {
        type: 'tool',
        name: toolCall.name,
        arguments: toolCall.arguments,
        executing: true,
        result: null,
        error: null
      }];
      
      scrollToBottom();
    });

    const unlistenToolResult = await listen('ai-tool-result', (event) => {
      const payload = event.payload;
      
      // Update last tool segment
      streamingSegments = streamingSegments.map((seg, index) => {
        if (seg.type === 'tool' && seg.executing && seg.name === payload.name) {
          return {
            ...seg,
            executing: false,
            result: payload.isError ? null : payload.result,
            error: payload.isError ? payload.error : null
          };
        }
        return seg;
      });
      scrollToBottom();
    });

    const unlistenDone = await listen('ai-stream-done', async () => {
      const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      
      if (isLoading) {
        // Add final text segment if any
        if (currentSegmentText.trim()) {
          streamingSegments = [...streamingSegments, {
            type: 'text',
            content: currentSegmentText
          }];
        }
        
        // Create message with segments
        messages = [...messages, {
          role: 'assistant',
          content: streamingContent || '',
          reasoning: streamingReasoning || '',
          segments: streamingSegments,
          timestamp
        }];
        
        // Save to database
        if (streamingContent) {
          await saveMessage('assistant', streamingContent);
        }
      }
      
      // Reset state
      streamingContent = '';
      streamingReasoning = '';
      streamingSegments = [];
      currentSegmentText = '';
      streamingSegments = [];
      currentSegmentText = '';
      isLoading = false;
      scrollToBottom();
    });

    const unlistenUpdate = await listen('conversation-updated', async (event) => {
      await loadConversations();
      if (activeConversationId !== event.payload.id) return;
    });

    // Keyboard shortcuts
    const handleKeydown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
        e.preventDefault();
        createNewConversation();
      }
      if ((e.metaKey || e.ctrlKey) && e.key === '/') {
        e.preventDefault();
        searchQuery = '';
        document.querySelector('aside input')?.focus();
      }
    };
    window.addEventListener('keydown', handleKeydown);

    unlistenHandlers.push(
      unlistenChunk, 
      unlistenReasoning,
      unlistenReasoningComplete,
      unlistenToolCall, 
      unlistenToolResult, 
      unlistenDone, 
      unlistenUpdate,
      () => window.removeEventListener('keydown', handleKeydown)
    );
  });

  onDestroy(() => {
    unlistenHandlers.forEach(fn => fn());
  });

  function handleScroll() {
    if (!messagesContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
    showScrollButton = scrollHeight - scrollTop - clientHeight > 200;
  }

  // --- DATABASE ACTIONS ---
  async function loadConversations() {
    try { conversations = await invoke('get_conversations', { mode: 'all', limit: 50 }); } catch (e) {}
  }
  async function createNewConversation() {
    try {
      const id = await invoke('create_conversation', { mode: 'unified', title: 'New Chat', contextPath: null, model: selectedModel });
      activeConversationId = id;
      messages = [];
      await loadConversations();
    } catch (e) {}
  }
  async function loadMessages(id) {
    activeConversationId = id;
    try {
      const msgs = await invoke('get_messages', { conversationId: id });
      messages = msgs.map(m => ({
        ...m,
        timestamp: new Date(m.timestamp * 1000).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
      }));
      scrollToBottom();
    } catch (e) {}
  }
  async function saveMessage(role, content) {
    if (!activeConversationId) return;
    try { await invoke('add_message', { conversationId: activeConversationId, role, content }); } catch (e) {}
  }
  async function deleteConversation(e, id) {
    e.stopPropagation();
    if (!confirm('Delete this conversation?')) return;
    try {
      await invoke('delete_conversation', { id });
      conversations = conversations.filter(c => c.id !== id);
      if (activeConversationId === id) {
        if (conversations.length > 0) loadMessages(conversations[0].id);
        else createNewConversation();
      }
    } catch (e) {}
  }
  
  // --- CONVERSATION MANAGEMENT ---
  async function renameConversation(id, newTitle) {
    try {
      await invoke('update_conversation_title', { id, title: newTitle });
      await loadConversations();
    } catch (e) {}
  }
  
  async function exportConversation(id) {
    try {
      const conv = conversations.find(c => c.id === id);
      const msgs = await invoke('get_messages', { conversationId: id });
      const exportData = {
        title: conv?.title || 'Exported Chat',
        exportedAt: new Date().toISOString(),
        model: selectedModel,
        messages: msgs.map(m => ({
          role: m.role,
          content: m.content,
          timestamp: new Date(m.timestamp * 1000).toISOString()
        }))
      };
      const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `${conv?.title || 'chat'}-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {}
  }
  
  // --- MESSAGE ACTIONS ---
  async function copyMessage(content, id) {
    try {
      await navigator.clipboard.writeText(content);
      copiedMessageId = id;
      setTimeout(() => copiedMessageId = null, 2000);
    } catch (e) {}
  }
  
  async function stopGeneration() {
    try {
      await invoke('cancel_ai_stream');
      if (streamingContent || streamingSegments.length > 0) {
        const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        
        // Add final text if any
        if (currentSegmentText.trim()) {
          streamingSegments = [...streamingSegments, {
            type: 'text',
            content: currentSegmentText
          }];
        }
        
        messages = [...messages, { 
          role: 'assistant', 
          content: streamingContent + '\n\n*[Stopped]*', 
          reasoning: streamingReasoning,
          segments: streamingSegments,
          timestamp 
        }];
        await saveMessage('assistant', streamingContent);
      }
    } catch (e) {
      console.error('Failed to stop:', e);
    }
    streamingContent = '';
    streamingReasoning = '';
    streamingSegments = [];
    currentSegmentText = '';
    isLoading = false;
  }

  function getGroupedConversations() {
    if (!searchQuery) {
      const groups = { 'Today': [], 'Yesterday': [], 'Previous 7 Days': [], 'Older': [] };
      const now = new Date();
      conversations.forEach(session => {
        const d = new Date((session.updated_at || session.created_at) * 1000);
        const diffDays = Math.floor((now - d) / (1000 * 60 * 60 * 24));
        if (diffDays === 0) groups['Today'].push(session);
        else if (diffDays === 1) groups['Yesterday'].push(session);
        else if (diffDays <= 7) groups['Previous 7 Days'].push(session);
        else groups['Older'].push(session);
      });
      return Object.entries(groups).filter(([_, items]) => items.length > 0);
    } else {
      const filtered = conversations.filter(s => s.title.toLowerCase().includes(searchQuery.toLowerCase()));
      return [['Search Results', filtered]];
    }
  }

  async function sendMessage() {
    if (!input.trim() || isLoading) return;
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    const userMessage = input.trim();
    input = '';
    
    // Reset streaming state
    streamingContent = ''; 
    streamingReasoning = '';
    streamingSegments = []; currentSegmentText = "";
    
    
    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    isLoading = true;
    scrollToBottom();
    
    try {
      const cwd = fsStore.activeWorkspace || '.';
      const mcpPrompt = await invoke('mcp_get_system_prompt', { 
        mode: 'code', 
        cwd: cwd 
      });
      
      const msgs = [
        { role: 'system', content: mcpPrompt },
        ...messages.filter(m => m.role !== 'system').slice(-6).map(m => ({ 
          role: m.role, 
          content: m.content 
        }))
      ];
      
      const config = {
        provider: configStore.settings.ai.provider,
        api_key: configStore.settings.ai.apiKey,
        base_url: configStore.settings.ai.baseUrl || 'https://api.kilo.ai/api/gateway/chat/completions',
        model_name: selectedModel,
        custom_models: []
      };
      
      await invoke('send_agent_message_stream', { messages: msgs, config: config });
    } catch (e) {
      messages = [...messages, { 
        role: 'assistant', 
        content: `**Error**: ${e}`, 
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) 
      }];
      isLoading = false;
    }
  }
  
  async function regenerateResponse(index) {
    if (isLoading || index < 1) return;
    const userMsg = messages[index - 1];
    if (userMsg?.role !== 'user') return;
    
    messages = messages.slice(0, index);
    isLoading = true;
    streamingContent = '';
    streamingReasoning = '';
    streamingSegments = []; currentSegmentText = "";
    
    
    try {
      const cwd = fsStore.activeWorkspace || '.';
      const mcpPrompt = await invoke('mcp_get_system_prompt', { 
        mode: 'code', 
        cwd: cwd 
      });
      
      const msgs = [
        { role: 'system', content: mcpPrompt },
        ...messages.filter(m => m.role !== 'system').slice(-6).map(m => ({ 
          role: m.role, 
          content: m.content 
        }))
      ];
      
      const config = {
        provider: configStore.settings.ai.provider,
        api_key: configStore.settings.ai.apiKey,
        base_url: configStore.settings.ai.baseUrl || 'https://api.kilo.ai/api/gateway/chat/completions',
        model_name: selectedModel,
        custom_models: []
      };
      
      await invoke('send_agent_message_stream', { messages: msgs, config: config });
    } catch (e) {
      messages = [...messages, { 
        role: 'assistant', 
        content: `**Error**: ${e}`, 
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) 
      }];
      isLoading = false;
    }
  }

  function scrollToBottom(smooth = true) {
    if (scrollAnchor) scrollAnchor.scrollIntoView({ behavior: smooth ? 'smooth' : 'auto', block: 'end' });
  }
</script>

<div class="flex h-full bg-background text-foreground animate-in fade-in duration-500">
  <!-- Sidebar -->
  <aside class="w-72 border-r border-border bg-muted/5 flex flex-col hidden md:flex transition-all duration-300">
    <div class="h-14 border-b border-border flex items-center px-4 font-semibold text-sm backdrop-blur-sm bg-background/50 sticky top-0 z-10 justify-between">
      <div class="flex items-center gap-2">
        <Bot size={18} class="text-primary" />
        <span class="text-foreground">History</span>
      </div>
      <button onclick={createNewConversation} class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-colors"><Plus size={18} /></button>
    </div>
    <div class="p-3 pb-0">
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2.5 text-muted-foreground" />
        <input type="text" bind:value={searchQuery} placeholder="Search..." class="w-full bg-background border border-border rounded-lg pl-8 pr-3 py-2 text-xs focus:ring-1 focus:ring-primary outline-none" />
      </div>
    </div>
    <div class="flex-1 overflow-y-auto p-3 space-y-4 scrollbar-thin scrollbar-thumb-muted">
      {#each getGroupedConversations() as [groupName, groupItems]}
        <div class="space-y-1">
          <div class="text-[10px] font-medium text-muted-foreground/70 px-2 uppercase tracking-wider">{groupName}</div>
          {#each groupItems as session}
            <div 
              class={cn("w-full px-3 py-2 text-sm rounded-lg flex items-center gap-3 border transition-all group relative overflow-hidden cursor-pointer", activeConversationId === session.id ? "bg-accent/50 text-foreground border-border shadow-sm" : "border-transparent hover:bg-muted/50 text-muted-foreground hover:text-foreground")}
              onclick={() => loadMessages(session.id)}
            >
              <div class={cn("p-1.5 rounded-md transition-colors", activeConversationId === session.id ? "bg-background text-primary" : "bg-muted/30")}><MessageSquare size={14} /></div>
              <div class="flex-1 min-w-0">
                {#if editingTitle && activeConversationId === session.id}
                  <input 
                    type="text" 
                    bind:value={editingTitleValue}
                    onkeydown={(e) => {
                      if (e.key === 'Enter') {
                        renameConversation(session.id, editingTitleValue);
                        editingTitle = false;
                      }
                      if (e.key === 'Escape') {
                        editingTitle = false;
                      }
                    }}
                    onblur={() => {
                      if (editingTitle) {
                        renameConversation(session.id, editingTitleValue);
                        editingTitle = false;
                      }
                    }}
                    class="w-full bg-background border border-primary rounded px-1 py-0.5 text-xs"
                    onclick={(e) => e.stopPropagation()}
                  />
                {:else}
                  <div class="truncate font-medium" ondblclick={() => { editingTitle = true; editingTitleValue = session.title; }}>{session.title}</div>
                {/if}
                <div class="text-[10px] opacity-60 mt-0.5 truncate">{new Date((session.updated_at || session.created_at) * 1000).toLocaleDateString()}</div>
              </div>
              
              <!-- Action buttons -->
              <div class={cn("flex items-center gap-0.5 transition-opacity", activeConversationId === session.id ? "opacity-100" : "opacity-0 group-hover:opacity-100")}>
                <button 
                  class="p-1 rounded hover:bg-primary/10 hover:text-primary transition-colors" 
                  onclick={(e) => { e.stopPropagation(); exportConversation(session.id); }}
                  title="Export conversation"
                >
                  <Download size={12} />
                </button>
                <button 
                  class="p-1 rounded hover:bg-destructive/10 hover:text-destructive transition-colors" 
                  onclick={(e) => deleteConversation(e, session.id)}
                  title="Delete conversation"
                >
                  <Trash2 size={12} />
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/each}
      
      {#if conversations.length === 0}
        <div class="text-center text-muted-foreground/50 text-xs py-8">
          No conversations yet
        </div>
      {/if}
    </div>
    <div class="p-4 border-t border-border bg-background/30 backdrop-blur-sm">
       <button class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground w-full px-2 py-1.5 rounded-md hover:bg-muted transition-colors"><Settings size={14} /> Agent Settings</button>
    </div>
  </aside>

  <!-- Main Area -->
  <main class="flex-1 flex flex-col relative bg-background/50 backdrop-blur-3xl">
    <header class="h-14 border-b border-border flex items-center justify-between px-6 bg-background/80 backdrop-blur-md sticky top-0 z-20 shadow-sm transition-all duration-300">
      <div class="flex items-center gap-4">
        <div class="w-64"><ModelSelector bind:selectedModel apiKey={configStore.settings.ai.apiKey} onModelChange={(m) => configStore.updateAiConfig({ selectedModel: m.id })} /></div>
        {#if messages.length > 0}
          <div class="text-xs text-muted-foreground flex items-center gap-1">
            <FolderTree size={12} />
            <span>{messages.length} messages</span>
          </div>
        {/if}
      </div>
      <div class="flex items-center gap-3">
        {#if activeConversationId}
          <button 
            onclick={() => exportConversation(activeConversationId)}
            class="p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
            title="Export conversation"
          >
            <Download size={16} />
          </button>
        {/if}
        <div class="text-xs text-muted-foreground flex items-center gap-1.5 bg-muted/30 px-2.5 py-1 rounded-full border border-border/50">
          <span class={cn("w-1.5 h-1.5 rounded-full animate-pulse", isLoading ? "bg-yellow-500" : "bg-green-500")}></span>
          {isLoading ? 'Thinking...' : 'Online'}
        </div>
      </div>
    </header>

    <div class="flex-1 overflow-y-auto p-4 md:p-8 space-y-8 scroll-smooth scrollbar-thin scrollbar-thumb-muted/50 hover:scrollbar-thumb-muted" bind:this={messagesContainer} onscroll={handleScroll}>
       {#if messages.length === 0 && !isLoading && !streamingContent}
         <div class="h-full flex flex-col items-center justify-center text-center space-y-8 opacity-60">
           <div class="relative">
             <div class="w-24 h-24 bg-gradient-to-br from-primary/20 to-purple-500/20 rounded-3xl flex items-center justify-center text-primary backdrop-blur-xl border border-primary/10 shadow-2xl shadow-primary/10"><Bot size={48} strokeWidth={1.5} /></div>
             <div class="absolute -bottom-2 -right-2 bg-background border border-border p-1.5 rounded-xl shadow-lg"><Sparkles size={16} class="text-yellow-500" /></div>
           </div>
           <div><h2 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-foreground to-muted-foreground">Karsa AI</h2><p class="text-muted-foreground mt-3">Ready to code.</p></div>
         </div>
       {/if}

        {#each visibleMessages as msg, i}
          <div class={cn("flex gap-4 max-w-4xl mx-auto group animate-in fade-in slide-in-from-bottom-4 duration-500", msg.role === 'user' ? "flex-row-reverse" : "")}>
            <div class={cn("w-10 h-10 rounded-xl flex items-center justify-center shrink-0 border border-border shadow-md transition-transform group-hover:scale-105", msg.role === 'user' ? "bg-gradient-to-br from-primary to-blue-600 text-primary-foreground" : "bg-card text-foreground")}>
              {#if msg.role === 'user'} <User size={18} /> {:else} <Bot size={18} /> {/if}
            </div>
            <div class={cn("flex flex-col max-w-[85%]", msg.role === 'user' ? "items-end" : "items-start")}>
              <div class="font-medium text-xs mb-1.5 text-muted-foreground flex items-center gap-2">
                {msg.role === 'user' ? 'You' : 'Karsa'} 
                <span class="text-[10px] opacity-50">• {msg.timestamp}</span>
              </div>
              
              {#if msg.role === 'assistant'}
                {#if msg.reasoning}
                  <ThinkingBlock content={msg.reasoning} isStreaming={false} />
                {/if}
                
                <!-- Render segments inline -->
                {#if msg.segments && msg.segments.length > 0}
                  {#each msg.segments as segment}
                    {#if segment.type === 'text'}
                      <div class="bg-card border border-border text-card-foreground rounded-2xl rounded-tl-sm px-6 py-4 text-sm shadow-sm mb-2">
                        <MarkdownRenderer content={segment.content} />
                      </div>
                    {:else if segment.type === 'tool'}
                      <div class="mb-2">
                        <MCPToolCall 
                          toolName={segment.name} 
                          arguments={segment.arguments}
                          result={segment.result}
                          error={segment.error}
                          executing={false}
                        />
                      </div>
                    {/if}
                  {/each}
                {:else}
                  <!-- Fallback for old messages without segments -->
                  <div class={cn("rounded-2xl px-6 py-4 text-sm shadow-sm transition-all hover:shadow-md select-text relative bg-card border border-border text-card-foreground rounded-tl-sm")}>
                    <MarkdownRenderer content={msg.content} />
                  </div>
                {/if}
              {:else}
                <!-- User message -->
                <div class={cn("rounded-2xl px-6 py-4 text-sm shadow-sm transition-all hover:shadow-md select-text relative bg-primary text-primary-foreground rounded-tr-sm")}>
                  <div class="whitespace-pre-wrap break-words leading-relaxed">{msg.content}</div>
                </div>
              {/if}
              
              <!-- Message Actions -->
              <div class={cn("flex items-center gap-1 mt-2 opacity-0 group-hover:opacity-100 transition-opacity", msg.role === 'user' ? "flex-row-reverse" : "")}>
                <button 
                  onclick={() => copyMessage(msg.content, i)} 
                  class="p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                  title="Copy"
                >
                  {#if copiedMessageId === i} <Check size={14} class="text-green-500" /> {:else} <Copy size={14} /> {/if}
                </button>
                {#if msg.role === 'assistant'}
                  <button 
                    onclick={() => regenerateResponse(i)} 
                    class="p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors"
                    title="Regenerate"
                  >
                    <RotateCcw size={14} />
                  </button>
                {/if}
              </div>
            </div>
          </div>
        {/each}

         <!-- Streaming Block -->
         {#if isLoading || streamingContent || streamingReasoning || streamingSegments.length > 0}
           <div class="flex gap-4 max-w-4xl mx-auto animate-in fade-in duration-300">
              <div class="w-10 h-10 rounded-xl bg-card border border-border flex items-center justify-center shrink-0">
                <Bot size={18} class="text-muted-foreground" />
              </div>
              <div class="flex flex-col max-w-[85%] items-start w-full">
                 <div class="font-medium text-xs mb-1.5 text-muted-foreground flex items-center gap-2">
                   Karsa 
                   {#if isLoading}
                     <span class="text-[10px] opacity-50">
                       {#if streamingSegments.some(s => s.type === 'tool' && s.executing)}
                         Running tools...
                       {:else if streamingReasoning}
                         Thinking...
                       {:else}
                         Writing...
                       {/if}
                     </span>
                     <button 
                       onclick={stopGeneration} 
                       class="ml-2 p-1 rounded bg-destructive/10 text-destructive hover:bg-destructive/20 transition-colors"
                       title="Stop"
                     >
                       <Square size={10} />
                     </button>
                   {/if}
                 </div>
                 
                 {#if streamingReasoning}
                   <ThinkingBlock content={streamingReasoning} isStreaming={true} />
                 {/if}

                 <!-- Render segments inline -->
                 {#each streamingSegments as segment}
                   {#if segment.type === 'text'}
                     <div class="bg-card border border-border text-card-foreground rounded-2xl rounded-tl-sm px-6 py-4 text-sm shadow-sm w-full mb-2">
                       <MarkdownRenderer content={segment.content} />
                     </div>
                   {:else if segment.type === 'tool'}
                     <div class="w-full mb-2">
                       <MCPToolCall 
                         toolName={segment.name} 
                         arguments={segment.arguments}
                         result={segment.result}
                         error={segment.error}
                         executing={segment.executing}
                       />
                     </div>
                   {/if}
                 {/each}
                 
                 <!-- Current streaming text -->
                 {#if currentSegmentText}
                   <div class="bg-card border border-border text-card-foreground rounded-2xl rounded-tl-sm px-6 py-4 text-sm shadow-sm w-full">
                      <MarkdownRenderer content={currentSegmentText} />
                      {#if isLoading}
                        <span class="inline-block w-1.5 h-4 bg-primary align-middle ml-1 animate-pulse"></span>
                      {/if}
                   </div>
                 {:else if isLoading && !streamingReasoning && streamingSegments.length === 0}
                   <div class="flex items-center gap-2 text-muted-foreground py-2">
                     <Loader2 size={16} class="animate-spin" />
                     <span class="text-xs">Thinking...</span>
                   </div>
                 {/if}
              </div>
           </div>
         {/if}
       <div class="h-32 shrink-0 w-full" bind:this={scrollAnchor}></div>
    </div>

    {#if showScrollButton}
      <button onclick={() => scrollToBottom(true)} class="absolute bottom-28 left-1/2 -translate-x-1/2 z-40 p-2 rounded-full bg-primary text-primary-foreground shadow-lg hover:scale-110 transition-all animate-in fade-in slide-in-from-bottom-2"><ChevronDown size={20} /></button>
    {/if}

    <div class="absolute bottom-6 left-0 right-0 px-4 md:px-0 flex justify-center z-30 pointer-events-none">
       <div class="w-full max-w-3xl bg-background/70 backdrop-blur-xl border border-border shadow-2xl rounded-2xl p-2 pointer-events-auto transition-all focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary/50 focus-within:shadow-primary/10 focus-within:bg-background/90 group">
          <div class="flex items-end gap-2 relative">
             <button class="p-3 text-muted-foreground hover:bg-muted hover:text-foreground rounded-xl transition-colors shrink-0" title="Attach file (coming soon)"><Plus size={20} /></button>
             <textarea 
               bind:value={input} 
               bind:this={inputTextarea}
               onkeydown={(e) => { 
                 if(e.key === 'Enter' && !e.shiftKey) { 
                   e.preventDefault(); 
                   sendMessage(); 
                 }
                 if(e.key === 'Escape' && isLoading) {
                   stopGeneration();
                 }
               }} 
               placeholder="Ask Karsa anything... (Enter to send, Shift+Enter for newline)" 
               class="flex-1 bg-transparent border-0 resize-none max-h-[200px] min-h-[44px] py-3 px-2 focus:ring-0 text-base placeholder:text-muted-foreground/60 scrollbar-hide leading-relaxed selection:bg-primary/20" 
               rows="1"
             ></textarea>
             {#if isLoading}
               <button 
                 onclick={stopGeneration} 
                 class="p-3 bg-destructive text-destructive-foreground rounded-xl hover:bg-destructive/90 transition-all shrink-0 mb-0.5 shadow-lg"
                 title="Stop generating (Esc)"
               >
                 <Square size={20} />
               </button>
             {:else}
               <button 
                 onclick={sendMessage} 
                 disabled={!input.trim()} 
                 class="p-3 bg-primary text-primary-foreground rounded-xl hover:bg-primary/90 disabled:opacity-50 transition-all shrink-0 mb-0.5 shadow-lg shadow-primary/20 hover:scale-105 active:scale-95"
                 title="Send message (Enter)"
               >
                 <Send size={20} strokeWidth={2.5} />
               </button>
             {/if}
          </div>
          <div class="flex items-center justify-between px-3 py-1 text-[10px] text-muted-foreground/50">
            <span>{messages.length} messages</span>
            <span>Press Enter to send, Shift+Enter for new line</span>
          </div>
       </div>
    </div>
  </main>
</div>
