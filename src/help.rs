use crate::{
    connection,
    keystroke::Keystroke,
    model::{Action, Binding},
};
use crossbeam::channel::{Receiver, RecvTimeoutError};
use itertools::Itertools;
use lazy_static::lazy_static;
use pango::LayoutExt;
use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

pub enum HelpMessage {
    Arm,
    Disarm,
    Update(Vec<Binding>),
    Draw,
    Cancel,
    Toggle,
}

pub struct HelpWindow {
    window: xcb::Window,
    is_visible: bool,
    width: u32,
    height: u32,
    header_column_widths: (u32, u32),         // title, keystrokes
    body_column_widths: (u32, u32, u32, u32), // modifiers, keystroke, arrow, title
    groups: Vec<(Option<&'static str>, Vec<(Keystroke, &'static str)>)>,
    system_bindings: BTreeMap<&'static str, Vec<Keystroke>>, // BTreeMap to retain sort order
}

impl HelpWindow {
    pub fn run(&mut self, rx: Receiver<HelpMessage>) {
        log::debug!("Help server started");

        let mut is_armed = false;
        loop {
            if !is_armed {
                match rx.recv() {
                    Ok(HelpMessage::Arm) => is_armed = true,
                    Ok(HelpMessage::Disarm) => (),
                    Ok(HelpMessage::Update(bindings)) => {
                        self.update(bindings);
                    }
                    Ok(HelpMessage::Draw) => {
                        self.draw();
                    }
                    Ok(HelpMessage::Cancel) => {
                        self.set_visible(false);
                    }
                    Ok(HelpMessage::Toggle) => {
                        self.set_visible(!self.is_visible);
                    }
                    Err(_) => break,
                }
            } else {
                is_armed = false;
                match rx.recv_timeout(Duration::from_secs(1)) {
                    Ok(HelpMessage::Arm) => is_armed = true,
                    Ok(HelpMessage::Disarm) => (),
                    Ok(HelpMessage::Update(bindings)) => {
                        self.update(bindings);
                    }
                    Ok(HelpMessage::Draw) => {
                        self.draw();
                    }
                    Ok(HelpMessage::Cancel) => {
                        self.set_visible(false);
                    }
                    Ok(HelpMessage::Toggle) => {
                        self.set_visible(!self.is_visible);
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        self.set_visible(true);
                    }
                    Err(_) => break,
                }
            }
        }

        log::debug!("Help server stopped");
    }

    pub fn new() -> HelpWindow {
        let connection = connection::connection();
        let screen = connection.get_setup().roots().nth(0).unwrap();
        let root = screen.root();
        let root_visual = screen.root_visual();

        let values = [
            (xcb::CW_BACK_PIXEL, screen.white_pixel()),
            (xcb::CW_EVENT_MASK, xcb::EVENT_MASK_EXPOSURE),
            (xcb::CW_OVERRIDE_REDIRECT, 1),
        ];

        let window = connection.generate_id();
        xcb::create_window(
            &connection,
            xcb::COPY_FROM_PARENT as u8,
            window,
            root,
            -100,
            -100,
            1,
            1,
            1,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            root_visual,
            &values,
        );

        HelpWindow {
            window,
            is_visible: false,
            width: 0,
            height: 0,
            header_column_widths: Default::default(),
            body_column_widths: Default::default(),
            groups: Default::default(),
            system_bindings: Default::default(),
        }
    }

    fn set_visible(&mut self, visible: bool) {
        if self.is_visible != visible {
            let connection = connection::connection();
            if visible {
                let root = connection.get_setup().roots().nth(0).unwrap().root();
                let geometry = xcb::get_geometry(connection, root).get_reply().unwrap();
                xcb::configure_window(
                    connection,
                    self.window,
                    &[
                        (xcb::CONFIG_WINDOW_X as u16, 0),
                        (
                            xcb::CONFIG_WINDOW_Y as u16,
                            (geometry.height() as u32 - self.height) / 2,
                        ),
                        (xcb::CONFIG_WINDOW_WIDTH as u16, self.width),
                        (xcb::CONFIG_WINDOW_HEIGHT as u16, self.height),
                    ],
                );
                xcb::map_window(connection, self.window);
            } else {
                xcb::unmap_window(connection, self.window);
            }
            connection.flush();
            self.is_visible = visible;
        }
    }

