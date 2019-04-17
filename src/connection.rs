use std::rc::Rc;

static mut CONNECTION: Option<Rc<xcb::Connection>> = None;
pub fn connection() -> Rc<xcb::Connection> {
    unsafe {
        CONNECTION
            .get_or_insert_with(|| {
                let (connection, _screen_number) = xcb::Connection::connect(None).unwrap();
                Rc::new(connection)
            })
            .clone()
    }
}
