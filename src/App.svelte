<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  // Stores
  import { fsStore } from './lib/stores/fileSystem.svelte.js';
  import { themeStore } from './lib/stores/theme.svelte.js';
  import { configStore } from './lib/stores/config.svelte.js';
  import { uiState } from './lib/stores/uiState.svelte.js';

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
  
  let showOnboarding = $state(true);
  let showChat = $state(configStore.settings.layout.chatVisible);
  let activeView = $state('dashboard');
  let activeMode = $state('editor'); 
  let showTerminal = $state(configStore.settings.layout.bottomPanelVisible);
  let showSettings = $state(false);
  let settingsTab = $state('general');
  let terminalHeight = $state(configStore.settings.layout.bottomPanelHeight || 250);
  let isResizingTerminal = $state(false);

  function openSettings(tab = 'general') {
    settingsTab = tab;
    showSettings = true;
  }

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
    themeStore.setTheme(configStore.settings.theme || 'karsa-dark');
    if (configStore.settings.appearance?.mode) {
      themeStore.setMode(configStore.settings.appearance.mode);
    }

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleThemeChange = () => {
      if (themeStore.mode === 'system') themeStore.applyTheme(themeStore.activeTheme);
    };
    mediaQuery.addEventListener('change', handleThemeChange);
    
    const u1 = await listen('terminal-state-changed', (e) => {
      showTerminal = e.payload;
      configStore.updateLayout({ bottomPanelVisible: showTerminal });
    });
    const u2 = await listen('chat-state-changed', (e) => {
      showChat = e.payload;
      configStore.updateLayout({ chatVisible: showChat });
    });

    try {
      const config = await invoke('get_ai_config');
      showOnboarding = !config.ai?.api_key;
    } catch (e) {
      showOnboarding = true;
    }

    return () => mediaQuery.removeEventListener('change', handleThemeChange);
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

<main class="h-screen w-screen flex flex-col bg-background text-foreground overflow-hidden font-sans selection:bg-primary/30 antialiased">
  
  {#if showOnboarding}
    <Onboarding onComplete={() => showOnboarding = false} />
  {:else}
    <TitleBar />

    <div class="shrink-0 z-[60] pt-8">
      <MenuBar onOpenSettings={openSettings} />
    </div>

    <div class="flex-1 flex overflow-hidden relative min-h-0">
      
      <div class="shrink-0 z-40 h-full">
        <ActivityBar 
          bind:activeView 
          bind:activeMode 
          onChatToggle={toggleChat} 
          onOpenSettings={openSettings} 
          onTerminalToggle={toggleTerminal}
        />
      </div>
      
      {#if activeMode === 'editor'}
        <div class="flex-1 flex overflow-hidden">
          
          {#if activeView === 'explorer'}
            <ResizablePanel side="left" initialSize={240} minSize={180} maxSize={400} className="bg-muted/5 border-r border-border/40 z-30">
              <FileExplorer />
            </ResizablePanel>
          {:else if activeView === 'search'}
            <ResizablePanel side="left" initialSize={280} className="bg-muted/5 border-r border-border/40 p-4 text-sm text-muted-foreground z-30">
              <div class="font-bold text-foreground mb-3 text-[10px] uppercase tracking-widest opacity-50">Global Search</div>
              <input type="text" placeholder="Find in files..." class="w-full bg-muted/20 border border-border/40 rounded-lg px-3 py-2 text-xs focus:ring-2 focus:ring-primary/20 outline-none" />
            </ResizablePanel>
          {/if}
          
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
                            <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none opacity-[0.03] select-none">
                               <div class="text-[120px] font-black tracking-tighter text-foreground">KARSA</div>
                            </div>
                          {/if}
                        </div>
                     </div>
                  {/if}
               {/if}
             </div>

             {#if showTerminal}
               <div class="shrink-0 z-20 terminal-panel" style="height: {terminalHeight}px">
                  <div class="h-[2px] bg-border hover:bg-primary/50 cursor-row-resize transition-colors" onmousedown={startTerminalResize}></div>
                  <div class="h-full bg-background/50 backdrop-blur-xl border-t border-border/40 flex flex-col overflow-hidden">
                      <Terminal onClose={toggleTerminal} />
                  </div>
               </div>
             {/if}
          </div>
          
          {#if showChat}
            <ResizablePanel side="right" initialSize={320} minSize={280} maxSize={600} className="border-l border-border/40 bg-background/50 backdrop-blur-xl z-20">
               <ChatPanel onClose={toggleChat} />
            </ResizablePanel>
          {/if}
        </div>
      
      {:else}
        <div class="flex-1 overflow-hidden bg-background relative z-0 animate-in zoom-in-95 duration-300">
           <AgentView />
        </div>
      {/if}
    </div>
    
    <div class="h-6 min-h-[24px] shrink-0 bg-background/80 backdrop-blur border-t border-border/20 flex items-center px-3 text-[9px] select-none justify-between z-50 text-muted-foreground/60 font-bold uppercase tracking-tighter">
       <div class="flex items-center gap-4">
          <button class="hover:text-primary flex items-center gap-1.5 transition-colors">
             <div class="w-1 h-1 rounded-full bg-primary animate-pulse shadow-[0_0_5px_var(--primary)]"></div>
             <span class="text-foreground/80">main*</span>
          </button>
          <span class="hover:text-foreground cursor-pointer transition-colors hidden sm:inline">0 Issues</span>
       </div>
       <div class="flex items-center gap-5">
          <span class="hover:text-foreground cursor-pointer transition-colors hidden sm:inline">Ln {uiState.editorStatus.line}, Col {uiState.editorStatus.column}</span>
          <span class="hover:text-foreground cursor-pointer transition-colors text-primary uppercase font-black">{uiState.editorStatus.language}</span>
          <button class={cn("transition-colors", showTerminal ? "text-primary" : "hover:text-foreground")} onclick={toggleTerminal}>Terminal</button>
       </div>
    </div>
  {/if}

  {#if showSettings}
    <Settings initialTab={settingsTab} onClose={() => showSettings = false} />
  {/if}
</main>

<style>
  :global(body) { user-select: none; }
  :global(body.resizing-terminal) { cursor: row-resize !important; }
  :global(body.resizing-terminal *) { pointer-events: none !important; }
</style>
