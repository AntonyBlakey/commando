#![feature(inner_deref, type_alias_enum_variants)]

mod action;
mod config;
mod help;
mod interpreter;
mod key_description;
mod keysource;
mod model;

use config::ConfigFile;
use interpreter::Interpreter;
use keysource::KeySource;
use model::Model;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Directories containing configuration
    #[structopt(parse(from_os_str))]
    dirs: Vec<PathBuf>,

    #[structopt(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

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

fn main() {
    let args = Opt::from_args();
    args.verbosity.setup_env_logger("commando").unwrap();

    let (connection, screen_number) = xcb::Connection::connect(None).unwrap();

    let keysource = KeySource::new(&connection, screen_number);

    let files: Vec<PathBuf> = args
        .dirs
        .iter()
        .filter(|d| d.is_dir())
        .map(|d| d.canonicalize().unwrap())
        .flat_map(|d| d.read_dir().unwrap())
        .map(|e| e.unwrap().path())
        .filter(|d| d.is_file())
        .collect();

    let css: Vec<PathBuf> = files
        .iter()
        .filter(|f| f.file_name().unwrap() == "help.css")
        .map(|f| f.clone())
        .collect();
    let javascript: Vec<PathBuf> = files
        .iter()
        .filter(|f| f.file_name().unwrap() == "help.js")
        .map(|f| f.clone())
        .collect();
    let definitions = files
        .iter()
        .filter(|f| {
            let ext = f.extension().unwrap();
            ext == "json5" || ext == "json"
        })
        .map(|f| std::fs::read_to_string(f).unwrap())
        .map(|source| ConfigFile::from_string(&source).unwrap())
        .flat_map(|config| config.definitions);

    let model = Model::new(definitions, css, javascript, &keysource);

    Interpreter::run(&model, &keysource);
}
