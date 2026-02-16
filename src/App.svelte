<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  // Stores
  import { fsStore } from './lib/stores/fileSystem.svelte.js';
  import { themeStore } from './lib/stores/theme.svelte.js';
  import { configStore } from './lib/stores/config.svelte.js';
  import { uiState } from './lib/stores/uiState.svelte.js';
  import { chatStore } from './lib/stores/chatStore.svelte.js';

  // Components
  import MenuBar from './lib/components/MenuBar.svelte';
  import ActivityBar from './lib/components/ActivityBar.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Editor from './lib/components/Editor/Monaco.svelte';
  import ChatPanel from './lib/components/Chat/ChatPanel.svelte';
  import TabBar from './lib/components/TabBar.svelte';
  import Dashboard from './lib/components/Dashboard.svelte';
  import Onboarding from './lib/components/Onboarding/Onboarding.svelte';
  import AgentView from './lib/components/AgentView.svelte';
  import ResizablePanel from './lib/components/ResizablePanel.svelte';
  import Terminal from './lib/components/Terminal.svelte';
  import Settings from './lib/components/Settings.svelte';
  import TitleBar from './lib/components/TitleBar.svelte';
  import FileExplorer from './lib/components/FileExplorer.svelte';
  import ContextMenu from './lib/components/ContextMenu.svelte';
  import Dialog from './lib/components/ui/Dialog.svelte';
  
  let showOnboarding = $state(true);
  let showChat = $state(configStore.settings.layout.chatVisible);
  let activeView = $state('dashboard');
  let activeMode = $state('editor'); 
  let showTerminal = $state(configStore.settings.layout.bottomPanelVisible);
  let showSettings = $state(false);
  let terminalHeight = $state(configStore.settings.layout.bottomPanelHeight || 250);
  let isResizingTerminal = $state(false);

  function startTerminalResize(e) {
    isResizingTerminal = true;
    document.body.classList.add('resizing-terminal');
    const startY = e.clientY;
    const startHeight = terminalHeight;
    
    function onMouseMove(e) {
      if (!isResizingTerminal) return;
      const delta = startY - e.clientY;
      const newHeight = Math.max(100, Math.min(window.innerHeight * 0.8, startHeight + delta));
      terminalHeight = newHeight;
    }
    
    function onMouseUp() {
      isResizingTerminal = false;
      document.body.classList.remove('resizing-terminal');
      configStore.updateLayout({ bottomPanelHeight: terminalHeight });
      window.removeEventListener('mousemove', onMouseMove);
      window.removeEventListener('mouseup', onMouseUp);
    }
    
    window.addEventListener('mousemove', onMouseMove);
    window.addEventListener('mouseup', onMouseUp);
  }

  onMount(async () => {
    themeStore.applyTheme(configStore.settings.theme || 'karsa-dark');
    
    // Auto-restore last workspace
    try {
      const lastWorkspace = await invoke('get_last_workspace');
      if (lastWorkspace) {
        console.log('Restoring workspace:', lastWorkspace);
        fsStore.setCurrentProjectDir(lastWorkspace);
      }
    } catch (e) {
      console.error('Failed to restore workspace:', e);
    }
    
    // Listeners
    const u1 = await listen('terminal-state-changed', (e) => {
      showTerminal = e.payload;
      configStore.updateLayout({ bottomPanelVisible: showTerminal });
    });
    const u2 = await listen('chat-state-changed', (e) => {
      showChat = e.payload;
      configStore.updateLayout({ chatVisible: showChat });
    });

    // Check if config exists and has API key
    try {
      const config = await invoke('get_ai_config');
      showOnboarding = !config.ai?.api_key;
    } catch (e) {
      showOnboarding = true;
    }
  });

  async function toggleChat() {
    showChat = !showChat;
    await invoke('toggle_chat', { visible: showChat }).catch(() => {});
    configStore.updateLayout({ chatVisible: showChat });
  }

  async function toggleTerminal() {
    showTerminal = !showTerminal;
    await invoke('toggle_terminal', { visible: showTerminal }).catch(() => {});
    configStore.updateLayout({ bottomPanelVisible: showTerminal });
  }
</script>

