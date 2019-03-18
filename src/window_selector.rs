use super::{key_source::KeySource, model::*};

pub struct WindowSelector<'a> {
    model: &'a Model,
    key_source: &'a KeySource<'a>,
}

impl<'a> WindowSelector<'a> {
    pub fn run(model: &'a Model, key_source: &'a KeySource<'a>) {
        Self::new(model, key_source).main_loop();
    }

    fn new(model: &'a Model, key_source: &'a KeySource<'a>) -> WindowSelector<'a> {
        WindowSelector { model, key_source }
    }

    fn main_loop(&self) {
        eprintln!(
            "{:?}",
            self.selectable_windows()
                .iter()
                .map(|w| self.get_window_name(*w))
                .collect::<Vec<_>>()
        );
    }

    fn selectable_windows(&self) -> Vec<xcb::xproto::Window> {
        let screen = self
            .key_source
            .connection()
            .get_setup()
            .roots()
            .nth(0)
            .unwrap();
        let query = xcb::xproto::query_tree(self.key_source.connection(), screen.root())
            .get_reply()
            .unwrap();
        query
            .children()
            .into_iter()
            .map(|&w| w)
            .filter(|&w| self.window_is_selectable(w))
            .collect::<Vec<_>>()
    }

    fn get_window_name(&self, window: xcb::xproto::Window) -> String {
        let atom_utf8_string =
            xcb::xproto::intern_atom(self.key_source.connection(), true, "UTF8_STRING")
                .get_reply()
                .unwrap()
                .atom();
        let property = xcb::xproto::get_property(
            self.key_source.connection(),
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

    fn window_is_selectable(&self, window: xcb::xproto::Window) -> bool {
        let is_viewable = xcb::xproto::get_window_attributes(self.key_source.connection(), window)
            .get_reply()
            .unwrap()
            .map_state()
            == xcb::xproto::MAP_STATE_VIEWABLE as u8;
        is_viewable && !self.get_window_name(window).is_empty()
    }
}
