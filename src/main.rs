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
struct Args {
    #[structopt(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,

    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// listen for key commands, showing help as appropriate
    #[structopt(name="listen")]
    Listen(ListenCommand),

    /// Select a window by showing key overlays
    #[structopt(name="select")]
    Select(SelectCommand),
}

#[derive(Debug, StructOpt)]
struct ListenCommand {
    /// Directories containing configuration
    #[structopt(parse(from_os_str))]
    config: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
struct SelectCommand {
    /// Files containing configuration
    #[structopt(parse(from_os_str))]
    config: Vec<PathBuf>,
}

fn main() {
    let args = Args::from_args();
    args.verbosity.setup_env_logger("commando").unwrap();

    let (connection, screen_number) = xcb::Connection::connect(None).unwrap();

    let keysource = KeySource::new(&connection, screen_number);

    match args.command {
        Command::Listen(ListenCommand { config }) => {
            let files: Vec<PathBuf> = config
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
        Command::Select(SelectCommand { config: _ }) => {}
    }
}
