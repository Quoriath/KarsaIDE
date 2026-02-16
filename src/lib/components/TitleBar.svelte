<script>
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Minus, Square, X, Zap } from 'lucide-svelte';

  const appWindow = getCurrentWindow();

  function minimize() { appWindow.minimize(); }
  function maximize() { appWindow.toggleMaximize(); }
  function close() { appWindow.close(); }
</script>

<div 
  data-tauri-drag-region 
  class="h-8 bg-background/80 backdrop-blur-xl flex items-center justify-between select-none fixed top-0 left-0 right-0 z-[100] border-b border-border/40"
>
  <div class="flex items-center gap-2.5 px-3 pointer-events-none">
    <div class="w-5 h-5 bg-primary/10 rounded flex items-center justify-center text-primary">
       <Zap size={12} fill="currentColor" strokeWidth={1} />
    </div>
    <span class="text-[10px] font-black tracking-tighter text-foreground/80 uppercase">KARSA <span class="text-primary italic">IDE</span></span>
  </div>

  <div class="flex h-full items-center">
    <button onclick={minimize} class="w-10 h-full flex items-center justify-center text-muted-foreground/60 hover:bg-muted/50 hover:text-foreground transition-all" tabindex="-1">
      <Minus size={12} />
    </button>
    <button onclick={maximize} class="w-10 h-full flex items-center justify-center text-muted-foreground/60 hover:bg-muted/50 hover:text-foreground transition-all" tabindex="-1">
      <Square size={10} />
    </button>
    <button onclick={close} class="w-10 h-full flex items-center justify-center text-muted-foreground/60 hover:bg-destructive hover:text-destructive-foreground transition-all" tabindex="-1">
      <X size={12} />
    </button>
  </div>
</div>
