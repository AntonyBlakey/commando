use super::{action::*, key_source::KeySource, model::*};
use crossbeam::{
    channel::{SendError, Sender},
    scope,
};

pub struct KeyDispatcher<'a> {
    model: &'a Model,
    key_source: &'a KeySource<'a>,
}

impl<'a> KeyDispatcher<'a> {
    pub fn run(model: &'a Model, key_source: &'a KeySource<'a>) {
        scope(|s| {
            let (tx, rx) = crossbeam::channel::bounded(0);
            s.spawn(|_| ActionServer::run(model, rx));
            KeyDispatcher::new(model, key_source).main_loop(tx).unwrap();
        })
        .unwrap();
    }

    fn new(model: &'a Model, key_source: &'a KeySource<'a>) -> KeyDispatcher<'a> {
        KeyDispatcher { model, key_source }
    }

    fn main_loop(&self, tx: Sender<ActionMessage>) -> Result<(), SendError<ActionMessage>> {
        self.key_source.grab_keys(self.model.bindings.keys());
        self.key_source.grab_keys(
            self.model
                .command_bindings
                .iter()
                .filter(|(_, c)| **c != Command::Cancel)
                .map(|(k, _)| k),
        );

        while let Some(key) = self.key_source.wait_for_key() {
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
                        self.key_source.grab_keyboard();
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
            while let Some(key) = self.key_source.wait_for_key() {
                match self.model.command_bindings.get(&key) {
                    Some(Command::Cancel) => {
                        self.key_source.ungrab_keyboard();
                        tx.send(ActionMessage::Cancel)?;
                        return Ok(());
                    }
                    Some(Command::ToggleHelp) => tx.send(ActionMessage::ToggleHelp)?,
                    None => {
                        for d in definitions {
                            // TODO: evaluate definition guard
                            match d.bindings.get(&key) {
                                Some(Binding::Exec { exec, .. }) => {
                                    self.key_source.ungrab_keyboard();
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
