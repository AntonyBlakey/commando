use crate::connection::connection;

#[macro_export]
macro_rules! key {

    // This is the "most general" formulation. Leave it here for my future reference.

    // (@modifier $v:ident $head:tt) => { $v.push(stringify!($head)) }; // Modifiers
    // (@key $v:ident $tail:tt) => { $v.push(stringify!($tail)) }; // Key
    // (@decompose $v:ident $tail:tt) => { key!(@key $v $tail) };
    // (@decompose $v:ident $head:tt $($tail:tt)+) => { key!(@modifier $v $head); key!(@decompose $v $($tail)+); };
    // (@len $len:literal $tail:tt) => { $len + 1 };
    // (@len $len:literal $head:tt $($tail:tt)+) => { key!(@len ($len + 1) $($tail)+) };
    // ($head:tt $(+ $tail:tt)*) => {{ let mut result = Vec::with_capacity(key!(@len 0 $head $($tail)*)); key!(@decompose result $head $($tail)*); result }};

    (@m) => { 0 };

    (@m Shift) => { xcb::xproto::KEY_BUT_MASK_SHIFT };
    (@m Control) =>{ xcb::xproto::KEY_BUT_MASK_CONTROL };
    (@m Ctrl) => { xcb::xproto::KEY_BUT_MASK_CONTROL };
    (@m Alt) => { xcb::xproto::KEY_BUT_MASK_MOD_1 };
    (@m Opt) => { xcb::xproto::KEY_BUT_MASK_MOD_1 };
    (@m Meta) => { xcb::xproto::KEY_BUT_MASK_MOD_1 };
    (@m Hyper) => { xcb::xproto::KEY_BUT_MASK_MOD_3 };
    (@m Super) => { xcb::xproto::KEY_BUT_MASK_MOD_4 };
    (@m Windows) => { xcb::xproto::KEY_BUT_MASK_MOD_4 };
    (@m Win) => { xcb::xproto::KEY_BUT_MASK_MOD_4 };
    (@m Command) => { xcb::xproto::KEY_BUT_MASK_MOD_4 };
    (@m Cmd) => { xcb::xproto::KEY_BUT_MASK_MOD_4 };

    (@m $head:ident $($tail:ident)+) => { key!(@m $head) | key!(@m $($tail)+) };

    ($key:tt) => { user::make_key(key!(@m), stringify!($key)) };
    ($m1:ident + $key:tt) => { user::make_key(key!(@m $m1), stringify!($key)) };
    ($m1:ident + $m2:ident + $key:tt) => { user::make_key(key!(@m $m1 $m2), stringify!($key)) };
    ($m1:ident + $m2:ident + $m3:ident + $key:tt) => { user::make_key(key!(@m $m1 $m2 $m3), stringify!($key)) };
    ($m1:ident + $m2:ident + $m3:ident + $m4:ident + $key:tt) => { user::make_key(key!(@m $m1 $m2 $m3 $m4), stringify!($key)) };
    ($m1:ident + $m2:ident + $m3:ident + $m4:ident + $m5:ident + $key:tt) => { user::make_key(key!(@m $m1 $m2 $m3 $m4 $m5), stringify!($key)) };
}

fn make_left_right_keys(modifiers: u32, key: &str) -> Vec<(u32, u8)> {
    make_key(modifiers, &format!("{}_L", key))
        .iter()
        .chain(make_key(modifiers, &format!("{}_R", key)).iter())
        .copied()
        .collect()
}

pub fn make_key(modifiers: u32, key: &str) -> Vec<(u32, u8)> {
    let connection = connection();
    let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);

    let mut result = Vec::new();

    match key {
        "Shift" => make_left_right_keys(modifiers, "Shift"),
        "Control" | "Ctrl" => make_left_right_keys(modifiers, "Control"),
        "Ctrl_L" => make_key(modifiers, "Control_L"),
        "Ctrl_R" => make_key(modifiers, "Control_R"),
        "Alt" | "Opt" => make_left_right_keys(modifiers, "Alt"),
        "Opt_L" => make_key(modifiers, "Alt_L"),
        "Opt_R" => make_key(modifiers, "Alt_R"),
        "Meta" => make_left_right_keys(modifiers, "Meta"),
        "Hyper" => make_left_right_keys(modifiers, "Hyper"),
        "Super" | "Windows" | "Win" | "Command" | "Cmd" => make_left_right_keys(modifiers, "Super"),
        "Windows_L" | "Win_L" | "Command_L" | "Cmd_L" => make_key(modifiers, "Super_L"),
        "Windows_R" | "Win_R" | "Command_R" | "Cmd_R" => make_key(modifiers, "Super_R"),
        _ => {
            let keysym = xkbcommon::xkb::keysym_from_name(key, xkbcommon::xkb::KEYSYM_NO_FLAGS);
            if keysym != xcb::NO_SYMBOL {
                for keycode in key_symbols.get_keycode(keysym) {
                    let keysym_unshifted = key_symbols.get_keysym(keycode, 0);
                    let keysym_shifted = key_symbols.get_keysym(keycode, 1);
                    // If the user specifies a shifted symbol AND the shift key,
                    // then it's impossible to press i.e we must ignore "shift-Q"
                    let keysym_is_shifted = keysym == keysym_shifted && keysym != keysym_unshifted;
                    if !keysym_is_shifted || modifiers & xcb::xproto::KEY_BUT_MASK_SHIFT == 0 {
                        result.push(if keysym_is_shifted {
                            (modifiers | xcb::xproto::KEY_BUT_MASK_SHIFT, keycode)
                        } else {
                            (modifiers, keycode)
                        });
                    }
                }
            }
            result
        }
    }
}