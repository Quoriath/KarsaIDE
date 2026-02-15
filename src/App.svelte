<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
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
  let activeMode = $state('editor'); 
  let showTerminal = $state(configStore.settings.layout.bottomPanelVisible);
  let showSettings = $state(false);

  onMount(async () => {
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
    <MenuBar onOpenSettings={() => showSettings = true} />

    <div class="flex-1 flex overflow-hidden relative min-h-0">
      <ActivityBar 
        bind:activeView 
        bind:activeMode 
        onChatToggle={toggleChat} 
        onOpenSettings={() => showSettings = true} 
        onTerminalToggle={toggleTerminal}
      />
      
      {#if activeMode === 'editor'}
        <div class="flex-1 flex overflow-hidden animate-in fade-in duration-200">
          
          {#if activeView === 'explorer'}
            <ResizablePanel side="left" initialSize={260} minSize={200} maxSize={400} className="bg-muted/5 border-r border-border z-10">
              <Sidebar />
            </ResizablePanel>
          {:else if activeView === 'search'}
            <ResizablePanel side="left" initialSize={300} className="bg-muted/5 border-r border-border p-4 text-sm text-muted-foreground">
              <div class="font-medium text-foreground mb-2 text-xs uppercase tracking-wider">Search</div>
              <input type="text" placeholder="Search files..." class="w-full bg-background border border-border rounded px-2 py-1.5 text-xs focus:ring-1 focus:ring-primary outline-none" />
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
                        <TabBar />
                        <div class="flex-1 relative bg-background">
                          <Editor />
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

             {#if showTerminal}
               <div class="h-[1px] bg-border hover:bg-primary/50 cursor-row-resize transition-colors z-20"></div>
               <div class="h-[200px] min-h-[100px] shrink-0 bg-background border-t border-border flex flex-col">
                  <Terminal onClose={toggleTerminal} />
               </div>
             {/if}
          </div>
          
          {#if showChat}
            <ResizablePanel side="right" initialSize={350} minSize={300} maxSize={600} className="border-l border-border bg-background z-20">
               <ChatPanel onClose={toggleChat} />
            </ResizablePanel>
          {/if}
        </div>
      
      {:else}
        <div class="flex-1 overflow-hidden bg-background relative z-0 animate-in zoom-in-95 duration-200">
           <AgentView />
        </div>
      {/if}
    </div>
    
    <!-- Status Bar (Transparan / Menyatu) -->
    <div class="h-6 min-h-[24px] bg-background/80 backdrop-blur-md border-t border-border flex items-center px-3 text-[10px] select-none justify-between z-30 shrink-0 text-muted-foreground">
       <div class="flex items-center gap-3">
          <button class="hover:text-foreground flex items-center gap-1.5 transition-colors">
             <div class="w-1.5 h-1.5 rounded-full bg-primary animate-pulse"></div>
             <span class="font-semibold text-foreground">main*</span>
          </button>
          <span class="hover:text-foreground cursor-pointer transition-colors">0 errors</span>
       </div>
       <div class="flex items-center gap-4">
          <span class="hover:text-foreground cursor-pointer transition-colors">Ln 12, Col 45</span>
          <span class="hover:text-foreground cursor-pointer transition-colors">UTF-8</span>
          <span class="hover:text-foreground cursor-pointer transition-colors text-primary font-medium">JavaScript</span>
          <button 
            class={showTerminal ? "text-primary font-medium" : "hover:text-foreground transition-colors"} 
            onclick={toggleTerminal} 
            title="Toggle Terminal"
          >
             Terminal
          </button>
       </div>
    </div>
  {/if}

  {#if showSettings}
    <SettingsModal onClose={() => showSettings = false} />
  {/if}
</main>
