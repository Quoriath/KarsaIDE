<script>
  import { onMount, onDestroy } from 'svelte';
  import * as monaco from 'monaco-editor';
  import { fsStore } from '../../stores/fileSystem.svelte.js';
  import { themeStore } from '../../stores/theme.svelte.js';
  import { uiState } from '../../stores/uiState.svelte.js';
  import Breadcrumbs from './Breadcrumbs.svelte';

  let editorContainer;
  let editor;

  const languageMap = {
    rs: 'rust', js: 'javascript', ts: 'typescript', svelte: 'svelte',
    html: 'html', css: 'css', json: 'json', md: 'markdown',
    toml: 'toml', jsx: 'javascript', tsx: 'typescript', py: 'python',
    kt: 'kotlin', java: 'java', c: 'c', cpp: 'cpp', h: 'c',
    go: 'go', yml: 'yaml', yaml: 'yaml', sh: 'shell',
    bash: 'shell', sql: 'sql', xml: 'xml', svg: 'xml'
  };

  function getLanguage(path) {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    const lang = languageMap[ext] || 'plaintext';
    return lang;
  }
  
  function updateMonacoTheme(themeName) {
    if (!monaco) return;
    
    const isDark = themeStore.activeTheme !== 'github-light';
    
    const colors = {
      'karsa-dark': { bg: '#09090b', fg: '#fafafa', line: '#27272a' },
      'dracula': { bg: '#282a36', fg: '#f8f8f2', line: '#44475a' },
      'github-light': { bg: '#ffffff', fg: '#24292f', line: '#eaeef2' },
      'cyberpunk': { bg: '#0d0214', fg: '#ffeeff', line: '#330033' },
      'nord': { bg: '#2e3440', fg: '#eceff4', line: '#3b4252' },
      'gruvbox': { bg: '#282828', fg: '#ebdbb2', line: '#3c3836' },
      'solarized-ocean': { bg: '#fdf6e3', fg: '#657b83', line: '#eee8d5' }
    }[themeName] || { bg: '#1e1e1e', fg: '#d4d4d4', line: '#333333' };

    monaco.editor.defineTheme('dynamic-theme', {
      base: isDark ? 'vs-dark' : 'vs',
      inherit: true,
      rules: [],
      colors: {
        'editor.background': colors.bg,
        'editor.foreground': colors.fg,
        'editorLineNumber.foreground': isDark ? '#52525b' : '#6e7781',
        'editor.selectionBackground': isDark ? '#3f3f46' : '#b6e3ff',
        'editor.lineHighlightBackground': colors.line + (isDark ? '40' : '40'),
        'editorCursor.foreground': '#3b82f6',
        'editorWhitespace.foreground': isDark ? '#27272a' : '#d0d7de',
        'editorIndentGuide.background': isDark ? '#27272a' : '#d0d7de',
        'editorIndentGuide.activeBackground': isDark ? '#52525b' : '#afb8c1'
      }
    });
    
    monaco.editor.setTheme('dynamic-theme');
  }

  onMount(() => {
    self.MonacoEnvironment = {
      getWorker: function () {
        return new Worker(
          new URL('monaco-editor/esm/vs/editor/editor.worker.js', import.meta.url),
          { type: 'module' }
        );
      }
    };

    updateMonacoTheme(themeStore.activeTheme);

    const initialLang = getLanguage(fsStore.activeFile?.path || '');
    uiState.updateEditorStatus({ language: initialLang });

    editor = monaco.editor.create(editorContainer, {
      value: fsStore.activeFileContent || '',
      language: initialLang,
      theme: 'dynamic-theme',
      automaticLayout: true,
      fontSize: 14,
      fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      renderLineHighlight: 'all',
      padding: { top: 10 },
      smoothScrolling: true,
      cursorBlinking: 'smooth',
      cursorSmoothCaretAnimation: 'on',
      bracketPairColorization: { enabled: true },
      guides: { indentation: true }
    });

    editor.onDidChangeModelContent(() => {
      if (fsStore.activeFile) {
        fsStore.updateActiveFileContent(editor.getValue());
      }
    });

    editor.onDidChangeCursorPosition((e) => {
      uiState.updateEditorStatus({
        line: e.position.lineNumber,
        column: e.position.column
      });
    });

    editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      fsStore.saveActiveFile();
    });

    // Watch theme changes
    $effect(() => {
       updateMonacoTheme(themeStore.activeTheme);
    });

    $effect(() => {
      if (fsStore.activeFile) {
        const model = editor.getModel();
        const currentValue = editor.getValue();
        const newLang = getLanguage(fsStore.activeFile.path);
        
        uiState.updateEditorStatus({ language: newLang });

        if (model) {
          monaco.editor.setModelLanguage(model, newLang);
        }
        
        if (currentValue !== fsStore.activeFileContent) {
          const position = editor.getPosition();
          editor.setValue(fsStore.activeFileContent);
          if (position) {
            editor.setPosition(position);
          }
        }
      }
    });
  });

  onDestroy(() => {
    editor?.dispose();
  });
</script>

<div class="h-full flex flex-col">
  <Breadcrumbs />
  <div class="flex-1 overflow-hidden h-full w-full" bind:this={editorContainer}></div>
</div>
",file_path: