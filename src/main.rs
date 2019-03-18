#![feature(inner_deref, type_alias_enum_variants)]

mod action;
mod config;
mod help;
mod interpreter;
mod key_description;
mod keysource;
mod model;

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
    /// Listen for key commands, showing help as appropriate
    #[structopt(name = "listen")]
    Listen(ListenCommand),

    /// Select a window by showing key overlays
    #[structopt(name = "select")]
    Select(SelectCommand),
}

#[derive(Debug, StructOpt)]
struct ListenCommand {
    /// Files containing configuration
    #[structopt(parse(from_os_str))]
    config: Vec<PathBuf>,
}

#[derive(Debug, StructOpt)]
struct SelectCommand {
    /// Files containing configuration
    #[structopt(parse(from_os_str))]
    config: Vec<PathBuf>,
}

fn files_from_config(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut result = Vec::new();

    for path in paths.iter().map(|p| p.canonicalize().unwrap()) {
        if path.is_file() {
            result.push(path);
        } else if path.is_dir() {
            result.extend(path.read_dir().unwrap().map(|e| e.unwrap().path()));
        }
    }

    result
}

fn main() {
    let args = Args::from_args();
    args.verbosity.setup_env_logger("commando").unwrap();

    let (connection, screen_number) = xcb::Connection::connect(None).unwrap();

    match args.command {
        Command::Listen(ListenCommand { config }) => {
            let keysource = KeySource::new(&connection, screen_number);
            let files = files_from_config(&config);
            let model = Model::new(files, &keysource);
            Interpreter::run(&model, &keysource);
        }
        Command::Select(SelectCommand { config: _ }) => {}
    }
}
