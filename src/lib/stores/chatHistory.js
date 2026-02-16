import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

function createChatHistoryStore() {
  const { subscribe, set, update } = writable([]);
  const currentSession = writable(null);
  
  return {
    subscribe,
    currentSession,
    
    async load(workspace) {
      try {
        const sessions = await invoke('get_chat_sessions', { workspace });
        set(sessions);
        return sessions;
      } catch (e) {
        console.error('Failed to load chat sessions:', e);
        return [];
      }
    },
    
    async save(session) {
      try {
        await invoke('save_chat_session', { session });
        update(sessions => {
          const index = sessions.findIndex(s => s.id === session.id);
          if (index >= 0) {
            sessions[index] = session;
          } else {
            sessions.unshift(session);
          }
          return sessions;
        });
      } catch (e) {
        console.error('Failed to save chat session:', e);
      }
    },
    
    createSession(workspace) {
      const now = Date.now();
      return {
        id: `session_${now}_${Math.random().toString(36).substr(2, 9)}`,
        workspace,
        messages: [],
        created_at: now,
        updated_at: now,
      };
    },
    
    async setCurrentSession(session) {
      currentSession.set(session);
      if (session) {
        await this.save(session);
      }
    },
  };
}

export const chatHistory = createChatHistoryStore();

// Recent folders store
function createRecentFoldersStore() {
  const { subscribe, set } = writable([]);
  
  return {
    subscribe,
    
    async load() {
      try {
        const folders = await invoke('get_recent_folders');
        set(folders);
        return folders;
      } catch (e) {
        console.error('Failed to load recent folders:', e);
        return [];
      }
    },
    
    async add(path, name) {
      try {
        await invoke('add_recent_folder', { path, name });
        await this.load();
      } catch (e) {
        console.error('Failed to add recent folder:', e);
      }
    },
  };
}

export const recentFolders = createRecentFoldersStore();

// Last workspace store
function createLastWorkspaceStore() {
  const { subscribe, set } = writable(null);
  
  return {
    subscribe,
    
    async load() {
      try {
        const workspace = await invoke('get_last_workspace');
        set(workspace);
        return workspace;
      } catch (e) {
        console.error('Failed to load last workspace:', e);
        return null;
      }
    },
    
    async save(path) {
      try {
        await invoke('set_last_workspace', { path });
        set(path);
      } catch (e) {
        console.error('Failed to save last workspace:', e);
      }
    },
  };
}

export const lastWorkspace = createLastWorkspaceStore();
