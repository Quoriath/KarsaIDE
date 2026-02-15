# 📚 Tauri Commands API Reference

## File Operations

### `read_file_content(path: string): Promise<string>`
Read file content as string.
```javascript
const content = await invoke('read_file_content', { path: '/path/to/file.txt' });
```

### `write_file_content(path: string, content: string): Promise<void>`
Write content to file (creates parent directories if needed).
```javascript
await invoke('write_file_content', { path: '/path/to/file.txt', content: 'Hello World' });
```

### `list_directory(path: string): Promise<FileEntry[]>`
List directory contents (sorted: folders first, then files alphabetically).
```javascript
const entries = await invoke('list_directory', { path: '/path/to/dir' });
// Returns: [{ name: "folder", path: "/path/to/dir/folder", is_dir: true }, ...]
```

### `file_exists(path: string): Promise<boolean>`
Check if file or directory exists.
```javascript
const exists = await invoke('file_exists', { path: '/path/to/file' });
```

### `create_directory(path: string): Promise<void>`
Create directory (creates parent directories if needed).
```javascript
await invoke('create_directory', { path: '/path/to/new/dir' });
```

### `create_file(path: string, content?: string): Promise<void>`
Create new file with optional content.
```javascript
await invoke('create_file', { path: '/path/to/new/file.txt', content: 'Initial content' });
```

### `delete_path(path: string): Promise<void>`
Delete file or directory (recursive for directories).
```javascript
await invoke('delete_path', { path: '/path/to/delete' });
```

### `rename_path(oldPath: string, newPath: string): Promise<void>`
Rename or move file/directory.
```javascript
await invoke('rename_path', { oldPath: '/old/path', newPath: '/new/path' });
```

### `copy_path(source: string, destination: string): Promise<void>`
Copy file or directory (recursive for directories).
```javascript
await invoke('copy_path', { source: '/source/path', destination: '/dest/path' });
```

---

## Terminal (PTY)

### `spawn_terminal(id: string, shell?: string): Promise<void>`
Spawn new terminal with PTY. Shell defaults to $SHELL env var or /bin/bash.
```javascript
await invoke('spawn_terminal', { id: 'terminal-1', shell: null });
```

### `write_to_terminal(id: string, data: number[]): Promise<void>`
Write data to terminal stdin (data is array of bytes).
```javascript
const encoder = new TextEncoder();
const bytes = encoder.encode('ls -la\n');
await invoke('write_to_terminal', { id: 'terminal-1', data: Array.from(bytes) });
```

### Event: `terminal-output`
Listen for terminal stdout/stderr output.
```javascript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('terminal-output', (event) => {
  const bytes = event.payload; // number[]
  const text = new TextDecoder().decode(new Uint8Array(bytes));
  console.log('Terminal output:', text);
});

// Cleanup
unlisten();
```

---

## AI / Chat

### `get_ai_config(): Promise<KarsaConfig>`
Get full configuration including AI, editor, terminal, session.
```javascript
const config = await invoke('get_ai_config');
// Returns: { ai: {...}, editor: {...}, terminal: {...}, session: {...} }
```

### `save_ai_config(config: KarsaConfig): Promise<void>`
Save full configuration to disk.
```javascript
await invoke('save_ai_config', { config: updatedConfig });
```

### `fetch_kilo_models(apiKey?: string): Promise<ModelInfo[]>`
Fetch available models from Kilo Code API.
```javascript
const models = await invoke('fetch_kilo_models', { apiKey: 'sk-...' });
// Returns: [{ id: "model-id", name: "Model Name", provider: "kilo" }, ...]
```

### `test_ai_connection(config: AIConfig): Promise<string>`
Test AI provider connection.
```javascript
const result = await invoke('test_ai_connection', { config: aiConfig });
// Returns: "Connection successful"
```

### `send_chat_completion(messages: ChatMessage[], config: AIConfig): Promise<string>`
Send chat completion request (non-streaming).
```javascript
const response = await invoke('send_chat_completion', {
  messages: [{ role: 'user', content: 'Hello' }],
  config: aiConfig
});
```

### `send_chat_completion_stream(messages: ChatMessage[], config: AIConfig): Promise<void>`
Send chat completion request with streaming response.
```javascript
await invoke('send_chat_completion_stream', {
  messages: [{ role: 'user', content: 'Hello' }],
  config: aiConfig
});
```

### Event: `ai-stream-chunk`
Listen for AI response chunks (streaming).
```javascript
const unlisten = await listen('ai-stream-chunk', (event) => {
  const chunk = event.payload; // string
  console.log('AI chunk:', chunk);
});
```

### Event: `ai-stream-done`
Listen for AI stream completion.
```javascript
const unlisten = await listen('ai-stream-done', () => {
  console.log('AI stream finished');
});
```

---

## Session Management

### `get_session(): Promise<SessionData>`
Get current session data (workspace, open files, etc).
```javascript
const session = await invoke('get_session');
// Returns: { last_workspace, open_files, active_file, recent_workspaces }
```

### `save_session(session: SessionData): Promise<void>`
Save session data to disk.
```javascript
await invoke('save_session', {
  session: {
    last_workspace: '/path/to/project',
    open_files: ['/path/file1.js'],
    active_file: '/path/file1.js',
    recent_workspaces: [{ path: '/path', name: 'Project', last_opened: Date.now() }]
  }
});
```

---

## Type Definitions

```typescript
interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
}

interface ModelInfo {
  id: string;
  name: string;
  provider: string;
}

interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

interface AIConfig {
  provider: 'kilo' | 'ollama' | 'openai' | 'custom';
  api_key?: string;
  base_url: string;
  model_name: string;
  custom_models: string[];
}

interface EditorConfig {
  font_size: number;
  tab_size: number;
  word_wrap: boolean;
  theme: string;
  auto_save: boolean;
  auto_save_delay: number;
}

interface TerminalConfig {
  shell?: string;
  font_size: number;
  scrollback: number;
}

interface SessionData {
  last_workspace?: string;
  open_files: string[];
  active_file?: string;
  recent_workspaces: RecentWorkspace[];
}

interface RecentWorkspace {
  path: string;
  name: string;
  last_opened: number;
}

interface KarsaConfig {
  ai: AIConfig;
  editor: EditorConfig;
  terminal: TerminalConfig;
  session: SessionData;
}
```

---

## Error Handling

All commands return `Promise<T>` or `Promise<void>`. Errors are thrown as strings.

```javascript
try {
  await invoke('delete_path', { path: '/some/path' });
} catch (error) {
  console.error('Failed to delete:', error);
  // error is a string like "Failed to delete file: Permission denied"
}
```

---

## Event Cleanup

Always unlisten from events when component unmounts:

```javascript
import { onDestroy } from 'svelte';
import { listen } from '@tauri-apps/api/event';

let unlisten;

onMount(async () => {
  unlisten = await listen('terminal-output', handleOutput);
});

onDestroy(() => {
  if (unlisten) unlisten();
});
```
