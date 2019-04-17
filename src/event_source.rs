use crate::{connection::connection, key_description::KeyDescription};
use std::{cell::RefCell, collections::HashSet};

pub struct EventSource {
    modifier_keycodes: HashSet<xcb::xproto::Keycode>,
    pushed_back_event: RefCell<Option<xcb::base::GenericEvent>>, // RefCell because we need interior mutability
}

impl EventSource {
    pub fn new() -> Self {
        let connection = connection();
        Self {
            modifier_keycodes: {
                let mmc = xcb::xproto::get_modifier_mapping(&connection);
                let mm = mmc.get_reply().unwrap();
                let width = mm.keycodes_per_modifier();
                let keycodes = mm.keycodes();
                let mut seen = HashSet::new();
                for mod_index in 0..8 {
                    for j in 0..width {
                        let keycode = keycodes[mod_index * (width as usize) + (j as usize)];
                        if keycode != 0 {
                            seen.insert(keycode);
                        }
                    }
                }
                seen
            },
            pushed_back_event: Default::default(),
        }
    }

    pub fn grab_keys<'a, T>(&self, descriptions: T)
    where
        T: Iterator<Item = &'a KeyDescription>,
    {
        let connection = connection();
        let root = connection.get_setup().roots().nth(0).unwrap().root();
        for desc in descriptions {
            xcb::xproto::grab_key(
                &connection,
                false,
                root,
                desc.modifiers(),
                desc.keycode(),
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_SYNC as u8,
            );
        }
        connection.flush();
    }

    pub fn grab_keyboard(&self) {
        let connection = connection();
        let root = connection.get_setup().roots().nth(0).unwrap().root();
        xcb::xproto::grab_keyboard(
            &connection,
            false,
            root,
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_SYNC as u8,
        );
        connection.flush();
    }

    pub fn ungrab_keyboard(&self) {
        let connection = connection();
        xcb::xproto::ungrab_keyboard(&connection, xcb::CURRENT_TIME);
        connection.flush();
    }

    pub fn wait_for_event<F>(&self, expose_handler: &F) -> Option<KeyDescription>
    where
        F: Fn(&xcb::ExposeEvent),
    {
        while let Some(event) = self.wait_for_raw_event() {
            if event.response_type() == xcb::KEY_PRESS {
                let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                if !self.modifier_keycodes.contains(&press_event.detail()) {
                    if let Some(key) = self.wait_for_event_release(&press_event, expose_handler) {
                        return Some(key);
                    }
                }
            } else if event.response_type() == xcb::EXPOSE {
                let expose_event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&event) };
                expose_handler(expose_event);
            }
        }

        return None;
    }

    fn wait_for_event_release<F>(
        &self,
        press_event: &xcb::KeyPressEvent,
        expose_handler: &F,
    ) -> Option<KeyDescription>
    where
        F: Fn(&xcb::ExposeEvent),
    {
        while let Some(event) = self.wait_for_raw_event() {
            match event.response_type() {
                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    if release_event.detail() != press_event.detail() {
                        self.wait_for_cancelled_key_release(press_event, expose_handler);
                        return None;
                    }
                    if release_event.state() != press_event.state() {
                        return None;
                    }
                    // We have a repeat if the next event is a press with identical
                    // detail, state and time. Thus we peek ahead to see if it's in
                    // the queue. If the queue is empty, that means it's not a repeat
                    // because the RELEASE+PRESS pair are queued together.
                    if let Some(next_event) = self.poll_for_raw_event() {
                        if next_event.response_type() == xcb::KEY_PRESS {
                            let second_press_event: &xcb::KeyPressEvent =
                                unsafe { xcb::cast_event(&next_event) };
                            if second_press_event.detail() == release_event.detail()
                                && second_press_event.state() == release_event.state()
                                && second_press_event.time() == release_event.time()
                            {
                                continue;
                            }
                        }
                        self.pushback_raw_event(next_event);
                    }
                    return Some(KeyDescription::from_key_press_event(&press_event));
                }
                xcb::KEY_PRESS => {
                    self.wait_for_cancelled_key_release(press_event, expose_handler);
                    return None;
                }
                xcb::EXPOSE => {
                    let expose_event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&event) };
                    expose_handler(expose_event);
                }
                _ => (),
            }
        }

        return None;
    }

    fn wait_for_cancelled_key_release<F>(
        &self,
        press_event: &xcb::KeyPressEvent,
        expose_handler: &F,
    ) where
        F: Fn(&xcb::ExposeEvent),
    {
        while let Some(event) = self.wait_for_raw_event() {
            if event.response_type() == xcb::KEY_RELEASE {
                let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                if release_event.detail() == press_event.detail() {
                    return;
                }
            } else if event.response_type() == xcb::EXPOSE {
                let expose_event: &xcb::ExposeEvent = unsafe { xcb::cast_event(&event) };
                expose_handler(expose_event);
            }
        }
    }

    fn poll_for_raw_event(&self) -> Option<xcb::base::GenericEvent> {
        let pushed_back_event = self.pushed_back_event.replace(None);
        if pushed_back_event.is_some() {
            return pushed_back_event;
        }

        self.allow_events();
        connection().poll_for_event()
    }

    fn wait_for_raw_event(&self) -> Option<xcb::base::GenericEvent> {
        let pushed_back_event = self.pushed_back_event.replace(None);
        if pushed_back_event.is_some() {
            return pushed_back_event;
        }

        self.allow_events();
        connection().wait_for_event()
    }

    fn pushback_raw_event(&self, event: xcb::base::GenericEvent) {
        self.pushed_back_event.replace(Some(event));
    }

    fn allow_events(&self) {
        let connection = connection();
        xcb::xproto::allow_events(
            &connection,
            xcb::ALLOW_SYNC_KEYBOARD as u8,
            xcb::CURRENT_TIME,
        );
        connection.flush();
    }
}
