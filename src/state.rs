use std::path::PathBuf;
use std::process::Child;
use std::sync::Arc;
use anyhow::Result;
use dreammaker::Context;
use dreammaker::objtree::ObjectTree;

/// Server state that persists across tool calls
pub struct ServerState {
    /// Currently parsed environment path
    pub environment_path: Option<PathBuf>,
    /// Parsed object tree (cached)
    pub objtree: Option<Arc<ObjectTree>>,
    /// Parsing context
    pub context: Option<Context>,
    /// Running DreamDaemon process
    pub game_process: Option<Child>,
    /// Port the game is running on
    pub game_port: Option<u16>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            environment_path: None,
            objtree: None,
            context: None,
            game_process: None,
            game_port: None,
        }
    }

    /// Clear cached parse state
    pub fn clear_cache(&mut self) {
        self.objtree = None;
        self.context = None;
    }

    /// Check if we have a valid parsed environment
    pub fn has_environment(&self) -> bool {
        self.objtree.is_some()
    }

    /// Set the running game process
    pub fn set_game_process(&mut self, process: Child, port: u16) {
        self.game_process = Some(process);
        self.game_port = Some(port);
    }

    /// Check if the game is currently running
    pub fn is_game_running(&mut self) -> bool {
        if let Some(ref mut process) = self.game_process {
            // Check if process is still alive
            match process.try_wait() {
                Ok(Some(_status)) => {
                    // Process has exited
                    self.game_process = None;
                    self.game_port = None;
                    false
                }
                Ok(None) => {
                    // Still running
                    true
                }
                Err(_) => {
                    // Error checking - assume dead
                    self.game_process = None;
                    self.game_port = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Stop the running game process
    pub fn stop_game_process(&mut self) -> Result<()> {
        if let Some(mut process) = self.game_process.take() {
            process.kill()?;
            process.wait()?;
        }
        self.game_port = None;
        Ok(())
    }
}
