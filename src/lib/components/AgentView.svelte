<script>
  import { onMount, tick } from 'svelte';
  import { configStore } from '../stores/config.svelte.js';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { chatStore } from '../stores/chatStore.svelte.js';
  import { 
    Send, Bot, User, Sparkles, Loader2, Plus, 
    MessageSquare, Trash2, ChevronDown, Copy, Check, 
    RotateCcw, Square, FolderTree, Cpu
  } from 'lucide-svelte';
  import MarkdownRenderer from './MarkdownRenderer.svelte';
  import ModelSelector from './ModelSelector.svelte';
  import ThinkingBlock from './Chat/ThinkingBlock.svelte';
  import MCPToolCall from './MCPToolCall.svelte';
  import { cn } from '../utils.js';

  let input = $state('');
  let showScrollButton = $state(false);
  let messagesContainer;
  let scrollAnchor;
  let inputTextarea;

  onMount(async () => {
    await chatStore.loadConversations();
    if (chatStore.conversations.length === 0) {
      await chatStore.createNewConversation();
    } else if (!chatStore.activeConversationId) {
      await chatStore.loadConversation(chatStore.conversations[0].id);
    }
  });

  function handleScroll() {
    if (!messagesContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = messagesContainer;
    showScrollButton = scrollHeight - scrollTop - clientHeight > 200;
  }

  async function handleSendMessage() {
    if (!input.trim() || chatStore.isLoading) return;
    const msg = input;
    input = '';
    await chatStore.sendMessage(msg);
    scrollToBottom();
  }

  function scrollToBottom(smooth = true) {
    if (scrollAnchor) {
      scrollAnchor.scrollIntoView({ behavior: smooth ? 'smooth' : 'auto', block: 'end' });
    }
  }

  function handleKeydown(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  }

  function autoResize(e) {
    const target = e.target;
    target.style.height = 'auto';
    target.style.height = `${Math.min(target.scrollHeight, 150)}px`;
  }

  $effect(() => {
    if (chatStore.messages.length || chatStore.streamingContent) {
      scrollToBottom();
    }
  });
</script>

<div class="flex h-full bg-background text-foreground overflow-hidden">
  <!-- Minimal History Sidebar -->
  <aside class="w-60 border-r border-border/20 bg-muted/[0.02] flex flex-col hidden lg:flex shrink-0 overflow-hidden transition-all duration-500">
    <div class="h-11 border-b border-border/20 flex items-center px-5 justify-between shrink-0">
      <div class="flex items-center gap-2">
        <MessageSquare size={13} class="text-primary/60" />
        <span class="text-[9px] font-black uppercase tracking-[0.2em] text-muted-foreground/80">Logs</span>
      </div>
      <button onclick={() => chatStore.createNewConversation()} class="p-1 hover:bg-muted rounded text-muted-foreground transition-all">
        <Plus size={14} />
      </button>
    </div>
    
    <div class="flex-1 overflow-y-auto p-2 space-y-0.5 scrollbar-hide">
      {#each chatStore.conversations as session}
        <button 
          class={cn(
            "w-full px-3 py-2 text-xs rounded-xl flex flex-col border transition-all group relative overflow-hidden text-left",
            chatStore.activeConversationId === session.id 
              ? "bg-primary/[0.03] border-primary/10 shadow-sm" 
              : "border-transparent hover:bg-muted/30 text-muted-foreground hover:text-foreground"
          )}
          onclick={() => chatStore.loadConversation(session.id)}
        >
          <span class={cn("truncate font-bold tracking-tight text-[10.5px]", chatStore.activeConversationId === session.id ? "text-primary" : "opacity-70 group-hover:opacity-100")}>
            {session.title}
          </span>
          <span class="text-[7px] opacity-20 font-mono tracking-tighter uppercase mt-0.5">
            {new Date(session.updated_at * 1000).toLocaleDateString()}
          </span>
        </button>
      {/each}
    </div>
  </aside>

  <!-- Main Chat Area -->
  <main class="flex-1 flex flex-col relative bg-background overflow-hidden selection:bg-primary/10">
    <!-- Header -->
    <header class="h-11 border-b border-border/20 flex items-center justify-between px-6 bg-background/80 backdrop-blur-xl shrink-0 z-20 shadow-sm transition-all duration-500">
      <div class="flex items-center gap-4 min-w-0">
        <div class="flex items-center gap-2.5">
           <div class="w-6 h-6 rounded-lg bg-primary/10 flex items-center justify-center text-primary border border-primary/10">
              <Cpu size={14} />
           </div>
           <div class="flex flex-col">
              <span class="text-[10px] font-black uppercase tracking-[0.15em] leading-none">Intelligence</span>
              <span class="text-[7px] text-primary/60 font-bold uppercase tracking-[0.2em] leading-none mt-1">Operational</span>
           </div>
        </div>

        <div class="hidden sm:flex items-center gap-3 min-w-0">
           <div class="w-[1px] h-3 bg-border/30 shrink-0"></div>
           <div class="scale-[0.85] origin-left">
             <ModelSelector 
               selectedModel={configStore.settings.ai.selectedModel} 
               apiKey={configStore.settings.ai.apiKey} 
               onModelChange={(m) => configStore.updateAiConfig({ selectedModel: m.id })} 
             />
           </div>
        </div>
      </div>

      <div class="flex items-center gap-3 shrink-0">
        {#if fsStore.activeWorkspace}
          <div class="hidden md:flex text-[8px] font-black text-muted-foreground/40 items-center gap-2 bg-muted/10 px-2.5 py-1 rounded-full border border-border/20 max-w-[150px] uppercase tracking-widest">
            <FolderTree size={10} />
            <span class="truncate">{fsStore.activeWorkspace.split('/').pop()}</span>
          </div>
        {/if}
        <div class="flex items-center gap-1.5 bg-green-500/5 px-2 py-1 rounded-md border border-green-500/10 text-[8px] font-black text-green-500/60 uppercase tracking-[0.2em]">
           Linked
        </div>
      </div>
    </header>

    <!-- Message Stream (Super Compact) -->
    <div 
      class="flex-1 overflow-y-auto p-4 lg:p-10 space-y-6 scroll-smooth no-scrollbar relative min-h-0" 
      bind:this={messagesContainer} 
      onscroll={handleScroll}
    >
       {#if chatStore.messages.length === 0 && !chatStore.isStreaming}
         <div class="h-full flex flex-col items-center justify-center text-center space-y-4 opacity-10 animate-in zoom-in-95 duration-1000">
           <Bot size={40} strokeWidth={1} class="text-primary animate-float" />
           <div class="space-y-1">
              <h2 class="text-xl font-black uppercase tracking-[0.3em]">AI Core</h2>
              <p class="text-[8px] font-bold tracking-[0.5em] text-muted-foreground uppercase">Waiting for input</p>
           </div>
         </div>
       {/if}

        {#each chatStore.messages as msg, i}
          <div class={cn(
            "flex gap-4 max-w-4xl mx-auto group animate-in fade-in slide-in-from-bottom-1 duration-300", 
            msg.role === 'user' ? "flex-row-reverse" : ""
          )}>
            <div class={cn(
              "w-7 h-7 rounded-lg flex items-center justify-center shrink-0 border shadow-sm mt-0.5 transition-all duration-500 group-hover:border-primary/20", 
              msg.role === 'user' ? "bg-primary text-primary-foreground border-primary/20" : "bg-muted/20 text-muted-foreground/60 border-border/30"
            )}>
              {#if msg.role === 'user'} <User size={14} /> {:else} <Bot size={14} /> {/if}
            </div>
            
            <div class={cn("flex flex-col max-w-[88%] min-w-0 flex-1", msg.role === 'user' ? "items-end" : "items-start")}>
              <div class="text-[8px] font-black uppercase tracking-[0.2em] mb-1 text-muted-foreground/20 flex items-center gap-2">
                {msg.role === 'user' ? 'Architect' : 'Karsa'} 
                <span class="opacity-50 tracking-tighter">• {msg.timestamp}</span>
              </div>
              
              <div class={cn(
                "rounded-[20px] px-4 py-2.5 text-[12.5px] leading-[1.45] transition-all duration-500 select-text relative w-fit max-w-full",
                msg.role === 'user' 
                  ? "bg-primary text-primary-foreground rounded-tr-none shadow-lg shadow-primary/5" 
                  : "bg-muted/20 border border-border/20 text-foreground/90 rounded-tl-none"
              )}>
                {#if msg.role === 'assistant'}
                  {#if msg.segments}
                    {#each msg.segments as segment}
                      {#if segment.type === 'thinking'}
                        <ThinkingBlock content={segment.content} isThinkingComplete={true} />
                      {:else if segment.type === 'text'}
                        <div class="w-full overflow-hidden mb-0.5 last:mb-0"><MarkdownRenderer content={segment.content} /></div>
                      {:else if segment.type === 'tool'}
                        <div class="my-2 max-w-full opacity-90 scale-95 origin-left"><MCPToolCall toolName={segment.name} arguments={segment.arguments} result={segment.result} error={segment.error} executing={false} /></div>
                      {/if}
                    {/each}
                  {:else}
                    <div class="w-full overflow-hidden"><MarkdownRenderer content={msg.content} /></div>
                  {/if}
                {:else}
                  <div class="whitespace-pre-wrap break-words leading-[1.45] opacity-95">{msg.content}</div>
                {/if}
              </div>
            </div>
          </div>
        {/each}

         {#if chatStore.isStreaming || chatStore.isLoading}
           <div class="flex gap-4 max-w-4xl mx-auto animate-in fade-in duration-300">
              <div class="w-7 h-7 rounded-lg bg-muted/20 border border-border/20 flex items-center justify-center shrink-0 mt-0.5 shadow-sm">
                <Bot size={14} class="text-primary/60 animate-pulse" />
              </div>
              <div class="flex flex-col max-w-[88%] items-start w-full min-w-0 flex-1">
                 <div class="text-[8px] font-black uppercase tracking-[0.2em] mb-1.5 text-primary/60 flex items-center gap-2">
                   Agent Processing
                   <div class="flex gap-0.5 opacity-40">
                      <span class="w-0.5 h-0.5 rounded-full bg-primary animate-bounce"></span>
                      <span class="w-0.5 h-0.5 rounded-full bg-primary animate-bounce delay-150"></span>
                   </div>
                 </div>
                 
                 {#if chatStore.streamingReasoning}
                   <ThinkingBlock 
                     content={chatStore.streamingReasoning} 
                     isStreaming={true} 
                     isThinkingComplete={chatStore.isThinkingComplete} 
                     isToolExecuting={chatStore.isToolExecuting} 
                   />
                 {/if}

                 {#each chatStore.streamingSegments as segment}
                   {#if segment.type === 'text'}
                     <div class="bg-muted/10 border border-border/20 rounded-[20px] rounded-tl-none px-4 py-2.5 text-[12.5px] shadow-sm w-fit max-w-full overflow-hidden mb-1 last:mb-0">
                       <MarkdownRenderer content={segment.content} />
                     </div>
                   {:else if segment.type === 'tool'}
                     <div class="w-full mb-2 opacity-80 scale-95 origin-left">
                       <MCPToolCall toolName={segment.name} arguments={segment.arguments} result={segment.result} error={segment.error} executing={segment.executing} />
                     </div>
                   {/if}
                 {/each}
                 
                 {#if chatStore.currentSegmentText || (chatStore.isLoading && !chatStore.streamingReasoning && chatStore.streamingSegments.length === 0)}
                   <div class="bg-muted/10 border border-border/20 rounded-[20px] rounded-tl-none px-4 py-2.5 text-[12.5px] shadow-sm w-fit max-w-full overflow-hidden">
                      {#if chatStore.currentSegmentText}
                        <MarkdownRenderer content={chatStore.currentSegmentText} />
                        <span class="inline-block w-1 h-3 bg-primary align-middle ml-1 animate-pulse rounded-full"></span>
                      {:else}
                        <div class="flex items-center gap-2 text-muted-foreground/40 py-0.5">
                           <Loader2 size={10} class="animate-spin" />
                           <span class="text-[9px] font-bold uppercase tracking-widest">Synthesizing...</span>
                        </div>
                      {/if}
                   </div>
                 {/if}

                 <button 
                   onclick={() => chatStore.stopGeneration()} 
                   class="mt-3 flex items-center gap-1.5 px-2 py-1 rounded-md border border-destructive/10 bg-destructive/[0.01] hover:bg-destructive/10 text-destructive text-[7px] font-black uppercase tracking-widest transition-all active:scale-95"
                 >
                   <Square size={8} fill="currentColor" /> Terminate
                 </button>
              </div>
           </div>
         {/if}
       <div class="h-10 shrink-0 w-full" bind:this={scrollAnchor}></div>
    </div>

    <!-- Scroll Button -->
    {#if showScrollButton}
      <button onclick={() => scrollToBottom(true)} class="absolute bottom-24 left-1/2 -translate-x-1/2 z-40 p-2 rounded-full bg-primary text-primary-foreground shadow-2xl hover:scale-110 active:scale-90 transition-all duration-300"><ChevronDown size={18} strokeWidth={3} /></button>
    {/if}

    <!-- Input Area (Ultra Lean) -->
    <div class="shrink-0 p-4 lg:p-6 bg-background border-t border-border/10">
       <div class="max-w-3xl mx-auto">
          <div class="bg-muted/[0.03] border border-border/30 shadow-inner rounded-3xl p-1.5 transition-all duration-500 focus-within:border-primary/20 group overflow-hidden">
             <div class="flex items-end gap-2 relative">
                <textarea 
                  bind:value={input} 
                  bind:this={inputTextarea}
                  onkeydown={handleKeydown}
                  oninput={autoResize}
                  placeholder="Collaborate..." 
                  class="flex-1 bg-transparent border-0 resize-none max-h-[150px] min-h-[40px] py-2 px-4 focus:ring-0 text-[13.5px] placeholder:text-muted-foreground/20 scrollbar-hide leading-[1.45] transition-all" 
                  rows="1"
                ></textarea>
                
                <button 
                  onclick={handleSendMessage} 
                  disabled={chatStore.isLoading || !input.trim()} 
                  class="p-2.5 bg-primary/90 text-primary-foreground rounded-2xl hover:bg-primary disabled:opacity-20 transition-all shrink-0 mb-1 shadow-md active:scale-90"
                >
                  {#if chatStore.isLoading}
                    <Loader2 size={18} class="animate-spin" />
                  {:else}
                    <Send size={18} strokeWidth={3} />
                  {/if}
                </button>
             </div>
             <div class="flex items-center justify-between px-4 pb-1 text-[7px] font-black uppercase tracking-[0.2em] text-muted-foreground/10 pointer-events-none">
                <span>Core Sync Active</span>
                <span>Enter to Submit</span>
             </div>
          </div>
       </div>
    </div>
  </main>
</div>

<style>
  .no-scrollbar::-webkit-scrollbar { display: none; }
</style>
