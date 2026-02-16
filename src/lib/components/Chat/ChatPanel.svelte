<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { tick, onMount, onDestroy } from 'svelte';
  import { 
    Send, X, Bot, User, Sparkles, FileCode, Loader2, Eraser, 
    Plus, MessageSquare, Trash2, RotateCcw, Copy, Check, 
    StopCircle, Terminal, Maximize2, Minimize2, Cpu
  } from 'lucide-svelte';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';
  import ThinkingBlock from './ThinkingBlock.svelte';
  import MCPToolCall from '../MCPToolCall.svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();

  let conversations = $state([]);
  let activeConversationId = $state(null);
  let messages = $state([]);
  let input = $state('');
  let isLoading = $state(false);
  let isStreaming = $state(false);
  let showHistory = $state(false);
  
  // Buffers
  let streamingContent = $state('');
  let streamingReasoning = $state('');
  let streamingSegments = $state([]);
  let currentSegmentText = $state('');
  let isToolExecuting = $state(false);
  let isThinkingComplete = $state(false);

  let messagesContainer;
  let textarea;
  let unlistenHandlers = [];

  const SYSTEM_PROMPT = "You are Karsa, an autonomous AI assistant integrated into the IDE. You have access to the file system and tools. Be precise, professional, and helpful. Always use markdown and code blocks for code.";

  onMount(async () => {
    await loadConversations();
    
    unlistenHandlers.push(
      await listen('ai-stream-chunk', (event) => {
        const chunk = typeof event.payload === 'string' ? event.payload : event.payload?.chunk || '';
        currentSegmentText += chunk;
        streamingContent += chunk;
        scrollToBottom();
      }),
      await listen('ai-stream-reasoning', (event) => {
        const reasoning = typeof event.payload === 'string' ? event.payload : '';
        streamingReasoning += reasoning;
        scrollToBottom();
      }),
      await listen('ai-reasoning-complete', (event) => {
        isThinkingComplete = true;
        scrollToBottom();
      }),
      await listen('ai-tool-call', (event) => {
        const toolCall = event.payload;
        if (currentSegmentText.trim()) {
          streamingSegments = [...streamingSegments, { type: 'text', content: currentSegmentText }];
          currentSegmentText = '';
        }
        streamingSegments = [...streamingSegments, {
          type: 'tool',
          name: toolCall.name,
          arguments: toolCall.arguments,
          executing: true,
          result: null,
          error: null
        }];
        isToolExecuting = true;
        scrollToBottom();
      }),
      await listen('ai-tool-result', (event) => {
        const payload = event.payload;
        streamingSegments = streamingSegments.map(seg => {
          if (seg.type === 'tool' && seg.executing && seg.name === payload.name) {
            return { ...seg, executing: false, result: payload.result, error: payload.error };
          }
          return seg;
        });
        isToolExecuting = false;
        scrollToBottom();
      }),
      await listen('ai-stream-done', async () => {
        if (isStreaming) {
          if (currentSegmentText.trim()) {
            streamingSegments = [...streamingSegments, { type: 'text', content: currentSegmentText }];
          }
          const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
          messages = [...messages, { 
            role: 'assistant', 
            content: streamingContent, 
            reasoning: streamingReasoning,
            segments: streamingSegments,
            timestamp 
          }];
          await saveMessage('assistant', streamingContent);
        }
        resetStreaming();
        scrollToBottom();
      })
    );
  });

  function resetStreaming() {
    streamingContent = '';
    streamingReasoning = '';
    streamingSegments = [];
    currentSegmentText = '';
    isStreaming = false;
    isLoading = false;
    isToolExecuting = false;
    isThinkingComplete = false;
  }

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
        ...m,
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
        model: configStore.settings.ai.selectedModel
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
    isStreaming = true;
    resetStreaming();
    isStreaming = true;
    isLoading = true;
    
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    await saveMessage('user', userMessage);
    
    await tick();
    scrollToBottom();

    try {
      const cwd = fsStore.activeWorkspace || '';
      const mcpPrompt = await invoke('mcp_get_system_prompt', { mode: 'code', cwd: cwd || 'no-workspace' });
      
      let contextContent = '';
      if (fsStore.activeFile) {
        contextContent = `\n\n[Current File: ${fsStore.activeFile.path}]\n\`\`\`\n${fsStore.activeFileContent}\n\`\`\``;
      }

      const msgs = [
        { role: 'system', content: mcpPrompt },
        ...messages.slice(-6).map(m => ({ role: m.role, content: m.content })),
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
      messages = [...messages, { role: 'assistant', content: `Error: ${e}`, timestamp }];
      resetStreaming();
    }
  }

  async function stopGeneration() {
    await invoke('cancel_ai_stream').catch(() => {});
    resetStreaming();
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
      sendMessage();
    }
  }

  function autoResize(e) {
    const target = e.target;
    target.style.height = 'auto';
    target.style.height = `${Math.min(target.scrollHeight, 180)}px`;
  }
</script>

