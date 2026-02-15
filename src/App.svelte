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
      // Mock check
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
    <!-- Top Menu Bar -->
    <MenuBar onOpenSettings={() => showSettings = true} />

    <div class="flex-1 flex overflow-hidden relative">
      <!-- Left: Activity Bar -->
      <ActivityBar bind:activeView bind:activeMode onChatToggle={toggleChat} onOpenSettings={() => showSettings = true} />
      
      {#if activeMode === 'editor'}
        <!-- EDITOR MODE LAYOUT -->
        <div class="flex-1 flex overflow-hidden animate-in fade-in duration-300">
          
          <!-- Resizable Sidebar -->
          {#if activeView === 'explorer'}
            <ResizablePanel 
              side="left" 
              initialSize={configStore.settings.layout.sidebarWidth || 280} 
              minSize={200} 
              maxSize={500}
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
              <div class="font-medium text-foreground mb-2">Search</div>
              <input type="text" placeholder="Search files..." class="w-full bg-background border border-border rounded px-2 py-1" />
            </ResizablePanel>
          {/if}
          
          <!-- Main Editor Area (Vertical Flex) -->
          <div class="flex-1 flex flex-col min-w-0 bg-background relative z-0">
             
             <!-- Top Half: Editor/Dashboard -->
             {#if activeView === 'dashboard' && fsStore.openFiles.length === 0}
                <div class="flex-1 overflow-hidden">
                  <Dashboard />
                </div>
             {:else}
                <div class="flex-1 flex flex-col min-h-0 relative">
                  <TabBar />
                  <div class="flex-1 relative bg-background">
                    <Editor />
                    <!-- Empty State -->
                    {#if !fsStore.activeFile}
                      <div class="absolute inset-0 flex flex-col items-center justify-center pointer-events-none opacity-20 select-none">
                         <div class="text-6xl font-bold tracking-tighter text-foreground">Karsa</div>
                         <div class="text-xs tracking-[0.5em] uppercase mt-4 text-muted-foreground">Intelligent IDE</div>
                      </div>
                    {/if}
                  </div>
                </div>
             {/if}

             <!-- Bottom Half: Resizable Terminal -->
             {#if showTerminal}
               <div class="h-1 bg-border hover:bg-primary/50 cursor-row-resize transition-colors z-20"></div>
               <div class="h-[200px] shrink-0 bg-background border-t border-border">
                  <Terminal />
               </div>
             {/if}
          </div>
          
          <!-- Resizable Chat Panel (Right) -->
          {#if showChat}
            <ResizablePanel 
              side="right" 
              initialSize={configStore.settings.layout.chatWidth || 400} 
              minSize={300} 
              maxSize={800}
              className="border-l border-border bg-background shadow-2xl z-20"
            >
               <ChatPanel onClose={toggleChat} />
            </ResizablePanel>
          {/if}
        </div>
      
      {:else}
        <!-- AGENT MODE LAYOUT -->
        <div class="flex-1 overflow-hidden bg-background relative z-0 animate-in zoom-in-95 duration-300">
           <AgentView />
        </div>
      {/if}
    </div>
    
    <!-- Status Bar (Optional) -->
    <div class="h-6 bg-primary text-primary-foreground flex items-center px-3 text-[10px] select-none justify-between z-30">
       <div class="flex items-center gap-3">
          <span class="font-bold">main*</span>
          <span class="opacity-80">0 errors, 0 warnings</span>
       </div>
       <div class="flex items-center gap-3">
          <span class="opacity-80">Ln 12, Col 45</span>
          <span class="opacity-80">UTF-8</span>
          <span class="opacity-80">JavaScript</span>
          <button class="hover:bg-primary-foreground/20 px-1 rounded" onclick={toggleTerminal} title="Toggle Terminal">
             Terminal
          </button>
       </div>
    </div>
  {/if}

  {#if showSettings}
    <SettingsModal onClose={() => showSettings = false} />
  {/if}
</main>
