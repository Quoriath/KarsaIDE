<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { onMount, tick } from 'svelte';
  import { Send, X, Bot, User, Sparkles, FileCode, Loader2 } from 'lucide-svelte';
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
    // Reset textarea height
    if (textarea) textarea.style.height = 'auto';
    
    isLoading = true;
    const timestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

    messages = [...messages, { role: 'user', content: userMessage, timestamp }];
    await tick();
    scrollToBottom();

    try {
      // Mock or Real Invoke
      // For now, let's assume real invoke works, if not we catch error
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

      const response = await invoke('send_chat_completion', {
        messages: msgs,
        config
      });

      const aiTimestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      messages = [...messages, { role: 'assistant', content: response, timestamp: aiTimestamp }];
    } catch (e) {
      console.error(e);
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
    target.style.height = `${Math.min(target.scrollHeight, 200)}px`;
  }
</script>

<aside class="w-[400px] h-full bg-background border-l border-border flex flex-col shadow-xl z-10 transition-all duration-300">
  <!-- Header -->
  <div class="h-14 px-4 border-b border-border flex items-center justify-between bg-card/50 backdrop-blur-sm">
    <div class="flex items-center gap-2.5">
      <div class="p-1.5 bg-primary/10 rounded-md text-primary">
        <Sparkles size={18} />
      </div>
      <div>
        <h2 class="text-sm font-semibold leading-none">AI Assistant</h2>
        <span class="text-[10px] text-muted-foreground">Powered by Karsa</span>
      </div>
    </div>
    <button onclick={onClose} class="p-2 hover:bg-muted rounded-md text-muted-foreground hover:text-foreground transition-colors">
      <X size={16} />
    </button>
  </div>

  <!-- Context Bar -->
  {#if fsStore.activeFile}
    <div class="px-4 py-2 bg-muted/30 border-b border-border flex items-center gap-2 text-xs text-muted-foreground">
      <FileCode size={12} />
      <span class="truncate font-mono">Context: {fsStore.activeFile.name}</span>
    </div>
  {/if}

  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-6 scroll-smooth" bind:this={messagesContainer}>
    {#if messages.length === 0}
      <div class="h-full flex flex-col items-center justify-center text-center p-8 space-y-4 opacity-50">
        <div class="p-4 bg-muted rounded-full">
           <Bot size={32} />
        </div>
        <div>
          <h3 class="font-medium text-foreground">How can I help?</h3>
          <p class="text-sm text-muted-foreground mt-1">Ask me to explain, refactor, or write code.</p>
        </div>
      </div>
    {/if}

    {#each messages as msg}
      <div class={cn("flex gap-3 max-w-full animate-in fade-in slide-in-from-bottom-2 duration-300", msg.role === 'user' ? "flex-row-reverse" : "")}>
        
        <!-- Avatar -->
        <div class={cn("w-8 h-8 rounded-full flex items-center justify-center shrink-0 border border-border", 
          msg.role === 'user' ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground")}>
          {#if msg.role === 'user'}
            <User size={14} />
          {:else}
            <Bot size={14} />
          {/if}
        </div>

        <!-- Bubble -->
        <div class={cn("flex flex-col max-w-[85%]", msg.role === 'user' ? "items-end" : "items-start")}>
          <div class={cn("rounded-2xl px-4 py-3 text-sm shadow-sm", 
            msg.role === 'user' 
              ? "bg-primary text-primary-foreground rounded-tr-sm" 
              : "bg-card border border-border text-card-foreground rounded-tl-sm")}>
            
            {#if msg.role === 'assistant'}
              <MarkdownRenderer content={msg.content} />
            {:else}
              <div class="whitespace-pre-wrap break-words">{msg.content}</div>
            {/if}
          </div>
          
          <span class="text-[10px] text-muted-foreground mt-1 px-1">
            {msg.timestamp}
          </span>
        </div>
      </div>
    {/each}

    {#if isLoading}
      <div class="flex gap-3 max-w-full animate-pulse">
        <div class="w-8 h-8 rounded-full bg-muted flex items-center justify-center shrink-0">
          <Bot size={14} class="text-muted-foreground" />
        </div>
        <div class="bg-card border border-border rounded-2xl rounded-tl-sm px-4 py-3 h-10 w-16 flex items-center justify-center">
           <div class="flex gap-1">
             <span class="w-1.5 h-1.5 bg-primary/50 rounded-full animate-bounce"></span>
             <span class="w-1.5 h-1.5 bg-primary/50 rounded-full animate-bounce delay-75"></span>
             <span class="w-1.5 h-1.5 bg-primary/50 rounded-full animate-bounce delay-150"></span>
           </div>
        </div>
      </div>
    {/if}
  </div>

  <!-- Input Area -->
  <div class="p-4 bg-background border-t border-border">
    <div class="relative flex items-end gap-2 bg-muted/50 border border-border rounded-xl p-2 focus-within:ring-2 focus-within:ring-primary/20 focus-within:border-primary transition-all">
      <textarea
        bind:this={textarea}
        bind:value={input}
        onkeydown={handleKeydown}
        oninput={autoResize}
        placeholder="Type a message..."
        class="flex-1 bg-transparent border-0 resize-none max-h-[200px] min-h-[24px] py-2 px-2 focus:ring-0 text-sm placeholder:text-muted-foreground scrollbar-hide"
        rows="1"
      ></textarea>
      
      <button 
        onclick={sendMessage}
        disabled={isLoading || !input.trim()}
        class="p-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors shrink-0 mb-0.5"
      >
        {#if isLoading}
          <Loader2 size={16} class="animate-spin" />
        {:else}
          <Send size={16} />
        {/if}
      </button>
    </div>
    <div class="text-[10px] text-center text-muted-foreground mt-2">
      Karsa can make mistakes. Verify code before use.
    </div>
  </div>
</aside>
