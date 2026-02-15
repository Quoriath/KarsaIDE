<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  
  // Stores
  import { fsStore } from './lib/stores/fileSystem.svelte.js';
  import { themeStore } from './lib/stores/theme.svelte.js';
  import { configStore } from './lib/stores/config.svelte.js';

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
  import SettingsModal from './lib/components/Settings/SettingsModal.svelte';
  
  let showOnboarding = $state(true);
  let showChat = $state(configStore.settings.layout.chatVisible);
  let activeView = $state('dashboard');
  let activeMode = $state('editor'); // 'editor' | 'agent'
  let showTerminal = $state(configStore.settings.layout.bottomPanelVisible);
  let showSettings = $state(false);

  onMount(async () => {
    // Initialize Theme
    themeStore.applyTheme(configStore.settings.theme || 'karsa-dark');

    try {
      const configExists = await invoke('file_exists', { path: 'karsa_config.json' });
      showOnboarding = !configExists;
    } catch (e) {
      showOnboarding = !localStorage.getItem('karsa_ai_config');
    }
  });

  function toggleChat() {
    showChat = !showChat;
    configStore.updateLayout({ chatVisible: showChat });
  }

  function toggleTerminal() {
    showTerminal = !showTerminal;
    configStore.updateLayout({ bottomPanelVisible: showTerminal });
  }
</script>

<main class="h-screen w-screen flex flex-col bg-background text-foreground overflow-hidden font-sans selection:bg-primary/30 antialiased">
  
  {#if showOnboarding}
    <Onboarding onComplete={() => showOnboarding = false} />
  {:else}
    <!-- Top Menu Bar (Fixed Height) -->
    <MenuBar onOpenSettings={() => showSettings = true} />

    <!-- Main Content Area (Flex Grow - Fills remaining space) -->
    <div class="flex-1 flex overflow-hidden relative min-h-0">
      <!-- Left: Activity Bar (Fixed Width) -->
      <ActivityBar bind:activeView bind:activeMode onChatToggle={toggleChat} onOpenSettings={() => showSettings = true} />
      
      {#if activeMode === 'editor'}
        <!-- EDITOR MODE LAYOUT -->
        <div class="flex-1 flex overflow-hidden animate-in fade-in duration-200">
          
          <!-- Resizable Sidebar -->
          {#if activeView === 'explorer'}
            <ResizablePanel 
              side="left" 
              initialSize={configStore.settings.layout.sidebarWidth || 260} 
              minSize={200} 
              maxSize={400}
              className="bg-muted/5 border-r border-border shadow-sm z-10"
            >
              <Sidebar />
            </ResizablePanel>
          {:else if activeView === 'search'}
            <ResizablePanel 
              side="left" 
              initialSize={300}
              className="bg-muted/5 border-r border-border p-4 text-sm text-muted-foreground"
            >
              <div class="font-medium text-foreground mb-2 text-xs uppercase tracking-wider">Search</div>
              <input type="text" placeholder="Search files..." class="w-full bg-background border border-border rounded px-2 py-1.5 text-xs focus:ring-1 focus:ring-primary outline-none" />
            </ResizablePanel>
          {/if}
          
          <!-- Main Editor Area (Vertical Flex) -->
          <div class="flex-1 flex flex-col min-w-0 bg-background relative z-0">
             
             <!-- Top Half: Editor/Dashboard -->
             <div class="flex-1 flex flex-col min-h-0 relative overflow-hidden">
               {#if activeView === 'dashboard' && fsStore.openFiles.length === 0}
                  <Dashboard />
               {:else}
                  {#if activeView === 'dashboard'}
                     <Dashboard />
                  {:else}
                     <div class="flex-1 flex flex-col h-full overflow-hidden">
                        <TabBar />
                        <div class="flex-1 relative bg-background">
                          <Editor />
                          <!-- Empty State -->
                          {#if !fsStore.activeFile}
                            <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none opacity-20 select-none">
                               <div class="text-6xl font-bold tracking-tighter text-foreground">Karsa</div>
                               <div class="text-[10px] tracking-[0.5em] uppercase mt-4 text-muted-foreground">Intelligent IDE</div>
                            </div>
                          {/if}
                        </div>
                     </div>
                  {/if}
               {/if}
             </div>

             <!-- Bottom Half: Resizable Terminal -->
             {#if showTerminal}
               <div class="h-[1px] bg-border hover:bg-primary/50 cursor-row-resize transition-colors z-20"></div>
               <div class="h-[200px] min-h-[100px] shrink-0 bg-background border-t border-border flex flex-col">
                  <Terminal />
               </div>
             {/if}
          </div>
          
          <!-- Resizable Chat Panel (Right) -->
          {#if showChat}
            <ResizablePanel 
              side="right" 
              initialSize={configStore.settings.layout.chatWidth || 350} 
              minSize={300} 
              maxSize={600}
              className="border-l border-border bg-background shadow-2xl z-20"
            >
               <ChatPanel onClose={toggleChat} />
            </ResizablePanel>
          {/if}
        </div>
      
      {:else}
        <!-- AGENT MODE LAYOUT -->
        <div class="flex-1 overflow-hidden bg-background relative z-0 animate-in zoom-in-95 duration-200">
           <AgentView />
        </div>
      {/if}
    </div>
    
    <!-- Status Bar (Fixed Height, Always Visible) -->
    <div class="h-6 min-h-[24px] bg-primary text-primary-foreground flex items-center px-3 text-[10px] select-none justify-between z-30 shrink-0 border-t border-primary-foreground/10">
       <div class="flex items-center gap-3">
          <span class="font-bold flex items-center gap-1.5">
             <div class="w-1.5 h-1.5 rounded-full bg-white animate-pulse"></div>
             main*
          </span>
          <span class="opacity-80">0 errors</span>
       </div>
       <div class="flex items-center gap-3">
          <span class="opacity-80">Ln 12, Col 45</span>
          <span class="opacity-80">UTF-8</span>
          <span class="opacity-80 font-mono">JavaScript</span>
          <button class="hover:bg-white/20 px-1.5 py-0.5 rounded transition-colors flex items-center gap-1" onclick={toggleTerminal} title="Toggle Terminal">
             Terminal
          </button>
       </div>
    </div>
  {/if}

  {#if showSettings}
    <SettingsModal onClose={() => showSettings = false} />
  {/if}
</main>
