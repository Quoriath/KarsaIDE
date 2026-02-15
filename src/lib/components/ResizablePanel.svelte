<script>
  import { onMount, onDestroy } from 'svelte';
  import { cn } from '../utils.js';

  let { 
    initialSize = 300,
    minSize = 200, 
    maxSize = 600, 
    side = 'left', // 'left' or 'right'
    className = '',
    children
  } = $props();

  let size = $state(initialSize);
  let isResizing = $state(false);
  let panelRef;

  function startResize(e) {
    e.preventDefault();
    isResizing = true;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'col-resize';
    document.body.classList.add('select-none'); // Prevent text selection
  }

  function handleMouseMove(e) {
    if (!isResizing) return;
    
    let newSize;
    if (side === 'left') {
      newSize = e.clientX;
    } else {
      newSize = window.innerWidth - e.clientX;
    }

    if (newSize >= minSize && newSize <= maxSize) {
      size = newSize;
    }
  }

  function stopResize() {
    isResizing = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.classList.remove('select-none');
    
    // Dispatch resize event for Monaco or other listeners
    window.dispatchEvent(new Event('resize'));
  }
</script>

<div 
  class={cn("relative flex h-full transition-[width] duration-0 ease-linear", className)}
  style="width: {size}px;"
  bind:this={panelRef}
>
  <div class="flex-1 h-full overflow-hidden">
     {@render children()}
  </div>
  
  <div 
    class={cn(
      "absolute top-0 bottom-0 w-1 cursor-col-resize hover:bg-primary/50 active:bg-primary z-50 transition-colors",
      side === 'left' ? "-right-1" : "-left-1"
    )}
    onmousedown={startResize}
  ></div>
</div>
