use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{BufRead, BufReader, Write};
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

        let shell_cmd = shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
        });

        let mut cmd = CommandBuilder::new(&shell_cmd);
        cmd.cwd(std::env::current_dir().unwrap_or_default());
        
        let child = pair.slave.spawn_command(cmd)?;
        
        let reader = pair.master.try_clone_reader()?;
        let writer = Arc::new(Mutex::new(pair.master.take_writer()?));
        let writer_clone = writer.clone();

        // Spawn reader thread
        std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut buffer = Vec::new();
            loop {
                buffer.clear();
                match reader.read_until(b'\n', &mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(_) => {
                        let _ = app.emit("terminal-output", buffer.clone());
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
}
