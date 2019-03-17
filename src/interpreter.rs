use super::{action::*, keysource::KeySource, model::*};
use crossbeam::{
    channel::{SendError, Sender},
    scope,
};

pub struct Interpreter<'a> {
    model: &'a Model,
    keysource: &'a KeySource<'a>,
}

impl<'a> Interpreter<'a> {
    pub fn run(model: &'a Model, keysource: &'a KeySource<'a>) {
        scope(|s| {
            let (tx, rx) = crossbeam::channel::bounded(0);
            s.spawn(|_| ActionServer::run(model, rx));
            Interpreter::new(model, keysource).main_loop(tx).unwrap();
        })
        .unwrap();
    }

    fn new(model: &'a Model, keysource: &'a KeySource<'a>) -> Interpreter<'a> {
        Interpreter { model, keysource }
    }

    fn main_loop(&self, tx: Sender<ActionMessage>) -> Result<(), SendError<ActionMessage>> {
        self.keysource.grab_keys(self.model.bindings.keys());
        self.keysource.grab_keys(
            self.model
                .command_bindings
                .iter()
                .filter(|(_, c)| **c != Command::Cancel)
                .map(|(k, _)| k),
        );

        while let Some(key) = self.keysource.wait_for_key() {
            match self.model.command_bindings.get(&key) {
                Some(Command::Cancel) => continue,
                Some(Command::ToggleHelp) => tx.send(ActionMessage::ToggleHelp)?,
                None => match self.model.bindings.get(&key) {
                    Some(Binding::Exec { exec, .. }) => {
                        tx.send(ActionMessage::Exec(exec.clone()))?
                    }
                    Some(Binding::Call { call, .. }) => {
                        tx.send(ActionMessage::Call(call.clone()))?
                    }
                    Some(Binding::Mode { mode, .. }) => {
                        self.keysource.grab_keyboard();
                        tx.send(ActionMessage::Enter)?;
                        self.modal_loop(mode, &tx)?;
                        tx.send(ActionMessage::Exit)?;
                    }
                    None => (),
                },
            }
        }

        Ok(())
    }

    fn modal_loop(
        &self,
        mode: &DefinitionId,
        tx: &Sender<ActionMessage>,
    ) -> Result<(), SendError<ActionMessage>> {
        if let Some(definitions) = self.model.definitions.get(mode) {
            tx.send(ActionMessage::Mode(mode.clone()))?;
            while let Some(key) = self.keysource.wait_for_key() {
                match self.model.command_bindings.get(&key) {
                    Some(Command::Cancel) => {
                        self.keysource.ungrab_keyboard();
                        tx.send(ActionMessage::Cancel)?;
                        return Ok(());
                    }
                    Some(Command::ToggleHelp) => tx.send(ActionMessage::ToggleHelp)?,
                    None => {
                        for d in definitions {
                            // TODO: evaluate definition guard
                            match d.bindings.get(&key) {
                                Some(Binding::Exec { exec, .. }) => {
                                    self.keysource.ungrab_keyboard();
                                    return tx.send(ActionMessage::Exec(exec.clone()));
                                }
                                Some(Binding::Call { call, .. }) => {
                                    tx.send(ActionMessage::Call(call.clone()))?;
                                    break; // out of the definitions loop;
                                }
                                Some(Binding::Mode { mode, .. }) => {
                                    return self.modal_loop(mode, tx);
                                }
                                None => (),
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
