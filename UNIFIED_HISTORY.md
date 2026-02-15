# 🎨 UI Fixes & Unified Chat History

## Fixed Issues ✅

### 1. Double Title Bar
**Problem:** Native OS title bar + Custom title bar showing together

**Solution:** Disable native window decorations
```json
// tauri.conf.json
{
  "app": {
    "windows": [{
      "decorations": false  // ← Added this
    }]
  }
}
```

**Result:**
- ✅ Only custom title bar visible
- ✅ Clean, modern look
- ✅ Custom minimize/maximize/close buttons work
- ✅ Drag-to-move still works

---

### 2. Unified Chat History

**Problem:** 
- Agent mode has history UI
- Editor chat has NO history UI
- Both should share same database

**Solution:** Add history sidebar to ChatPanel (Editor mode)

---

## Unified Chat History Architecture

```
┌─────────────────────────────────────────┐
│         SQLite Database                 │
│  ┌────────────────────────────────────┐ │
│  │     conversations table            │ │
│  │  - mode: 'editor' | 'agent'        │ │
│  │  - title, context_path, model      │ │
│  │  - created_at, updated_at          │ │
│  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────┐ │
│  │     messages table                 │ │
│  │  - conversation_id (FK)            │ │
│  │  - role: 'user' | 'assistant'      │ │
│  │  - content, timestamp              │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
           ▲                    ▲
           │                    │
    ┌──────┴──────┐      ┌─────┴──────┐
    │ Editor Chat │      │ Agent Chat │
    │ (mode='editor')    │ (mode='agent')
    └─────────────┘      └────────────┘
```

### Same Database, Different Modes

**Editor Chat:**
```typescript
await invoke('create_conversation', {
  mode: 'editor',  // ← Editor mode
  title: 'Chat 2026-02-15',
  contextPath: '/path/to/file.js',
  model: 'minimax/minimax-m2.5:free'
});
```

**Agent Chat:**
```typescript
await invoke('create_conversation', {
  mode: 'agent',  // ← Agent mode
  title: 'Agent Session',
  contextPath: null,
  model: 'minimax/minimax-m2.5:free'
});
```

### Query by Mode

```typescript
// Get only editor conversations
const editorChats = await invoke('get_conversations', {
  mode: 'editor',
  limit: 50
});

// Get only agent conversations
const agentChats = await invoke('get_conversations', {
  mode: 'agent',
  limit: 50
});

// Get all conversations
const allChats = await invoke('get_conversations', {
  mode: null,  // No filter
  limit: 100
});
```

---

## ChatPanel (Editor) Features

### New UI Elements

1. **History Button** 📜
   - Toggle conversation list
   - Shows all editor mode chats
   - Click to load conversation

2. **New Chat Button** ➕
   - Create new conversation
   - Auto-saves to database
   - Switches to new chat

3. **History Sidebar**
   - Collapsible list
   - Shows title + date
   - Delete button per conversation
   - Active conversation highlighted

### UI Layout

```
┌─────────────────────────────────────┐
│ ✨ Assistant    [📜][➕][⚙️] [✕]   │ ← Header
├─────────────────────────────────────┤
│ 📜 History (Collapsible)            │
│  ┌─────────────────────────────┐   │
│  │ Chat 2026-02-15        [🗑️] │   │ ← Active
│  │ Feb 15, 2026                │   │
│  ├─────────────────────────────┤   │
│  │ Previous Chat          [🗑️] │   │
│  │ Feb 14, 2026                │   │
│  └─────────────────────────────┘   │
├─────────────────────────────────────┤
│ Messages Area                       │
│ ...                                 │
└─────────────────────────────────────┘
```

---

## Implementation Details

### ChatPanel Changes

**Added:**
```typescript
// State
let conversations = $state([]);
let activeConversationId = $state(null);
let showHistory = $state(false);

// Functions
async function loadConversations() { ... }
async function loadConversation(id) { ... }
async function createNewConversation() { ... }
async function deleteConversation(id) { ... }
async function saveMessage(role, content) { ... }
```

**Removed:**
```typescript
// Old localStorage-based history
function saveHistory() { ... }
function exportChat() { ... }
function importChat() { ... }
```

### Database Integration

**On Mount:**
```typescript
onMount(async () => {
  await loadConversations();  // Load from DB
  
  // Load last conversation
  if (conversations.length > 0) {
    await loadConversation(conversations[0].id);
  }
});
```

**On Send Message:**
```typescript
async function sendMessage() {
  // Add user message
  messages = [...messages, { role: 'user', content: userMessage }];
  await saveMessage('user', userMessage);  // ← Save to DB
  
  // Get AI response
  const response = await invoke('send_chat_completion', { ... });
  
  // Add AI message
  messages = [...messages, { role: 'assistant', content: response }];
  await saveMessage('assistant', response);  // ← Save to DB
}
```

---

## Benefits

### 1. Persistent History
✅ Survives app restart
✅ No data loss
✅ Searchable (future)

### 2. Unified System
✅ Same database for both modes
✅ Consistent API
✅ Easy to query across modes

### 3. Better UX
✅ Visual history list
✅ Quick conversation switching
✅ Delete old conversations
✅ See conversation dates

### 4. Scalability
✅ SQLite handles thousands of conversations
✅ Indexed queries (fast)
✅ Foreign key constraints (data integrity)

---

## Database Schema

```sql
CREATE TABLE conversations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mode TEXT NOT NULL,           -- 'editor' | 'agent'
  title TEXT NOT NULL,
  context_path TEXT,            -- File path (editor mode)
  model TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX idx_conversations_mode ON conversations(mode);
CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);

CREATE TABLE messages (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  conversation_id INTEGER NOT NULL,
  role TEXT NOT NULL,           -- 'user' | 'assistant'
  content TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
);

CREATE INDEX idx_messages_conversation ON messages(conversation_id);
```

---

## Future Enhancements

### Search & Filter
```typescript
// Search conversations by title/content
const results = await invoke('search_conversations', {
  query: 'react hooks',
  mode: 'editor'
});
```

### Export/Import
```typescript
// Export conversation
const data = await invoke('export_conversation', { id: 123 });

// Import conversation
await invoke('import_conversation', { data });
```

### Conversation Metadata
```typescript
// Add tags, favorites, etc.
await invoke('update_conversation', {
  id: 123,
  tags: ['react', 'typescript'],
  favorite: true
});
```

### Cross-Mode View
```typescript
// View all conversations (editor + agent)
const allChats = await invoke('get_conversations', {
  mode: null  // No filter
});
```

---

## Testing Checklist

- [x] Disable native title bar
- [x] Custom title bar works
- [x] Add history button to ChatPanel
- [x] Add new chat button
- [x] Implement conversation list UI
- [x] Load conversations from database
- [x] Switch between conversations
- [x] Delete conversations
- [x] Save messages to database
- [x] Auto-create conversation on first message
- [ ] Test on Windows (native title bar)
- [ ] Test on macOS (native title bar)
- [ ] Test with many conversations (performance)

---

## Migration Notes

### For Existing Users

**Old localStorage data:**
- Still accessible in browser DevTools
- Not automatically migrated
- Can be manually imported (future feature)

**New database:**
- Starts fresh
- All new chats saved to SQLite
- Persistent across sessions

---

**UI fixed! History unified! 🎉**
