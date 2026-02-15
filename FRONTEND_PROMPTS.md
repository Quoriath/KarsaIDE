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

---

## NEW: Kilo Models Fetch & Session Management

### 11. Fetch Kilo Models (Onboarding.svelte & Settings.svelte)

```
Implementasikan fetch models dari Kilo Code API:

1. Di Onboarding.svelte, saat user pilih provider "Kilo" dan input API key:
   - Add button "Fetch Models" next to API key input
   - onClick: call invoke('fetch_kilo_models', { apiKey: apiKey })
   - Response: array of { id, name, provider }
   - Populate dropdown dengan models yang di-fetch
   - Show loading spinner saat fetching
   - Error handling jika fetch gagal

2. Di Settings.svelte AI tab:
   - Same functionality: Fetch Models button
   - Refresh models list
   - Cache models in localStorage untuk 1 jam
   - Show last fetched time

3. Model selection:
   - Dropdown dengan search/filter
   - Show model name + id
   - Save selected model ke config
   - Persist selection across sessions

State: availableModels = $state([]), isFetchingModels = $state(false)
```

### 12. Session Persistence (App.svelte & fileSystem store)

```
Implementasikan session management untuk persist workspace state:

1. Di App.svelte onMount:
   - Call invoke('get_session')
   - Load last_workspace, open_files, active_file
   - Restore workspace state automatically

2. Di fileSystem.svelte.js store:
   - Setiap open file: update session via invoke('save_session', session)
   - Setiap close file: update session
   - Setiap change active file: update session
   - Debounce save (500ms) untuk performa

3. Session data structure:
   {
     last_workspace: "/path/to/project",
     open_files: ["/path/file1.js", "/path/file2.rs"],
     active_file: "/path/file1.js",
     recent_workspaces: [
       { path: "/path", name: "Project", last_opened: timestamp }
     ]
   }

4. Recent workspaces:
   - Add workspace saat open folder
   - Limit 10 recent workspaces
   - Sort by last_opened (descending)
   - Show in Dashboard.svelte

5. Auto-restore on startup:
   - Reopen last workspace
   - Reopen all files yang open sebelumnya
   - Set active file
   - Show notification: "Restored session"

Error handling: Jika file tidak exist, skip dan remove dari session
```

### 13. Settings Persistence (Settings.svelte)

```
Implementasikan full settings persistence:

1. Load settings onMount:
   - invoke('get_ai_config') → get full config
   - Populate all form fields
   - Show current values

2. Save settings onChange:
   - Debounce 1 second
   - invoke('save_ai_config', config)
   - Show toast: "Settings saved"

3. Settings structure:
   {
     ai: {
       provider: "kilo",
       api_key: "sk-...",
       base_url: "https://...",
       model_name: "model-id",
       custom_models: []
     },
     editor: {
       font_size: 14,
       tab_size: 2,
       word_wrap: true,
       theme: "dark",
       auto_save: true,
       auto_save_delay: 1000
     },
     terminal: {
       shell: "/bin/bash",
       font_size: 13,
       scrollback: 1000
     },
     session: { ... }
   }

4. Validation:
   - API key format check
   - Font size range (10-24)
   - Tab size options (2, 4, 8)
   - Auto-save delay range (500-5000ms)

5. Reset to defaults:
   - Button "Reset to Defaults"
   - Confirm dialog
   - Restore default config
   - Reload UI

UI: Form dengan sections, save indicator, validation errors
```

---

## Implementation Order (Updated):

1. **Critical:** Fetch Kilo Models, Session Persistence
2. **High:** Settings Persistence, Open Folder, File Operations
3. **Medium:** Chat History, Command Palette, Status Bar
4. **Low:** Git Integration, Search & Replace

