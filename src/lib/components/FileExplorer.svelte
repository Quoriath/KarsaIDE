<script>
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { ChevronRight, ChevronDown, File, Folder, FolderOpen } from 'lucide-svelte';
  import { onMount } from 'svelte';
  
  let expandedFolders = $state(new Set());
  
  // Auto-expand root when tree loads
  $effect(() => {
    if (fsStore.fileTree && !expandedFolders.has(fsStore.fileTree.path)) {
      expandedFolders.add(fsStore.fileTree.path);
      expandedFolders = new Set(expandedFolders);
    }
  });
  
  function toggleFolder(path) {
    if (expandedFolders.has(path)) {
      expandedFolders.delete(path);
    } else {
      expandedFolders.add(path);
    }
    expandedFolders = new Set(expandedFolders);
  }
  
  function handleFileClick(path) {
    fsStore.openFile(path);
  }
  
  function FileTreeItem({ node, depth = 0 }) {
    const isExpanded = expandedFolders.has(node.path);
    const paddingLeft = depth * 12;
    
    return {
      node,
      depth,
      isExpanded,
      paddingLeft
    };
  }
</script>

<div class="h-full flex flex-col bg-background border-r border-border">
  <div class="p-2 border-b border-border">
    <div class="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
      Explorer
    </div>
    {#if fsStore.currentProjectDir}
      <div class="text-[10px] text-muted-foreground mt-1 truncate">
        {fsStore.currentProjectDir.split('/').pop()}
      </div>
    {/if}
  </div>
  
  <div class="flex-1 overflow-y-auto text-sm">
    {#if fsStore.fileTree}
      {@render renderTree(fsStore.fileTree, 0)}
    {:else}
      <div class="p-4 text-center text-muted-foreground text-xs">
        No folder open
      </div>
    {/if}
  </div>
</div>

{#snippet renderTree(node, depth)}
  {#if node.is_dir}
    <div>
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div 
        class="flex items-center gap-1 px-2 py-1 hover:bg-accent cursor-pointer"
        style="padding-left: {depth * 12 + 8}px"
        onclick={() => toggleFolder(node.path)}
      >
        {#if expandedFolders.has(node.path)}
          <ChevronDown size={14} class="text-muted-foreground" />
          <FolderOpen size={14} class="text-blue-400" />
        {:else}
          <ChevronRight size={14} class="text-muted-foreground" />
          <Folder size={14} class="text-blue-400" />
        {/if}
        <span class="truncate">{node.name}</span>
      </div>
      
      {#if expandedFolders.has(node.path) && node.children}
        {#each node.children as child}
          {@render renderTree(child, depth + 1)}
        {/each}
      {/if}
    </div>
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="flex items-center gap-1 px-2 py-1 hover:bg-accent cursor-pointer"
      style="padding-left: {depth * 12 + 22}px"
      onclick={() => handleFileClick(node.path)}
    >
      <File size={14} class="text-muted-foreground" />
      <span class="truncate">{node.name}</span>
    </div>
  {/if}
{/snippet}
