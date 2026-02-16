<script>
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { chatStore } from '../../stores/chatStore.svelte.js';
  import { tick, onMount } from 'svelte';
  import { 
    Send, X, Bot, User, FileCode, Loader2, Plus, MessageSquare, 
    RotateCcw, StopCircle, Cpu
  } from 'lucide-svelte';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';
  import ThinkingBlock from './ThinkingBlock.svelte';
  import MCPToolCall from '../MCPToolCall.svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();

  let input = $state('');
  let showHistory = $state(false);
  let messagesContainer;
  let textarea;

  onMount(async () => {
    await chatStore.loadConversations();
    if (chatStore.conversations.length > 0 && !chatStore.activeConversationId) {
      await chatStore.loadConversation(chatStore.conversations[0].id);
    }
  });

  async function handleSendMessage() {
    if (!input.trim() || chatStore.isLoading) return;
    const msg = input;
    input = '';
    await chatStore.sendMessage(msg);
    scrollToBottom();
  }

  function scrollToBottom() {
    if (messagesContainer) {
      setTimeout(() => {
        messagesContainer.scrollTop = messagesContainer.scrollHeight;
      }, 50);
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
    target.style.height = `${Math.min(target.scrollHeight, 100)}px`;
  }

  $effect(() => {
    if (chatStore.messages.length || chatStore.streamingContent) {
      scrollToBottom();
    }
  });
</script>

<div class="h-full flex flex-col bg-background border-l border-border/30 overflow-hidden relative selection:bg-primary/10">
  <!-- Ultra-Slim Header -->
  <div class="h-9 px-3 border-b border-border/30 flex items-center justify-between bg-muted/[0.02] shrink-0">
    <div class="flex items-center gap-2 min-w-0">
       <Cpu size={12} class="text-primary/60" />
       <div class="flex items-baseline gap-1.5 min-w-0">
          <span class="text-[9px] font-black text-foreground uppercase tracking-wider shrink-0">Sidekick</span>
          <span class="text-[7px] text-muted-foreground uppercase tracking-widest truncate opacity-50">{configStore.settings.ai.selectedModel?.split('/').pop() || 'IDLE'}</span>
       </div>
    </div>
    
    <div class="flex items-center gap-0.5 shrink-0 ml-2">
      <button onclick={() => chatStore.createNewConversation()} class="p-1 hover:bg-muted rounded text-muted-foreground/60 hover:text-foreground transition-colors" title="New Chat">
        <Plus size={13} />
      </button>
      <button onclick={() => showHistory = !showHistory} class={cn("p-1 rounded transition-all", showHistory ? "bg-primary/10 text-primary" : "text-muted-foreground/60 hover:bg-muted")} title="History">
        <MessageSquare size={13} />
      </button>
      <div class="w-[1px] h-2.5 bg-border/30 mx-1"></div>
      <button onclick={onClose} class="p-1 hover:bg-destructive/5 hover:text-destructive rounded text-muted-foreground/40 transition-all">
        <X size={14} />
      </button>
    </div>
  </div>

  <!-- History List (Compact) -->
  {#if showHistory}
    <div class="absolute inset-x-0 top-9 bottom-0 bg-background/98 backdrop-blur-2xl z-50 animate-in fade-in duration-200 overflow-y-auto border-b border-border/30 scrollbar-hide">
       <div class="p-1.5 space-y-0.5">
         {#each chatStore.conversations as conv}
            <button 
              class={cn("w-full text-left px-3 py-2 text-[10px] rounded-lg flex items-center justify-between group transition-all border border-transparent", chatStore.activeConversationId === conv.id ? "bg-primary/5 text-foreground border-primary/10" : "hover:bg-muted text-muted-foreground/70")}
              onclick={() => {chatStore.loadConversation(conv.id); showHistory = false;}}
            >
               <span class="truncate font-bold flex-1">{conv.title}</span>
               <span class="text-[7px] opacity-20 shrink-0 ml-2 font-mono uppercase">{new Date(conv.updated_at * 1000).toLocaleDateString()}</span>
            </button>
         {/each}
       </div>
    </div>
  {/if}

  <!-- Messages Area (Tighter spacing) -->
  <div 
    class="flex-1 overflow-y-auto p-3 space-y-4 scroll-smooth scrollbar-hide"
    bind:this={messagesContainer}
  >
    {#if chatStore.messages.length === 0 && !chatStore.isStreaming}
      <div class="h-full flex flex-col items-center justify-center text-center p-4 opacity-10 animate-in zoom-in-95 duration-1000">
        <Bot size={24} class="text-primary mb-2" />
        <p class="text-[8px] font-black uppercase tracking-[0.3em]">AI Core Standby</p>
      </div>
    {/if}

    {#each chatStore.messages as msg}
      <div class={cn(
        "flex gap-2 max-w-full animate-in fade-in duration-300",
        msg.role === 'user' ? "flex-row-reverse" : "flex-row"
      )}>
        <!-- No icons for ultra-minimalism, or keep them tiny -->
        <div class={cn(
          "w-5 h-5 rounded-md flex items-center justify-center shrink-0 border shadow-sm mt-0.5 transition-all",
          msg.role === 'user' ? "bg-primary text-primary-foreground border-primary/10" : "bg-muted/30 text-muted-foreground/60 border-border/20"
        )}>
          {#if msg.role === 'user'} <User size={9} /> {:else} <Bot size={9} /> {/if}
        </div>

        <div class={cn(
          "flex flex-col gap-0.5 min-w-0 flex-1",
          msg.role === 'user' ? "items-end" : "items-start"
        )}>
          <div class={cn(
            "relative rounded-xl px-3 py-1.5 text-[12.5px] leading-[1.4] transition-all duration-300 w-fit max-w-full",
            msg.role === 'user' 
              ? "bg-primary text-primary-foreground rounded-tr-none ml-2 shadow-lg shadow-primary/5" 
              : "bg-muted/20 border border-border/20 text-foreground/90 rounded-tl-none mr-2"
          )}>
            {#if msg.role === 'assistant'}
              {#if msg.reasoning}
                <ThinkingBlock content={msg.reasoning} isThinkingComplete={true} />
              {/if}
              
              {#if msg.segments}
                {#each msg.segments as seg}
                  {#if seg.type === 'text'}
                    <div class="w-full overflow-hidden mb-0.5 last:mb-0"><MarkdownRenderer content={seg.content} /></div>
                  {:else if seg.type === 'tool'}
                    <div class="my-1.5 max-w-full overflow-x-auto opacity-90"><MCPToolCall toolName={seg.name} arguments={seg.arguments} result={seg.result} error={seg.error} executing={false} /></div>
                  {/if}
                {/each}
              {:else}
                <div class="w-full overflow-hidden"><MarkdownRenderer content={msg.content} /></div>
              {/if}
            {:else}
              <div class="whitespace-pre-wrap break-words leading-[1.4] opacity-95">{msg.content}</div>
            {/if}
          </div>
          <span class="text-[6px] font-black text-muted-foreground/20 uppercase px-1 tracking-tighter">{msg.timestamp}</span>
        </div>
      </div>
    {/each}

    <!-- Streaming Content -->
    {#if chatStore.isStreaming || chatStore.isLoading}
       <div class="flex gap-2 animate-in fade-in">
          <div class="w-5 h-5 rounded-md bg-muted/20 border border-border/20 flex items-center justify-center shrink-0 mt-0.5">
             <Bot size={9} class="text-primary/60 animate-pulse" />
          </div>
          <div class="flex flex-col gap-1 flex-1 min-w-0 pr-2">
             {#if chatStore.streamingReasoning}
                <ThinkingBlock 
                  content={chatStore.streamingReasoning} 
                  isStreaming={true} 
                  isThinkingComplete={chatStore.isThinkingComplete} 
                  isToolExecuting={chatStore.isToolExecuting} 
                />
             {/if}

             {#each chatStore.streamingSegments as seg}
               {#if seg.type === 'text'}
                 <div class="bg-muted/10 border border-border/20 rounded-xl rounded-tl-none px-3 py-1.5 text-[12.5px] max-w-full overflow-hidden mb-0.5 last:mb-0">
                    <MarkdownRenderer content={seg.content} />
                 </div>
               {:else if seg.type === 'tool'}
                 <div class="max-w-full overflow-x-auto opacity-80 scale-95 origin-left">
                    <MCPToolCall toolName={seg.name} arguments={seg.arguments} result={seg.result} error={seg.error} executing={seg.executing} />
                 </div>
               {/if}
             {/each}
             
             {#if chatStore.currentSegmentText || (chatStore.isLoading && !chatStore.streamingReasoning && chatStore.streamingSegments.length === 0)}
               <div class="bg-muted/10 border border-border/20 rounded-xl rounded-tl-none px-3 py-1.5 text-[12.5px] max-w-full overflow-hidden">
                  {#if chatStore.currentSegmentText}
                    <MarkdownRenderer content={chatStore.currentSegmentText} />
                    <span class="inline-block w-1 h-3 bg-primary animate-pulse ml-1 align-middle rounded-full"></span>
                  {:else}
                    <div class="flex items-center gap-2 text-muted-foreground/40 py-0.5">
                       <Loader2 size={9} class="animate-spin" />
                       <span class="text-[9px] font-black uppercase tracking-[0.2em]">Processing</span>
                    </div>
                  {/if}
               </div>
             {/if}
             
             <button 
              onclick={() => chatStore.stopGeneration()}
              class="self-start flex items-center gap-1.5 px-2 py-0.5 rounded-md border border-destructive/10 bg-destructive/[0.02] hover:bg-destructive/10 text-destructive text-[7px] font-black uppercase tracking-widest transition-all mt-1"
             >
                <StopCircle size={9} />
                Terminate
             </button>
          </div>
       </div>
    {/if}
  </div>

  <!-- Ultra-Compact Input -->
  <div class="p-2 bg-background border-t border-border/20 shrink-0">
    <div class="relative flex flex-col gap-1 bg-muted/[0.03] border border-border/30 rounded-xl p-1 focus-within:border-primary/30 transition-all duration-500 shadow-inner">
      
      <textarea
        bind:this={textarea}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={autoResize}
        placeholder="Type..."
        class="bg-transparent border-0 resize-none max-h-[100px] min-h-[28px] py-1.5 px-2 focus:ring-0 text-[12px] placeholder:text-muted-foreground/20 scrollbar-hide leading-[1.4] transition-all"
        rows="1"
      ></textarea>
      
      <div class="flex items-center justify-between px-1.5 pb-0.5">
        <div class="min-w-0">
          {#if fsStore.activeFile}
            <div class="flex items-center gap-1 text-[7px] font-black text-primary/40 px-1 py-0.5 truncate max-w-[80px] uppercase tracking-tighter">
              {fsStore.activeFile.name}
            </div>
          {/if}
        </div>

        <button 
          onclick={handleSendMessage}
          disabled={chatStore.isLoading || !input.trim()}
          class="p-1.5 bg-primary/90 text-primary-foreground rounded-lg hover:bg-primary disabled:opacity-20 transition-all shadow-sm active:scale-90"
        >
          {#if chatStore.isLoading} <Loader2 size={11} class="animate-spin" /> {:else} <Send size={11} strokeWidth={3} /> {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .scrollbar-hide::-webkit-scrollbar { display: none; }
</style>
