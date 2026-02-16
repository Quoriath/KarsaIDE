import { invoke } from '@tauri-apps/api/core';

class FileSystemStore {
  currentProjectDir = $state(null);
  activeWorkspace = $state(null);
  openFiles = $state([]);
  activeFile = $state(null);
  activeFileContent = $state('');
  directoryContents = $state([]);
  fileTree = $state(null);
  clipboard = $state({ path: null, operation: null }); // { path: string, operation: 'copy' | 'cut' }

  async setProjectDir(path) {
    this.currentProjectDir = path;
    this.activeWorkspace = path;
    await this.refreshExplorer();
  }

  setFileTree(tree) {
    this.fileTree = tree;
  }

  async refreshExplorer() {
    if (this.currentProjectDir) {
      try {
        const tree = await invoke('scan_workspace', { path: this.currentProjectDir, depth: 8 });
        this.fileTree = tree;
      } catch (e) {
        console.error('Failed to refresh explorer:', e);
      }
    }
  }

  async openFile(path) {
    const alreadyOpen = this.openFiles.find(f => f.path === path);
    if (!alreadyOpen) {
      try {
        const content = await invoke('read_file_content', { path });
        const name = path.split(/[/\\\\]/).pop() || path;
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

  // --- FILE OPERATIONS ---

  async createFile(path, content = '') {
    try {
      await invoke('create_file', { path, content });
      await this.refreshExplorer();
    } catch (e) {
      console.error('Failed to create file:', e);
      throw e;
    }
  }

  async createDirectory(path) {
    try {
      await invoke('create_directory', { path });
      await this.refreshExplorer();
    } catch (e) {
      console.error('Failed to create directory:', e);
      throw e;
    }
  }

  async renamePath(oldPath, newPath) {
    try {
      await invoke('rename_path', { oldPath, newPath });
      
      // Update open files if renamed
      this.openFiles = this.openFiles.map(f => {
        if (f.path === oldPath) {
          return { ...f, path: newPath, name: newPath.split(/[/\\\\]/).pop() };
        }
        if (f.path.startsWith(oldPath + '/')) {
          const relativePart = f.path.substring(oldPath.length);
          return { ...f, path: newPath + relativePart };
        }
        return f;
      });

      if (this.activeFile?.path === oldPath) {
        this.setActiveFile(newPath);
      }

      await this.refreshExplorer();
    } catch (e) {
      console.error('Failed to rename:', e);
      throw e;
    }
  }

  async deletePath(path) {
    try {
      await invoke('delete_path', { path });
      
      // Close file if it was open
      this.closeFile(path);
      const toClose = this.openFiles.filter(f => f.path.startsWith(path + '/'));
      toClose.forEach(f => this.closeFile(f.path));

      await this.refreshExplorer();
    } catch (e) {
      console.error('Failed to delete:', e);
      throw e;
    }
  }

  async movePath(source, destination) {
    try {
      await invoke('rename_path', { oldPath: source, newPath: destination });
      
      this.openFiles = this.openFiles.map(f => {
        if (f.path === source) {
          return { ...f, path: destination, name: destination.split(/[/\\\\]/).pop() };
        }
        if (f.path.startsWith(source + '/')) {
          const relativePart = f.path.substring(source.length);
          return { ...f, path: destination + relativePart };
        }
        return f;
      });

      await this.refreshExplorer();
    } catch (e) {
      console.error('Failed to move:', e);
      throw e;
    }
  }

  // --- CLIPBOARD OPERATIONS ---

  copyToClipboard(path) {
    this.clipboard = { path, operation: 'copy' };
  }

  cutToClipboard(path) {
    this.clipboard = { path, operation: 'cut' };
  }

  async pasteFromClipboard(destinationDir) {
    if (!this.clipboard.path) return;

    const sourcePath = this.clipboard.path;
    const fileName = sourcePath.split(/[/\\]/).pop();
    const destinationPath = `${destinationDir}/${fileName}`;

    try {
      if (this.clipboard.operation === 'copy') {
        await invoke('copy_path', { source: sourcePath, destination: destinationPath });
      } else if (this.clipboard.operation === 'cut') {
        await this.movePath(sourcePath, destinationPath);
        this.clipboard = { path: null, operation: null }; // Clear after cut
      }
      await this.refreshExplorer();
    } catch (e) {
      console.error(`Failed to ${this.clipboard.operation}:`, e);
      throw e;
    }
  }

  getRelativePath(path) {
    if (!this.currentProjectDir) return path;
    const rel = path.replace(this.currentProjectDir, '');
    return rel.startsWith('/') || rel.startsWith('\\\\') ? rel.substring(1) : rel;
  }

  async revealInExplorer(path) {
    try {
      await invoke('reveal_in_explorer', { path });
    } catch (e) {
      console.error('Failed to reveal in explorer:', e);
    }
  }
}

export const fsStore = new FileSystemStore();
