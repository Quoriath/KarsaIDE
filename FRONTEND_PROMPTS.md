# 📋 Frontend Implementation Prompts

## Priority 1: Core Functionality

### 1. Open Folder & File Tree (Sidebar.svelte)
Implementasikan open folder dengan recursive file tree. Import open dari '@tauri-apps/plugin-dialog', call open({ directory: true }), lalu invoke('list_directory'). Tampilkan tree dengan folders (chevron expand/collapse), files (icon by extension), nested indentation. Add right-click menu (New File, Delete, Rename). Lazy load subdirectories on expand. Show file count, highlight active file. State: expandedFolders = new Set().

### 2. File Operations (fileSystem store)
Tambahkan CRUD di fileSystem.svelte.js: createFile(path, content) → invoke('write_file_content'), deleteFile(path) → invoke('delete_path') dengan confirm dialog, renameFile, createFolder, moveFile, copyFile. Add undo stack, toast notifications, auto-save dengan debounce. Error handling dengan user-friendly messages.

### 3. Settings Panel (Settings.svelte - NEW)
Buat Settings component dengan tabs: General (theme, font, auto-save), AI (provider, API key, model, temperature), Editor (tab size, word wrap, line numbers), Terminal (shell, font size), Shortcuts (keybindings). Save via invoke('save_ai_config'). UI: sidebar tabs + form controls.

## Priority 2: Enhanced UX

### 4. Chat History (AgentView.svelte)
Add localStorage persistence: load history onMount, save on messages change. Limit 100 messages. Add "Clear History" & "Export Chat" buttons. Show timestamps, group by date. Sidebar dengan conversation list.

### 5. Command Palette (CommandPalette.svelte - NEW)
Modal dengan fuzzy search (Ctrl+Shift+P). Commands: Open File, New File, Save All, Toggle Terminal, Change Theme, AI Ask. Show shortcuts, recent commands, category grouping. Keyboard navigation, execute on Enter.

### 6. Status Bar (StatusBar.svelte - NEW)
Bottom bar dengan: Left (Git branch, encoding, language), Center (line:col, selection), Right (AI status, terminal status, notifications). Height 24px, clickable items.

## Priority 3: Advanced Features

### 7. Notifications (Notifications.svelte - NEW)
Toast system: types (success, error, warning, info), auto-dismiss (3s), manual close, stack max 5, top-right position, slide animations. Usage: notificationStore.add({ type, title, message }).

### 8. Search & Replace (Search.svelte - NEW)
Panel (Ctrl+F): search input, regex/case/whole word toggles, results count, next/prev, highlight. Replace (Ctrl+H): replace input, Replace/Replace All. Search in files (Ctrl+Shift+F): across files, filters, results by file.

### 9. Git Integration (Git.svelte - NEW)
Detect .git, show branch in status bar. File indicators (M/A/D/U). Source Control panel: staged/unstaged, commit message, push/pull. Diff viewer, branch switcher. Use invoke('shell_execute', { cmd: 'git status' }).

### 10. Recent Workspaces (Dashboard.svelte)
Store recentWorkspaces in localStorage. Grid of recent folders dengan last opened time, quick actions (Open, Remove, Pin). Recent files section. Quick actions: New File, Open Folder. Tips carousel.

---

## Implementation Notes:
- Svelte 5 runes: $state, $derived, $effect
- Lucide icons for all UI
- Colors: #1e1e1e, #252526, #2d2d2d
- Error handling & loading states
- Keyboard shortcuts
- Responsive & accessible (ARIA)
