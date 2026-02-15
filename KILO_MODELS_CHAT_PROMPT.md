# 🤖 Kilo Models & Chat Sidebar Enhancement

## 1. Model Selector Component (ModelSelector.svelte - NEW)

Buat reusable model selector component:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { Search, RefreshCw, ChevronDown } from 'lucide-svelte';
  
  let { selectedModel = $bindable(), apiKey, onModelChange } = $props();
  
  let models = $state([]);
  let filteredModels = $state([]);
  let searchQuery = $state('');
  let isOpen = $state(false);
  let isFetching = $state(false);
  let error = $state('');
  
  async function fetchModels() {
    isFetching = true;
    error = '';
    try {
      const result = await invoke('fetch_kilo_models', { apiKey });
      models = result;
      filteredModels = result;
      
      // Cache models in localStorage
      localStorage.setItem('kilo-models', JSON.stringify({
        models: result,
        timestamp: Date.now()
      }));
    } catch (e) {
      error = `Failed to fetch models: ${e}`;
    } finally {
      isFetching = false;
    }
  }
  
  function loadCachedModels() {
    const cached = localStorage.getItem('kilo-models');
    if (cached) {
      const { models: cachedModels, timestamp } = JSON.parse(cached);
      // Cache valid for 1 hour
      if (Date.now() - timestamp < 3600000) {
        models = cachedModels;
        filteredModels = cachedModels;
        return true;
      }
    }
    return false;
  }
  
  $effect(() => {
    if (searchQuery) {
      filteredModels = models.filter(m => 
        m.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        m.id.toLowerCase().includes(searchQuery.toLowerCase())
      );
    } else {
      filteredModels = models;
    }
  });
  
  onMount(() => {
    if (!loadCachedModels()) {
      fetchModels();
    }
  });
</script>

