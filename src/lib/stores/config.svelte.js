import { invoke } from '@tauri-apps/api/core';

class ConfigStore {
  settings = $state({
    theme: 'karsa-dark',
    ai: {
      provider: 'kilo',
      apiKey: '',
      baseUrl: 'https://api.kilo.ai/api/gateway/chat/completions',
      models: ['minimax/minimax-m2.5:free', 'z-ai/glm-5:free'],
      selectedModel: 'minimax/minimax-m2.5:free'
    },
    editor: {
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      minimap: true,
      wordWrap: 'on'
    },
    layout: {
      sidebarVisible: true,
      sidebarWidth: 280,
      chatVisible: false,
      chatWidth: 400,
      bottomPanelVisible: true,
      bottomPanelHeight: 200
    }
  });

  constructor() {
    this.load();
  }

  async load() {
    try {
      // Try to load from disk via Tauri (mocked here if backend missing)
      // In a real app, we'd use tauri-plugin-store or read a config file
      const stored = localStorage.getItem('karsa_config');
      if (stored) {
        const parsed = JSON.parse(stored);
        this.settings = { ...this.settings, ...parsed };
      }
    } catch (e) {
      console.error('Failed to load config', e);
    }
  }

  save() {
    try {
      localStorage.setItem('karsa_config', JSON.stringify(this.settings));
      // In real app: await invoke('save_config', { config: this.settings });
    } catch (e) {
      console.error('Failed to save config', e);
    }
  }

  updateAiConfig(config) {
    this.settings.ai = { ...this.settings.ai, ...config };
    this.save();
    
    // Sync specifically with backend AI service as requested
    invoke('save_ai_config', { config: this.settings.ai }).catch(err => {
        console.error('Failed to sync AI config to backend:', err);
    });
  }

  updateLayout(layout) {
    this.settings.layout = { ...this.settings.layout, ...layout };
    this.save();
  }

  setTheme(themeName) {
    this.settings.theme = themeName;
    this.save();
  }
}

export const configStore = new ConfigStore();
