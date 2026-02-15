<script>
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { cn } from '../utils.js';
  import { Terminal as TerminalIcon, X, Minimize2, Trash2, Plus, ChevronDown } from 'lucide-svelte';

  let container;
  let activeTab = $state('terminal-1');
  let terminals = $state([
    { id: 'terminal-1', label: 'Terminal 1', history: [], command: '' }
  ]);
  let unlisteners = [];
  let nextTerminalId = 2;

  const tabs = [
    { id: 'output', label: 'Output', icon: 'file-text' },
    { id: 'problems', label: 'Problems', icon: 'alert-circle' },
    { id: 'debug', label: 'Debug Console', icon: 'bug' },
  ];

  onMount(async () => {
    // Spawn first terminal
    await spawnTerminal('terminal-1');
  });

  onDestroy(() => {
    unlisteners.forEach(fn => fn && fn());
  });

  async function spawnTerminal(id) {
    try {
      await invoke('spawn_terminal', { id, shell: null });
      
      const unlisten = await listen('terminal-output', (event) => {
        const bytes = event.payload;
        const text = new TextDecoder().decode(new Uint8Array(bytes));
        
        const terminal = terminals.find(t => t.id === id);
        if (terminal) {
          terminal.history = [...terminal.history, { type: 'output', text }];
          scrollToBottom();
        }
      });
      
      unlisteners.push(unlisten);
    } catch (e) {
      console.error('Failed to spawn terminal:', e);
      const terminal = terminals.find(t => t.id === id);
      if (terminal) {
        terminal.history = [{ type: 'error', text: `Failed to start terminal: ${e}` }];
      }
    }
  }

  async function addTerminal() {
    const id = `terminal-${nextTerminalId++}`;
    terminals = [...terminals, { id, label: `Terminal ${nextTerminalId - 1}`, history: [], command: '' }];
    activeTab = id;
    await spawnTerminal(id);
  }

  function closeTerminal(id) {
    if (terminals.length === 1) return; // Keep at least one
    terminals = terminals.filter(t => t.id !== id);
    if (activeTab === id) {
      activeTab = terminals[0].id;
    }
  }

  async function runCommand(terminal) {
    if (!terminal.command.trim()) return;
    
    const cmd = terminal.command.trim() + '\n';
    terminal.history = [...terminal.history, { type: 'cmd', text: `$ ${cmd.trim()}` }];
    
    try {
      const encoder = new TextEncoder();
      const data = encoder.encode(cmd);
      await invoke('write_to_terminal', { 
        id: terminal.id, 
        data: Array.from(data)
      });
    } catch (e) {
      terminal.history = [...terminal.history, { type: 'error', text: `Error: ${e}` }];
    }

    terminal.command = '';
    scrollToBottom();
  }

  function handleKeydown(e, terminal) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      runCommand(terminal);
    }
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (container) container.scrollTop = container.scrollHeight;
    }, 10);
  }

  function clearTerminal(terminal) {
    terminal.history = [];
  }

  $: currentTerminal = terminals.find(t => t.id === activeTab);
  $: isTerminalTab = activeTab.startsWith('terminal-');
</script>

<div class="h-full flex flex-col bg-[#1e1e1e] border-t border-[#2d2d2d]">
  <!-- Header with Tabs -->
  <div class="h-9 px-2 border-b border-[#2d2d2d] flex items-center justify-between select-none bg-[#252526]">
    <div class="flex items-center gap-1 overflow-x-auto scrollbar-thin">
      {#each terminals as terminal}
        <button
          class={cn(
            "flex items-center gap-2 text-xs font-medium px-3 py-1.5 rounded-t transition-colors group",
            activeTab === terminal.id 
              ? "bg-[#1e1e1e] text-white" 
              : "text-gray-400 hover:text-white hover:bg-[#2a2a2a]"
          )}
          onclick={() => activeTab = terminal.id}
        >
          <TerminalIcon size={14} />
          <span>{terminal.label}</span>
          {#if terminals.length > 1}
            <button
              class="opacity-0 group-hover:opacity-100 hover:bg-[#3e3e42] rounded p-0.5"
              onclick={(e) => { e.stopPropagation(); closeTerminal(terminal.id); }}
            >
              <X size={12} />
            </button>
          {/if}
        </button>
      {/each}
      
      <button
        class="text-gray-400 hover:text-white hover:bg-[#2a2a2a] p-1.5 rounded transition-colors"
        onclick={addTerminal}
        title="New Terminal"
      >
        <Plus size={14} />
      </button>

      <div class="w-px h-4 bg-[#2d2d2d] mx-1"></div>

      {#each tabs as tab}
        <button
          class={cn(
            "text-xs font-medium px-3 py-1.5 rounded-t transition-colors",
            activeTab === tab.id 
              ? "bg-[#1e1e1e] text-white" 
              : "text-gray-400 hover:text-white hover:bg-[#2a2a2a]"
          )}
          onclick={() => activeTab = tab.id}
        >
          {tab.label}
        </button>
      {/each}
    </div>

    <div class="flex items-center gap-1 text-gray-400">
      {#if isTerminalTab && currentTerminal}
        <button 
          class="p-1 hover:text-white hover:bg-[#2a2a2a] rounded" 
          title="Clear Terminal" 
          onclick={() => clearTerminal(currentTerminal)}
        >
          <Trash2 size={14} />
        </button>
      {/if}
      <button class="p-1 hover:text-white hover:bg-[#2a2a2a] rounded" title="Minimize">
        <Minimize2 size={14} />
      </button>
      <button class="p-1 hover:text-white hover:bg-[#2a2a2a] rounded" title="Close Panel">
        <X size={14} />
      </button>
    </div>
  </div>

  <!-- Content Area -->
  <div class="flex-1 overflow-hidden">
    {#if isTerminalTab && currentTerminal}
      <div class="h-full flex flex-col p-2 font-mono text-sm overflow-y-auto" bind:this={container}>
        {#each currentTerminal.history as line}
          <div class={cn(
            "whitespace-pre-wrap break-words",
            line.type === 'cmd' ? 'text-green-400 font-semibold' : 
            line.type === 'error' ? 'text-red-400' : 
            'text-gray-300'
          )}>
            {line.text}
          </div>
        {/each}
        
        <div class="flex items-start gap-2 mt-1">
          <span class="text-green-400 font-bold flex-shrink-0">➜</span>
          <input 
            type="text" 
            bind:value={currentTerminal.command}
            onkeydown={(e) => handleKeydown(e, currentTerminal)}
            class="bg-transparent border-none outline-none text-white flex-1 font-mono"
            spellcheck="false"
            autocomplete="off"
            autofocus
            placeholder="Type command..."
          />
        </div>
      </div>
    {:else if activeTab === 'output'}
      <div class="h-full p-4 text-sm text-gray-400">
        <div class="flex items-center gap-2 mb-2">
          <ChevronDown size={16} />
          <span class="font-semibold">Build Output</span>
        </div>
        <div class="text-xs text-gray-500">No output yet. Run a build command to see output here.</div>
      </div>
    {:else if activeTab === 'problems'}
      <div class="h-full p-4 text-sm text-gray-400">
        <div class="text-xs text-gray-500">No problems detected.</div>
      </div>
    {:else if activeTab === 'debug'}
      <div class="h-full p-4 text-sm text-gray-400">
        <div class="text-xs text-gray-500">Debug console ready. Start debugging to see output.</div>
      </div>
    {/if}
  </div>
</div>

<style>
  .scrollbar-thin::-webkit-scrollbar {
    height: 4px;
  }
</style>
