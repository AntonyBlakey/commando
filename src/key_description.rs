#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct KeyDescription {
    shift: bool,
    lock: bool,
    control: bool,
    mod1: bool,
    mod2: bool,
    mod3: bool,
    mod4: bool,
    mod5: bool,
    keycode: xcb::xproto::Keycode,
}

impl KeyDescription {
    fn from_keycode_and_state(keycode: xcb::xproto::Keycode, state: u16) -> Self {
        KeyDescription {
            shift: state & (xcb::xproto::KEY_BUT_MASK_SHIFT as u16) != 0,
            lock: state & (xcb::xproto::KEY_BUT_MASK_LOCK as u16) != 0,
            control: state & (xcb::xproto::KEY_BUT_MASK_CONTROL as u16) != 0,
            mod1: state & (xcb::xproto::KEY_BUT_MASK_MOD_1 as u16) != 0,
            mod2: state & (xcb::xproto::KEY_BUT_MASK_MOD_2 as u16) != 0,
            mod3: state & (xcb::xproto::KEY_BUT_MASK_MOD_3 as u16) != 0,
            mod4: state & (xcb::xproto::KEY_BUT_MASK_MOD_4 as u16) != 0,
            mod5: state & (xcb::xproto::KEY_BUT_MASK_MOD_5 as u16) != 0,
            keycode: keycode,
        }
    }

    pub fn from_key_press_event(event: &xcb::KeyPressEvent) -> KeyDescription {
        KeyDescription::from_keycode_and_state(event.detail(), event.state())
    }

    pub fn from_string(string: &str, syms: &xcb_util::keysyms::KeySymbols) -> Vec<KeyDescription> {
        let mut result = Vec::new();

        let tokens: Vec<&str> = string.split('-').collect();
        if let Some((keysym_name, raw_modifiers)) = tokens.split_last() {
            let mut shift = false;
            let mut control = false;
            let mut mod1 = false;
            let mut mod3 = false;
            let mut mod4 = false;
            for x in raw_modifiers {
                match x.to_lowercase().as_str() {
                    "shift" | "s" => shift = true,
                    "control" | "ctrl" | "c" => control = true,
                    "alt" | "a" | "opt" | "o" | "meta" | "m" => mod1 = true,
                    "hyper" | "h" => mod3 = true,
                    "super" | "windows" | "win" | "w" | "command" | "cmd" => mod4 = true,
                    _ => (),
                }
            }

            let keysym =
                xkbcommon::xkb::keysym_from_name(keysym_name, xkbcommon::xkb::KEYSYM_NO_FLAGS);
            // TODO: check it's a valid keysym
            for keycode in syms.get_keycode(keysym) {
                let keysym_unshifted = syms.get_keysym(keycode, 0);
                let keysym_shifted = syms.get_keysym(keycode, 1);
                let mut kd = KeyDescription {
                    shift: shift,
                    lock: false,
                    control: control,
                    mod1: mod1,
                    mod2: false,
                    mod3: mod3,
                    mod4: mod4,
                    mod5: false,
                    keycode: keycode,
                };
                // If the user specifies a shifted symbol AND the shift key,
                // then it's impossible to press i.e we must ignore "shift-Q"
                let keysym_is_shifted = keysym == keysym_shifted && keysym != keysym_unshifted;
                if !keysym_is_shifted || !kd.shift {
                    kd.shift |= keysym_is_shifted;
                    result.push(kd);
                }
            }
        }

        result
    }

    pub fn modifiers(&self) -> u16 {
        let mut result: u16 = 0;
        if self.mod1 {
            result |= xcb::xproto::KEY_BUT_MASK_MOD_1 as u16
        };
        if self.mod2 {
            result |= xcb::xproto::KEY_BUT_MASK_MOD_2 as u16
        };
        if self.mod3 {
            result |= xcb::xproto::KEY_BUT_MASK_MOD_3 as u16
        };
        if self.mod4 {
            result |= xcb::xproto::KEY_BUT_MASK_MOD_4 as u16
        };
        if self.mod5 {
            result |= xcb::xproto::KEY_BUT_MASK_MOD_5 as u16
        };
        if self.control {
            result |= xcb::xproto::KEY_BUT_MASK_CONTROL as u16
        };
        if self.lock {
            result |= xcb::xproto::KEY_BUT_MASK_LOCK as u16
        };
        if self.shift {
            result |= xcb::xproto::KEY_BUT_MASK_SHIFT as u16
        };
        result
    }

    pub fn keycode(&self) -> xcb::xproto::Keycode {
        self.keycode
    }

    //     pub fn pretty_string(&self, syms: &xcb_util::keysyms::KeySymbols) -> String {
    //         let f1 = {
    //             let keysym = syms.get_keysym(self.keycode, 0);
    //             if keysym == xcb::base::NO_SYMBOL {
    //                 None
    //             } else {
    //                 let keysym_name = unsafe {
    //                     std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
    //                         .to_str()
    //                         .unwrap()
    //                 };
    //                 Some(format!(
    //                     "{}{}{}{}{}{}{}{}{}",
    //                     if self.mod1 { "alt-" } else { "" },
    //                     if self.mod2 { "mod2-" } else { "" },
    //                     if self.mod3 { "hyper-" } else { "" },
    //                     if self.mod4 { "super-" } else { "" },
    //                     if self.mod5 { "mod5-" } else { "" },
    //                     if self.control { "control-" } else { "" },
    //                     if self.lock { "lock-" } else { "" },
    //                     if self.shift { "shift-" } else { "" },
    //                     keysym_name
    //                 ))
    //             }
    //         };
    //         let f2 = {
    //             let keysym = syms.get_keysym(self.keycode, 1);
    //             if !self.shift || keysym == xcb::base::NO_SYMBOL {
    //                 None
    //             } else {
    //                 let keysym_name = unsafe {
    //                     std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
    //                         .to_str()
    //                         .unwrap()
    //                 };
    //                 Some(format!(
    //                     "{}{}{}{}{}{}{}{}",
    //                     if self.mod1 { "alt-" } else { "" },
    //                     if self.mod2 { "mod2-" } else { "" },
    //                     if self.mod3 { "hyper-" } else { "" },
    //                     if self.mod4 { "super-" } else { "" },
    //                     if self.mod5 { "mod5-" } else { "" },
    //                     if self.control { "control-" } else { "" },
    //                     if self.lock { "lock-" } else { "" },
    //                     keysym_name,
    //                 ))
    //             }
    //         };

    //         match (f1, f2) {
    //             (Some(a), Some(b)) => format!("{} / {}", a, b),
    //             (Some(a), None) => a,
    //             (None, Some(b)) => b,
    //             (None, None) => String::from("Invalid"),
    //         }
    //     }
}