    fn update(&mut self, bindings: Vec<Binding>) {
        self.set_bindings(bindings);

        if let Ok(surface) = connection::get_cairo_surface(self.window) {
            let cairo_context = cairo::Context::new(&surface);
            if let Some(layout) = pangocairo::functions::create_layout(&cairo_context) {
                let font_description =
                    // pango::FontDescription::from_string("Operator Mono SSm Light 11px");
                    pango::FontDescription::from_string("Noto Sans 11px");
                let key_font_description =
                    pango::FontDescription::from_string("Noto Sans Mono 11px");
                let symbol_font_description =
                    pango::FontDescription::from_string("Lucida Grande 11px");

                self.height = 0;
                self.width = 0;

                if self.system_bindings.is_empty() {
                    self.header_column_widths = (0, 0);
                } else {
                    self.height += 10;

                    let mut width_1: u32 = 0;
                    let mut width_2: u32 = 0;

                    for (label, keystrokes) in &self.system_bindings {
                        layout.set_font_description(&font_description);
                        layout.set_text(label);
                        let w1 = layout.get_pixel_size().0 as u32;
                        layout.set_text(": ");
                        let w2 = layout.get_pixel_size().0 as u32;
                        width_1 = width_1.max(w1 + w2);

                        let mut w = 0;
                        for (index, keystroke) in keystrokes.iter().enumerate() {
                            let (w1, w2) = keystroke.process_help(
                                &cairo_context,
                                &key_font_description,
                                &symbol_font_description,
                                false,
                            );
                            w += w1 + w2;

                            if index < keystrokes.len() - 1 {
                                layout.set_text(" / ");
                                w += layout.get_pixel_size().0 as u32;
                            }
                        }
                        width_2 = width_2.max(w);

                        self.height += 14;
                    }

                    self.header_column_widths = (width_1, width_2);
                    self.width = self.width.max(10 + width_1 + width_2 + 10);

                    self.height += 10;
                }

                if self.groups.is_empty() {
                    self.body_column_widths = (0, 0, 0, 0);
                } else {
                    self.height += 10;

                    let mut width_1: u32 = 0;
                    let mut width_2: u32 = 0;
                    layout.set_font_description(&font_description);
                    layout.set_text("\u{2794}");
                    let width_3 = layout.get_pixel_size().0 as u32;
                    let mut width_4: u32 = 0;

                    for (group, group_bindings) in &self.groups {
                        if let Some(_) = group {
                            self.height += 8 + 14 + 2 + 4;
                        }

                        for (keystroke, label) in group_bindings {
                            let (w1, w2) = keystroke.process_help(
                                &cairo_context,
                                &key_font_description,
                                &symbol_font_description,
                                false,
                            );
                            width_1 = width_1.max(w1);
                            width_2 = width_2.max(w2);
                            layout.set_text(label);
                            width_4 = width_4.max(layout.get_pixel_size().0 as u32);

                            self.height += 14;
                        }
                    }
                    self.body_column_widths = (width_1, width_2, width_3, width_4);
                    self.width = self
                        .width
                        .max(10 + width_1 + width_2 + 10 + width_3 + 10 + width_4 + 10);

                    self.height += 10;
                }


                log::debug!("Resize help window to {} x {}", self.width, self.height);
            }
        }

        let connection = connection::connection();
        if let Ok(attributes) = xcb::get_window_attributes(connection, self.window).get_reply() {
            if attributes.map_state() == xcb::MAP_STATE_VIEWABLE as u8 {
                let root = connection.get_setup().roots().nth(0).unwrap().root();
                let geometry = xcb::get_geometry(connection, root).get_reply().unwrap();
                xcb::configure_window(
                    connection,
                    self.window,
                    &[
                        (xcb::CONFIG_WINDOW_X as u16, 0),
                        (
                            xcb::CONFIG_WINDOW_Y as u16,
                            (geometry.height() as u32 - self.height) / 2,
                        ),
                        (xcb::CONFIG_WINDOW_WIDTH as u16, self.width),
                        (xcb::CONFIG_WINDOW_HEIGHT as u16, self.height),
                    ],
                );
                connection.flush();
                self.draw();
            }
        }
    }

