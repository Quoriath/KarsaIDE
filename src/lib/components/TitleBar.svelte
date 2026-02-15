<script>
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Minus, Square, X, AppWindow } from 'lucide-svelte';

  const appWindow = getCurrentWindow();

  function minimize() { appWindow.minimize(); }
  function maximize() { appWindow.toggleMaximize(); }
  function close() { appWindow.close(); }
</script>

<div data-tauri-drag-region class="h-8 bg-background flex items-center justify-between select-none fixed top-0 left-0 right-0 z-[60] border-b border-border/50">
  <!-- Left: App Icon/Name -->
  <div class="flex items-center gap-2 px-3 pointer-events-none">
    <div class="w-4 h-4 bg-primary/20 rounded flex items-center justify-center text-primary">
       <AppWindow size={10} strokeWidth={3} />
    </div>
    <span class="text-xs font-semibold text-muted-foreground/80 tracking-wide">Karsa IDE</span>
  </div>

  <!-- Center: Title (Optional) -->
  <!-- <div class="text-xs text-muted-foreground pointer-events-none">Start Page</div> -->

  <!-- Right: Window Controls -->
  <div class="flex h-full">
    <button onclick={minimize} class="w-10 h-full flex items-center justify-center text-muted-foreground hover:bg-muted/50 hover:text-foreground transition-colors" tabindex="-1">
      <Minus size={14} />
    </button>
    <button onclick={maximize} class="w-10 h-full flex items-center justify-center text-muted-foreground hover:bg-muted/50 hover:text-foreground transition-colors" tabindex="-1">
      <Square size={12} />
    </button>
    <button onclick={close} class="w-10 h-full flex items-center justify-center text-muted-foreground hover:bg-red-500 hover:text-white transition-colors" tabindex="-1">
      <X size={14} />
    </button>
  </div>
</div>
