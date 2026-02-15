use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use anyhow::Result;

pub struct Terminal {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Terminal {
    pub fn spawn(app: AppHandle, shell: Option<String>) -> Result<Self> {
        let pty_system = native_pty_system();
        
        let pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let shell_cmd = shell.unwrap_or_else(Self::detect_shell);

        let mut cmd = CommandBuilder::new(&shell_cmd);
        cmd.cwd(std::env::current_dir().unwrap_or_default());
        
        // Suppress Powerlevel10k instant prompt warning
        cmd.env("POWERLEVEL9K_INSTANT_PROMPT", "quiet");
        
        let child = pair.slave.spawn_command(cmd)?;
        
        let reader = pair.master.try_clone_reader()?;
        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
        let writer_clone = writer.clone();

        // Spawn reader thread with immediate output
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let _ = app.emit("terminal-output", buffer[..n].to_vec());
                    }
                    Err(_) => break,
                }
            }
        });

        // Monitor child in separate thread (not tokio)
        std::thread::spawn(move || {
            let mut child_mut = child;
            let _ = child_mut.wait();
        });

        Ok(Self { writer: writer_clone })
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(data)?;
        writer.flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn resize(&self, _rows: u16, _cols: u16) -> Result<()> {
        // Future: implement PTY resize
        Ok(())
    }

    /// Auto-detect shell based on OS
    fn detect_shell() -> String {
        #[cfg(target_os = "windows")]
        {
            // Windows: PowerShell > CMD
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
            // macOS: zsh (default since Catalina) > bash
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string())
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: $SHELL env > zsh > bash > sh
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

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            // Fallback for other Unix-like systems
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        }
    }

    /// Get shell info for display
    pub fn get_shell_info() -> ShellInfo {
        let shell = Self::detect_shell();
        let shell_name = std::path::Path::new(&shell)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        ShellInfo {
            path: shell,
            name: shell_name,
            os: std::env::consts::OS.to_string(),
        }
    }
}

#[derive(serde::Serialize)]
pub struct ShellInfo {
    pub path: String,
    pub name: String,
    pub os: String,
}
