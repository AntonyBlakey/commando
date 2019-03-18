fn get_window_name(connection: &xcb::base::Connection, window: xcb::xproto::Window) -> String {
    let atom_utf8_string = xcb::xproto::intern_atom(&connection, true, "UTF8_STRING")
        .get_reply()
        .unwrap()
        .atom();
    let property = xcb::xproto::get_property(
        connection,
        false,
        window,
        xcb::xproto::ATOM_WM_NAME,
        atom_utf8_string,
        0,
        256,
    )
    .get_reply()
    .unwrap();
    String::from(std::str::from_utf8(property.value()).unwrap())
}

fn window_is_selectable(connection: &xcb::base::Connection, window: xcb::xproto::Window) -> bool {
    let is_viewable = xcb::xproto::get_window_attributes(&connection, window)
        .get_reply()
        .unwrap()
        .map_state()
        == xcb::xproto::MAP_STATE_VIEWABLE as u8;
    is_viewable && !get_window_name(connection, window).is_empty()
}

fn selectable_windows(connection: &xcb::base::Connection) -> Vec<xcb::xproto::Window> {
    let screen = connection.get_setup().roots().nth(0).unwrap();
    let query = xcb::xproto::query_tree(&connection, screen.root())
        .get_reply()
        .unwrap();
    query
        .children()
        .into_iter()
        .map(|&w| w)
        .filter(|&w| window_is_selectable(connection, w))
        .collect::<Vec<_>>()
}