<div class="h-screen w-screen flex flex-col bg-background text-foreground overflow-hidden font-sans selection:bg-primary/30 antialiased relative">
  
  {#if showOnboarding}
    <Onboarding onComplete={() => showOnboarding = false} />
  {:else}
    <!-- Layered layout to ensure everything fits -->
    <TitleBar />

    <div class="flex-1 flex flex-col min-h-0 pt-10"> <!-- pt-10 for TitleBar height -->
      <!-- Top: Menu -->
      <div class="shrink-0 z-50">
        <MenuBar onOpenSettings={() => showSettings = true} />
      </div>

      <!-- Main: Sidebar + Content + Chat -->
      <div class="flex-1 flex overflow-hidden relative min-h-0">
        <!-- Activity Bar (Fixed Left) -->
        <div class="shrink-0 z-40 h-full">
          <ActivityBar 
            bind:activeView 
            bind:activeMode 
            onChatToggle={toggleChat} 
            onOpenSettings={() => showSettings = true} 
            onTerminalToggle={toggleTerminal}
          />
        </div>
        
        {#if activeMode === 'editor'}
          <div class="flex-1 flex overflow-hidden animate-in fade-in duration-300">
            <!-- Left Sidebar -->
            {#if activeView === 'explorer'}
              <ResizablePanel side="left" initialSize={260} minSize={200} maxSize={400} className="bg-muted/5 border-r border-border/40 z-30">
                <FileExplorer />
              </ResizablePanel>
            {:else if activeView === 'search'}
              <ResizablePanel side="left" initialSize={300} className="bg-muted/5 border-r border-border/40 p-4 text-sm text-muted-foreground z-30">
                <div class="font-bold text-foreground mb-4 text-[10px] uppercase tracking-widest">Search Workspace</div>
                <input type="text" placeholder="Search files..." class="w-full bg-muted/20 border border-border/40 rounded-xl px-3 py-2 text-xs focus:ring-2 focus:ring-primary/20 outline-none transition-all shadow-inner" />
              </ResizablePanel>
            {/if}
            
            <!-- Center Content -->
            <div class="flex-1 flex flex-col min-w-0 bg-background relative z-0">
               <div class="flex-1 flex flex-col min-h-0 relative overflow-hidden">
                 {#if activeView === 'dashboard' && fsStore.openFiles.length === 0}
                    <Dashboard />
                 {:else}
                    {#if activeView === 'dashboard'}
                       <Dashboard />
                    {:else}
                       <div class="flex-1 flex flex-col h-full overflow-hidden">
                          <div class="shrink-0">
                             <TabBar />
                          </div>
                          <div class="flex-1 relative bg-background min-h-0">
                            <Editor />
                            {#if !fsStore.activeFile}
                              <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none opacity-10 select-none">
                                 <div class="text-8xl font-black tracking-tighter text-foreground italic">KARSA</div>
                                 <div class="text-[10px] tracking-[1em] uppercase mt-6 text-primary font-bold">INTELLIGENT STUDIO</div>
                              </div>
                            {/if}
                          </div>
                       </div>
                    {/if}
                 {/if}
               </div>

               {#if showTerminal}
                 <div class="shrink-0 z-20 terminal-panel" style="height: {terminalHeight}px">
                    <div 
                      class="h-[2px] bg-border hover:bg-primary transition-colors cursor-row-resize active:bg-primary shadow-[0_-2px_10px_rgba(0,0,0,0.2)]"
                      onmousedown={startTerminalResize}
                    ></div>
                    <div class="h-full bg-background border-t border-border/40 flex flex-col overflow-hidden">
                        <Terminal onClose={toggleTerminal} />
                    </div>
                 </div>
               {/if}
            </div>
            
            <!-- Right Chat Sidebar -->
            {#if showChat}
              <ResizablePanel side="right" initialSize={350} minSize={300} maxSize={600} className="border-l border-border/40 bg-background/50 backdrop-blur-3xl z-20 overflow-hidden">
                 <ChatPanel onClose={toggleChat} />
              </ResizablePanel>
            {/if}
          </div>
        {:else}
          <!-- AGENT LAYOUT (Synced) -->
          <div class="flex-1 overflow-hidden bg-background relative z-0 animate-in zoom-in-95 duration-300">
             <AgentView />
          </div>
        {/if}
      </div>

      <!-- Bottom Footer (Consistently visible) -->
      <div class="h-7 shrink-0 bg-background border-t border-border/40 flex items-center px-4 text-[9px] font-bold select-none justify-between z-50 text-muted-foreground/60 uppercase tracking-widest transition-all duration-300">
         <div class="flex items-center gap-4">
            <button class="hover:text-primary flex items-center gap-2 transition-colors group">
               <div class="w-1.5 h-1.5 rounded-full bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)] group-hover:animate-ping"></div>
               <span class="text-foreground">Production*</span>
            </button>
            <div class="flex items-center gap-1.5 hover:text-foreground cursor-pointer transition-colors">
               <div class="w-[1px] h-3 bg-border/50"></div>
               <span>0 Disruptions</span>
            </div>
         </div>
         <div class="flex items-center gap-5">
            <div class="flex items-center gap-1.5">
               <span>Line {uiState.editorStatus.line}</span>
               <span class="opacity-30">/</span>
               <span>Col {uiState.editorStatus.column}</span>
            </div>
            <div class="flex items-center gap-1.5 text-primary">
               <div class="w-[1px] h-3 bg-border/50"></div>
               <span>{uiState.editorStatus.language}</span>
            </div>
            <button 
              class={`transition-all duration-300 flex items-center gap-1.5 hover:text-foreground ${showTerminal ? "text-primary font-black" : ""}`} 
              onclick={toggleTerminal}
            >
               <div class="w-[1px] h-3 bg-border/50"></div>
               Terminal
            </button>
         </div>
      </div>
    </div>
  {/if}

  {#if showSettings}
    <Settings onClose={() => showSettings = false} />
  {/if}

  <ContextMenu />
  <Dialog />
</div>

<style>
  :global(body) {
    user-select: none;
    margin: 0;
    padding: 0;
    overflow: hidden;
  }
  
  :global(body.resizing-terminal) {
    cursor: row-resize !important;
  }
  
  :global(body.resizing-terminal *) {
    pointer-events: none !important;
  }
</style>
