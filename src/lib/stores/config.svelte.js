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

  isLoaded = $state(false);

  constructor() {
    this.load();
  }

  async load() {
    try {
      // Load from backend (secure storage)
      const backendConfig = await invoke('get_ai_config');
      
      // Merge with defaults
      this.settings = {
        ...this.settings,
        ai: {
          ...this.settings.ai,
          provider: backendConfig.ai?.provider || this.settings.ai.provider,
          apiKey: backendConfig.ai?.api_key || this.settings.ai.apiKey,
          baseUrl: backendConfig.ai?.base_url || this.settings.ai.baseUrl,
          selectedModel: backendConfig.ai?.model_name || this.settings.ai.selectedModel,
          models: backendConfig.ai?.custom_models || this.settings.ai.models
        },
        editor: backendConfig.editor || this.settings.editor
      };

      // Load layout from localStorage (UI state)
      const layoutStored = localStorage.getItem('karsa_layout');
      if (layoutStored) {
        this.settings.layout = { ...this.settings.layout, ...JSON.parse(layoutStored) };
      }

      this.isLoaded = true;
    } catch (e) {
      console.error('Failed to load config:', e);
      this.isLoaded = true;
    }
  }

  async save() {
    try {
      // Save to backend
      await invoke('save_ai_config', {
        config: {
          ai: {
            provider: this.settings.ai.provider,
            api_key: this.settings.ai.apiKey,
            base_url: this.settings.ai.baseUrl,
            model_name: this.settings.ai.selectedModel,
            custom_models: this.settings.ai.models
          },
          editor: this.settings.editor,
          terminal: {},
          session: {},
          security: {
            auto_execute_shell: false,
            auto_delete_files: false,
            auto_move_files: false,
            auto_save_files: true
          }
        }
      });

      // Save layout to localStorage
      localStorage.setItem('karsa_layout', JSON.stringify(this.settings.layout));
    } catch (e) {
      console.error('Failed to save config:', e);
    }
  }

  updateAiConfig(config) {
    this.settings.ai = { ...this.settings.ai, ...config };
    this.save();
  }

  updateLayout(layout) {
    this.settings.layout = { ...this.settings.layout, ...layout };
    localStorage.setItem('karsa_layout', JSON.stringify(this.settings.layout));
  }

  setTheme(themeName) {
    this.settings.theme = themeName;
    localStorage.setItem('karsa_theme', themeName);
  }
}

export const configStore = new ConfigStore();
