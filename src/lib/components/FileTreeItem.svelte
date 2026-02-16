<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { uiState } from '../stores/uiState.svelte.js';
  import { 
    ChevronRight, ChevronDown, Folder, FolderOpen, 
    File, FileCode, FileJson, Loader2, FilePlus, Pencil, Trash2,
    Code, FileText, Image as ImageIcon, Box
  } from 'lucide-svelte';
  import { cn } from '../utils.js';
  import FileTreeItem from './FileTreeItem.svelte';

  let { item, depth = 0 } = $props();

  let expanded = $state(depth === 0); // Root expanded by default
  let children = $state(item.children || null);
  let loading = $state(false);

  // Advanced icons based on file type
  function getFileIcon(name) {
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'json': return { icon: FileJson, color: 'text-yellow-400' };
      case 'js': case 'jsx': return { icon: FileCode, color: 'text-yellow-300' };
      case 'ts': case 'tsx': return { icon: FileCode, color: 'text-blue-400' };
      case 'svelte': return { icon: FileCode, color: 'text-orange-500' };
      case 'rs': return { icon: Code, color: 'text-orange-400' };
      case 'css': case 'scss': return { icon: FileText, color: 'text-blue-500' };
      case 'html': return { icon: Code, color: 'text-orange-600' };
      case 'md': return { icon: FileText, color: 'text-muted-foreground' };
      case 'png': case 'jpg': case 'jpeg': case 'svg': return { icon: ImageIcon, color: 'text-pink-400' };
      case 'toml': case 'yaml': case 'yml': return { icon: Box, color: 'text-purple-400' };
      default: return { icon: File, color: 'text-muted-foreground/60' };
    }
  }

  async function toggleExpand(e) {
    e.stopPropagation();
    if (!item.is_dir) {
      fsStore.openFile(item.path);
      return;
    }

    expanded = !expanded;

    if (expanded && !children) {
      loading = true;
      try {
        children = await invoke('list_directory', { path: item.path }).catch(() => []);
      } catch (err) {
        console.error("Failed to list dir", err);
        children = [];
      } finally {
        loading = false;
      }
    }
  }

  function handleContextMenu(e) {
    e.preventDefault();
    const actions = [
      { 
        label: 'New File', 
        icon: FilePlus, 
        fn: async (i) => {
            const name = window.prompt("Enter file name:");
            if (!name) return;
            const dir = i.is_dir ? i.path : i.path.substring(0, i.path.lastIndexOf('/'));
            const path = `${dir}/${name}`;
            await invoke('create_file', { path, content: '' });
        } 
      },
      { 
        label: 'New Folder', 
        icon: Folder, 
        fn: async (i) => {
            const name = window.prompt("Enter folder name:");
            if (!name) return;
            const dir = i.is_dir ? i.path : i.path.substring(0, i.path.lastIndexOf('/'));
            const path = `${dir}/${name}`;
            await invoke('create_directory', { path });
        } 
      },
      { separator: true },
      { 
        label: 'Rename', 
        icon: Pencil, 
        fn: async (i) => {
            const name = window.prompt("Enter new name:", i.name);
            if (!name || name === i.name) return;
            const parent = i.path.substring(0, i.path.lastIndexOf('/'));
            const newPath = `${parent}/${name}`;
            await invoke('rename_path', { oldPath: i.path, newPath });
        } 
      },
      { 
        label: 'Delete', 
        icon: Trash2, 
        danger: true, 
        fn: async (i) => {
            if (!confirm(`Delete ${i.name}?`)) return;
            await invoke('delete_path', { path: i.path });
        } 
      },
    ];
    uiState.openContextMenu(e, item, actions);
  }

  const isActive = $derived(fsStore.activeFile?.path === item.path);
</script>

<div class="animate-in fade-in duration-300">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class={cn(
      "w-full flex items-center gap-2 py-1 px-3 hover:bg-muted/50 transition-all text-[13px] select-none group cursor-pointer relative",
      isActive ? "bg-primary/10 text-primary font-medium" : "text-muted-foreground/90 hover:text-foreground"
    )}
    style="padding-left: {depth * 14 + 12}px"
    onclick={toggleExpand}
    oncontextmenu={handleContextMenu}
  >
    <!-- Highlight Line -->
    {#if isActive}
       <div class="absolute left-0 top-0 bottom-0 w-1 bg-primary shadow-[0_0_10px_hsl(var(--primary))]"></div>
    {/if}

    {#if item.is_dir}
      <div class="flex items-center gap-2 flex-1 min-w-0">
        <span class={cn("transition-transform duration-200", expanded ? "rotate-0" : "-rotate-90")}>
           <ChevronDown size={14} class="opacity-40" />
        </span>
        <div class="relative shrink-0">
          {#if expanded}
            <FolderOpen size={16} class="text-primary shrink-0 transition-transform group-hover:scale-110" />
          {:else}
            <Folder size={16} class="text-primary shrink-0 transition-transform group-hover:scale-110" />
          {/if}
          {#if loading}
            <div class="absolute -top-1 -right-1 bg-background rounded-full p-0.5">
               <Loader2 size={8} class="animate-spin text-primary" />
            </div>
          {/if}
        </div>
        <span class="truncate">{item.name}</span>
      </div>
    {:else}
      {@const iconInfo = getFileIcon(item.name)}
      <div class="flex items-center gap-2 flex-1 min-w-0">
        <span class="w-3.5 shrink-0"></span> <!-- Placeholder for chevron space -->
        <iconInfo.icon size={16} class={cn("shrink-0 transition-all group-hover:scale-110", iconInfo.color)} />
        <span class="truncate">{item.name}</span>
      </div>
    {/if}
    
    {#if item.is_dir && !loading}
       <span class="text-[10px] opacity-0 group-hover:opacity-40 transition-opacity font-bold uppercase ml-auto">{children?.length || 0}</span>
    {/if}
  </div>

  {#if expanded && children}
    <div class="relative">
      <!-- Vertical guide lines -->
      <div 
        class="absolute left-[20px] top-0 bottom-0 w-px bg-border/20 transition-colors group-hover:bg-border/40"
        style="left: {depth * 14 + 19}px"
      ></div>
      
      <div class="space-y-[1px]">
        {#each children as child}
          <FileTreeItem item={child} depth={depth + 1} />
        {/each}
        {#if children.length === 0}
          <div 
            class="py-1 px-4 text-[11px] text-muted-foreground/30 italic font-medium" 
            style="padding-left: {(depth + 1) * 14 + 12}px"
          >
            (Empty)
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
