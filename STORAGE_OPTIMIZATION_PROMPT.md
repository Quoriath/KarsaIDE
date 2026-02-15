# 🗄️ Storage & Optimization Implementation

## Backend DONE ✅

1. ✅ SQLite database untuk chat history
2. ✅ Stale-while-revalidate cache untuk models
3. ✅ Secure config storage (platform-specific)
4. ✅ UI state events (toggle_terminal, toggle_chat)

## Frontend Implementation

### 1. Fix Hardcoded Models di AgentView

```svelte
<!-- AgentView.svelte -->
<script>
  import { invoke } from '@tauri-apps/api/core';
  import ModelSelector from './ModelSelector.svelte';
  
  let availableModels = $state([]);
  let selectedModel = $state(configStore.settings.ai.selectedModel);
  
  onMount(async () => {
    // Fetch models (uses cache automatically)
    try {
      availableModels = await invoke('fetch_kilo_models', { 
        apiKey: configStore.settings.ai.apiKey,
        forceRefresh: false  // Use cache if available
      });
    } catch (e) {
      console.error('Failed to fetch models:', e);
    }
  });
  
  // Replace hardcoded dropdown with:
  <ModelSelector 
    bind:selectedModel 
    models={availableModels}
    onModelChange={(model) => {
      configStore.updateAiConfig({ selectedModel: model.id });
    }}
  />
</script>
```

### 2. Chat History dengan Database

```svelte
<!-- ChatPanel.svelte -->
<script>
  let conversations = $state([]);
  let activeConversationId = $state(null);
  
  async function loadConversations() {
    conversations = await invoke('get_conversations', {
      mode: 'editor',  // or 'agent'
      limit: 50
    });
  }
  
  async function createNewConversation() {
    const id = await invoke('create_conversation', {
      mode: 'editor',
      title: 'New Chat',
      contextPath: fsStore.activeFile?.path,
      model: selectedModel
    });
    activeConversationId = id;
  }
  
  async function saveMessage(role, content) {
    await invoke('add_message', {
      conversationId: activeConversationId,
      role,
      content
    });
  }
  
  async function loadMessages(conversationId) {
    const msgs = await invoke('get_messages', { conversationId });
    messages = msgs;
  }
  
  onMount(() => {
    loadConversations();
  });
</script>

<!-- UI: Sidebar dengan conversation list -->
<div class="flex">
  <div class="w-64 border-r">
    <button onclick={createNewConversation}>New Chat</button>
    {#each conversations as conv}
      <button onclick={() => loadMessages(conv.id)}>
        {conv.title}
      </button>
    {/each}
  </div>
  <div class="flex-1">
    <!-- Messages -->
  </div>
</div>
```

### 3. UI State Events

```svelte
<!-- App.svelte -->
<script>
  import { listen } from '@tauri-apps/api/event';
  
  let terminalVisible = $state(true);
  let chatVisible = $state(false);
  
  onMount(async () => {
    await listen('terminal-state-changed', (event) => {
      terminalVisible = event.payload;
    });
    
    await listen('chat-state-changed', (event) => {
      chatVisible = event.payload;
    });
  });
  
  async function toggleTerminal() {
    terminalVisible = !terminalVisible;
    await invoke('toggle_terminal', { visible: terminalVisible });
  }
  
  async function toggleChat() {
    chatVisible = !chatVisible;
    await invoke('toggle_chat', { visible: chatVisible });
  }
</script>
```

### 4. Model Fetch dengan Force Refresh

```svelte
<button onclick={async () => {
  isFetching = true;
  try {
    const models = await invoke('fetch_kilo_models', {
      apiKey: configStore.settings.ai.apiKey,
      forceRefresh: true  // Bypass cache
    });
    availableModels = models;
  } finally {
    isFetching = false;
  }
}}>
  <RefreshCw class={isFetching ? 'animate-spin' : ''} />
  Refresh Models
</button>
```

---

## Storage Locations

### Linux
- Config: `~/.config/karsa-ide/karsa_config.json`
- Database: `~/.local/share/karsa-ide/chat_history.db`
- Cache: `~/.cache/karsa-ide/kilo_models.json`

### Windows
- Config: `%APPDATA%\karsa-ide\karsa_config.json`
- Database: `%LOCALAPPDATA%\karsa-ide\chat_history.db`
- Cache: `%LOCALAPPDATA%\karsa-ide\kilo_models.json`

### macOS
- Config: `~/Library/Application Support/karsa-ide/karsa_config.json`
- Database: `~/Library/Application Support/karsa-ide/chat_history.db`
- Cache: `~/Library/Caches/karsa-ide/kilo_models.json`

---

## API Reference

### Database Commands

```typescript
// Create conversation
const id = await invoke('create_conversation', {
  mode: 'editor' | 'agent',
  title: string,
  contextPath?: string,
  model: string
}): Promise<number>

// Get conversations
const conversations = await invoke('get_conversations', {
  mode?: 'editor' | 'agent',
  limit?: number
}): Promise<Conversation[]>

// Add message
const msgId = await invoke('add_message', {
  conversationId: number,
  role: 'user' | 'assistant',
  content: string
}): Promise<number>

// Get messages
const messages = await invoke('get_messages', {
  conversationId: number
}): Promise<Message[]>

// Delete conversation
await invoke('delete_conversation', { id: number })
```

### Cache Commands

```typescript
// Fetch models (with cache)
const models = await invoke('fetch_kilo_models', {
  apiKey?: string,
  forceRefresh?: boolean  // Default: false
}): Promise<ModelInfo[]>
```

### UI State Commands

```typescript
// Toggle terminal
await invoke('toggle_terminal', { visible: boolean })

// Toggle chat
await invoke('toggle_chat', { visible: boolean })

// Listen to events
await listen('terminal-state-changed', (event) => {
  console.log('Terminal visible:', event.payload);
});

await listen('chat-state-changed', (event) => {
  console.log('Chat visible:', event.payload);
});
```

---

## Implementation Checklist

- [ ] Remove hardcoded models from AgentView
- [ ] Add ModelSelector component to AgentView
- [ ] Implement conversation list UI
- [ ] Add create/load/delete conversation
- [ ] Save messages to database
- [ ] Load messages from database
- [ ] Add UI state event listeners
- [ ] Fix close button with toggle commands
- [ ] Add force refresh button for models
- [ ] Show cache status (last updated)
- [ ] Add conversation search/filter
- [ ] Add export conversation feature

---

## Security Notes

✅ **API Keys:** Stored in platform-specific secure location
✅ **Database:** Local SQLite, no network access
✅ **Cache:** Temporary, can be cleared
✅ **Config:** JSON with proper permissions (600)

Backend handles all storage securely! 🔒
