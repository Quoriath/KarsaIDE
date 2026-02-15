<script>
  import { onMount, onDestroy, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { configStore } from '../stores/config.svelte.js';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { Send, Bot, User, Sparkles, Loader2, Maximize2, Minimize2, Settings, Plus, X, Cpu, Globe, Key, History } from 'lucide-svelte';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import { cn } from '../utils.js';

  let messages = $state([]);
  let input = $state('');
  let isLoading = $state(false);
  let showModelSelector = $state(false);
  let streamingContent = $state(''); // Buffer for incoming chunks
  let messagesContainer;
  let unlistenHandlers = []; // Store unlisten functions
  
  // Use configStore for persistent selection
  let selectedModel = $state(configStore.settings.ai.selectedModel || 'gemini-1.5-pro');

  const availableModels = $derived(
     Array.isArray(configStore.settings.ai.models) 
       ? configStore.settings.ai.models 
       : (configStore.settings.ai.models || '').split(',').map(m => m.trim())
  );

  onMount(async () => {
    // Listen for chunks
    const unlistenChunk = await listen('ai-stream-chunk', (event) => {
      // Event payload is expected to be the string chunk
      const chunk = typeof event.payload === 'string' ? event.payload : event.payload?.chunk || '';
      streamingContent += chunk;
      scrollToBottom();
    });

    // Listen for completion
    const unlistenDone = await listen('ai-stream-done', () => {
      if (streamingContent.trim()) {
        const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        messages = [...messages, { role: 'assistant', content: streamingContent, timestamp }];
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

  function selectModel(model) {
    selectedModel = model;
    configStore.updateAiConfig({ selectedModel: model });
    showModelSelector = false;
  }

  async function sendMessage() {
    if (!input.trim() || isLoading) return;
    
    const userMessage = input.trim();
    input = '';
    isLoading = true;
    streamingContent = ''; // Reset buffer
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    await tick();
    scrollToBottom();
    
    try {
      // Build context from active file if available
      let context = null;
      if (fsStore.activeFile) {
        context = {
          file: fsStore.activeFile.name,
          content: fsStore.activeFileContent
        };
      }

      // Invoke backend command to START the stream
      // The backend will emit events: 'ai-stream-chunk' and 'ai-stream-done'
      await invoke('stream_chat_completion', {
        message: userMessage,
        model: selectedModel,
        context: context,
        config: configStore.settings.ai
      });
    } catch (e) {
      console.error('Failed to start stream:', e);
      messages = [...messages, { role: 'assistant', content: `**Error**: ${e.toString()}`, timestamp: new Date().toLocaleTimeString() }];
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
      // Use requestAnimationFrame for smoother scrolling during heavy rendering
      requestAnimationFrame(() => {
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      });
    }
  }
</script>

<div class="flex h-full bg-background text-foreground animate-in fade-in duration-500">
  
  <!-- Agent Sidebar (History/Context) -->
  <aside class="w-72 border-r border-border bg-muted/5 flex flex-col hidden md:flex transition-all duration-300">
    <div class="h-14 border-b border-border flex items-center px-4 font-semibold text-sm backdrop-blur-sm bg-background/50 sticky top-0 z-10">
      <div class="p-1.5 bg-primary/10 text-primary rounded-md mr-2">
        <Bot size={18} />
      </div>
      <span class="text-foreground">Agent History</span>
      <button class="ml-auto p-1.5 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-colors" title="New Chat">
        <Plus size={18} />
      </button>
    </div>
    
    <div class="flex-1 overflow-y-auto p-3 space-y-1 scrollbar-thin scrollbar-thumb-muted">
      <div class="text-xs font-medium text-muted-foreground px-2 py-1 uppercase tracking-wider mb-1">Today</div>
      
      <button class="w-full text-left px-3 py-2.5 text-sm rounded-lg bg-accent/50 text-foreground font-medium flex items-center gap-3 border border-border hover:border-primary/50 transition-all group shadow-sm">
        <div class="p-1.5 bg-background rounded-md text-primary border border-border group-hover:border-primary/30 transition-colors">
           <Cpu size={14} />
        </div>
        <div class="flex-1 min-w-0">
           <div class="truncate">Project Refactoring</div>
           <div class="text-[10px] text-muted-foreground mt-0.5 truncate">Analyze src/App.svelte...</div>
        </div>
      </button>
    </div>

    <!-- Stats / Token Usage -->
    <div class="p-4 border-t border-border bg-background/30 backdrop-blur-sm">
       <div class="text-xs font-medium text-muted-foreground mb-2 flex justify-between">
          <span>Context Usage</span>
          <span class="text-primary">24%</span>
       </div>
       <div class="h-1.5 w-full bg-muted rounded-full overflow-hidden">
          <div class="h-full bg-gradient-to-r from-primary to-purple-500 w-[24%] rounded-full"></div>
       </div>
       <div class="mt-2 flex justify-between text-[10px] text-muted-foreground">
          <span>2,405 tokens</span>
          <span>128k limit</span>
       </div>
    </div>
  </aside>

  <!-- Main Chat Area -->
  <main class="flex-1 flex flex-col relative bg-background/50 backdrop-blur-3xl">
    <!-- Chat Header -->
    <header class="h-14 border-b border-border flex items-center justify-between px-6 bg-background/80 backdrop-blur-md sticky top-0 z-20 shadow-sm transition-all duration-300">
      
      <!-- Model Selector -->
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
      
      <!-- Context Indicator -->
      <div class="flex items-center gap-3">
         <div class="flex -space-x-2">
            <div class="w-6 h-6 rounded-full bg-blue-500 border-2 border-background flex items-center justify-center text-[10px] text-white font-bold">A</div>
            <div class="w-6 h-6 rounded-full bg-purple-500 border-2 border-background flex items-center justify-center text-[10px] text-white font-bold">M</div>
         </div>
         <div class="h-4 w-[1px] bg-border mx-1"></div>
         <div class="text-xs text-muted-foreground flex items-center gap-1.5 bg-muted/30 px-2.5 py-1 rounded-full border border-border/50">
            <span class={cn("w-1.5 h-1.5 rounded-full animate-pulse", isLoading ? "bg-yellow-500" : "bg-green-500")}></span>
            {isLoading ? 'Thinking...' : 'Online'}
         </div>
      </div>
    </header>

    <!-- Messages -->
    <div class="flex-1 overflow-y-auto p-4 md:p-8 space-y-8 scroll-smooth pb-40 scrollbar-thin scrollbar-thumb-muted/50 hover:scrollbar-thumb-muted" bind:this={messagesContainer}>
       {#if messages.length === 0 && !isLoading}
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

       {#each messages as msg}
         <div class={cn("flex gap-4 max-w-4xl mx-auto group animate-in fade-in slide-in-from-bottom-4 duration-500", msg.role === 'user' ? "flex-row-reverse" : "")}>
           
           <!-- Avatar -->
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
           
           <!-- Message Body -->
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
       {#if isLoading}
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
                      <!-- Cursor effect -->
                      <span class="inline-block w-1.5 h-4 bg-primary align-middle ml-1 animate-pulse"></span>
                   {:else}
                      <!-- Bouncing Dots Typing Indicator -->
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

    <!-- Input Area (Floating) -->
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
