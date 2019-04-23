use super::connection::connection;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct Keystroke {
    modifiers: u16,
    keycode: u8,
}

impl Keystroke {
    pub fn make_raw(tokens: &[&str]) -> Vec<Self> {
        match tokens.split_last() {
            Some((key, modifiers)) => Self::make(modifiers, key),
            None => Default::default(),
        }
    }
    pub fn make(modifiers: &[&str], key: &str) -> Vec<Self> {
        match key {
            // Alternate names for modifier keys
            "Windows" | "Win" => Self::make(modifiers, "Super"),
            "Windows_L" | "Win_L" => Self::make(modifiers, "Super_L"),
            "Windows_R" | "Win_R" => Self::make(modifiers, "Super_R"),

            "Command" | "Cmd" => Self::make(modifiers, "Super"),
            "Command_L" | "Cmd_L" => Self::make(modifiers, "Super_L"),
            "Command_R" | "Cmd_R" => Self::make(modifiers, "Super_R"),

            "Ctrl" => Self::make(modifiers, "Control"),
            "Ctrl_L" => Self::make(modifiers, "Control_L"),
            "Ctrl_R" => Self::make(modifiers, "Control_R"),

            "Opt" => Self::make(modifiers, "Alt"),
            "Opt_L" => Self::make(modifiers, "Alt_L"),
            "Opt_R" => Self::make(modifiers, "Alt_R"),

            // Modifier keys that actually have *_L and *_R
            "Hyper" => Self::make_left_right(modifiers, "Hyper"),
            "Super" => Self::make_left_right(modifiers, "Super"),
            "Meta" => Self::make_left_right(modifiers, "Meta"),
            "Control" => Self::make_left_right(modifiers, "Control"),
            "Alt" => Self::make_left_right(modifiers, "Alt"),
            "Shift" => Self::make_left_right(modifiers, "Shift"),

            // Normal keys
            _ => {
                let connection = connection();
                let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);

                // TODO: look these up dynamically using the xmodmap code in elucidate.rs
                let mod_mask = modifiers.iter().fold(0, |accum, &m| {
                    accum
                        | match m {
                            "Hyper" => xcb::KEY_BUT_MASK_MOD_3 as u16,
                            "Super" | "Windows" | "Win" | "Command" | "Cmd" => {
                                xcb::KEY_BUT_MASK_MOD_4 as u16
                            }
                            "Control" | "Ctrl" => xcb::KEY_BUT_MASK_CONTROL as u16,
                            "Alt" | "Opt" | "Meta" => xcb::KEY_BUT_MASK_MOD_1 as u16,
                            "Shift" => xcb::KEY_BUT_MASK_SHIFT as u16,
                            _ => 0,
                        }
                });

                let mut result = Vec::new();
                let keysym = xkbcommon::xkb::keysym_from_name(key, xkbcommon::xkb::KEYSYM_NO_FLAGS);
                if keysym != xcb::NO_SYMBOL {
                    for keycode in key_symbols.get_keycode(keysym) {
                        let keysym_unshifted = key_symbols.get_keysym(keycode, 0);
                        let keysym_shifted = key_symbols.get_keysym(keycode, 1);
                        // If the key specifies a shifted symbol AND the shift key,
                        // then it's impossible to press i.e we must ignore "shift-Q"
                        let keysym_is_shifted =
                            keysym == keysym_shifted && keysym != keysym_unshifted;
                        if !keysym_is_shifted || mod_mask & xcb::KEY_BUT_MASK_SHIFT as u16 == 0 {
                            result.push(if keysym_is_shifted {
                                Self {
                                    modifiers: mod_mask | xcb::KEY_BUT_MASK_SHIFT as u16,
                                    keycode,
                                }
                            } else {
                                Self {
                                    modifiers: mod_mask,
                                    keycode,
                                }
                            });
                        }
                    }
                }
                result
            }
        }
    }

    pub fn parse(string: &str) -> Vec<Self> {
        let tokens: Vec<&str> = string.split('-').collect();
        match tokens.split_last() {
            Some((keysym_name, raw_modifiers)) => Self::make(raw_modifiers, keysym_name),
            None => Default::default(),
        }
    }

    pub fn modifiers(&self) -> u16 {
        self.modifiers
    }

    pub fn keycode(&self) -> u8 {
        self.keycode
    }

    fn make_left_right(modifiers: &[&str], key: &str) -> Vec<Self> {
        Self::make(modifiers, &format!("{}_L", key))
            .iter()
            .chain(Self::make(modifiers, &format!("{}_R", key)).iter())
            .copied()
            .collect()
    }
}

