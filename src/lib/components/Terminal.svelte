<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { cn } from '../utils.js';
  import { Terminal as TerminalIcon, X, Plus } from 'lucide-svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from 'xterm-addon-fit';
  import { WebLinksAddon } from 'xterm-addon-web-links';
  import 'xterm/css/xterm.css';

  let { onClose } = $props();

  let terminalContainer;
  let xterm;
  let fitAddon;
  let termId = 'term-1';
  let unlistenFn;

  onMount(async () => {
    // Initialize xterm.js
    xterm = new Terminal({
      cursorBlink: true,
      fontSize: 14,
      fontFamily: 'JetBrains Mono, Fira Code, monospace',
      theme: {
        background: '#1e1e1e',
        foreground: '#d4d4d4',
        cursor: '#d4d4d4',
        black: '#000000',
        red: '#cd3131',
        green: '#0dbc79',
        yellow: '#e5e510',
        blue: '#2472c8',
        magenta: '#bc3fbc',
        cyan: '#11a8cd',
        white: '#e5e5e5',
        brightBlack: '#666666',
        brightRed: '#f14c4c',
        brightGreen: '#23d18b',
        brightYellow: '#f5f543',
        brightBlue: '#3b8eea',
        brightMagenta: '#d670d6',
        brightCyan: '#29b8db',
        brightWhite: '#e5e5e5'
      },
      scrollback: 1000,
      convertEol: true
    });

    fitAddon = new FitAddon();
    xterm.loadAddon(fitAddon);
    xterm.loadAddon(new WebLinksAddon());

    xterm.open(terminalContainer);
    fitAddon.fit();

    // Handle input
    xterm.onData(async (data) => {
      try {
        const bytes = new TextEncoder().encode(data);
        await invoke('write_to_terminal', { 
          id: termId, 
          data: Array.from(bytes) 
        });
      } catch (err) {
        console.error('Failed to write to terminal:', err);
      }
    });

    // Spawn terminal
    try {
      await invoke('spawn_terminal', { id: termId, shell: null });
      
      // Listen for output
      unlistenFn = await listen('terminal-output', (event) => {
        const data = event.payload;
        const text = new TextDecoder().decode(new Uint8Array(data));
        xterm.write(text);
      });
    } catch (err) {
      xterm.write(`\r\n\x1b[31mError spawning terminal: ${err}\x1b[0m\r\n`);
    }

    // Fit on resize
    const resizeObserver = new ResizeObserver(() => {
      fitAddon?.fit();
    });
    resizeObserver.observe(terminalContainer);

    return () => {
      resizeObserver.disconnect();
    };
  });

  onDestroy(() => {
    if (unlistenFn) unlistenFn();
    if (xterm) xterm.dispose();
  });
</script>

<div class="terminal-wrapper">
  <div class="terminal-header">
    <div class="tabs">
      <div class="tab active">
        <TerminalIcon size={14} />
        <span>Terminal</span>
      </div>
    </div>
    <button class="close-btn" onclick={onClose}>
      <X size={14} />
    </button>
  </div>
  
  <div class="terminal-content" bind:this={terminalContainer}></div>
</div>

<style>
  .terminal-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
  }

  .terminal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.25rem 0.5rem;
    background: #252526;
    border-bottom: 1px solid #3e3e42;
  }

  .tabs {
    display: flex;
    gap: 0.25rem;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.25rem 0.75rem;
    font-size: 0.75rem;
    color: #cccccc;
    background: transparent;
    border: none;
    cursor: pointer;
    border-radius: 3px;
  }

  .tab.active {
    background: #1e1e1e;
  }

  .close-btn {
    display: flex;
    align-items: center;
    padding: 0.25rem;
    background: none;
    border: none;
    color: #cccccc;
    cursor: pointer;
    border-radius: 3px;
  }

  .close-btn:hover {
    background: #3e3e42;
  }

  .terminal-content {
    flex: 1;
    padding: 0.5rem;
    overflow: hidden;
  }

  :global(.xterm) {
    height: 100%;
  }

  :global(.xterm-viewport) {
    overflow-y: auto !important;
  }
</style>


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
