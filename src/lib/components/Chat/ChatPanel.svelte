<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { configStore } from '../../stores/config.svelte.js';
  import { tick, onMount } from 'svelte';
  import { 
    Send, X, Bot, User, Sparkles, FileCode, Loader2, Eraser, 
    Settings, Download, Upload, Sliders 
  } from 'lucide-svelte';
  import MarkdownRenderer from '../MarkdownRenderer.svelte';
  import ModelSelector from '../ModelSelector.svelte';
  import { cn } from '../../utils.js';

  let { onClose } = $props();

  let messages = $state([]);
  let input = $state('');
  let isLoading = $state(false);
  let showSettings = $state(false);
  let messagesContainer;
  let textarea;
  
  // Settings
  let temperature = $state(0.7);
  let maxTokens = $state(2000);
  let selectedModel = $state(configStore.settings.ai.selectedModel || 'gemini-1.5-pro');

  const SYSTEM_PROMPT = "You are Karsa, an autonomous coding agent. You have access to the user's open file. Answer concisely and provide code blocks when relevant. Use markdown.";

  onMount(() => {
    // Load last session specific to sidebar
    const saved = localStorage.getItem('karsa-sidebar-chat');
    if (saved) {
      try { messages = JSON.parse(saved); } catch (e) {}
    }
  });

  function saveHistory() {
    localStorage.setItem('karsa-sidebar-chat', JSON.stringify(messages.slice(-50)));
  }

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
      // Build proper config with all required fields (map camelCase to snake_case)
      const config = {
        provider: configStore.settings.ai.provider,
        api_key: configStore.settings.ai.apiKey || null,
        base_url: configStore.settings.ai.baseUrl || 'https://api.kilo.ai/api/gateway/chat/completions',
        model_name: selectedModel,
        custom_models: configStore.settings.ai.models || []
      };
      
      let contextContent = '';
      if (fsStore.activeFile) {
        contextContent = `\n\nCurrent file: ${fsStore.activeFile.name}\n\`\`\`\n${fsStore.activeFileContent}\n\`\`\``;
      }

      const msgs = [
        { role: 'system', content: SYSTEM_PROMPT },
        ...messages.slice(-6).map(m => ({ role: m.role, content: m.content })),
        { role: 'user', content: `${userMessage}${contextContent}` }
      ];

      const response = await invoke('send_chat_completion', { messages: msgs, config });

      const aiTimestamp = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      messages = [...messages, { role: 'assistant', content: response, timestamp: aiTimestamp }];
      saveHistory();
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

  function exportChat() {
    const data = JSON.stringify(messages, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `karsa-sidebar-chat-${Date.now()}.json`;
    a.click();
  }

  function importChat(event) {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          messages = JSON.parse(e.target.result);
          saveHistory();
        } catch (err) {
          console.error(err);
        }
      };
      reader.readAsText(file);
    }
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
      <button class={cn("p-1 hover:bg-muted rounded transition-colors", showSettings ? "text-primary bg-primary/10" : "text-muted-foreground")} title="Settings" onclick={() => showSettings = !showSettings}>
        <Sliders size={14} />
      </button>
      <label class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors cursor-pointer" title="Import Chat">
        <Upload size={14} />
        <input type="file" accept=".json" onchange={importChat} class="hidden" />
      </label>
      <button class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors" title="Export Chat" onclick={exportChat}>
        <Download size={14} />
      </button>
      <button class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors" title="Clear Chat" onclick={() => { messages = []; saveHistory(); }}>
        <Eraser size={14} />
      </button>
      <div class="w-[1px] h-3 bg-border mx-1"></div>
      <button onclick={onClose} class="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors">
        <X size={14} />
      </button>
    </div>
  </div>

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
  <div class="p-3 bg-background border-t border-border shrink-0">
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
