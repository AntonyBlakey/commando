use super::model::Model;
use crossbeam::channel::{Receiver, RecvError, RecvTimeoutError};
use std::time::Duration;

pub enum ActionMessage {
    Mode(String),
    Exec(String),
    Call(String),
    ToggleHelp,
    Enter,
    Cancel,
    Exit,
}

pub struct ActionServer<'a> {
    model: &'a Model,
    definition_id: Option<String>,
    // help: HelpEngine,
}

impl<'a> ActionServer<'a> {
    pub fn run(model: &'a Model, rx: Receiver<ActionMessage>) {
        ActionServer::new(model).main_loop(rx);
    }

    fn new(model: &Model) -> ActionServer {
        ActionServer {
            model,
            definition_id: None,
            // help: Default::default(),
        }
    }

    fn main_loop(&mut self, rx: Receiver<ActionMessage>) {
        loop {
            if self.definition_id.is_none()
            /*|| self.help.is_showing()*/
            {
                match rx.recv() {
                    Ok(action) => self.handle_action(&action),
                    Err(RecvError) => return,
                }
            } else {
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(action) => self.handle_action(&action),
                    Err(RecvTimeoutError::Timeout) => self.show_help(),
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }
        }
    }

    fn handle_action(&mut self, action: &ActionMessage) {
        match action {
            ActionMessage::Mode(id) => self.mode(&id),
            ActionMessage::Exec(command_line) => self.exec(&command_line),
            ActionMessage::Call(command_line) => self.call(&command_line),
            ActionMessage::ToggleHelp => self.toggle_help(),
            ActionMessage::Enter => self.enter(),
            ActionMessage::Cancel => self.cancel(),
            ActionMessage::Exit => self.exit(),
        }
    }

    fn cancel(&mut self) {
        self.hide_help();
    }

    fn enter(&mut self) {}

    fn mode(&mut self, id: &String) {
        self.definition_id = Some(id.clone());
        self.refresh_help();
    }

    fn exit(&mut self) {
        self.definition_id = None;
        self.hide_help();
    }

    fn exec(&mut self, command_line: &String) {
        self.hide_help();
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command_line)
            .spawn()
            .expect(&format!("Failed to spawn {}", command_line));
    }

    fn call(&mut self, command_line: &String) {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(command_line)
            .spawn()
            .expect(&format!("Failed to spawn {}", command_line));
    }

    fn show_help(&mut self) {
        // self.help.show(&self.model, &self.definition_id);
    }

    fn refresh_help(&mut self) {
        // if self.help.is_showing() {
        //     self.help.show(&self.model, &self.definition_id);
        // }
    }

    fn hide_help(&mut self) {
        // self.help.hide(&self.model);
    }

    fn toggle_help(&mut self) {
        // self.help.toggle(&self.model, &self.definition_id);
    }
}