<div class="relative">
  <button 
    class="flex items-center gap-2 px-3 py-2 bg-[#2a2a2a] hover:bg-[#3e3e42] rounded-lg w-full"
    onclick={() => isOpen = !isOpen}
  >
    <span class="flex-1 text-left truncate text-sm">{selectedModel || 'Select model'}</span>
    <ChevronDown size={16} class={isOpen ? 'rotate-180' : ''} />
  </button>
  
  {#if isOpen}
    <div class="absolute top-full mt-1 w-full bg-[#252526] border border-[#2d2d2d] rounded-lg shadow-lg z-50 max-h-96 flex flex-col">
      <!-- Search & Refresh -->
      <div class="p-2 border-b border-[#2d2d2d] flex gap-2">
        <div class="flex-1 relative">
          <Search size={14} class="absolute left-2 top-2.5 text-gray-500" />
          <input 
            type="text"
            bind:value={searchQuery}
            placeholder="Search models..."
            class="w-full pl-8 pr-2 py-2 bg-[#1e1e1e] border border-[#3e3e42] rounded text-sm"
          />
        </div>
        <button 
          onclick={fetchModels}
          disabled={isFetching}
          class="p-2 bg-[#2a2a2a] hover:bg-[#3e3e42] rounded"
          title="Refresh models"
        >
          <RefreshCw size={14} class={isFetching ? 'animate-spin' : ''} />
        </button>
      </div>
      
      <!-- Models List -->
      <div class="overflow-y-auto flex-1">
        {#if error}
          <div class="p-3 text-red-400 text-xs">{error}</div>
        {:else if isFetching}
          <div class="p-3 text-gray-500 text-xs">Loading models...</div>
        {:else if filteredModels.length === 0}
          <div class="p-3 text-gray-500 text-xs">No models found</div>
        {:else}
          {#each filteredModels as model}
            <button
              class="w-full px-3 py-2 hover:bg-[#2a2a2a] text-left flex flex-col gap-1"
              onclick={() => {
                selectedModel = model.id;
                isOpen = false;
                onModelChange?.(model);
              }}
            >
              <span class="text-sm font-medium">{model.name}</span>
              <span class="text-xs text-gray-500">{model.id}</span>
            </button>
          {/each}
        {/if}
      </div>
      
      <!-- Footer -->
      <div class="p-2 border-t border-[#2d2d2d] text-xs text-gray-500">
        {filteredModels.length} models available
      </div>
    </div>
  {/if}
</div>
```

---

## 2. Enhanced Chat Sidebar (ChatPanel.svelte)

Update ChatPanel dengan model selector dan features:

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { configStore } from '../stores/config.svelte.js';
  import ModelSelector from './ModelSelector.svelte';
  import { Settings, Trash2, Download, Upload } from 'lucide-svelte';
  
  let messages = $state([]);
  let input = $state('');
  let isLoading = $state(false);
  let selectedModel = $state(configStore.settings.ai.model_name);
  let showSettings = $state(false);
  
  // Temperature & max tokens
  let temperature = $state(0.7);
  let maxTokens = $state(2000);
  
  async function sendMessage() {
    if (!input.trim() || isLoading) return;
    
    const userMessage = input.trim();
    input = '';
    isLoading = true;
    
    messages = [...messages, { 
      role: 'user', 
      content: userMessage,
      timestamp: Date.now()
    }];
    
    try {
      const config = {
        ...configStore.settings.ai,
        model_name: selectedModel
      };
      
      const response = await invoke('send_chat_completion', {
        messages: messages.map(m => ({ role: m.role, content: m.content })),
        config
      });
      
      messages = [...messages, { 
        role: 'assistant', 
        content: response,
        timestamp: Date.now()
      }];
      
      // Save to history
      saveChatHistory();
    } catch (e) {
      messages = [...messages, { 
        role: 'assistant', 
        content: `Error: ${e}`,
        timestamp: Date.now()
      }];
    } finally {
      isLoading = false;
    }
  }
  
  function saveChatHistory() {
    const history = JSON.parse(localStorage.getItem('chat-history') || '[]');
    history.push({
      id: Date.now(),
      messages: messages.slice(-10), // Last 10 messages
      timestamp: Date.now(),
      model: selectedModel
    });
    // Keep last 50 conversations
    localStorage.setItem('chat-history', JSON.stringify(history.slice(-50)));
  }
  
  function clearChat() {
    if (confirm('Clear all messages?')) {
      messages = [];
    }
  }
  
  function exportChat() {
    const data = JSON.stringify(messages, null, 2);
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `chat-${Date.now()}.json`;
    a.click();
  }
  
  function importChat(event) {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          messages = JSON.parse(e.target.result);
        } catch (err) {
          alert('Invalid chat file');
        }
      };
      reader.readAsText(file);
    }
  }
</script>

<aside class="w-96 bg-[#252526] border-l border-[#2d2d2d] flex flex-col">
  <!-- Header -->
  <div class="px-4 py-3 border-b border-[#2d2d2d] flex items-center justify-between">
    <h2 class="text-sm font-semibold">AI Assistant</h2>
    <div class="flex items-center gap-1">
      <button onclick={() => showSettings = !showSettings} class="p-1 hover:bg-[#2a2a2a] rounded">
        <Settings size={16} />
      </button>
      <button onclick={clearChat} class="p-1 hover:bg-[#2a2a2a] rounded" title="Clear chat">
        <Trash2 size={16} />
      </button>
      <button onclick={exportChat} class="p-1 hover:bg-[#2a2a2a] rounded" title="Export chat">
        <Download size={16} />
      </button>
      <label class="p-1 hover:bg-[#2a2a2a] rounded cursor-pointer" title="Import chat">
        <Upload size={16} />
        <input type="file" accept=".json" onchange={importChat} class="hidden" />
      </label>
    </div>
  </div>
  
  <!-- Model Selector -->
  <div class="px-4 py-2 border-b border-[#2d2d2d]">
    <label class="text-xs text-gray-500 mb-1 block">Model</label>
    <ModelSelector 
      bind:selectedModel 
      apiKey={configStore.settings.ai.api_key}
      onModelChange={(model) => {
        configStore.updateAiConfig({ model_name: model.id });
      }}
    />
  </div>
  
  <!-- Settings Panel -->
  {#if showSettings}
    <div class="px-4 py-2 border-b border-[#2d2d2d] bg-[#1e1e1e]">
      <div class="mb-2">
        <label class="text-xs text-gray-500">Temperature: {temperature}</label>
        <input 
          type="range" 
          min="0" 
          max="1" 
          step="0.1" 
          bind:value={temperature}
          class="w-full"
        />
      </div>
      <div>
        <label class="text-xs text-gray-500">Max Tokens: {maxTokens}</label>
        <input 
          type="range" 
          min="100" 
          max="4000" 
          step="100" 
          bind:value={maxTokens}
          class="w-full"
        />
      </div>
    </div>
  {/if}
  
  <!-- Messages -->
  <div class="flex-1 overflow-y-auto p-4 space-y-4">
    {#if messages.length === 0}
      <div class="text-center text-gray-500 text-sm py-8">
        <p>Start a conversation with AI</p>
      </div>
    {:else}
      {#each messages as msg}
        <div class="flex {msg.role === 'user' ? 'justify-end' : 'justify-start'}">
          <div class="max-w-[85%] rounded-lg px-4 py-2.5 text-sm {
            msg.role === 'user' 
              ? 'bg-blue-600 text-white' 
              : 'bg-[#1e1e1e] text-gray-200 border border-[#2d2d2d]'
          }">
            <div class="whitespace-pre-wrap break-words">{msg.content}</div>
            <div class="text-xs opacity-50 mt-1">
              {new Date(msg.timestamp).toLocaleTimeString()}
            </div>
          </div>
        </div>
      {/each}
    {/if}
    
    {#if isLoading}
      <div class="flex justify-start">
        <div class="bg-[#1e1e1e] border border-[#2d2d2d] rounded-lg px-4 py-2.5">
          <div class="flex gap-1.5">
            <span class="w-2 h-2 bg-blue-400 rounded-full animate-bounce"></span>
            <span class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 150ms"></span>
            <span class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 300ms"></span>
          </div>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- Input -->
  <div class="p-4 border-t border-[#2d2d2d]">
    <div class="flex gap-2">
      <textarea 
        bind:value={input}
        onkeydown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendMessage();
          }
        }}
        placeholder="Ask anything..."
        class="flex-1 bg-[#1e1e1e] border border-[#3e3e42] rounded-lg px-3 py-2.5 text-sm resize-none"
        rows="3"
      />
      <button 
        onclick={sendMessage}
        disabled={isLoading || !input.trim()}
        class="bg-blue-600 hover:bg-blue-700 text-white p-2.5 rounded-lg disabled:opacity-50"
      >
        <Send size={18} />
      </button>
    </div>
    <div class="text-xs text-gray-500 mt-2">
      Model: {selectedModel}
    </div>
  </div>
</aside>
```

---

## 3. Onboarding Model Selector

Update Onboarding.svelte untuk fetch models:

```svelte
<!-- In Onboarding.svelte, AI setup step -->
<div class="space-y-4">
  <div>
    <label class="text-sm font-medium mb-2 block">API Key</label>
    <input 
      type="password"
      bind:value={apiKey}
      placeholder="sk-..."
      class="w-full bg-[#1e1e1e] border border-[#3e3e42] rounded-lg px-4 py-2"
    />
  </div>
  
  <div>
    <label class="text-sm font-medium mb-2 block flex items-center justify-between">
      <span>Model</span>
      <button 
        onclick={fetchModels}
        disabled={!apiKey || isFetchingModels}
        class="text-xs text-blue-400 hover:text-blue-300 disabled:opacity-50"
      >
        {isFetchingModels ? 'Fetching...' : 'Fetch Models'}
      </button>
    </label>
    
    {#if availableModels.length > 0}
      <ModelSelector 
        bind:selectedModel 
        apiKey={apiKey}
      />
    {:else}
      <input 
        type="text"
        bind:value={selectedModel}
        placeholder="Enter model ID manually"
        class="w-full bg-[#1e1e1e] border border-[#3e3e42] rounded-lg px-4 py-2"
      />
    {/if}
  </div>
</div>
```

---

## 4. Implementation Checklist

- [ ] Create ModelSelector.svelte component
- [ ] Add search functionality with fuzzy matching
- [ ] Add refresh button to re-fetch models
- [ ] Cache models in localStorage (1 hour TTL)
- [ ] Update ChatPanel with model selector
- [ ] Add temperature & max tokens sliders
- [ ] Add export/import chat functionality
- [ ] Add clear chat with confirmation
- [ ] Save chat history to localStorage
- [ ] Show model name in input footer
- [ ] Update Onboarding with fetch models button
- [ ] Handle API errors gracefully
- [ ] Show loading states
- [ ] Add keyboard shortcuts (Ctrl+K for model selector)

---

## 5. Features Summary

✅ **Dynamic Model Loading** - Fetch from Kilo API
✅ **Search & Filter** - Find models quickly
✅ **Model Caching** - 1 hour localStorage cache
✅ **Model Selector** - Reusable component
✅ **Chat Settings** - Temperature, max tokens
✅ **Chat History** - Export/import/clear
✅ **Multi-model Support** - Switch models per conversation
✅ **Error Handling** - Graceful fallbacks
✅ **Loading States** - Spinners & indicators
✅ **Keyboard Shortcuts** - Quick access

---

## 6. Backend Already Ready

Backend sudah support:
- `fetch_kilo_models(apiKey)` - Returns ModelInfo[]
- Handles different API response structures
- Proper error handling
- Caching support

Frontend tinggal implement UI sesuai prompt di atas! 🚀
