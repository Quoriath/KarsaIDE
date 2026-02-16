use std::env;

#[derive(Debug, Clone)]
pub struct Environment {
    pub os: String,
    pub os_version: String,
    pub shell: String,
    pub home: String,
    pub hostname: String,
    pub username: String,
    pub cwd: String,
    pub lang: String,
    pub terminal: String,
}

impl Environment {
    pub fn detect() -> Self {
        let os = Self::detect_os();
        let shell = Self::detect_shell(&os);
        let home = env::var("HOME")
            .or_else(|_| env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());
        let username = whoami::username();
        let lang = env::var("LANG")
            .or_else(|_| env::var("LC_ALL"))
            .unwrap_or_else(|_| "en_US.UTF-8".to_string());
        let terminal = Self::detect_terminal();

        Self {
            os,
            os_version: Self::detect_os_version(),
            shell,
            home,
            hostname,
            username,
            cwd: env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string()),
            lang,
            terminal,
        }
    }

    fn detect_os() -> String {
        if cfg!(target_os = "linux") {
            if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
                for line in contents.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return line
                            .replace("PRETTY_NAME=", "")
                            .trim_matches('"')
                            .to_string();
                    }
                }
            }
            "Linux".to_string()
        } else if cfg!(target_os = "macos") {
            "macOS".to_string()
        } else if cfg!(target_os = "windows") {
            "Windows".to_string()
        } else {
            env::consts::OS.to_string()
        }
    }

    fn detect_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = std::process::Command::new("cmd")
                .args(["/C", "ver"])
                .output()
            {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }

        "unknown".to_string()
    }

    fn detect_shell(os: &str) -> String {
        if cfg!(target_os = "windows") {
            env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".to_string())
        } else {
            env::var("SHELL").unwrap_or_else(|_| {
                if os.contains("macOS") {
                    "/bin/zsh".to_string()
                } else {
                    "/bin/bash".to_string()
                }
            })
        }
    }

    fn detect_terminal() -> String {
        env::var("TERM")
            .or_else(|_| env::var("TERM_PROGRAM"))
            .unwrap_or_else(|_| {
                if cfg!(target_os = "windows") {
                    "cmd".to_string()
                } else {
                    "unknown".to_string()
                }
            })
    }

    pub fn to_context_string(&self) -> String {
        format!(
            r#"## System Environment

- OS: {} {}
- Shell: {}
- Terminal: {}
- Home: {}
- User: {}@{}
- Language: {}
- Current Directory: {}"#,
            self.os,
            self.os_version,
            self.shell,
            self.terminal,
            self.home,
            self.username,
            self.hostname,
            self.lang,
            self.cwd
        )
    }

    pub fn shell_command_prefix(&self) -> (&'static str, &'static str) {
        if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        }
    }
}
