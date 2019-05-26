use super::{
    connection, help,
    keystroke::Keystroke,
    model::{Action, Context, Model},
};
use crossbeam::channel::{SendError, Sender};

pub struct KeyDispatcher {
    model: Model,
    last_release: (xcb::Keycode, u16, xcb::Timestamp),
}

impl KeyDispatcher {
    pub fn run(model: Model) {
        let (sender, receiver) = crossbeam::channel::bounded(0);
        std::thread::spawn(move || help::HelpWindow::new().run(receiver));
        KeyDispatcher {
            model,
            last_release: (0, 0, 0),
        }
        .run_event_loop(None, &sender)
        .unwrap();
    }

    fn run_event_loop(
        &mut self,
        mode: Option<&str>,
        tx: &Sender<help::HelpMessage>,
    ) -> Result<(), SendError<help::HelpMessage>> {
        let context = Context {};
        let mode_name = mode.unwrap_or("@root");

        log::debug!("Enter runloop for mode {}", mode_name);

        let bindings = self.model.get_applicable_bindings(mode_name, &Context {});
        tx.send(help::HelpMessage::Update(bindings))?;
        match mode {
            None => connection::grab_keys(&self.model.get_root_grab_keys()),
            Some(_) => tx.send(help::HelpMessage::Arm)?,
        }

        while let Some(keystroke) = self.wait_for_keystroke(tx) {
            log::debug!("Got keystroke {}", keystroke);
            tx.send(help::HelpMessage::Disarm)?;
            if let Some(binding) = self.model.get_binding(mode_name, &Context {}, keystroke) {
                match binding.action() {
                    Action::Cancel => {
                        connection::ungrab_keyboard();
                        tx.send(help::HelpMessage::Cancel)?;
                        if mode.is_some() {
                            break;
                        }
                    }

                    Action::ToggleHelp => tx.send(help::HelpMessage::Toggle)?,

                    Action::Mode(new_mode) => {
                        if mode.is_none() {
                            connection::grab_keyboard();
                        }
                        self.run_event_loop(Some(new_mode), tx)?;
                        if mode.is_none() {
                            let bindings =
                                self.model.get_applicable_bindings(mode_name, &Context {});
                            tx.send(help::HelpMessage::Update(bindings))?;
                        }
                    }

                    Action::Call(action) => {
                        action(&context);
                    }

                    Action::Exec(action) => {
                        tx.send(help::HelpMessage::Cancel)?;
                        connection::ungrab_keyboard();
                        action(&context);
                        if mode.is_some() {
                            break;
                        }
                    }
                }
            }
        }

        log::debug!("Exit runloop for mode {}", mode_name);

        Ok(())
    }

    pub fn wait_for_keystroke(&mut self, tx: &Sender<help::HelpMessage>) -> Option<Keystroke> {
        let mut last_modifier = None;
        while let Some(event) = connection::wait_for_event() {
            match event.response_type() {
                xcb::KEY_PRESS => {
                    last_modifier = None;
                    let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                    if press_event.detail() == self.last_release.0
                        && press_event.state() == self.last_release.1
                        && press_event.time() == self.last_release.2
                    {
                        // These conditions indicate a key repeat
                        self.wait_for_key_release(&press_event, tx);
                        continue;
                    }
                    let key = Keystroke::from(press_event);
                    if !key.is_modifier() {
                        if self.wait_for_key_release(&press_event, tx) {
                            return Some(key);
                        }
                    } else {
                        last_modifier = Some((key, press_event.detail()));
                    }
                }

                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    self.last_release = (
                        release_event.detail(),
                        release_event.state(),
                        release_event.time(),
                    );
                    if let Some((key, detail)) = last_modifier {
                        if detail == release_event.detail() {
                            return Some(key);
                        }
                    }
                    last_modifier = None;
                }

                xcb::EXPOSE => {
                    tx.send(help::HelpMessage::Draw).unwrap();
                }

                _ => {}
            }
        }

        return None;
    }

    fn wait_for_key_release(
        &mut self,
        press_event: &xcb::KeyPressEvent,
        tx: &Sender<help::HelpMessage>,
    ) -> bool {
        while let Some(event) = connection::wait_for_event() {
            match event.response_type() {
                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    self.last_release = (
                        release_event.detail(),
                        release_event.state(),
                        release_event.time(),
                    );
                    if release_event.detail() != press_event.detail() {
                        self.wait_for_cancelled_key_release(press_event, tx);
                        return false;
                    }
                    if release_event.state() != press_event.state() {
                        return false;
                    }
                    return true;
                }

                xcb::KEY_PRESS => {
                    self.wait_for_cancelled_key_release(press_event, tx);
                    return false;
                }

                xcb::EXPOSE => {
                    tx.send(help::HelpMessage::Draw).unwrap();
                }

                _ => {}
            }
        }

        return false;
    }

    fn wait_for_cancelled_key_release(
        &mut self,
        press_event: &xcb::KeyPressEvent,
        tx: &Sender<help::HelpMessage>,
    ) {
        while let Some(event) = connection::wait_for_event() {
            match event.response_type() {
                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    self.last_release = (
                        release_event.detail(),
                        release_event.state(),
                        release_event.time(),
                    );
                    if release_event.detail() == press_event.detail() {
                        return;
                    }
                }

                xcb::EXPOSE => {
                    tx.send(help::HelpMessage::Draw).unwrap();
                }

                _ => {}
            }
        }
    }
}
