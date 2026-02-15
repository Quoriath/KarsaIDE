<script>
  import { invoke } from '@tauri-apps/api/core';
  import { configStore } from '../stores/config.svelte.js';
  import { X, Save, RefreshCw, Settings as SettingsIcon, Brain } from 'lucide-svelte';
  import IntelligenceTab from './IntelligenceTab.svelte';

  let { onClose } = $props();
  
  let activeTab = $state('general');

  let provider = $state(configStore.settings.ai.provider);
  let apiKey = $state(configStore.settings.ai.apiKey);
  let baseUrl = $state(configStore.settings.ai.baseUrl);
  let selectedModel = $state(configStore.settings.ai.selectedModel);
  let availableModels = $state([]);
  let isFetchingModels = $state(false);
  let isSaving = $state(false);

  async function fetchModels() {
    if (!apiKey) return;
    
    isFetchingModels = true;
    try {
      availableModels = await invoke('fetch_kilo_models', {
        apiKey,
        forceRefresh: true
      });
    } catch (e) {
      console.error('Failed to fetch models:', e);
    } finally {
      isFetchingModels = false;
    }
  }

  async function saveSettings() {
    isSaving = true;
    try {
      configStore.updateAiConfig({
        provider,
        apiKey,
        baseUrl,
        selectedModel,
        models: availableModels.length > 0 ? availableModels.map(m => m.id) : configStore.settings.ai.models
      });
      
      onClose?.();
    } catch (e) {
      console.error('Failed to save settings:', e);
    } finally {
      isSaving = false;
    }
  }
</script>

<div class="settings-overlay" onclick={onClose}>
  <div class="settings-panel" onclick={(e) => e.stopPropagation()}>
    <div class="settings-header">
      <h2>Settings</h2>
      <button class="close-btn" onclick={onClose}>
        <X size={20} />
      </button>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-border">
      <button
        class="px-4 py-2 text-sm flex items-center gap-2 {activeTab === 'general' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground hover:text-foreground'}"
        onclick={() => activeTab = 'general'}
      >
        <SettingsIcon size={14} />
        General
      </button>
      <button
        class="px-4 py-2 text-sm flex items-center gap-2 {activeTab === 'intelligence' ? 'border-b-2 border-primary text-primary' : 'text-muted-foreground hover:text-foreground'}"
        onclick={() => activeTab = 'intelligence'}
      >
        <Brain size={14} />
        Intelligence
      </button>
    </div>

    <div class="settings-content">
      {#if activeTab === 'general'}
      <section>
        <h3>AI Provider</h3>
        
        <label>
          <span>Provider</span>
          <select bind:value={provider}>
            <option value="kilo">Kilo Code</option>
            <option value="ollama">Ollama</option>
            <option value="openai">OpenAI</option>
            <option value="custom">Custom</option>
          </select>
        </label>

        <label>
          <span>API Key</span>
          <input 
            type="password" 
            bind:value={apiKey}
            placeholder="Enter your API key"
          />
        </label>

        <label>
          <span>Base URL</span>
          <input 
            type="text" 
            bind:value={baseUrl}
            placeholder="https://api.example.com"
          />
        </label>

        <label>
          <span>Model</span>
          <div class="model-selector">
            <select bind:value={selectedModel}>
              {#each availableModels.length > 0 ? availableModels : configStore.settings.ai.models as model}
                <option value={typeof model === 'string' ? model : model.id}>
                  {typeof model === 'string' ? model : model.name || model.id}
                </option>
              {/each}
            </select>
            <button 
              class="refresh-btn" 
              onclick={fetchModels}
              disabled={isFetchingModels || !apiKey}
            >
              <RefreshCw size={16} class={isFetchingModels ? 'spin' : ''} />
            </button>
          </div>
        </label>
      </section>
      {:else if activeTab === 'intelligence'}
        <IntelligenceTab />
      {/if}
    </div>

    {#if activeTab === 'general'}
    <div class="settings-footer">
      <button class="btn-secondary" onclick={onClose}>Cancel</button>
      <button class="btn-primary" onclick={saveSettings} disabled={isSaving}>
        <Save size={16} />
        {isSaving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
    {/if}
  </div>
</div>

<style>
  .settings-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .settings-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border);
  }

  .settings-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    padding: 0.25rem;
    display: flex;
    align-items: center;
  }

  .close-btn:hover {
    color: var(--text-primary);
  }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  section {
    margin-bottom: 2rem;
  }

  section h3 {
    margin: 0 0 1rem 0;
    font-size: 1rem;
    color: var(--text-primary);
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-bottom: 1rem;
  }

  label span {
    font-size: 0.875rem;
    color: var(--text-secondary);
  }

  input, select {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.5rem;
    color: var(--text-primary);
    font-size: 0.875rem;
  }

  input:focus, select:focus {
    outline: none;
    border-color: var(--accent);
  }

  .model-selector {
    display: flex;
    gap: 0.5rem;
  }

  .model-selector select {
    flex: 1;
  }

  .refresh-btn {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0.5rem;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .refresh-btn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .refresh-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .settings-footer {
    display: flex;
    gap: 0.75rem;
    justify-content: flex-end;
    padding: 1rem 1.5rem;
    border-top: 1px solid var(--border);
  }

  button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .btn-secondary {
    background: var(--bg-primary);
    border: 1px solid var(--border);
    color: var(--text-primary);
  }

  .btn-secondary:hover {
    background: var(--bg-hover);
  }

  .btn-primary {
    background: var(--accent);
    border: none;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
