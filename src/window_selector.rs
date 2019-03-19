use super::{event_source::EventSource, model::*};
use horrorshow::{append_html, helper::doctype, html, Raw};
use image::GenericImageView;
use sha2::{digest::Digest, Sha256};
use std::{collections::HashMap, io::Write};

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

        let mut window_to_image_map = HashMap::new();
        let mut key_to_window_map = HashMap::new();
        for (window, key) in self
            .target_windows(&screen)
            .iter()
            // TODO: get from config
            .zip("asdfghjklqwertyuiopzxcvbnm1234567890".chars())
        {
            let new_id = connection.generate_id();
            let image = self.get_image(key);

            xcb::create_window(
                connection,
                xcb::COPY_FROM_PARENT as u8,
                new_id,
                root,
                window.pos.0,
                window.pos.1,
                image.width() as u16,
                image.height() as u16,
                0,
                xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
                root_visual,
                &values,
            );

            xcb::map_window(connection, new_id);

            window_to_image_map.insert(new_id, image);
            key_to_window_map.insert(key.to_string(), window.id);
        }

        connection.flush();

        let expose_handler = |event: &xcb::ExposeEvent| {
            self.draw_image_on_window(
                window_to_image_map.get(&event.window()).unwrap(),
                event.window(),
            );
        };

        self.event_source.grab_keyboard();
        while let Some(key) = self.event_source.wait_for_event(&expose_handler) {
            match self.model.command_bindings.get(&key) {
                Some(Command::Cancel) => break,
                Some(_) => continue,
                None => {
                    if key.modifiers() == 0 {
                        let keysym = self.event_source.key_symbols().get_keysym(key.keycode(), 0);
                        if keysym != xcb::base::NO_SYMBOL {
                            let keysym_name = unsafe {
                                std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
                                    .to_str()
                                    .unwrap()
                            };
                            match key_to_window_map.get(keysym_name) {
                                Some(window_id) => {
                                    println!("{}", window_id);
                                    break;
                                }
                                None => {}
                            }
                        }
                    }
                }
            }
        }
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

    fn get_image(&self, key: char) -> image::DynamicImage {
        let html = html! {
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
        };

        let path = Self::html_to_image(&html);
        let bytes = std::fs::read(&path).unwrap();
        image::load_from_memory_with_format(&bytes, image::ImageFormat::PNG).unwrap()
    }

    fn html_to_image<T>(html: &T) -> String
    where
        T: std::fmt::Display,
    {
        let string = html.to_string();
        let mut hasher = Sha256::new();
        hasher.input(&string);
        let image_path = format!("/tmp/commando.{:x}.png", hasher.result());

        if !std::fs::metadata(&image_path).is_ok() {
            write!(
                std::fs::File::create("/tmp/commando.html").unwrap(),
                "{}",
                string
            )
            .unwrap();

            let chromium_status = std::process::Command::new("chromium-browser")
                .arg("--headless")
                .arg("--screenshot=/tmp/commando.png")
                .arg("--window-size=2560x1440")
                .arg("/tmp/commando.html")
                .status();
            if let Err(err) = chromium_status {
                eprintln!("Error creating image from html: {:?}", err);
            }

            let convert_status = std::process::Command::new("convert")
                .arg("/tmp/commando.png")
                .arg("-trim")
                .arg("-shave")
                .arg("1x1")
                .arg(&image_path)
                .status();
            if let Err(err) = convert_status {
                eprintln!("Error creating image from html: {:?}", err);
            }
        }

        image_path
    }

    fn draw_image_on_window(&self, image: &image::DynamicImage, window_id: u32) {
        let connection = self.connection();
        let gc_id = connection.generate_id();
        xcb::xproto::create_gc(connection, gc_id, window_id, &[]);
        let pixels = image.to_bgra().into_raw();
        xcb::xproto::put_image(
            connection,
            xcb::xproto::IMAGE_FORMAT_Z_PIXMAP as u8,
            window_id,
            gc_id,
            image.width() as u16,
            image.height() as u16,
            0,
            0,
            0,
            24,
            &pixels,
        );
        xcb::xproto::free_gc(connection, gc_id);
        connection.flush();
    }
}
