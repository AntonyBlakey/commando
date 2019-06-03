use crate::keystroke::Keystroke;
use cairo::XCBSurface;
use std::collections::HashSet;

pub fn connection() -> &'static xcb::Connection {
    static mut CONNECTION: Option<xcb::Connection> = None;
    unsafe {
        CONNECTION.get_or_insert_with(|| {
            let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();
            connection
        })
    }
}

pub fn modifier_keycodes() -> &'static HashSet<xcb::xproto::Keycode> {
    static mut MODIFIER_KEYCODES: Option<HashSet<xcb::xproto::Keycode>> = None;
    unsafe {
        MODIFIER_KEYCODES.get_or_insert_with(|| {
            let connection = connection();
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
        })
    }
}

pub fn grab_keys(keystrokes: &Vec<Keystroke>) {
    let root = connection().get_setup().roots().nth(0).unwrap().root();
    for desc in keystrokes {
        xcb::xproto::grab_key(
            connection(),
            false,
            root,
            desc.modifiers(),
            desc.keycode(),
            xcb::GRAB_MODE_ASYNC as u8,
            xcb::GRAB_MODE_SYNC as u8,
        );
    }
    connection().flush();
}

pub fn ungrab_all_keys() {
    let root = connection().get_setup().roots().nth(0).unwrap().root();
    xcb::xproto::ungrab_key(
        connection(),
        xcb::GRAB_ANY as u8,
        root,
        xcb::MOD_MASK_ANY as u16,
    );
    connection().flush();
}

pub fn grab_keyboard() {
    let root = connection().get_setup().roots().nth(0).unwrap().root();
    match xcb::xproto::grab_keyboard(
        connection(),
        false,
        root,
        xcb::CURRENT_TIME,
        xcb::GRAB_MODE_ASYNC as u8,
        xcb::GRAB_MODE_SYNC as u8,
    )
    .get_reply()
    .unwrap()
    .status() as u32
    {
        xcb::xproto::GRAB_STATUS_SUCCESS => log::debug!("Grab keyboard: Success"),
        xcb::xproto::GRAB_STATUS_ALREADY_GRABBED => log::debug!("Grab keyboard: Already Grabbed"),
        xcb::xproto::GRAB_STATUS_INVALID_TIME => log::debug!("Grab keyboard: Invalid Time"),
        xcb::xproto::GRAB_STATUS_NOT_VIEWABLE => log::debug!("Grab keyboard: Not Viewable"),
        xcb::xproto::GRAB_STATUS_FROZEN => log::debug!("Grab keyboard: Frozen"),
        x => log::debug!("Grab keyboard: Unknown status: {}", x),
    }
    connection().flush();
}

pub fn ungrab_keyboard() {
    log::debug!("Ungrab keyboard");
    xcb::xproto::ungrab_keyboard(connection(), xcb::CURRENT_TIME);
    connection().flush();
}

pub fn allow_events() {
    xcb::xproto::allow_events(
        connection(),
        xcb::ALLOW_SYNC_KEYBOARD as u8,
        xcb::CURRENT_TIME,
    );
    connection().flush();
}

pub fn wait_for_event() -> Option<xcb::base::GenericEvent> {
    allow_events();
    connection().wait_for_event()
}

pub fn get_cairo_surface(window: xcb::Window) -> Result<cairo::Surface, xcb::GenericError> {
    let connection = connection();

    let geometry = xcb::get_geometry(&connection, window).get_reply()?;
    let cairo_connection = unsafe {
        cairo::XCBConnection::from_raw_none(
            connection.get_raw_conn() as *mut cairo_sys::xcb_connection_t
        )
    };

    let cairo_drawable = cairo::XCBDrawable(window);

    let screen = connection.get_setup().roots().nth(0).unwrap();
    let mut visual = screen
        .allowed_depths()
        .filter(|d| d.depth() == screen.root_depth())
        .flat_map(|d| d.visuals())
        .find(|v| v.visual_id() == screen.root_visual())
        .unwrap();
    let cairo_visualtype = unsafe {
        cairo::XCBVisualType::from_raw_none(
            (&mut visual.base as *mut xcb::ffi::xproto::xcb_visualtype_t)
                as *mut cairo_sys::xcb_visualtype_t,
        )
    };

    Ok(cairo::Surface::create(
        &cairo_connection,
        &cairo_drawable,
        &cairo_visualtype,
        geometry.width() as i32,
        geometry.height() as i32,
    ))
}
