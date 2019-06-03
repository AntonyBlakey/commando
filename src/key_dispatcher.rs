use super::{
    connection, help,
    keystroke::Keystroke,
    model::{Action, Context, Model},
};
use crossbeam::channel::{SendError, Sender};

pub struct KeyDispatcher {
    model: Model,
    help_tx: Sender<help::HelpMessage>,
    last_release: (xcb::Keycode, u16, xcb::Timestamp),
    keyboard_is_grabbed: bool,
}

impl KeyDispatcher {
    pub fn run(model: Model) {
        let (sender, receiver) = crossbeam::channel::bounded(0);
        std::thread::spawn(move || help::HelpWindow::new().run(receiver));
        KeyDispatcher {
            model,
            help_tx: sender,
            last_release: (0, 0, 0),
            keyboard_is_grabbed: false,
        }
        .run_top_level_event_loop()
        .unwrap();
    }

    fn run_top_level_event_loop(&mut self) -> Result<(), SendError<help::HelpMessage>> {
        log::debug!("Enter top level runloop");

        let context = Context {};
        let bindings = self.model.get_applicable_bindings("@root", &context);
        self.help_tx.send(help::HelpMessage::Update(bindings))?;
        connection::grab_keys(&self.model.get_root_grab_keys());

        while let Some(keystroke) = self.wait_for_keystroke() {
            connection::ungrab_keyboard();
            self.help_tx.send(help::HelpMessage::Disarm)?;
            if let Some(binding) = self.model.get_binding("@root", &context, keystroke) {
                self.handle_action(&context, &binding.action())?;
                match binding.action() {
                    Action::Mode(_) => {
                        let bindings = self.model.get_applicable_bindings("@root", &context);
                        self.help_tx.send(help::HelpMessage::Update(bindings))?;
                    }
                    _ => {}
                }
            }
        }

        log::debug!("Exit top level runloop");

        Ok(())
    }

    fn run_modal_event_loop(&mut self, mode: &str) -> Result<(), SendError<help::HelpMessage>> {
        log::debug!("Enter runloop for mode {}", mode);

        let context = Context {};
        let bindings = self.model.get_applicable_bindings(mode, &context);
        self.help_tx.send(help::HelpMessage::Update(bindings))?;
        self.help_tx.send(help::HelpMessage::Arm)?;

        while let Some(keystroke) = self.wait_for_keystroke() {
            self.help_tx.send(help::HelpMessage::Disarm)?;
            if let Some(binding) = self.model.get_binding(mode, &context, keystroke) {
                self.handle_action(&context, &binding.action())?;
                match binding.action() {
                    Action::Cancel | Action::Mode(_) | Action::Exec(_) => break,
                    _ => {}
                }
            }
        }

        log::debug!("Exit runloop for mode {}", mode);

        Ok(())
    }

    fn handle_action(
        &mut self,
        context: &Context,
        action: &Action,
    ) -> Result<(), SendError<help::HelpMessage>> {
        match action {
            Action::Cancel => {
                self.help_tx.send(help::HelpMessage::Cancel)?;
            }

            Action::Mode(new_mode) => {
                self.set_keyboard_is_grabbed(true);
                self.run_modal_event_loop(new_mode)?;
                self.set_keyboard_is_grabbed(false);
            }

            Action::Exec(action) => {
                self.help_tx.send(help::HelpMessage::Cancel)?;
                self.set_keyboard_is_grabbed(false);
                action(context);
            }

            Action::Call(action) => action(context),

            Action::ToggleHelp => self.help_tx.send(help::HelpMessage::Toggle)?,
        }

        Ok(())
    }

    fn wait_for_keystroke(&mut self) -> Option<Keystroke> {
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
                        self.wait_for_key_release(press_event.detail());
                        continue;
                    }
                    let key = Keystroke::from(press_event);
                    if !key.is_modifier() {
                        if self.wait_for_key_release(press_event.detail())
                            == Some(press_event.state())
                        {
                            log::debug!("Got keystroke {}", key);
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
                            log::debug!("Got keystroke {}", key);
                            return Some(key);
                        }
                    }
                    last_modifier = None;
                }

                xcb::EXPOSE => {
                    self.help_tx.send(help::HelpMessage::Draw).unwrap();
                }

                _ => {}
            }
        }

        return None;
    }

    fn wait_for_key_release(&mut self, keycode: xcb::Keycode) -> Option<u16> {
        let mut is_cancelled = false;
        while let Some(event) = connection::wait_for_event() {
            match event.response_type() {
                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    self.last_release = (
                        release_event.detail(),
                        release_event.state(),
                        release_event.time(),
                    );
                    if release_event.detail() == keycode {
                        return if is_cancelled {
                            None
                        } else {
                            Some(release_event.state())
                        };
                    }
                }

                xcb::KEY_PRESS => {
                    is_cancelled = true;
                }

                xcb::EXPOSE => {
                    self.help_tx.send(help::HelpMessage::Draw).unwrap();
                }

                _ => {}
            }
        }

        return None;
    }

    fn set_keyboard_is_grabbed(&mut self, keyboard_is_grabbed: bool) {
        if keyboard_is_grabbed != self.keyboard_is_grabbed {
            if keyboard_is_grabbed {
                connection::grab_keyboard();
            } else {
                connection::ungrab_keyboard();
            }
            self.keyboard_is_grabbed = keyboard_is_grabbed;
        }
    }
}
