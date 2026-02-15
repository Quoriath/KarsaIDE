# 🖥️ Terminal Auto-Detection

## Overview

Terminal sekarang otomatis detect shell native berdasarkan OS!

## Shell Detection Logic

### Windows
```
1. PowerShell Core (pwsh.exe) - Modern, cross-platform
2. Windows PowerShell (powershell.exe) - Built-in
3. CMD (cmd.exe) - Fallback
```

### macOS
```
1. $SHELL environment variable
2. /bin/zsh - Default since Catalina
```

### Linux
```
1. $SHELL environment variable
2. /bin/zsh - Modern shell
3. /bin/bash - Traditional
4. /bin/sh - POSIX fallback
```

## API

### Get Shell Info

```typescript
import { invoke } from '@tauri-apps/api/core';

const shellInfo = await invoke('get_shell_info');
// Returns:
// {
//   path: "/bin/zsh",
//   name: "zsh",
//   os: "linux"
// }
```

### Spawn Terminal (Auto-detect)

```typescript
// Auto-detect shell
await invoke('spawn_terminal', {
  id: 'terminal-1',
  shell: null  // null = auto-detect
});
```

### Spawn Terminal (Custom Shell)

```typescript
// Use specific shell
await invoke('spawn_terminal', {
  id: 'terminal-1',
  shell: '/bin/bash'  // Force bash
});

// Windows PowerShell
await invoke('spawn_terminal', {
  id: 'terminal-1',
  shell: 'powershell.exe'
});
```

## Frontend Implementation

### Display Shell Info

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  
  let shellInfo = $state(null);
  
  onMount(async () => {
    shellInfo = await invoke('get_shell_info');
  });
</script>

{#if shellInfo}
  <div class="shell-info">
    <span class="shell-icon">🖥️</span>
    <span>{shellInfo.name}</span>
    <span class="os-badge">{shellInfo.os}</span>
  </div>
{/if}
```

### Terminal Component

```svelte
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  
  let terminalId = 'main-terminal';
  let output = $state('');
  let shellInfo = $state(null);
  
  onMount(async () => {
    // Get shell info
    shellInfo = await invoke('get_shell_info');
    
    // Spawn terminal (auto-detect)
    await invoke('spawn_terminal', {
      id: terminalId,
      shell: null
    });
    
    // Listen to output
    await listen('terminal-output', (event) => {
      const text = new TextDecoder().decode(new Uint8Array(event.payload));
      output += text;
    });
  });
  
  async function sendCommand(cmd) {
    const data = new TextEncoder().encode(cmd + '\n');
    await invoke('write_to_terminal', {
      id: terminalId,
      data: Array.from(data)
    });
  }
</script>

<div class="terminal">
  <div class="terminal-header">
    <span>Terminal - {shellInfo?.name || 'Loading...'}</span>
    <span class="os">{shellInfo?.os}</span>
  </div>
  <pre class="terminal-output">{output}</pre>
  <input 
    type="text" 
    placeholder="Type command..."
    onkeydown={(e) => {
      if (e.key === 'Enter') {
        sendCommand(e.target.value);
        e.target.value = '';
      }
    }}
  />
</div>
```

## Shell Priority

### Why This Order?

**Windows:**
- PowerShell Core: Modern, cross-platform, best features
- Windows PowerShell: Built-in, widely available
- CMD: Legacy fallback

**macOS:**
- zsh: Default since macOS Catalina (2019)
- Respects user's $SHELL preference

**Linux:**
- Respects $SHELL (user's choice)
- zsh: Modern features (completion, themes)
- bash: Most common, reliable
- sh: POSIX standard, always available

## Benefits

✅ **Zero Configuration** - Works out of the box
✅ **OS Native** - Uses system's default shell
✅ **User Preference** - Respects $SHELL environment
✅ **Fallback Chain** - Always finds a working shell
✅ **Cross-Platform** - Linux, macOS, Windows

## Technical Details

### Detection Implementation

```rust
fn detect_shell() -> String {
    #[cfg(target_os = "windows")]
    {
        // Try PowerShell Core > PowerShell > CMD
        if let Ok(pwsh) = which::which("pwsh") {
            return pwsh.to_string_lossy().to_string();
        }
        if let Ok(powershell) = which::which("powershell") {
            return powershell.to_string_lossy().to_string();
        }
        "cmd.exe".to_string()
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            return shell;
        }
        
        for shell in &["/bin/zsh", "/bin/bash", "/bin/sh"] {
            if std::path::Path::new(shell).exists() {
                return shell.to_string();
            }
        }
        
        "/bin/sh".to_string()
    }
}
```

### Dependencies

```toml
[dependencies]
portable-pty = "0.8"  # Cross-platform PTY
which = "6.0"         # Find executables in PATH
```

## Testing

### Linux
```bash
# Test auto-detection
SHELL=/bin/bash cargo run
SHELL=/bin/zsh cargo run

# Test with no SHELL env
unset SHELL && cargo run
```

### macOS
```bash
# Should use zsh by default
cargo run

# Test custom shell
SHELL=/bin/bash cargo run
```

### Windows
```powershell
# Should use PowerShell
cargo run

# Test with CMD
$env:ComSpec = "cmd.exe"
cargo run
```

## Future Enhancements

- [ ] Shell-specific features (PowerShell modules, zsh plugins)
- [ ] Custom shell profiles (.bashrc, .zshrc)
- [ ] Shell theme detection
- [ ] Multi-shell tabs
- [ ] Shell history persistence

---

**Made with ❤️ for cross-platform terminal experience**
