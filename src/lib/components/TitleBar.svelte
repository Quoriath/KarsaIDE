<script>
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Minus, Square, X, AppWindow, Command } from 'lucide-svelte';

  const appWindow = getCurrentWindow();

  function minimize() { appWindow.minimize(); }
  function maximize() { appWindow.toggleMaximize(); }
  function close() { appWindow.close(); }
</script>

<div data-tauri-drag-region class="h-10 bg-background/80 backdrop-blur-md flex items-center justify-between select-none fixed top-0 left-0 right-0 z-[100] border-b border-border/40 px-1">
  <!-- Left: App Branding -->
  <div class="flex items-center gap-3 px-3 pointer-events-none">
    <div class="w-5 h-5 bg-primary/20 rounded-md flex items-center justify-center text-primary border border-primary/10">
       <AppWindow size={12} strokeWidth={3} />
    </div>
    <div class="flex flex-col">
      <span class="text-[11px] font-black text-foreground tracking-tighter uppercase leading-none">Karsa</span>
      <span class="text-[8px] font-bold text-primary tracking-[0.2em] uppercase leading-none mt-0.5">Studio</span>
    </div>
  </div>

  <!-- Center: Search/Command (Visual Only for now) -->
  <div class="hidden md:flex flex-1 max-w-md mx-4">
     <div class="w-full flex items-center gap-2 px-3 py-1 bg-muted/20 border border-border/40 rounded-lg text-muted-foreground/50 hover:bg-muted/30 transition-all cursor-text pointer-events-auto">
        <Command size={12} />
        <span class="text-[10px] font-medium">Search files or run commands...</span>
        <span class="ml-auto text-[9px] bg-muted/50 px-1 rounded border border-border/50">Ctrl P</span>
     </div>
  </div>

  <!-- Right: Window Controls -->
  <div class="flex items-center h-full pointer-events-auto">
    <button onclick={minimize} class="w-10 h-8 flex items-center justify-center text-muted-foreground hover:bg-muted/50 hover:text-foreground transition-all rounded-md" title="Minimize">
      <Minus size={14} />
    </button>
    <button onclick={maximize} class="w-10 h-8 flex items-center justify-center text-muted-foreground hover:bg-muted/50 hover:text-foreground transition-all rounded-md" title="Maximize">
      <Square size={12} />
    </button>
    <button onclick={close} class="w-10 h-8 flex items-center justify-center text-muted-foreground hover:bg-red-500/10 hover:text-red-500 transition-all rounded-md" title="Close">
      <X size={14} />
    </button>
  </div>
</div>