    fn draw(&self) {
        if let Ok(surface) = connection::get_cairo_surface(self.window) {
            let cairo_context = cairo::Context::new(&surface);
            if let Some(layout) = pangocairo::functions::create_layout(&cairo_context) {
                let font_description =
                    // pango::FontDescription::from_string("Operator Mono SSm Light 11px");
                    pango::FontDescription::from_string("Noto Sans 11px");
                let key_font_description =
                    pango::FontDescription::from_string("Noto Sans Mono 11px");
                let symbol_font_description =
                    pango::FontDescription::from_string("Lucida Grande 11px");

                layout.set_font_description(&font_description);

                cairo_context.set_source_rgb(1.0, 1.0, 0.95);
                cairo_context.move_to(0.0, 0.0);
                cairo_context.line_to(self.width as f64, 0.0);
                cairo_context.line_to(self.width as f64, self.height as f64);
                cairo_context.line_to(0.0, self.height as f64);
                cairo_context.close_path();
                cairo_context.fill();


                let mut y = 0.0;

                if !self.system_bindings.is_empty() {
                    cairo_context.set_source_rgb(0.9, 1.0, 0.9);
                    cairo_context.move_to(0.0, 0.0);
                    cairo_context.rel_line_to(self.width as f64, 0.0);
                    cairo_context
                        .rel_line_to(0.0, (10 + self.system_bindings.len() * 14 + 10) as f64);
                    cairo_context.rel_line_to(0.0 - self.width as f64, 0.0);
                    cairo_context.close_path();
                    cairo_context.fill();

                    cairo_context.set_source_rgb(0.8, 0.9, 0.8);
                    cairo_context.move_to(
                        0.0,
                        (10 + self.system_bindings.len() * 14 + 10) as f64 - 0.5,
                    );
                    cairo_context.rel_line_to(self.width as f64, 0.0);
                    cairo_context.set_line_width(1.0);
                    cairo_context.stroke();

                    cairo_context.set_source_rgb(0.0, 0.0, 0.0);

                    y += 10.0;

                    let x_column_1 = 10.0;
                    let x_column_2 = x_column_1 + self.header_column_widths.0 as f64;
                    for (label, keystrokes) in &self.system_bindings {
                        let mut x = x_column_1;
                        cairo_context.move_to(x, y);
                        layout.set_text(label);
                        pangocairo::functions::show_layout(&cairo_context, &layout);
                        x += layout.get_pixel_size().0 as f64;

                        cairo_context.move_to(x, y);
                        layout.set_text(": ");
                        pangocairo::functions::show_layout(&cairo_context, &layout);

                        let mut x = x_column_2;
                        for (index, keystroke) in keystrokes.iter().enumerate() {
                            cairo_context.move_to(x, y);
                            let (w1, w2) = keystroke.process_help(
                                &cairo_context,
                                &key_font_description,
                                &symbol_font_description,
                                false,
                            );
                            x += w1 as f64;

                            cairo_context.move_to(x, y);
                            keystroke.process_help(
                                &cairo_context,
                                &key_font_description,
                                &symbol_font_description,
                                true,
                            );
                            x += w2 as f64;

                            if index < keystrokes.len() - 1 {
                                cairo_context.move_to(x, y);
                                layout.set_text(" / ");
                                pangocairo::functions::show_layout(&cairo_context, &layout);
                                x += layout.get_pixel_size().0 as f64;
                            }
                        }

                        y += 14.0;
                    }

                    y += 10.0;
                }

                if !self.groups.is_empty() {
                    y += 10.0;

                    let x_column_1 = 10.0;
                    let x_column_2 = x_column_1 + self.body_column_widths.0 as f64;
                    let x_column_3 = x_column_2 + self.body_column_widths.1 as f64 + 10.0;
                    let x_column_4 = x_column_3 + self.body_column_widths.2 as f64 + 10.0;
                    let x_right = x_column_4 + self.body_column_widths.3 as f64;
                    for (group, group_bindings) in &self.groups {
                        if let Some(group_name) = group {
                            y += 8.0;
                            cairo_context.set_source_rgb(0.0, 0.5, 0.0);
                            cairo_context.move_to(x_column_1, y);
                            layout.set_text(group_name);
                            pangocairo::functions::show_layout(&cairo_context, &layout);
                            y += 14.0;

                            y += 2.0;
                            cairo_context.set_source_rgb(0.7, 0.85, 0.7);
                            cairo_context.move_to(x_column_1, y + 0.5);
                            cairo_context.line_to(x_right, y + 0.5);
                            cairo_context.set_line_width(1.0);
                            cairo_context.stroke();
                            y += 4.0;
                        }

                        for (keystroke, label) in group_bindings {
                            cairo_context.set_source_rgb(0.0, 0.0, 0.0);

                            cairo_context.move_to(x_column_2, y);
                            keystroke.process_help(
                                &cairo_context,
                                &key_font_description,
                                &symbol_font_description,
                                true,
                            );

                            cairo_context.move_to(x_column_4, y);
                            layout.set_text(label);
                            pangocairo::functions::show_layout(&cairo_context, &layout);

                            cairo_context.set_source_rgb(0.7, 0.7, 0.7);

                            cairo_context.move_to(x_column_3, y);
                            layout.set_text("\u{2794}");
                            pangocairo::functions::show_layout(&cairo_context, &layout);

                            y += 14.0;
                        }
                    }

                }
            }
            connection::connection().flush();
        }
    }

