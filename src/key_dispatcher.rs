use super::{
    connection, help,
    keystroke::Keystroke,
    model::{Action, Context, Model},
};
use crossbeam::channel::{SendError, Sender};

pub struct KeyDispatcher {
    model: Model,
    help_window: help::HelpWindow,
}

impl KeyDispatcher {
    pub fn run(model: Model) {
        let (tx, rx) = crossbeam::channel::bounded(0);
        let help_window = help::HelpWindow::new();
        let w = help_window.window().clone();
        std::thread::spawn(move || super::help::run(w, rx));
        KeyDispatcher { model, help_window }
            .run_event_loop(None, &tx)
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
        self.help_window.update(bindings);
        match mode {
            None => connection::grab_keys(&self.model.get_root_grab_keys()),
            Some(_) => tx.send(help::HelpMessage::Arm)?,
        }

        while let Some(keystroke) = self.wait_for_keystroke() {
            log::debug!("Got keystroke {}", keystroke);
            tx.send(help::HelpMessage::Disarm)?;
            if let Some(binding) = self.model.get_binding(mode_name, &Context {}, keystroke) {
                match binding.action() {
                    Action::Cancel => {
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
                            connection::ungrab_keyboard();
                        }
                    }
                    Action::Call(action) => {
                        action(&context);
                    }
                    Action::Exec(action) => {
                        tx.send(help::HelpMessage::Cancel)?;
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

    pub fn wait_for_keystroke(&self) -> Option<Keystroke> {
        let mut last_modifier = None;
        while let Some(event) = connection::wait_for_event() {
            if event.response_type() == xcb::KEY_PRESS {
                last_modifier = None;
                let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                let key = Keystroke::from(press_event);
                if !key.is_modifier() {
                    if self.wait_for_key_release(&press_event) {
                        return Some(key);
                    }
                } else {
                    last_modifier = Some((key, press_event.detail()));
                }
            } else if event.response_type() == xcb::KEY_RELEASE {
                let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                if let Some((key, detail)) = last_modifier {
                    if detail == release_event.detail() {
                        return Some(key);
                    }
                }
                last_modifier = None;
            } else if event.response_type() == xcb::EXPOSE {
                self.help_window.draw();
            }
        }

        return None;
    }

    fn wait_for_key_release(&self, press_event: &xcb::KeyPressEvent) -> bool {
        while let Some(event) = connection::wait_for_event() {
            match event.response_type() {
                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    if release_event.detail() != press_event.detail() {
                        self.wait_for_cancelled_key_release(press_event);
                        return false;
                    }
                    if release_event.state() != press_event.state() {
                        return false;
                    }
                    // We have a repeat if the next event is a press with identical
                    // detail, state and time. Thus we peek ahead to see if it's in
                    // the queue. If the queue is empty, that means it's not a repeat
                    // because the RELEASE+PRESS pair are queued together.
                    if let Some(next_event) = connection::poll_for_event() {
                        eprintln!("Poll has value");
                        if next_event.response_type() == xcb::KEY_PRESS {
                            eprintln!("Repeat poll check");
                            let second_press_event: &xcb::KeyPressEvent =
                                unsafe { xcb::cast_event(&next_event) };
                            if second_press_event.detail() == release_event.detail()
                                && second_press_event.state() == release_event.state()
                                && second_press_event.time() == release_event.time()
                            {
                                eprintln!("   ... match");
                                continue;
                            }
                            eprintln!("   ... fail");
                        }
                        connection::pushback_event(next_event);
                    } else {
                        eprintln!("Poll failed");
                    }
                    return true;
                }
                xcb::KEY_PRESS => {
                    self.wait_for_cancelled_key_release(press_event);
                    return false;
                }
                xcb::EXPOSE => {
                    self.help_window.draw();
                }
                _ => (),
            }
        }

        return false;
    }

    fn wait_for_cancelled_key_release(&self, press_event: &xcb::KeyPressEvent) {
        while let Some(event) = connection::wait_for_event() {
            if event.response_type() == xcb::KEY_RELEASE {
                let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                if release_event.detail() == press_event.detail() {
                    return;
                }
            } else if event.response_type() == xcb::EXPOSE {
                self.help_window.draw();
            }
        }
    }
}
