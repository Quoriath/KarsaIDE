<script>
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { ChevronRight, FileCode, Folder } from 'lucide-svelte';
  
  let pathParts = $derived(fsStore.activeFile?.path?.split('/') || []);
  let fileName = $derived(pathParts[pathParts.length - 1] || '');
  let folders = $derived(pathParts.slice(0, -1));
</script>

<div class="h-9 px-4 flex items-center gap-1.5 text-[11px] text-muted-foreground bg-background/50 backdrop-blur-sm border-b border-border/50 overflow-x-auto no-scrollbar select-none">
  {#if fsStore.activeFile}
    <div class="flex items-center gap-1.5 shrink-0">
      <Folder size={12} class="text-primary/70" />
      <span class="hover:text-foreground cursor-default transition-colors">KarsaProject</span>
    </div>

    {#each folders as folder}
      {#if folder}
        <ChevronRight size={10} class="shrink-0 opacity-40" />
        <span class="hover:text-foreground cursor-default transition-colors shrink-0">{folder}</span>
      {/if}
    {/each}

    {#if fileName}
      <ChevronRight size={10} class="shrink-0 opacity-40" />
      <div class="flex items-center gap-1.5 shrink-0 text-foreground font-medium">
        <FileCode size={12} class="text-primary" />
        <span>{fileName}</span>
      </div>
    {/if}
  {:else}
    <div class="flex items-center gap-1.5 opacity-50 italic">
      <FileCode size={12} />
      <span>No file selected</span>
    </div>
  {/if}
</div>

<style>
  .no-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .no-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
