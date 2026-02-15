<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { cn } from '../utils.js';
  import { 
    Terminal as TerminalIcon, 
    FileText, 
    AlertCircle, 
    Bug, 
    X, 
    Trash2, 
    Plus
  } from 'lucide-svelte';

  let { onClose } = $props();

  let container;
  let activeTab = $state('terminal');
  let command = $state('');
  let terminalOutput = $state(''); 
  let termId = 'term-1'; 
  let unlistenFn;

  const tabs = [
    { id: 'terminal', label: 'Terminal', icon: TerminalIcon, active: true },
    { id: 'output', label: 'Output', icon: FileText, active: false },
    { id: 'problems', label: 'Problems', icon: AlertCircle, active: false, badge: 0 },
    { id: 'debug', label: 'Debug Console', icon: Bug, active: false },
  ];

  const encoder = new TextEncoder();
  const decoder = new TextDecoder();

  onMount(async () => {
    try {
      await invoke('spawn_terminal', { id: termId, shell: null });
      unlistenFn = await listen('terminal-output', (event) => {
        const payload = event.payload;
        if (payload.id === termId && payload.data) {
           const text = decoder.decode(new Uint8Array(payload.data));
           terminalOutput += text;
           scrollToBottom();
        }
      });
    } catch (err) {
      terminalOutput += `\nError spawning terminal: ${err}\n`;
    }
  });

  onDestroy(() => {
    if (unlistenFn) unlistenFn();
  });

  async function sendCommand() {
    if (!command) return;
    try {
      const input = command + '\n';
      const data = Array.from(encoder.encode(input));
      await invoke('write_to_terminal', { id: termId, data });
      command = ''; 
    } catch (err) {
      console.error('Failed to write to terminal:', err);
    }
  }

  function handleKeydown(e) {
    if (e.key === 'Enter') {
        sendCommand();
    }
  }

  function scrollToBottom() {
    if (container) {
        requestAnimationFrame(() => {
            container.scrollTop = container.scrollHeight;
        });
    }
  }

  function clearTerminal() {
    terminalOutput = '';
  }
</script>

<div class="h-full flex flex-col bg-background border-t border-border">
  <!-- Terminal Header (Compact) -->
  <div class="h-7 min-h-[28px] flex items-center justify-between select-none bg-muted/10 border-b border-border px-2">
    <div class="flex items-center gap-4 h-full">
      {#each tabs as tab}
        <button
          class={cn(
            "flex items-center gap-1.5 text-[11px] h-full uppercase tracking-wide font-medium transition-all relative px-1",
            activeTab === tab.id 
              ? "text-foreground" 
              : "text-muted-foreground hover:text-foreground"
          )}
          onclick={() => activeTab = tab.id}
        >
          {tab.label}
          {#if tab.badge}
            <span class="ml-0.5 px-1 py-[1px] rounded-full bg-destructive text-destructive-foreground text-[9px] leading-none min-w-[14px] text-center">{tab.badge}</span>
          {/if}
          {#if activeTab === tab.id}
             <div class="absolute bottom-0 left-0 right-0 h-[2px] bg-primary"></div>
          {/if}
        </button>
      {/each}
    </div>
    
    <div class="flex items-center gap-1">
      <button class="p-1 hover:bg-muted/30 rounded text-muted-foreground hover:text-foreground transition-colors" title="New Terminal">
        <Plus size={12} />
      </button>
      <button class="p-1 hover:bg-muted/30 rounded text-muted-foreground hover:text-foreground transition-colors" title="Clear" onclick={clearTerminal}>
        <Trash2 size={12} />
      </button>
      <button class="p-1 hover:bg-muted/30 rounded text-muted-foreground hover:text-foreground transition-colors" title="Close" onclick={onClose}>
        <X size={12} />
      </button>
    </div>
  </div>

  <!-- Terminal Content -->
  <div class="flex-1 p-2 font-mono text-[12px] leading-relaxed overflow-y-auto scrollbar-thin scrollbar-thumb-muted bg-black/20" bind:this={container}>
    {#if activeTab === 'terminal'}
      <div class="whitespace-pre-wrap break-all text-foreground/80 font-mono tracking-tight selection:bg-primary/30">
        {terminalOutput}
      </div>
      
      <!-- Input Line -->
      <div class="flex items-center gap-2 mt-0.5 group">
        <span class="text-green-500 font-bold select-none text-[11px]">➜</span>
        <span class="text-blue-400 font-bold select-none text-[11px]">~</span>
        <input 
          type="text" 
          bind:value={command}
          onkeydown={handleKeydown}
          class="bg-transparent border-none outline-none text-foreground flex-1 caret-primary p-0 m-0 font-mono text-[12px]"
          spellcheck="false"
          autocomplete="off"
          autofocus
        />
      </div>
    {:else}
      <div class="flex flex-col items-center justify-center h-full text-muted-foreground/50">
        <span class="text-[10px] uppercase tracking-widest">No output</span>
      </div>
    {/if}
  </div>
</div>
