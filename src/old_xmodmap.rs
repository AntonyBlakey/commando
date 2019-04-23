// This isn't used - I just wrote it to ensure I understand the keymap apis

use std::ffi::CStr;

fn xmodmap_pke(conn: &xcb::base::Connection) {
    let setup = conn.get_setup();
    let min_keycode = unsafe { (*setup.ptr).min_keycode };
    let max_keycode = unsafe { (*setup.ptr).max_keycode };

    let syms = xcb_util::keysyms::KeySymbols::new(&conn);

    for keycode in min_keycode..=max_keycode {
        print!("keycode {:3} =", keycode);
        let mut last_printed = -1;
        for c in 0..2 {
            let ksym = syms.get_keysym(keycode, c);
            if ksym != 0 {
                for _ in (last_printed + 1)..c {
                    print!(" NoSymbol");
                }
                let kstr = unsafe {
                    CStr::from_ptr(x11::xlib::XKeysymToString(ksym.into()))
                        .to_str()
                        .expect("Couldn't create Rust string from C string")
                };
                    print!(" {}", kstr);
                if kstr.len() > 1 && kstr.chars().next().unwrap().is_lowercase() {
                    let kgraphic = xkbcommon::xkb::keysym_to_utf8(ksym);
                    print!("={:?}", kgraphic.chars().next().unwrap());
                }
                last_printed = c;
            }
        }
        println!();
    }
}

fn xmodmap_pm(conn: &xcb::base::Connection) {
    let syms = xcb_util::keysyms::KeySymbols::new(&conn);

    let mmc = xcb::xproto::get_modifier_mapping(&conn);
    let mm = mmc.get_reply().unwrap();
    let width = mm.keycodes_per_modifier();

    println!(
        "Up to {} keys per modifier, (keycodes in parentheses):",
        width
    );
    println!();
    
    let keycodes = mm.keycodes();

    [
        "shift", "lock", "control", "mod1", "mod2", "mod3", "mod4", "mod5",
    ]
    .into_iter()
    .enumerate()
    .for_each(|(mod_index, mod_name)| {
        let mut seen = std::collections::HashSet::new();
        let mut need_comma = false;
        print!("{:10}", mod_name);
        for j in 0..width {
            let keycode = keycodes[mod_index * (width as usize) + (j as usize)];
            for c in 0..8 {
                let ksym = syms.get_keysym(keycode, c);
                if ksym != 0 {
                    if !seen.contains(&keycode) {
                        seen.insert(keycode);
                        let kstr = unsafe {
                            CStr::from_ptr(x11::xlib::XKeysymToString(ksym.into()))
                                .to_str()
                                .expect("Couldn't create Rust string from C string")
                        };
                        if need_comma {
                            print!(",");
                        }
                        print!("  {} ({:#x})", kstr, keycode);
                        need_comma = true;
                    }
                }
            }
        }
        println!();
    });
}