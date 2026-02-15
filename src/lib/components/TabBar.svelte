<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { FileCode, File, FileJson, X, Circle } from 'lucide-svelte';
  import { cn } from '../utils.js';

  function closeTab(e, path) {
    e.stopPropagation();
    fsStore.closeFile(path);
  }

  function selectTab(path) {
    fsStore.setActiveFile(path);
  }

  function getFileIcon(name) {
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'json': return FileJson;
      case 'js':
      case 'ts':
      case 'jsx':
      case 'tsx':
      case 'svelte':
      case 'html':
      case 'css':
        return FileCode;
      default: return File;
    }
  }
</script>

<div class="flex items-center bg-muted/10 border-b border-border overflow-x-auto scrollbar-thin h-9 shrink-0 select-none">
  {#if fsStore.openFiles.length === 0}
    <div class="px-4 text-xs text-muted-foreground italic flex items-center h-full w-full bg-background/50">
      Double-click a file to open
    </div>
  {:else}
    {#each fsStore.openFiles as file}
      {@const IconCmp = getFileIcon(file.name)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class={cn(
          "group flex items-center gap-2 px-3 h-full text-xs border-r border-border hover:bg-muted/30 transition-all min-w-[120px] max-w-[200px] relative cursor-pointer",
          fsStore.activeFile?.path === file.path 
            ? "bg-background border-t-2 border-t-primary text-foreground font-medium" 
            : "bg-muted/10 text-muted-foreground border-t-2 border-t-transparent"
        )}
        onclick={() => selectTab(file.path)}
      >
        <IconCmp size={14} class={cn("shrink-0", fsStore.activeFile?.path === file.path ? "text-primary" : "opacity-70")} />
        
        <span class="truncate flex-1">{file.name}</span>
        
        <button 
          class={cn(
            "p-0.5 rounded-sm hover:bg-muted hover:text-foreground transition-all shrink-0 opacity-0 group-hover:opacity-100",
            file.modified ? "opacity-100" : ""
          )}
          onclick={(e) => closeTab(e, file.path)}
          title="Close"
        >
          {#if file.modified}
             <div class="w-2 h-2 bg-primary rounded-full group-hover:hidden"></div>
             <X size={12} class="hidden group-hover:block" />
          {:else}
             <X size={12} />
          {/if}
        </button>
      </div>
    {/each}
  {/if}
</div>
