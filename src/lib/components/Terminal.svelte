<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { themeStore } from '../stores/theme.svelte.js';
  import { cn } from '../utils.js';
  import { Terminal as TerminalIcon, X, Minimize2, Trash2 } from 'lucide-svelte';

  let container;
  let activeTab = $state('terminal');
  let history = $state([]);
  let command = $state('');
  let terminalId = 'main-terminal';
  let unlisten;

  const tabs = [
    { id: 'terminal', label: 'Terminal', active: true },
    { id: 'output', label: 'Output', active: false },
    { id: 'problems', label: 'Problems', active: false },
    { id: 'debug', label: 'Debug Console', active: false },
  ];

  onMount(async () => {
    try {
      // Spawn terminal
      await invoke('spawn_terminal', { 
        id: terminalId, 
        shell: null // Use default shell
      });

      // Listen for terminal output
      unlisten = await listen('terminal-output', (event) => {
        const bytes = event.payload;
        const text = new TextDecoder().decode(new Uint8Array(bytes));
        history = [...history, { type: 'output', text }];
        scrollToBottom();
      });
    } catch (e) {
      console.error('Failed to spawn terminal:', e);
      history = [{ type: 'error', text: `Failed to start terminal: ${e}` }];
    }
  });

  onDestroy(() => {
    if (unlisten) unlisten();
  });

  async function runCommand() {
    if (!command.trim()) return;
    
    const cmd = command.trim() + '\n';
    history = [...history, { type: 'cmd', text: `> ${cmd.trim()}` }];
    
    try {
      const encoder = new TextEncoder();
      const data = encoder.encode(cmd);
      await invoke('write_to_terminal', { 
        id: terminalId, 
        data: Array.from(data)
      });
    } catch (e) {
      history = [...history, { type: 'error', text: `Error: ${e}` }];
    }

    command = '';
    scrollToBottom();
  }

  function handleKeydown(e) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      runCommand();
    }
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (container) container.scrollTop = container.scrollHeight;
    }, 10);
  }

  function clearTerminal() {
    history = [];
  }
</script>

<div class="h-full flex flex-col bg-background border-t border-border">
  <!-- Terminal Header/Tabs -->
  <div class="h-9 px-4 border-b border-border flex items-center justify-between select-none bg-muted/20">
    <div class="flex items-center gap-4">
      {#each tabs as tab}
        <button
          class={cn(
            "text-xs font-medium border-b-2 px-1 py-2 transition-colors",
            activeTab === tab.id 
              ? "border-primary text-foreground" 
              : "border-transparent text-muted-foreground hover:text-foreground"
          )}
          onclick={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </div>
    <div class="flex items-center gap-2 text-muted-foreground">
      <button class="p-1 hover:text-foreground hover:bg-muted/50 rounded" title="Clear Terminal" onclick={clearTerminal}>
        <Trash2 size={14} />
      </button>
      <button class="p-1 hover:text-foreground hover:bg-muted/50 rounded" title="Minimize">
        <Minimize2 size={14} />
      </button>
      <button class="p-1 hover:text-foreground hover:bg-muted/50 rounded" title="Close Panel">
        <X size={14} />
      </button>
    </div>
  </div>

  <!-- Terminal Content -->
  <div class="flex-1 p-2 font-mono text-xs overflow-y-auto scrollbar-thin scrollbar-thumb-muted" bind:this={container}>
    {#each history as line}
      <div class={cn(
        line.type === 'cmd' ? 'text-primary mt-1 font-bold' : 
        line.type === 'error' ? 'text-red-500' : 
        'text-foreground'
      )}>
        {line.text}
      </div>
    {/each}
    
    <div class="flex items-start gap-2 mt-1 group">
      <span class="text-green-500 font-bold">➜</span>
      <input 
        type="text" 
        bind:value={command}
        onkeydown={handleKeydown}
        class="bg-transparent border-none outline-none text-foreground flex-1 caret-primary font-mono"
        spellcheck="false"
        autocomplete="off"
        autofocus
        placeholder="Type command..."
      />
    </div>
  </div>
</div>