    fn set_bindings(&mut self, bindings: Vec<Binding>) {
        let (mut system_bindings, mut groups): (Vec<Binding>, Vec<Binding>) =
            bindings.into_iter().partition(|b| match b.action() {
                Action::Cancel | Action::ToggleHelp => true,
                _ => false,
            });

        system_bindings.sort_by_key(|b| b.label());
        self.system_bindings = system_bindings
            .iter()
            .group_by(|b| b.label())
            .into_iter()
            .map(|(label, bindings)| (label, bindings.into_iter().map(|b| b.keystroke()).collect()))
            .collect();

        groups.sort_by_key(|b| b.group());
        self.groups = groups
            .iter()
            .group_by(|b| b.group())
            .into_iter()
            .map(|(group, bindings)| {
                (
                    group,
                    bindings
                        .into_iter()
                        .map(|b| (b.keystroke(), b.label()))
                        .collect(),
                )
            })
            .collect();
    }
}

impl Drop for HelpWindow {
    fn drop(&mut self) {
        xcb::destroy_window(&connection::connection(), self.window);
    }
}

impl Keystroke {
    fn process_help(
        &self,
        cairo_context: &cairo::Context,
        text_font: &pango::FontDescription,
        symbol_font: &pango::FontDescription,
        draw: bool,
    ) -> (u32, u32) {
        if let Some(layout) = pangocairo::functions::create_layout(&cairo_context) {
            let connection = connection::connection();
            let key_symbols = xcb_util::keysyms::KeySymbols::new(&connection);

            let (keysym, hide_shift) = match (
                key_symbols.get_keysym(self.keycode(), 0),
                key_symbols.get_keysym(self.keycode(), 1),
            ) {
                (xcb::base::NO_SYMBOL, xcb::base::NO_SYMBOL) => return (0, 0),
                (a, xcb::base::NO_SYMBOL) => (a, false),
                (xcb::base::NO_SYMBOL, b) => (b, true),
                (a, b) => {
                    if self.made_with_shift() {
                        (a, false)
                    } else {
                        (b, true)
                    }
                }
            };

            layout.set_font_description(text_font);
            let text_baseline = layout.get_baseline();
            layout.set_font_description(symbol_font);
            let symbol_baseline = layout.get_baseline();
            let symbol_baseline_offset =
                (text_baseline - symbol_baseline) as f64 / pango::SCALE as f64;

            let raw_keysym_name = unsafe {
                std::ffi::CStr::from_ptr(x11::xlib::XKeysymToString(keysym.into()))
                    .to_str()
                    .unwrap()
            };

            let (keysym_name, is_symbol) = KEYSYM_NAME_DISPLAY_FORM
                .get(raw_keysym_name)
                .copied()
                .unwrap_or_else(|| (raw_keysym_name, false));
            layout.set_font_description(if is_symbol { symbol_font } else { text_font });
            layout.set_text(keysym_name);
            let width_2 = layout.get_pixel_size().0;
            if draw {
                if is_symbol {
                    cairo_context.rel_move_to(0.0, symbol_baseline_offset);
                    pangocairo::functions::show_layout(&cairo_context, &layout);
                    cairo_context.rel_move_to(0.0, -symbol_baseline_offset);
                } else {
                    pangocairo::functions::show_layout(&cairo_context, &layout);
                }
            }

            let mut width = 0;

            // spacing between modifiers and key
            width += 1;
            cairo_context.rel_move_to(-1.0, 0.0);

            for (name, display_form, is_symbol) in &*MODIFIER_NAME_DISPLAY_FORM {
                if match *name {
                    "shift" => {
                        !hide_shift && self.modifiers() & xcb::KEY_BUT_MASK_SHIFT as u16 != 0
                    }
                    "control" => self.modifiers() & xcb::KEY_BUT_MASK_CONTROL as u16 != 0,
                    "alt" => self.modifiers() & xcb::KEY_BUT_MASK_MOD_1 as u16 != 0,
                    "mod2" => self.modifiers() & xcb::KEY_BUT_MASK_MOD_2 as u16 != 0,
                    "hyper" => self.modifiers() & xcb::KEY_BUT_MASK_MOD_3 as u16 != 0,
                    "super" => self.modifiers() & xcb::KEY_BUT_MASK_MOD_4 as u16 != 0,
                    "mod5" => self.modifiers() & xcb::KEY_BUT_MASK_MOD_5 as u16 != 0,
                    _ => false,
                } {
                    layout.set_font_description(if *is_symbol { symbol_font } else { text_font });
                    layout.set_text(display_form);
                    let w = layout.get_pixel_size().0;
                    width += w;
                    if draw {
                        if *is_symbol {
                            cairo_context.rel_move_to(-w as f64, symbol_baseline_offset);
                            pangocairo::functions::show_layout(&cairo_context, &layout);
                            cairo_context.rel_move_to(0.0, -symbol_baseline_offset);
                        } else {
                            cairo_context.rel_move_to(-w as f64, 0.0);
                            pangocairo::functions::show_layout(&cairo_context, &layout);
                        }
                    }
                }
            }

            (width as u32, width_2 as u32)
        } else {
            (0, 0)
        }
    }
}

