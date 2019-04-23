use super::{
    action::*, connection, connection::connection, help::HelpWindow, keystroke::Keystroke, model::*,
};
use crossbeam::{
    channel::{SendError, Sender},
    scope,
};

pub struct KeyDispatcher<'a> {
    model: &'a Model,
    mode: &'static str,
    help_window: HelpWindow,
}

impl<'a> KeyDispatcher<'a> {
    pub fn run(model: Model) {
        scope(|s| {
            let (tx, rx) = crossbeam::channel::bounded(0);
            // s.spawn(|_| ActionServer::run(&model, rx));
            KeyDispatcher::new(&model).run_root_event_loop(tx).unwrap();
        })
        .unwrap();
    }

    fn new(model: &'a Model) -> KeyDispatcher {
        KeyDispatcher {
            model,
            mode: "@reset",
            help_window: HelpWindow::new(),
        }
    }

    fn run_root_event_loop(
        &mut self,
        tx: Sender<ActionMessage>,
    ) -> Result<(), SendError<ActionMessage>> {
        let connection = connection();
        let bindings = self.model.get_applicable_bindings("@root", &Context {});
        self.help_window.update(bindings);
        xcb::map_window(&connection, self.help_window.window());

        while let Some(e) = connection::wait_for_event() {
            match e.response_type() & 0x7f {
                xcb::EXPOSE => {
                    let event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&e) };
                    self.help_window.expose(event);
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn main_loop(&self, tx: Sender<ActionMessage>) -> Result<(), SendError<ActionMessage>> {
        // self.event_source
        //     .grab_keys(self.model.bindings.keys());
        // self.event_source.grab_keys(
        //     self.model
        //         .command_bindings
        //         .iter()
        //         .filter(|(_, c)| **c != Command::Cancel)
        //         .map(|(k, _)| k),
        // );

        // while let Some(key) = self.event_source.wait_for_event(&|_| {}) {
        //     match self.model.command_bindings.get(&key) {
        //         Some(Command::Cancel) => continue,
        //         Some(Command::ToggleHelp) => tx.send(ActionMessage::ToggleHelp)?,
        //         None => match self.model.bindings.get(&key) {
        //             Some(Binding::Exec { exec, .. }) => {
        //                 tx.send(ActionMessage::Exec(exec.clone()))?
        //             }
        //             Some(Binding::Call { call, .. }) => {
        //                 tx.send(ActionMessage::Call(call.clone()))?
        //             }
        //             Some(Binding::Mode { mode, .. }) => {
        //                 self.event_source.grab_keyboard();
        //                 tx.send(ActionMessage::Enter)?;
        //                 self.modal_loop(mode, &tx)?;
        //                 tx.send(ActionMessage::Exit)?;
        //             }
        //             None => (),
        //         },
        //     }
        // }

        Ok(())
    }

    fn modal_loop(
        &self,
        mode: &'static str,
        tx: &Sender<ActionMessage>,
    ) -> Result<(), SendError<ActionMessage>> {
        // match self.model.definitions.get(mode) {
        //     Some(definitions) => {
        //         tx.send(ActionMessage::Mode(mode.clone()))?;
        //         while let Some(key) = self.event_source.wait_for_event(&|_| {}) {
        //             match self.model.command_bindings.get(&key) {
        //                 Some(Command::Cancel) => {
        //                     self.event_source.ungrab_keyboard();
        //                     tx.send(ActionMessage::Cancel)?;
        //                     return Ok(());
        //                 }
        //                 Some(Command::ToggleHelp) => tx.send(ActionMessage::ToggleHelp)?,
        //                 None => {
        //                     for d in definitions {
        //                         // TODO: evaluate definition guard
        //                         match d.bindings.get(&key) {
        //                             Some(Binding::Exec { exec, .. }) => {
        //                                 self.event_source.ungrab_keyboard();
        //                                 return tx.send(ActionMessage::Exec(exec.clone()));
        //                             }
        //                             Some(Binding::Call { call, .. }) => {
        //                                 tx.send(ActionMessage::Call(call.clone()))?;
        //                                 break; // out of the definitions loop;
        //                             }
        //                             Some(Binding::Mode { mode, .. }) => {
        //                                 return self.modal_loop(mode, tx);
        //                             }
        //                             None => (),
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //     }
        //     None => self.event_source.ungrab_keyboard(),
        // }

        Ok(())
    }

    // fn run_modal_event_loop(&mut self) -> Option<String> {
    //     let mut key_press_count = 0;
    //     let mut selected_label: Option<String> = None;
    //     connection::grab_keyboard();
    //     connection::allow_events();
    //     let connection = connection();
    //     let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);
    //     while let Some(e) = connection.wait_for_event() {
    //         match e.response_type() & 0x7f {
    //             xcb::KEY_PRESS => {
    //                 let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&e) };
    //                 if key_press_count == 0 {
    //                     let keycode = press_event.detail();
    //                     let keysym = key_symbols.get_keysym(keycode, 0);
    //                     if keysym != xcb::base::NO_SYMBOL {
    //                         let cstr = unsafe {
    //                             std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
    //                         };
    //                         selected_label =
    //                             cstr.to_str().ok().map(|s| s.to_owned().to_uppercase());
    //                     }
    //                 } else {
    //                     selected_label = None;
    //                 }
    //                 key_press_count += 1;
    //             }
    //             xcb::KEY_RELEASE => {
    //                 key_press_count -= 1;
    //                 if key_press_count == 0 {
    //                     break;
    //                 }
    //             }
    //             _ => {
    //                 self.dispatch_wm_event(&e);
    //             }
    //         }
    //         connection::allow_events();
    //     }
    //     connection::ungrab_keyboard();
    //     connection::allow_events();
    //     selected_label
    // }

    // pub fn wait_for_event<F>(&self, expose_handler: &F) -> Option<Keystroke>
    // where
    //     F: Fn(&xcb::ExposeEvent),
    // {
    //     while let Some(event) = connection::wait_for_event() {
    //         if event.response_type() == xcb::KEY_PRESS {
    //             let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
    //             if !self.modifier_keycodes.contains(&press_event.detail()) {
    //                 if let Some(key) = self.wait_for_event_release(&press_event, expose_handler) {
    //                     return Some(key);
    //                 }
    //             }
    //         } else if event.response_type() == xcb::EXPOSE {
    //             let expose_event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&event) };
    //             expose_handler(expose_event);
    //         }
    //     }

    //     return None;
    // }

    // fn wait_for_event_release<F>(
    //     &self,
    //     press_event: &xcb::KeyPressEvent,
    //     expose_handler: &F,
    // ) -> Option<Keystroke>
    // where
    //     F: Fn(&xcb::ExposeEvent),
    // {
    //     while let Some(event) = connection::wait_for_event() {
    //         match event.response_type() {
    //             xcb::KEY_RELEASE => {
    //                 let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
    //                 if release_event.detail() != press_event.detail() {
    //                     self.wait_for_cancelled_key_release(press_event, expose_handler);
    //                     return None;
    //                 }
    //                 if release_event.state() != press_event.state() {
    //                     return None;
    //                 }
    //                 // We have a repeat if the next event is a press with identical
    //                 // detail, state and time. Thus we peek ahead to see if it's in
    //                 // the queue. If the queue is empty, that means it's not a repeat
    //                 // because the RELEASE+PRESS pair are queued together.
    //                 if let Some(next_event) = connection::poll_for_event() {
    //                     if next_event.response_type() == xcb::KEY_PRESS {
    //                         let second_press_event: &xcb::KeyPressEvent =
    //                             unsafe { xcb::cast_event(&next_event) };
    //                         if second_press_event.detail() == release_event.detail()
    //                             && second_press_event.state() == release_event.state()
    //                             && second_press_event.time() == release_event.time()
    //                         {
    //                             continue;
    //                         }
    //                     }
    //                     connection::pushback_event(next_event);
    //                 }
    //                 return Some(press_event.into());
    //             }
    //             xcb::KEY_PRESS => {
    //                 self.wait_for_cancelled_key_release(press_event, expose_handler);
    //                 return None;
    //             }
    //             xcb::EXPOSE => {
    //                 let expose_event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&event) };
    //                 expose_handler(expose_event);
    //             }
    //             _ => (),
    //         }
    //     }

    //     return None;
    // }

    // fn wait_for_cancelled_key_release<F>(
    //     &self,
    //     press_event: &xcb::KeyPressEvent,
    //     expose_handler: &F,
    // ) where
    //     F: Fn(&xcb::ExposeEvent),
    // {
    //     while let Some(event) = connection::wait_for_event() {
    //         if event.response_type() == xcb::KEY_RELEASE {
    //             let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
    //             if release_event.detail() == press_event.detail() {
    //                 return;
    //             }
    //         } else if event.response_type() == xcb::EXPOSE {
    //             let expose_event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&event) };
    //             expose_handler(expose_event);
    //         }
    //     }
    // }
}