<div class="h-full flex flex-col bg-background/95 backdrop-blur-xl border-l border-border/50">
  <!-- Header: Enhanced to match AgentView -->
  <div class="h-14 min-h-[56px] px-4 border-b border-border/40 flex items-center justify-between bg-muted/5 shrink-0 shadow-sm">
    <div class="flex items-center gap-3">
       <div class="w-8 h-8 rounded-xl bg-primary/10 flex items-center justify-center text-primary border border-primary/20">
          <Cpu size={18} />
       </div>
       <div class="flex flex-col">
          <span class="text-[11px] font-bold text-foreground">Sidekick Agent</span>
          <div class="flex items-center gap-1.5">
            <span class="w-1.5 h-1.5 rounded-full bg-green-500"></span>
            <span class="text-[9px] text-muted-foreground uppercase tracking-widest">{configStore.settings.ai.selectedModel?.split('/').pop() || 'Pro Mode'}</span>
          </div>
       </div>
    </div>
    
    <div class="flex items-center gap-1">
      <button onclick={createNewConversation} class="p-1.5 hover:bg-muted rounded-lg text-muted-foreground hover:text-foreground transition-all">
        <Plus size={16} />
      </button>
      <button onclick={() => showHistory = !showHistory} class={cn("p-1.5 rounded-lg transition-all", showHistory ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-muted")}>
        <MessageSquare size={16} />
      </button>
      <div class="w-[1px] h-4 bg-border/50 mx-1"></div>
      <button onclick={onClose} class="p-1.5 hover:bg-destructive/10 hover:text-destructive rounded-lg text-muted-foreground transition-all">
        <X size={18} />
      </button>
    </div>
  </div>

  {#if showHistory}
    <div class="border-b border-border/40 bg-muted/10 max-h-64 overflow-y-auto animate-in slide-in-from-top duration-300">
       {#each conversations as conv}
          <button 
            class="w-full text-left px-4 py-3 text-xs hover:bg-muted/50 border-b border-border/20 last:border-0 flex items-center justify-between group transition-colors"
            onclick={() => {loadConversation(conv.id); showHistory = false;}}
          >
             <div class="flex items-center gap-3 min-w-0">
                <div class="p-1.5 rounded-md bg-muted/50 group-hover:bg-primary/10 group-hover:text-primary transition-colors">
                   <MessageSquare size={12} />
                </div>
                <span class="truncate font-medium">{conv.title}</span>
             </div>
             <span class="text-[10px] opacity-30 shrink-0">{new Date(conv.updated_at * 1000).toLocaleDateString()}</span>
          </button>
       {/each}
    </div>
  {/if}

  <!-- Messages Area -->
  <div 
    class="flex-1 overflow-y-auto p-4 space-y-8 scroll-smooth no-scrollbar"
    bind:this={messagesContainer}
  >
    {#if messages.length === 0 && !isStreaming}
      <div class="h-full flex flex-col items-center justify-center text-center p-8 opacity-40 animate-in zoom-in-95 duration-700">
        <div class="w-16 h-16 rounded-3xl bg-primary/5 border border-primary/10 flex items-center justify-center mb-6 shadow-inner animate-float">
          <Bot size={32} class="text-primary" />
        </div>
        <h3 class="text-sm font-bold text-foreground mb-2">Intelligent Sidebar</h3>
        <p class="text-[11px] text-muted-foreground max-w-[200px] leading-relaxed italic">
          I have full access to your code and tools here.
        </p>
      </div>
    {/if}

    {#each messages as msg, i}
      <div class={cn(
        "group flex gap-3.5 max-w-full animate-in fade-in slide-in-from-bottom-2 duration-500",
        msg.role === 'user' ? "flex-row-reverse" : "flex-row"
      )}>
        <div class={cn(
          "w-8 h-8 rounded-xl flex items-center justify-center shrink-0 border shadow-sm transition-transform duration-300 group-hover:scale-105",
          msg.role === 'user' ? "bg-primary text-primary-foreground border-primary/20" : "bg-card text-foreground border-border/60"
        )}>
          {#if msg.role === 'user'} <User size={14} /> {:else} <Bot size={14} /> {/if}
        </div>

        <div class={cn(
          "flex flex-col gap-1.5",
          msg.role === 'user' ? "items-end max-w-[85%]" : "items-start max-w-[92%]"
        )}>
          <div class={cn(
            "relative rounded-2xl px-4 py-3 text-[13px] leading-relaxed shadow-sm transition-all duration-300",
            msg.role === 'user' 
              ? "bg-primary text-primary-foreground rounded-tr-none" 
              : "bg-muted/40 border border-border/40 text-foreground rounded-tl-none"
          )}>
            {#if msg.role === 'assistant'}
              {#if msg.reasoning}
                <ThinkingBlock content={msg.reasoning} isThinkingComplete={true} />
              {/if}
              
              {#if msg.segments}
                {#each msg.segments as seg}
                  {#if seg.type === 'text'}
                    <MarkdownRenderer content={seg.content} />
                  {:else if seg.type === 'tool'}
                    <div class="my-3"><MCPToolCall toolName={seg.name} arguments={seg.arguments} result={seg.result} error={seg.error} executing={false} /></div>
                  {/if}
                {/each}
              {:else}
                <MarkdownRenderer content={msg.content} />
              {/if}
            {:else}
              <div class="whitespace-pre-wrap break-words">{msg.content}</div>
            {/if}

            <!-- Quick Actions -->
            <div class={cn(
              "absolute -bottom-6 opacity-0 group-hover:opacity-100 transition-all flex items-center gap-2",
              msg.role === 'user' ? "right-0" : "left-0"
            )}>
               <button 
                class="p-1 text-[10px] text-muted-foreground hover:text-primary transition-colors flex items-center gap-1"
                onclick={() => navigator.clipboard.writeText(msg.content)}
               >
                 <Copy size={10} /> Copy
               </button>
               {#if msg.role === 'assistant'}
                 <button 
                  class="p-1 text-[10px] text-muted-foreground hover:text-primary transition-colors flex items-center gap-1"
                  onclick={() => {input = messages[i-1]?.content || ''; sendMessage();}}
                 >
                   <RotateCcw size={10} /> Retry
                 </button>
               {/if}
            </div>
          </div>
          <span class="text-[9px] text-muted-foreground/40 px-1">{msg.timestamp}</span>
        </div>
      </div>
    {/each}

    <!-- Streaming Content -->
    {#if isStreaming || isLoading}
       <div class="flex gap-3.5 animate-in fade-in">
          <div class="w-8 h-8 rounded-xl bg-card border border-border/60 flex items-center justify-center shrink-0 shadow-sm">
             <Bot size={14} class="text-primary animate-pulse" />
          </div>
          <div class="flex flex-col gap-2 max-w-[92%] w-full">
             {#if streamingReasoning}
                <ThinkingBlock content={streamingReasoning} isStreaming={true} isThinkingComplete={isThinkingComplete} isToolExecuting={isToolExecuting} />
             {/if}

             {#each streamingSegments as seg}
               {#if seg.type === 'text'}
                 <div class="bg-card border border-border/40 rounded-2xl rounded-tl-none px-4 py-3 text-[13px] shadow-sm">
                    <MarkdownRenderer content={seg.content} />
                 </div>
               {:else if seg.type === 'tool'}
                 <MCPToolCall toolName={seg.name} arguments={seg.arguments} result={seg.result} error={seg.error} executing={seg.executing} />
               {/if}
             {/each}
             
             {#if currentSegmentText || (isLoading && !streamingReasoning && streamingSegments.length === 0)}
               <div class="bg-card border border-border/40 rounded-2xl rounded-tl-none px-4 py-3 text-[13px] shadow-sm">
                  {#if currentSegmentText}
                    <MarkdownRenderer content={currentSegmentText} />
                    <span class="inline-block w-1.5 h-3.5 bg-primary animate-pulse ml-1 align-middle rounded-sm"></span>
                  {:else}
                    <div class="flex items-center gap-2 text-muted-foreground/60">
                       <Loader2 size={12} class="animate-spin" />
                       <span class="text-[11px]">Consulting models...</span>
                    </div>
                  {/if}
               </div>
             {/if}
             
             <button 
              onclick={stopGeneration}
              class="self-start flex items-center gap-2 px-3 py-1.5 rounded-full border border-border bg-background hover:bg-muted text-[10px] text-muted-foreground transition-all active:scale-95"
             >
                <StopCircle size={12} class="text-destructive" />
                Stop Generation
             </button>
          </div>
       </div>
    {/if}
  </div>

  <!-- Enhanced Input Area -->
  <div class="p-4 bg-background border-t border-border/40 shrink-0">
    <div class="relative flex flex-col gap-2 bg-muted/20 border border-border/40 rounded-2xl p-2 focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary/40 transition-all duration-300 shadow-inner">
      
      <textarea
        bind:this={textarea}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={autoResize}
        placeholder="Message Sidekick Agent..."
        class="bg-transparent border-0 resize-none max-h-[180px] min-h-[44px] py-2 px-3 focus:ring-0 text-[13px] placeholder:text-muted-foreground/30 scrollbar-hide leading-relaxed"
        rows="1"
      ></textarea>
      
      <div class="flex items-center justify-between px-2 pb-1">
        <div class="flex items-center gap-2">
          {#if fsStore.activeFile}
            <div class="flex items-center gap-1.5 text-[9px] font-bold text-primary/70 bg-primary/10 px-2 py-0.5 rounded-md border border-primary/20">
              <FileCode size={10} /> {fsStore.activeFile.name}
            </div>
          {/if}
          <div class="text-[9px] text-muted-foreground/40 font-mono">Enter to send</div>
        </div>

        <button 
          onclick={sendMessage}
          disabled={isLoading || !input.trim()}
          class="p-2 bg-primary text-primary-foreground rounded-xl hover:bg-primary/90 disabled:opacity-30 transition-all duration-300 shadow-lg shadow-primary/20 active:scale-95"
        >
          {#if isLoading} <Loader2 size={16} class="animate-spin" /> {:else} <Send size={16} strokeWidth={2.5} /> {/if}
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .no-scrollbar::-webkit-scrollbar { display: none; }
  .no-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
</style>