lazy_static! {
    static ref MODIFIER_NAME_DISPLAY_FORM: Vec<(&'static str, &'static str, bool)> = {
        vec![
            ("super", "\u{2318}", true),
            ("shift", "\u{21e7}", true),
            ("alt", "\u{2325}", true),
            ("control", "\u{2303}", true),
            ("mod2", "mod2-", false),
            ("mod5", "mod5-", false),
            ("hyper", "hyper-", false),
        ]
    };
    static ref KEYSYM_NAME_DISPLAY_FORM: HashMap<&'static str, (&'static str, bool)> = {
        let mut m = HashMap::new();
        m.insert("Tab", ("\u{21e5}", true));

        m.insert("Return", ("&crarr", true));
        m.insert("Escape", ("Esc", false));
        m.insert("BackSpace", ("&#9003", true));
        m.insert("Delete", ("&#8998", true));
        m.insert("Up", ("&uarr", true));
        m.insert("Down", ("&darr", true));
        m.insert("Left", ("&larr", true));
        m.insert("Right", ("&rarr", true));
        m.insert("PageUp", ("&#8670", true));
        m.insert("PageDown", ("&#8671", true));
        m.insert("Home", ("&#8598", true));
        m.insert("End", ("&#8600", true));
        m.insert("space", ("\u{2423}", true));

        m.insert("plus", ("+", false));
        m.insert("minus", ("-", false));
        m.insert("less", ("<", false));
        m.insert("greater", (">", false));
        m.insert("equal", ("=", false));
        m.insert("semicolon", (";", false));
        m.insert("apostrophe", ("'", false));
        m.insert("grave", ("`", false));
        m.insert("backslash", ("\\", false));
        m.insert("comma", (",", false));
        m.insert("period", (".", false));
        m.insert("question", ("?", false));
        m.insert("bar", ("|", false));
        m.insert("asciitilde", ("~", false));
        m.insert("quotedbl", ("\"", false));
        m.insert("colon", (":", false));
        m.insert("underscore", ("_", false));
        m.insert("asterisk", ("*", false));
        m.insert("ampersand", ("&", false));
        m.insert("asciicircum", ("^", false));
        m.insert("percent", ("%", false));
        m.insert("dollar", ("$", false));
        m.insert("numbersign", ("#", false));
        m.insert("at", ("@", false));
        m.insert("exclam", ("!", false));
        m.insert("bracketleft", ("[", false));
        m.insert("bracketright", ("]", false));
        m.insert("braceleft", ("{", false));
        m.insert("braceright", ("}", false));
        m.insert("parenleft", ("(", false));
        m.insert("parenright", (")", false));
        m
    };
    static ref KEYSYM_NAME_SORT_ORDER: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        let symbols = [
            "1..9",
            "parenleft",
            "parenright",
            "bracketleft",
            "bracketright",
            "braceleft",
            "braceright",
            "less",
            "greater",
            "plus",
            "minus",
            "equal",
            "slash",
            "backslash",
            "underscore",
            "bar",
            "semicolon",
            "colon",
            "apostrophe",
            "quotedbl",
            "grave",
            "asciitilde",
            "comma",
            "period",
            "question",
            "numbersign",
            "exclam",
            "at",
            "dollar",
            "percent",
            "asciicircum",
            "ampersand",
            "asterisk",
            "Up",
            "Down",
            "Left",
            "Right",
            "BackSpace",
            "Delete",
            "PageUp",
            "PageDown",
            "Home",
            "End",
            "Tab",
            "Return",
            "space",
            "Escape",
        ];
        for i in 0..symbols.len() {
            m.insert(symbols[i], i as u8);
        }
        m
    };
}
