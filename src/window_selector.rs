use std::{io::Write, path::PathBuf};
use horrorshow::{append_html, helper::doctype, html, Raw};
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
        WindowSelector {
            model,
            event_source,
        }
    }

    fn main_loop(&self) {
        let connection = self.connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let values = [
            (xcb::CW_BACK_PIXEL, screen.black_pixel()),
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
        ];
        let root = screen.root();
        let root_visual = screen.root_visual();

        // TODO: get this from config and cache the images
        // for char in "asdfghjklqwertyuiopzxcvbnm1234567890".chars() {
        for char in "as".chars() {
            self.generate_image(char);
        }

        for window in self.target_windows(&screen) {
            let new_id = connection.generate_id();

            xcb::create_window(
                connection,
                xcb::COPY_FROM_PARENT as u8,
                new_id,
                root,
                window.pos.0,
                window.pos.1,
                100,
                100,
                0,
                xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                root_visual,
                &values,
            );

            xcb::map_window(connection, new_id);
        }

        connection.flush();

        self.event_source.grab_keyboard();
        self.event_source.wait_for_event(None);
        self.event_source.ungrab_keyboard();
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
    
    fn generate_image(&self, key: char) {
        let path = PathBuf::from("/tmp/commando.select.html");
        let mut file = std::fs::File::create(&path).unwrap(); 
        write!(
            file,
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        style(type="text/css") {
                            @ for f in self.model.files.iter().filter(|f| f.file_name().unwrap() == "select.css") {
                                : Raw(std::fs::read_to_string(f).unwrap());
                            }
                        }
                    }
                    body {
                        div(id="body") {
                            : key 
                        }
                    }
                }
            }
        )
        .unwrap();

        {
            let status = std::process::Command::new("chromium-browser")
                .arg("--headless")
                .arg("--screenshot=/tmp/commando.select.png")
                .arg("--window-size=256x256")
                .arg(path.to_str().unwrap())
                .status();
            if let Err(err) = status {
                eprintln!("command failed with {:?}", err);
            }
        }
        {
            let status = std::process::Command::new("convert")
                .arg("/tmp/commando.select.png")
                .arg("-trim")
                .arg("-shave")
                .arg("1x1")
                .arg("/tmp/commando.select.trim.png")
                .status();
            if let Err(err) = status {
                eprintln!("command failed with {:?}", err);
            }
        }
    }
}