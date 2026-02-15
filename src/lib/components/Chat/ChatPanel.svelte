<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { tick } from 'svelte';
  import { Send, X, Bot, User, Sparkles, FileCode, Loader2, Eraser } from 'lucide-svelte';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();

  let messages = $state([]);
  let input = $state('');
  let isLoading = $state(false);
  let messagesContainer;
  let textarea;

  const SYSTEM_PROMPT = "You are Karsa, an autonomous coding agent. You have access to the user's open file. Answer concisely and provide code blocks when relevant. Use markdown.";

  async function sendMessage() {
    if (!input.trim() || isLoading) return;

    const userMessage = input.trim();
    input = '';
    if (textarea) textarea.style.height = 'auto';
    
    isLoading = true;
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    await tick();
    scrollToBottom();

    try {
      const config = await invoke('get_ai_config').catch(() => ({}));
      
      let contextContent = '';
      if (fsStore.activeFile) {
        contextContent = `\n\nCurrent file: ${fsStore.activeFile.name}\n\`\`\`\n${fsStore.activeFileContent}\n\`\`\``;
      }

      const msgs = [
        { role: 'system', content: SYSTEM_PROMPT },
        ...messages.slice(-4).map(m => ({ role: m.role, content: m.content })),
        { role: 'user', content: `${userMessage}${contextContent}` }
      ];

      const response = await invoke('send_chat_completion', { messages: msgs, config });

      const aiTimestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      messages = [...messages, { role: 'assistant', content: response, timestamp: aiTimestamp }];
    } catch (e) {
      const errTimestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      messages = [...messages, { role: 'assistant', content: `**Error**: ${e}`, timestamp: errTimestamp }];
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
  <div class="h-9 min-h-[36px] px-3 border-b border-border flex items-center justify-between bg-muted/10">
    <span class="text-xs font-semibold text-muted-foreground uppercase tracking-wider">AI Assistant</span>
    <div class="flex items-center gap-1">
      <button class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors" title="Clear Chat" onclick={() => messages = []}>
        <Eraser size={14} />
      </button>
      <button onclick={onClose} class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors">
        <X size={14} />
      </button>
    </div>
  </div>

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
      <div class={cn("flex flex-col gap-1 max-w-full animate-in fade-in slide-in-from-bottom-2 duration-300", msg.role === 'user' ? "items-end" : "items-start")}>
        <div class={cn("rounded-2xl px-3 py-2 text-sm shadow-sm max-w-[90%]", 
          msg.role === 'user' 
            ? "bg-primary text-primary-foreground rounded-tr-sm" 
            : "bg-muted/30 border border-border text-foreground rounded-tl-sm")}>
          
          {#if msg.role === 'assistant'}
            <MarkdownRenderer content={msg.content} />
          {:else}
            <div class="whitespace-pre-wrap break-words">{msg.content}</div>
          {/if}
        </div>
        
        <span class="text-[10px] text-muted-foreground opacity-50 px-1">
          {msg.role === 'user' ? 'You' : 'Karsa'} • {msg.timestamp}
        </span>
      </div>
    {/each}

    {#if isLoading}
      <div class="flex items-center gap-2 text-xs text-muted-foreground px-2">
        <Bot size={12} />
        <span class="animate-pulse">Thinking...</span>
      </div>
    {/if}
  </div>

  <!-- Input Area -->
  <div class="p-3 bg-background border-t border-border">
    <div class="relative flex items-end gap-2 bg-muted/30 border border-border rounded-xl p-1.5 focus-within:ring-1 focus-within:ring-primary/30 focus-within:border-primary/50 transition-all shadow-sm">
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
        class="p-1.5 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors shrink-0 mb-0.5"
      >
        {#if isLoading}
          <Loader2 size={14} class="animate-spin" />
        {:else}
          <Send size={14} />
        {/if}
      </button>
    </div>
    {#if fsStore.activeFile}
      <div class="flex items-center gap-1.5 mt-2 px-1">
         <FileCode size={10} class="text-muted-foreground" />
         <span class="text-[10px] text-muted-foreground truncate max-w-[200px]">{fsStore.activeFile.name}</span>
      </div>
    {/if}
  </div>
</div>