impl From<&xcb::KeyPressEvent> for Keystroke {
    fn from(event: &xcb::KeyPressEvent) -> Self {
        Self {
            modifiers: event.state()
                & (xcb::KEY_BUT_MASK_SHIFT
                    | xcb::KEY_BUT_MASK_LOCK
                    | xcb::KEY_BUT_MASK_CONTROL
                    | xcb::KEY_BUT_MASK_MOD_1
                    | xcb::KEY_BUT_MASK_MOD_2
                    | xcb::KEY_BUT_MASK_MOD_3
                    | xcb::KEY_BUT_MASK_MOD_4
                    | xcb::KEY_BUT_MASK_MOD_5) as u16,
            keycode: event.detail(),
        }
    }
}

impl Display for Keystroke {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        let connection = connection();
        let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);

        let f1 = {
            let keysym = key_symbols.get_keysym(self.keycode, 0);
            if keysym == xcb::base::NO_SYMBOL {
                None
            } else {
                let keysym_name = unsafe {
                    std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
                        .to_str()
                        .unwrap()
                };
                Some(format!(
                    "{}{}{}{}{}{}{}{}",
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_3 as u16 != 0 {
                        "hyper-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_4 as u16 != 0 {
                        "super-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_2 as u16 != 0 {
                        "mod2-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_5 as u16 != 0 {
                        "mod5-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_CONTROL as u16 != 0 {
                        "control-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_1 as u16 != 0 {
                        "alt-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_SHIFT as u16 != 0 {
                        "shift-"
                    } else {
                        ""
                    },
                    keysym_name
                ))
            }
        };
        let f2 = {
            let keysym = key_symbols.get_keysym(self.keycode, 1);
            if !self.modifiers & xcb::KEY_BUT_MASK_SHIFT as u16 != 0
                || keysym == xcb::base::NO_SYMBOL
            {
                None
            } else {
                let keysym_name = unsafe {
                    std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
                        .to_str()
                        .unwrap()
                };
                Some(format!(
                    "{}{}{}{}{}{}{}",
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_3 as u16 != 0 {
                        "hyper-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_4 as u16 != 0 {
                        "super-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_2 as u16 != 0 {
                        "mod2-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_5 as u16 != 0 {
                        "mod5-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_CONTROL as u16 != 0 {
                        "control-"
                    } else {
                        ""
                    },
                    if self.modifiers & xcb::KEY_BUT_MASK_MOD_1 as u16 != 0 {
                        "alt-"
                    } else {
                        ""
                    },
                    keysym_name
                ))
            }
        };

        match (f1, f2) {
            (Some(a), Some(b)) => write!(formatter, "{} / {}", a, b),
            (Some(a), None) => write!(formatter, "{}", a),
            (None, Some(b)) => write!(formatter, "{}", b),
            (None, None) => write!(formatter, "Invalid Key Description"),
        }
    }
}

#[macro_export]
macro_rules! key {
    // The unfolding of the modifier sequence is required to get around a weakness in Rust's macro pattern matching
    (@m $($m:ident)* + $key:tt) => { $crate::keystroke::Keystroke::make(&[ $(stringify!($m)),*], stringify!($key)) };
    ($key:tt) => { key!(@m + $key) };
    ($m1:ident + $key:tt) => { key!(@m $m1 + $key) };
    ($m1:ident + $m2:ident + $key:tt) => { key!(@m $m1 $m2 + $key) };
    ($m1:ident + $m2:ident + $m3:ident + $key:tt) => { key!(@m $m1 $m2 $m3 + $key) };
    ($m1:ident + $m2:ident + $m3:ident + $m4:ident + $key:tt) => { key!(@m $m1 $m2 $m3 $m4 + $key) };
    ($m1:ident + $m2:ident + $m3:ident + $m4:ident + $m5:ident + $key:tt) => { key!(@m $m1 $m2 $m3 $m3 $m5 + $key) };
}
