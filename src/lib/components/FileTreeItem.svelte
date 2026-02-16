<script>
  import { invoke } from '@tauri-apps/api/core';
  import { fsStore } from '../stores/fileSystem.svelte.js';
  import { uiState } from '../stores/uiState.svelte.js';
  import { 
    ChevronRight, ChevronDown, Folder, FolderOpen, 
    File, FileCode, FileJson, Loader2, FilePlus, Pencil, Trash2,
    FileText, Hash, Image as ImageIcon, Braces, Copy, ExternalLink,
    Terminal, Scissors, Clipboard
  } from 'lucide-svelte';
  import { cn } from '../utils.js';

  let { item, depth = 0 } = $props();

  let expanded = $state(depth === 0);
  let children = $state(item.children || null);
  let loading = $state(false);
  let isDragOver = $state(false);

  function getFileIcon(name) {
    const ext = name.split('.').pop()?.toLowerCase();
    switch (ext) {
      case 'json': case 'lock': return FileJson;
      case 'js': case 'ts': case 'jsx': case 'tsx': case 'svelte': return FileCode;
      case 'rs': case 'go': case 'py': case 'cpp': case 'c': return Braces;
      case 'css': case 'scss': case 'less': return Hash;
      case 'md': case 'txt': return FileText;
      case 'png': case 'jpg': case 'jpeg': case 'svg': return ImageIcon;
      default: return File;
    }
  }

  async function toggleExpand(e) {
    e?.stopPropagation();
    if (!item.is_dir) {
      fsStore.openFile(item.path);
      return;
    }

    expanded = !expanded;

    if (expanded && !children) {
      loading = true;
      try {
        children = await invoke('list_directory', { path: item.path });
      } catch (err) {
        console.error("Failed to list dir", err);
        children = [];
      } finally {
        loading = false;
      }
    }
  }

  // --- ACTIONS ---

  async function handleNewFile() {
    const name = window.prompt("New File Name:");
    if (!name) return;
    const parent = item.is_dir ? item.path : item.path.substring(0, item.path.lastIndexOf('/'));
    await fsStore.createFile(`${parent}/${name}`);
  }

  async function handleNewFolder() {
    const name = window.prompt("New Folder Name:");
    if (!name) return;
    const parent = item.is_dir ? item.path : item.path.substring(0, item.path.lastIndexOf('/'));
    await fsStore.createDirectory(`${parent}/${name}`);
  }

  async function handleRename() {
    const newName = window.prompt("Rename to:", item.name);
    if (!newName || newName === item.name) return;
    const parent = item.path.substring(0, item.path.lastIndexOf('/'));
    await fsStore.renamePath(item.path, `${parent}/${newName}`);
  }

  async function handleDelete() {
    uiState.showDialog({
      title: 'Delete Item',
      message: `Permanently delete '${item.name}'?`,
      confirmLabel: 'Delete',
      variant: 'danger',
      onConfirm: () => fsStore.deletePath(item.path)
    });
  }

  function handleCopyPath() { navigator.clipboard.writeText(item.path); }
  function handleCopyRelativePath() { navigator.clipboard.writeText(fsStore.getRelativePath(item.path)); }
  
  function handleCut() { fsStore.cutToClipboard(item.path); }
  function handleCopy() { fsStore.copyToClipboard(item.path); }
  
  async function handlePaste() {
    const dest = item.is_dir ? item.path : item.path.substring(0, item.path.lastIndexOf('/'));
    await fsStore.pasteFromClipboard(dest);
  }

  function handleContextMenu(e) {
    e.preventDefault();
    const canPaste = fsStore.clipboard.path !== null;
    
    const actions = [
      { label: 'New File', icon: FilePlus, fn: handleNewFile },
      { label: 'New Folder', icon: Folder, fn: handleNewFolder },
      { separator: true },
      { label: 'Cut', icon: Scissors, fn: handleCut },
      { label: 'Copy', icon: Copy, fn: handleCopy },
      { label: 'Paste', icon: Clipboard, fn: handlePaste, disabled: !canPaste },
      { separator: true },
      { label: 'Rename', icon: Pencil, fn: handleRename },
      { label: 'Delete', icon: Trash2, danger: true, fn: handleDelete },
      { separator: true },
      { label: 'Copy Path', icon: Copy, fn: handleCopyPath },
      { label: 'Copy Relative Path', icon: Copy, fn: handleCopyRelativePath },
      { separator: true },
      { label: 'Reveal in File Explorer', icon: ExternalLink, fn: (i) => fsStore.revealInExplorer(i.path) },
    ];
    uiState.openContextMenu(e, item, actions);
  }

  // --- DRAG AND DROP ---

  function onDragStart(e) {
    uiState.draggedItem = item;
    e.dataTransfer.setData('text/plain', item.path);
    e.dataTransfer.effectAllowed = 'move';
    const dragImg = document.createElement('div');
    dragImg.className = 'bg-primary/20 p-2 rounded border border-primary/40 text-[10px] text-foreground font-bold uppercase fixed -top-[1000px]';
    dragImg.innerText = item.name;
    document.body.appendChild(dragImg);
    e.dataTransfer.setDragImage(dragImg, 0, 0);
    setTimeout(() => dragImg.remove(), 0);
  }

  function onDragOver(e) {
    if (!item.is_dir || uiState.draggedItem?.path === item.path) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    isDragOver = true;
  }

  function onDragLeave() {
    isDragOver = false;
  }

  async function onDrop(e) {
    e.preventDefault();
    isDragOver = false;
    const sourcePath = uiState.draggedItem?.path;
    if (!sourcePath) return;
    
    const itemName = sourcePath.split(/[/\\]/).pop();
    const targetPath = `${item.path}/${itemName}`;
    if (sourcePath === targetPath) return;

    uiState.showDialog({
      title: 'Move Item',
      message: `Move '${itemName}' into '${item.name}'?`,
      confirmLabel: 'Move',
      onConfirm: async () => {
        try { await fsStore.movePath(sourcePath, targetPath); } 
        catch (err) { alert(`Move failed: ${err}`); }
      }
    });
    uiState.draggedItem = null;
  }

  const isActive = $derived(fsStore.activeFile?.path === item.path);
  const isCut = $derived(fsStore.clipboard.operation === 'cut' && fsStore.clipboard.path === item.path);
