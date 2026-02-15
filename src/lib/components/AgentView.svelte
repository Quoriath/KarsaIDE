<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { configStore } from '../stores/config.svelte.js';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { 
    Send, Bot, User, Sparkles, Loader2, Maximize2, Minimize2, Settings, 
    Plus, X, Cpu, Globe, Key, History, Trash2, Download, Search, MessageSquare 
  } from 'lucide-svelte';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import { cn } from '../utils.js';

  // --- STATE MANAGEMENT ---
  let sessions = $state([]);
  let activeSessionId = $state(null);
  let searchQuery = $state('');
  let input = $state('');
  let isLoading = $state(false);
  let showModelSelector = $state(false);
  let streamingContent = $state(''); 
  
  let messagesContainer;
  let unlistenHandlers = [];

  // Derived State
  let activeSession = $derived(sessions.find(s => s.id === activeSessionId));
  let currentMessages = $derived(activeSession ? activeSession.messages : []);
  
  // Persistent Config
  let selectedModel = $state(configStore.settings.ai.selectedModel || 'gemini-1.5-pro');
  const availableModels = $derived(
     Array.isArray(configStore.settings.ai.models) 
       ? configStore.settings.ai.models 
       : (configStore.settings.ai.models || '').split(',').map(m => m.trim())
  );

  // --- LIFECYCLE & PERSISTENCE ---

  onMount(async () => {
    // 1. Load History
    const savedHistory = localStorage.getItem('karsa-chat-history');
    if (savedHistory) {
      try {
        sessions = JSON.parse(savedHistory);
      } catch (e) {
        console.error('Failed to load chat history', e);
      }
    }

    // Ensure at least one session exists
    if (sessions.length === 0) {
      createNewSession();
    } else {
      // Select most recent
      activeSessionId = sessions[0].id;
    }

    // 2. Setup Stream Listeners
    const unlistenChunk = await listen('ai-stream-chunk', (event) => {
      const chunk = typeof event.payload === 'string' ? event.payload : event.payload?.chunk || '';
      streamingContent += chunk;
      scrollToBottom();
    });

    const unlistenDone = await listen('ai-stream-done', () => {
      if (streamingContent.trim()) {
        addMessageToActiveSession({ 
          role: 'assistant', 
          content: streamingContent, 
          timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) 
        });
      }
      streamingContent = '';
      isLoading = false;
      scrollToBottom();
    });

    unlistenHandlers.push(unlistenChunk, unlistenDone);
  });

  onDestroy(() => {
    unlistenHandlers.forEach(fn => fn());
  });

  // 3. Save History on Change (Auto-save)
  $effect(() => {
    if (sessions.length > 0) {
      localStorage.setItem('karsa-chat-history', JSON.stringify(sessions));
    }
  });

  // --- ACTIONS ---

  function createNewSession() {
    const newSession = {
      id: crypto.randomUUID(),
      title: 'New Chat',
      date: new Date().toISOString(),
      messages: []
    };
    sessions = [newSession, ...sessions];
    activeSessionId = newSession.id;
    input = '';
    streamingContent = '';
  }

  function deleteSession(e, id) {
    e.stopPropagation();
    sessions = sessions.filter(s => s.id !== id);
    if (activeSessionId === id) {
      activeSessionId = sessions[0]?.id || null;
      if (!activeSessionId) createNewSession();
    }
  }

  function clearCurrentHistory() {
    if (confirm('Are you sure you want to clear this conversation?')) {
      updateActiveSessionMessages([]);
    }
  }

  function exportChat() {
    if (!activeSession) return;
    const data = JSON.stringify(activeSession, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `karsa-chat-${activeSession.id.slice(0,8)}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  // --- LOGIC HELPERS ---

  function addMessageToActiveSession(msg) {
    if (!activeSessionId) return;
    
    // Find index of active session to mutate it (Svelte 5 requirement for reactivity on array items)
    const index = sessions.findIndex(s => s.id === activeSessionId);
    if (index !== -1) {
      const updatedMessages = [...sessions[index].messages, msg];
      
      // Limit to 100 messages for performance
      if (updatedMessages.length > 100) {
        updatedMessages.shift();
      }

      // Update session with new message and title if needed
      const updatedSession = { ...sessions[index], messages: updatedMessages };
      
      // Auto-title: if it's the first user message
      if (updatedMessages.length === 1 && msg.role === 'user') {
        updatedSession.title = msg.content.slice(0, 30) + (msg.content.length > 30 ? '...' : '');
      }
      
      // Reassign to trigger reactivity
      const newSessions = [...sessions];
      newSessions[index] = updatedSession;
      sessions = newSessions;
    }
  }

  function updateActiveSessionMessages(newMessages) {
    const index = sessions.findIndex(s => s.id === activeSessionId);
    if (index !== -1) {
      const newSessions = [...sessions];
      newSessions[index] = { ...newSessions[index], messages: newMessages };
      sessions = newSessions;
    }
  }

  function getGroupedSessions() {
    if (!searchQuery) {
      // Group by date logic
      const groups = { 'Today': [], 'Yesterday': [], 'Previous 7 Days': [], 'Older': [] };
      const now = new Date();
      
      sessions.forEach(session => {
        const d = new Date(session.date);
        const diffDays = Math.floor((now - d) / (1000 * 60 * 60 * 24));
        
        if (diffDays === 0) groups['Today'].push(session);
        else if (diffDays === 1) groups['Yesterday'].push(session);
        else if (diffDays <= 7) groups['Previous 7 Days'].push(session);
        else groups['Older'].push(session);
      });
      
      return Object.entries(groups).filter(([_, items]) => items.length > 0);
    } else {
      // Filter by search
      const filtered = sessions.filter(s => 
        s.title.toLowerCase().includes(searchQuery.toLowerCase()) || 
        s.messages.some(m => m.content.toLowerCase().includes(searchQuery.toLowerCase()))
      );
      return [['Search Results', filtered]];
    }
  }

  // --- CORE CHAT LOGIC ---

  function selectModel(model) {
    selectedModel = model;
    configStore.updateAiConfig({ selectedModel: model });
    showModelSelector = false;
  }

  async function sendMessage() {
    if (!input.trim() || isLoading) return;
    
    if (!activeSessionId) createNewSession();

    const userMessage = input.trim();
    input = '';
    isLoading = true;
    streamingContent = ''; 
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    addMessageToActiveSession({ role: 'user', content: userMessage, timestamp });
    
    await tick();
    scrollToBottom();
    
    try {
      let context = null;
      if (fsStore.activeFile) {
        context = {
          file: fsStore.activeFile.name,
          content: fsStore.activeFileContent
        };
      }

      await invoke('stream_chat_completion', {
        message: userMessage,
        model: selectedModel,
        context: context,
        config: configStore.settings.ai
      });
    } catch (e) {
      console.error('Failed to start stream:', e);
      addMessageToActiveSession({ 
        role: 'assistant', 
        content: `**Error**: ${e.toString()}`, 
        timestamp: new Date().toLocaleTimeString() 
      });
      isLoading = false;
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
    target.style.height = `${Math.min(target.scrollHeight, 200)}px`;
  }

  function scrollToBottom() {
    if (messagesContainer) {
      requestAnimationFrame(() => {
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      });
    }
  }
</script>

<div class="flex h-full bg-background text-foreground animate-in fade-in duration-500">
  
  <!-- Agent Sidebar (History/Context) -->
  <aside class="w-72 border-r border-border bg-muted/5 flex flex-col hidden md:flex transition-all duration-300">
    <div class="h-14 border-b border-border flex items-center px-4 font-semibold text-sm backdrop-blur-sm bg-background/50 sticky top-0 z-10 justify-between">
      <div class="flex items-center gap-2">
        <Bot size={18} class="text-primary" />
        <span class="text-foreground">History</span>
      </div>
      <button 
        onclick={createNewSession}
        class="p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-colors" 
        title="New Chat"
      >
        <Plus size={18} />
      </button>
    </div>

    <!-- Search History -->
    <div class="p-3 pb-0">
      <div class="relative">
        <Search size={14} class="absolute left-2.5 top-2.5 text-muted-foreground" />
        <input 
          type="text" 
          bind:value={searchQuery}
          placeholder="Search chats..." 
          class="w-full bg-background border border-border rounded-lg pl-8 pr-3 py-2 text-xs focus:ring-1 focus:ring-primary outline-none"
        />
      </div>
    </div>
    
    <!-- Session List -->
    <div class="flex-1 overflow-y-auto p-3 space-y-4 scrollbar-thin scrollbar-thumb-muted">
      {#each getGroupedSessions() as [groupName, groupItems]}
        <div class="space-y-1">
          <div class="text-[10px] font-medium text-muted-foreground/70 px-2 uppercase tracking-wider">{groupName}</div>
          {#each groupItems as session}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
              class={cn(
                "w-full px-3 py-2 text-sm rounded-lg flex items-center gap-3 border transition-all group relative overflow-hidden cursor-pointer",
                activeSessionId === session.id 
                  ? "bg-accent/50 text-foreground border-border shadow-sm" 
                  : "border-transparent hover:bg-muted/50 text-muted-foreground hover:text-foreground"
              )}
              onclick={() => activeSessionId = session.id}
            >
              <div class={cn("p-1.5 rounded-md transition-colors", activeSessionId === session.id ? "bg-background text-primary" : "bg-muted/30")}>
                 <MessageSquare size={14} />
              </div>
              <div class="flex-1 min-w-0">
                 <div class="truncate font-medium">{session.title}</div>
                 <div class="text-[10px] opacity-60 mt-0.5 truncate">{new Date(session.date).toLocaleDateString()}</div>
              </div>
              
              <!-- Delete Button (Visible on Hover/Active) -->
              <button 
                class={cn("absolute right-2 top-1/2 -translate-y-1/2 p-1.5 rounded hover:bg-destructive/10 hover:text-destructive transition-colors bg-background/80 backdrop-blur-sm", activeSessionId === session.id ? "opacity-100" : "opacity-0 group-hover:opacity-100")}
                onclick={(e) => deleteSession(e, session.id)}
                title="Delete Chat"
              >
                <Trash2 size={12} />
              </button>
            </div>
          {/each}
        </div>
      {/each}
    </div>

    <!-- Stats / Footer -->
    <div class="p-4 border-t border-border bg-background/30 backdrop-blur-sm">
       <button class="flex items-center gap-2 text-xs text-muted-foreground hover:text-foreground w-full px-2 py-1.5 rounded-md hover:bg-muted transition-colors">
         <Settings size={14} />
         Agent Settings
       </button>
    </div>
  </aside>

  <!-- Main Chat Area -->
  <main class="flex-1 flex flex-col relative bg-background/50 backdrop-blur-3xl">
    <!-- Chat Header -->
    <header class="h-14 border-b border-border flex items-center justify-between px-6 bg-background/80 backdrop-blur-md sticky top-0 z-20 shadow-sm transition-all duration-300">
      
      <!-- Model Selector -->
      <div class="flex items-center gap-4">
        <div class="relative">
          <button 
            class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-muted/50 hover:bg-muted text-sm font-medium transition-all border border-border hover:border-primary/50 group"
            onclick={() => showModelSelector = !showModelSelector}
          >
             <Sparkles size={14} class="text-primary group-hover:scale-110 transition-transform" />
             <span>{selectedModel}</span>
             <Globe size={12} class="ml-2 opacity-50" />
          </button>
          
          {#if showModelSelector}
            <div class="absolute top-full left-0 mt-2 w-64 bg-popover border border-border rounded-xl shadow-2xl p-1.5 z-50 animate-in fade-in zoom-in-95 duration-200 origin-top-left ring-1 ring-border">
              <div class="px-2 py-1.5 text-xs font-medium text-muted-foreground">Select Model</div>
              {#each availableModels as model}
                <button 
                  class={cn("w-full text-left px-3 py-2 text-sm rounded-lg hover:bg-accent flex items-center gap-2 transition-colors", selectedModel === model ? "bg-accent text-accent-foreground font-medium" : "text-popover-foreground")}
                  onclick={() => selectModel(model)}
                >
                  <Cpu size={14} class={selectedModel === model ? "text-primary" : "opacity-50"} />
                  <span class="truncate">{model}</span>
                  {#if selectedModel === model}
                    <div class="ml-auto w-1.5 h-1.5 rounded-full bg-primary shadow-[0_0_8px_rgba(59,130,246,0.5)]"></div>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </div>
      
      <!-- Header Actions -->
      <div class="flex items-center gap-2">
         {#if activeSession && activeSession.messages.length > 0}
           <button class="p-2 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-colors" title="Export Chat" onclick={exportChat}>
              <Download size={16} />
           </button>
           <button class="p-2 hover:bg-destructive/10 rounded-md text-muted-foreground hover:text-destructive transition-colors" title="Clear History" onclick={clearCurrentHistory}>
              <Trash2 size={16} />
           </button>
           <div class="h-4 w-[1px] bg-border mx-1"></div>
         {/if}
         
         <div class="text-xs text-muted-foreground flex items-center gap-1.5 bg-muted/30 px-2.5 py-1 rounded-full border border-border/50">
            <span class={cn("w-1.5 h-1.5 rounded-full animate-pulse", isLoading ? "bg-yellow-500" : "bg-green-500")}></span>
            {isLoading ? 'Thinking...' : 'Online'}
         </div>
      </div>
    </header>

    <!-- Messages -->
    <div class="flex-1 overflow-y-auto p-4 md:p-8 space-y-8 scroll-smooth pb-40 scrollbar-thin scrollbar-thumb-muted/50 hover:scrollbar-thumb-muted" bind:this={messagesContainer}>
       {#if currentMessages.length === 0 && !isLoading && !streamingContent}
         <div class="h-full flex flex-col items-center justify-center text-center space-y-8 opacity-0 animate-in fade-in slide-in-from-bottom-8 duration-700 fill-mode-forwards" style="animation-delay: 100ms">
           <div class="relative">
             <div class="w-24 h-24 bg-gradient-to-br from-primary/20 to-purple-500/20 rounded-3xl flex items-center justify-center text-primary backdrop-blur-xl border border-primary/10 shadow-2xl shadow-primary/10">
                <Bot size={48} strokeWidth={1.5} />
             </div>
             <div class="absolute -bottom-2 -right-2 bg-background border border-border p-1.5 rounded-xl shadow-lg">
                <Sparkles size={16} class="text-yellow-500" />
             </div>
           </div>
           
           <div>
             <h2 class="text-3xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-foreground to-muted-foreground">How can I help you build?</h2>
             <p class="text-muted-foreground mt-3 max-w-md mx-auto text-lg leading-relaxed">I have full access to your codebase. Ask me to refactor, explain, or generate new features.</p>
           </div>

           <div class="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-2xl w-full px-4">
              {#each [
                { title: 'Refactor Code', desc: 'Modernize legacy patterns', icon: Sparkles, color: 'text-yellow-500' },
                { title: 'Explain Logic', desc: 'Understand complex functions', icon: Globe, color: 'text-blue-500' },
                { title: 'Generate Tests', desc: 'Ensure code reliability', icon: Key, color: 'text-green-500' },
                { title: 'Optimize', desc: 'Improve performance', icon: Cpu, color: 'text-purple-500' }
              ] as action}
                <button 
                  onclick={() => input = action.title} 
                  class="p-4 bg-card/50 backdrop-blur-sm border border-border rounded-xl hover:bg-muted/80 text-left transition-all hover:scale-[1.02] hover:shadow-lg hover:border-primary/20 group relative overflow-hidden"
                >
                   <div class="absolute inset-0 bg-gradient-to-r from-transparent via-primary/5 to-transparent translate-x-[-100%] group-hover:translate-x-[100%] transition-transform duration-1000"></div>
                   <div class="flex items-start gap-3 relative z-10">
                      <div class={cn("p-2 rounded-lg bg-background border border-border shadow-sm", action.color)}>
                        <action.icon size={18} />
                      </div>
                      <div>
                        <div class="font-medium text-foreground group-hover:text-primary transition-colors">{action.title}</div>
                        <div class="text-xs text-muted-foreground mt-0.5">{action.desc}</div>
                      </div>
                   </div>
                </button>
              {/each}
           </div>
         </div>
       {/if}

       <!-- Render Messages -->
       {#each currentMessages as msg}
         <div class={cn("flex gap-4 max-w-4xl mx-auto group animate-in fade-in slide-in-from-bottom-4 duration-500", msg.role === 'user' ? "flex-row-reverse" : "")}>
           <div class={cn("w-10 h-10 rounded-xl flex items-center justify-center shrink-0 border border-border shadow-md transition-transform group-hover:scale-105", 
             msg.role === 'user' 
               ? "bg-gradient-to-br from-primary to-blue-600 text-primary-foreground" 
               : "bg-card text-foreground")}>
             {#if msg.role === 'user'}
               <User size={18} />
             {:else}
               <Bot size={18} />
             {/if}
           </div>
           
           <div class={cn("flex flex-col max-w-[85%]", msg.role === 'user' ? "items-end" : "items-start")}>
             <div class="font-medium text-xs mb-1.5 text-muted-foreground flex items-center gap-2">
                {msg.role === 'user' ? 'You' : 'Karsa'}
                <span class="text-[10px] opacity-50">• {msg.timestamp}</span>
             </div>
             
             <div class={cn("rounded-2xl px-6 py-4 text-sm shadow-sm transition-all hover:shadow-md", 
               msg.role === 'user' 
                 ? "bg-primary text-primary-foreground rounded-tr-sm" 
                 : "bg-card border border-border text-card-foreground rounded-tl-sm")}>
               
               {#if msg.role === 'assistant'}
                 <MarkdownRenderer content={msg.content} />
               {:else}
                 <div class="whitespace-pre-wrap break-words leading-relaxed">{msg.content}</div>
               {/if}
             </div>
           </div>
         </div>
       {/each}

       <!-- Streaming Response / Typing Indicator -->
       {#if isLoading || streamingContent}
          <div class="flex gap-4 max-w-4xl mx-auto animate-in fade-in duration-300">
             <div class="w-10 h-10 rounded-xl bg-card border border-border flex items-center justify-center shrink-0">
               <Bot size={18} class="text-muted-foreground" />
             </div>
             <div class="flex flex-col max-w-[85%] items-start">
                <div class="font-medium text-xs mb-1.5 text-muted-foreground flex items-center gap-2">
                   Karsa
                   <span class="text-[10px] opacity-50">• Typing...</span>
                </div>
                <div class="bg-card border border-border text-card-foreground rounded-2xl rounded-tl-sm px-6 py-4 text-sm shadow-sm min-w-[60px]">
                   {#if streamingContent}
                      <MarkdownRenderer content={streamingContent} />
                      <span class="inline-block w-1.5 h-4 bg-primary align-middle ml-1 animate-pulse"></span>
                   {:else}
                      <div class="flex items-center gap-1.5 py-1">
                         <span class="w-2 h-2 bg-primary/60 rounded-full animate-bounce" style="animation-delay: 0ms"></span>
                         <span class="w-2 h-2 bg-primary/60 rounded-full animate-bounce" style="animation-delay: 150ms"></span>
                         <span class="w-2 h-2 bg-primary/60 rounded-full animate-bounce" style="animation-delay: 300ms"></span>
                      </div>
                   {/if}
                </div>
             </div>
          </div>
       {/if}
    </div>

    <!-- Input Area -->
    <div class="absolute bottom-6 left-0 right-0 px-4 md:px-0 flex justify-center z-30 pointer-events-none">
       <div class="w-full max-w-3xl bg-background/70 backdrop-blur-xl border border-border shadow-2xl rounded-2xl p-2 pointer-events-auto transition-all focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary/50 focus-within:shadow-primary/10 focus-within:bg-background/90 group">
          <div class="flex items-end gap-2 relative">
             <button class="p-3 text-muted-foreground hover:bg-muted hover:text-foreground rounded-xl transition-colors shrink-0" title="Attach File">
                <Plus size={20} />
             </button>
             
             <textarea
               bind:value={input}
               onkeydown={handleKeydown}
               oninput={autoResize}
               placeholder="Ask Karsa anything..."
               class="flex-1 bg-transparent border-0 resize-none max-h-[200px] min-h-[44px] py-3 px-2 focus:ring-0 text-base placeholder:text-muted-foreground/60 scrollbar-hide leading-relaxed selection:bg-primary/20"
               rows="1"
             ></textarea>
             
             <button 
               onclick={sendMessage}
               disabled={isLoading || !input.trim()}
               class="p-3 bg-primary text-primary-foreground rounded-xl hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-all shrink-0 mb-0.5 shadow-lg shadow-primary/20 hover:scale-105 active:scale-95"
             >
               {#if isLoading}
                 <Loader2 size={20} class="animate-spin" />
               {:else}
                 <Send size={20} strokeWidth={2.5} />
               {/if}
             </button>
          </div>
       </div>
    </div>
  </main>
</div>
