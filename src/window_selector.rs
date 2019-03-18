use super::{event_source::EventSource, model::*};

#[derive(Debug)]
struct TargetWindow {
    id: xcb::xproto::Window,
    name: String,
    pos: (i16, i16),
    size: (u16, u16),
}

pub struct WindowSelector<'a> {
    model: &'a Model,
    event_source: &'a EventSource<'a>,
}

impl<'a> WindowSelector<'a> {
    pub fn run(model: &'a Model, event_source: &'a EventSource<'a>) {
        Self::new(model, event_source).main_loop();
    }

    fn new(model: &'a Model, event_source: &'a EventSource<'a>) -> WindowSelector<'a> {
        WindowSelector { model, event_source }
    }

    fn main_loop(&self) {
        let connection = self.connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let values = [
            (xcb::CW_BACK_PIXEL, screen.black_pixel()),
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
        ];

        for w in self.target_windows(&screen) {
            let new_id = connection.generate_id();

            xcb::create_window(
                connection,
                xcb::COPY_FROM_PARENT as u8,
                new_id,
                screen.root(),
                w.pos.0,
                w.pos.1,
                100,
                100,
                0,
                xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                screen.root_visual(),
                &values,
            );

            xcb::map_window(connection, new_id);
        }

        connection.flush();

        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    fn connection(&self) -> &xcb::Connection {
        self.event_source.connection()
    }

    fn target_windows(&self, screen: &xcb::Screen) -> Vec<TargetWindow> {
        let mut result = Vec::new();

        let atom_utf8_string = xcb::xproto::intern_atom(self.connection(), true, "UTF8_STRING")
            .get_reply()
            .unwrap()
            .atom();

        let query = xcb::xproto::query_tree(self.connection(), screen.root())
            .get_reply()
            .unwrap();

        for &id in query.children() {
            let match_state = xcb::xproto::get_window_attributes(self.connection(), id)
                .get_reply()
                .unwrap()
                .map_state();
            if match_state == xcb::xproto::MAP_STATE_VIEWABLE as u8 {
                let property = xcb::xproto::get_property(
                    self.connection(),
                    false,
                    id,
                    xcb::xproto::ATOM_WM_NAME,
                    atom_utf8_string,
                    0,
                    256,
                )
                .get_reply()
                .unwrap();
                let name = std::str::from_utf8(property.value()).unwrap();
                if !name.is_empty() {
                    let g = xcb::xproto::get_geometry(self.connection(), id)
                        .get_reply()
                        .unwrap();
                    result.push(TargetWindow {
                        id,
                        name: String::from(name),
                        pos: (g.x(), g.y()),
                        size: (g.width(), g.height()),
                    });
                }
            }
        }

        result.sort_by_key(|w| w.pos.0);
        result.sort_by_key(|w| w.pos.1);
        result
    }
}
