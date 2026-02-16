import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';

export interface ChatMessage {
  role: string;
  content: string;
  timestamp: number;
  thinking?: string;
  toolCalls?: any[];
  toolResults?: any[];
}

export interface ChatSession {
  id: string;
  workspace: string | null;
  messages: ChatMessage[];
  created_at: number;
  updated_at: number;
}

function createChatHistoryStore() {
  const { subscribe, set, update } = writable<ChatSession[]>([]);
  const currentSession = writable<ChatSession | null>(null);
  
  return {
    subscribe,
    currentSession,
    
    async load(workspace: string | null) {
      try {
        const sessions = await invoke<ChatSession[]>('get_chat_sessions', { workspace });
        set(sessions);
        return sessions;
      } catch (e) {
        console.error('Failed to load chat sessions:', e);
        return [];
      }
    },
    
    async save(session: ChatSession) {
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
    
    createSession(workspace: string | null): ChatSession {
      const now = Date.now();
      return {
        id: `session_${now}_${Math.random().toString(36).substr(2, 9)}`,
        workspace,
        messages: [],
        created_at: now,
        updated_at: now,
      };
    },
    
    async setCurrentSession(session: ChatSession | null) {
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
  const { subscribe, set } = writable<any[]>([]);
  
  return {
    subscribe,
    
    async load() {
      try {
        const folders = await invoke<any[]>('get_recent_folders');
        set(folders);
        return folders;
      } catch (e) {
        console.error('Failed to load recent folders:', e);
        return [];
      }
    },
    
    async add(path: string, name: string) {
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
  const { subscribe, set } = writable<string | null>(null);
  
  return {
    subscribe,
    
    async load() {
      try {
        const workspace = await invoke<string | null>('get_last_workspace');
        set(workspace);
        return workspace;
      } catch (e) {
        console.error('Failed to load last workspace:', e);
        return null;
      }
    },
    
    async save(path: string) {
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
