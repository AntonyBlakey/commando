use crate::{connection, keystroke::Keystroke, model::Binding};
use itertools::Itertools;
use pango::LayoutExt;

pub struct HelpWindow {
    window: xcb::Window,
    column_widths: (u32, u32, u32),
    groups: Vec<(Option<&'static str>, Vec<(Keystroke, &'static str)>)>,
}

impl HelpWindow {
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
            0,
            xcb::WINDOW_CLASS_INPUT_OUTPUT as u16,
            root_visual,
            &values,
        );

        HelpWindow {
            window,
            column_widths: Default::default(),
            groups: Default::default(),
        }
    }

    pub fn window(&self) -> xcb::Window {
        self.window
    }

    pub fn update(&mut self, bindings: Vec<Binding>) {
        if let Ok(surface) = connection::get_cairo_surface(self.window) {
            let cairo_context = cairo::Context::new(&surface);
            if let Some(layout) = pangocairo::functions::create_layout(&cairo_context) {
                let mut mut_bindings = bindings;
                mut_bindings.sort_by_key(|b| b.group());
                self.groups = mut_bindings
                    .iter()
                    .group_by(|b| b.group())
                    .into_iter()
                    .map(|(group, group_bindings)| {
                        (
                            group,
                            group_bindings
                                .into_iter()
                                .map(|b| (b.keystroke(), b.label()))
                                .collect(),
                        )
                    })
                    .collect();

                let font_description =
                    pango::FontDescription::from_string("Operator Mono SSm Light 11px");
                layout.set_font_description(&font_description);
                layout.set_text("\u{2794}");
                let width_2 = layout.get_pixel_size().0 as u32;

                let mut height = 0;
                let mut width_1 = 0;
                let mut width_3 = 0;
                for (group, group_bindings) in &self.groups {
                    if let Some(_) = group {
                        height += 4 + 14 + 4;
                    }
                    for (keystroke, label) in group_bindings {
                        height += 14;
                        layout.set_text(keystroke.to_string().as_str());
                        width_1 = width_1.max(layout.get_pixel_size().0 as u32);
                        layout.set_text(label);
                        width_3 = width_3.max(layout.get_pixel_size().0 as u32);
                    }
                }

                self.column_widths = (width_1, width_2, width_3);

                xcb::configure_window(
                    &connection::connection(),
                    self.window,
                    &[
                        (xcb::CONFIG_WINDOW_X as u16, 0),
                        (xcb::CONFIG_WINDOW_Y as u16, (1440 - (10 + height + 10)) / 2),
                        (
                            xcb::CONFIG_WINDOW_WIDTH as u16,
                            10 + width_1 + 10 + width_2 + 10 + width_3 + 10,
                        ),
                        (xcb::CONFIG_WINDOW_HEIGHT as u16, 10 + height + 10),
                    ],
                );
            }
        }
    }

    pub fn expose(&self, event: &xcb::ExposeEvent) {
        if let Ok(surface) = connection::get_cairo_surface(self.window) {
            let cairo_context = cairo::Context::new(&surface);

            if let Some(layout) = pangocairo::functions::create_layout(&cairo_context) {
                let font_description =
                    pango::FontDescription::from_string("Operator Mono SSm Light 11px");
                layout.set_font_description(&font_description);

                let x_column_1 = 10.0;
                let x_column_2 = x_column_1 + self.column_widths.0 as f64 + 10.0;
                let x_column_3 = x_column_2 + self.column_widths.1 as f64 + 10.0;
                let x_right = x_column_3 + self.column_widths.2 as f64;

                let mut y = 10.0;
                for (group, group_bindings) in &self.groups {
                    if let Some(group_name) = group {
                        y += 4.0;
                        cairo_context.set_source_rgb(0.0, 0.5, 0.0);
                        cairo_context.move_to(x_column_1, y);
                        layout.set_text(group_name);
                        pangocairo::functions::show_layout(&cairo_context, &layout);
                        y += 14.0;

                        cairo_context.set_source_rgb(0.7, 0.85, 0.7);
                        cairo_context.move_to(x_column_1, y + 0.5);
                        cairo_context.line_to(x_right, y + 0.5);
                        cairo_context.set_line_width(1.0);
                        cairo_context.stroke();
                        y += 4.0;
                    }

                    for (keystroke, label) in group_bindings {
                        cairo_context.set_source_rgb(0.0, 0.0, 0.0);

                        cairo_context.move_to(x_column_1, y);
                        layout.set_text(keystroke.to_string().as_str());
                        pangocairo::functions::show_layout(&cairo_context, &layout);

                        cairo_context.move_to(x_column_3, y);
                        layout.set_text(label);
                        pangocairo::functions::show_layout(&cairo_context, &layout);

                        cairo_context.set_source_rgb(0.7, 0.7, 0.7);

                        cairo_context.move_to(x_column_2, y);
                        layout.set_text("\u{2794}");
                        pangocairo::functions::show_layout(&cairo_context, &layout);

                        y += 14.0;
                    }
                }

            }
        }
    }

}

impl Drop for HelpWindow {
    fn drop(&mut self) {
        xcb::destroy_window(&connection::connection(), self.window);
    }
}
