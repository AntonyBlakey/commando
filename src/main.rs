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
