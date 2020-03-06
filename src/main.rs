#![feature(inner_deref, trait_alias)]
#![recursion_limit = "128"]

#[macro_use]
mod keystroke;

#[macro_use]
mod model;

mod connection;
mod help;
mod key_dispatcher;

mod ceramic;

use key_dispatcher::KeyDispatcher;
use model::*;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(flatten)]
    verbosity: clap_verbosity_flag::Verbosity,
}

fn main() {
    let args = Args::from_args();
    args.verbosity.setup_env_logger("commando").unwrap();
    KeyDispatcher::run(create_model());
}

fn create_model() -> Model {
    let mut model = Model::new();

    model.extend_with(&bindings!(
        global {
            Escape         => { "Cancel Operation" cancel }
            Ctrl + g       => { "Cancel Operation" cancel }
            Cmd + question => { "Toggle/Move Help" toggle help }
        }
        root {
            Command => { "Application" => application }
        }
    ));

    ceramic::extend_model(&mut model);

    model
}