</script>

<div 
  class={cn("group/item select-none relative", isCut ? "opacity-50" : "")}
  draggable="true"
  ondragstart={onDragStart}
  ondragover={onDragOver}
  ondragleave={onDragLeave}
  ondrop={onDrop}
>
  <button
    class={cn(
      "w-full flex items-center gap-2 py-1 pr-2 transition-all duration-200 border-l-2",
      isActive 
        ? "bg-primary/10 text-primary border-primary" 
        : "text-muted-foreground/80 hover:bg-muted/30 border-transparent hover:text-foreground",
      isDragOver ? "bg-primary/20 border-primary outline-1 outline-dashed outline-primary/50" : ""
    )}
    style="padding-left: {depth * 12 + 12}px"
    onclick={toggleExpand}
    oncontextmenu={handleContextMenu}
  >
    <div class="flex items-center gap-2 min-w-0 pointer-events-none">
      {#if item.is_dir}
        <div class="flex items-center justify-center w-4 h-4">
          <span class={cn("transition-transform duration-200", expanded ? "rotate-0" : "-rotate-90")}>
            <ChevronDown size={12} class="opacity-50" />
          </span>
        </div>
        {#if expanded}
          <FolderOpen size={15} class="text-primary/70 shrink-0" />
        {:else}
          <Folder size={15} class="text-primary/70 shrink-0" />
        {/if}
      {:else}
        <div class="w-4 shrink-0"></div>
        {@const IconCmp = getFileIcon(item.name)}
        <IconCmp size={15} class={cn("shrink-0 transition-opacity", isActive ? "opacity-100" : "opacity-60 group-hover/item:opacity-90")} />
      {/if}

      <span class={cn("truncate text-[12.5px] tracking-tight", isActive ? "font-semibold" : "font-medium")}>
        {item.name}
      </span>
    </div>
    
    {#if loading}
      <Loader2 size={10} class="animate-spin ml-auto opacity-40" />
    {/if}
  </button>

  {#if expanded && children}
    <div class="animate-in fade-in slide-in-from-top-1 duration-200">
      {#each children as child}
        <svelte:self item={child} depth={depth + 1} />
      {/each}
      {#if children.length === 0}
        <div class="py-1 px-4 text-[11px] text-muted-foreground/40 italic" style="padding-left: {(depth + 1) * 12 + 32}px">
          No items
        </div>
      {/if}
    </div>
  {/if}
</div>
