import { invoke } from '@tauri-apps/api/core';

class FileSystemStore {
  currentProjectDir = $state(null);
  openFiles = $state([]);
  activeFile = $state(null);
  activeFileContent = $state('');
  directoryContents = $state([]);
  fileTree = $state(null);

  async setProjectDir(path) {
    this.currentProjectDir = path;
    await this.refreshDirectory(path);
  }

  setFileTree(tree) {
    this.fileTree = tree;
  }

  async refreshDirectory(path) {
    try {
      const entries = await invoke('list_directory', { path });
      this.directoryContents = entries;
    } catch (e) {
      console.error('Failed to list directory:', e);
    }
  }

  async openFile(path) {
    const alreadyOpen = this.openFiles.find(f => f.path === path);
    if (!alreadyOpen) {
      try {
        const content = await invoke('read_file_content', { path });
        const name = path.split(/[/\\]/).pop() || path;
        this.openFiles.push({ path, name, content, modified: false });
      } catch (e) {
        console.error('Failed to read file:', e);
        return;
      }
    }
    this.setActiveFile(path);
  }

  setActiveFile(path) {
    const file = this.openFiles.find(f => f.path === path);
    if (file) {
      this.activeFile = file;
      this.activeFileContent = file.content;
    }
  }

  updateActiveFileContent(content) {
    this.activeFileContent = content;
    if (this.activeFile) {
      this.activeFile.content = content;
      this.activeFile.modified = true;
    }
  }

  async saveActiveFile() {
    if (!this.activeFile) return;
    try {
      await invoke('write_file_content', { 
        path: this.activeFile.path, 
        content: this.activeFileContent 
      });
      this.activeFile.modified = false;
    } catch (e) {
      console.error('Failed to save file:', e);
    }
  }

  closeFile(path) {
    const idx = this.openFiles.findIndex(f => f.path === path);
    if (idx !== -1) {
      this.openFiles.splice(idx, 1);
      if (this.activeFile?.path === path) {
        this.activeFile = this.openFiles[0] || null;
        this.activeFileContent = this.activeFile?.content || '';
      }
    }
  }
}

export const fsStore = new FileSystemStore();
