use super::key_description::KeyDescription;
use std::{cell::RefCell, collections::HashSet};

pub struct KeySource<'a> {
    connection: &'a xcb::Connection,
    screen_number: i32,
    modifier_keycodes: HashSet<xcb::xproto::Keycode>,
    key_symbols: xcb_util::keysyms::KeySymbols<'a>,
    pushed_back_event: RefCell<Option<xcb::base::GenericEvent>>, // RefCell because we need interior mutability
}

impl<'a> KeySource<'a> {
    pub fn new(connection: &'a xcb::Connection, screen_number: i32) -> Self {
        Self {
            connection,
            screen_number,
            modifier_keycodes: {
                let mmc = xcb::xproto::get_modifier_mapping(connection);
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
            key_symbols: xcb_util::keysyms::KeySymbols::new(connection),
            pushed_back_event: Default::default(),
        }
    }

    pub fn key_symbols(&self) -> &xcb_util::keysyms::KeySymbols {
        &self.key_symbols
    }

    pub fn grab_keys<T>(&self, descriptions: T)
    where
        T: Iterator<Item = &'a KeyDescription>,
    {
        for desc in descriptions {
            xcb::xproto::grab_key(
                self.connection,
                false,
                self.screen().root(),
                desc.modifiers(),
                desc.keycode(),
                xcb::GRAB_MODE_ASYNC as u8,
                xcb::GRAB_MODE_SYNC as u8,
            );
        }
        self.connection.flush();
    }

    // pub fn ungrab_keys(&self) {
    //     xcb::xproto::ungrab_key(
    //         self.connection,
    //         xcb::GRAB_ANY as u8,
    //         self.screen().root(),
    //         xcb::MOD_MASK_ANY as u16,
    //     );
    //     self.connection.flush();
    // }

    pub fn grab_keyboard(&self) {
        xcb::xproto::grab_keyboard(
            self.connection,
            false,
            self.screen().root(),
            xcb::CURRENT_TIME,
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_SYNC as u8,
        );
        self.connection.flush();
    }

    pub fn ungrab_keyboard(&self) {
        xcb::xproto::ungrab_keyboard(self.connection, xcb::CURRENT_TIME);
        self.connection.flush();
    }

    pub fn poll_for_event(&self) -> Option<xcb::base::GenericEvent> {
        let pushed_back_event = self.pushed_back_event.replace(None);
        if pushed_back_event.is_some() {
            return pushed_back_event;
        }

        self.allow_events();
        self.connection.poll_for_event()
    }

    pub fn pushback_event(&self, event: xcb::base::GenericEvent) {
        self.pushed_back_event.replace(Some(event));
    }

    pub fn wait_for_key(&self) -> Option<KeyDescription> {
        while let Some(event) = self.wait_for_event() {
            if event.response_type() == xcb::KEY_PRESS {
                let press_event: &xcb::KeyPressEvent = unsafe { xcb::cast_event(&event) };
                if !self.modifier_keycodes.contains(&press_event.detail()) {
                    if let Some(key) = self.wait_for_key_release(&press_event) {
                        return Some(key);
                    }
                }
            }
        }

        return None;
    }

    fn wait_for_key_release(&self, press_event: &xcb::KeyPressEvent) -> Option<KeyDescription> {
        while let Some(event) = self.wait_for_event() {
            match event.response_type() {
                xcb::KEY_RELEASE => {
                    let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                    if release_event.detail() != press_event.detail() {
                        self.wait_for_cancelled_key_release(press_event);
                        return None;
                    }
                    if release_event.state() != press_event.state() {
                        return None;
                    }
                    // We have a repeat if the next event is a press with identical
                    // detail, state and time. Thus we peek ahead to see if it's in
                    // the queue. If the queue is empty, that means it's not a repeat
                    // because the RELEASE+PRESS pair are queued together.
                    if let Some(next_event) = self.poll_for_event() {
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
                        self.pushback_event(next_event);
                    }
                    return Some(KeyDescription::from_key_press_event(&press_event));
                }
                xcb::KEY_PRESS => {
                    self.wait_for_cancelled_key_release(press_event);
                    return None;
                }
                _ => (),
            }
        }

        return None;
    }

    fn wait_for_cancelled_key_release(&self, press_event: &xcb::KeyPressEvent) {
        while let Some(event) = self.wait_for_event() {
            if event.response_type() == xcb::KEY_RELEASE {
                let release_event: &xcb::KeyReleaseEvent = unsafe { xcb::cast_event(&event) };
                if release_event.detail() == press_event.detail() {
                    return;
                }
            }
        }
    }

    fn allow_events(&self) {
        xcb::xproto::allow_events(
            self.connection,
            xcb::ALLOW_SYNC_KEYBOARD as u8,
            xcb::CURRENT_TIME,
        );
        self.connection.flush();
    }

    fn wait_for_event(&self) -> Option<xcb::base::GenericEvent> {
        let pushed_back_event = self.pushed_back_event.replace(None);
        if pushed_back_event.is_some() {
            return pushed_back_event;
        }

        self.allow_events();
        self.connection.wait_for_event()
    }

    fn screen(&self) -> xcb::Screen {
        self.connection
            .get_setup()
            .roots()
            .nth(self.screen_number as usize)
            .unwrap()
    }
}
