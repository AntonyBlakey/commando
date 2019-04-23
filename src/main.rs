#![feature(inner_deref, type_alias_enum_variants, iter_copied, trait_alias)]
#![recursion_limit = "128"]

#[macro_use]
mod keystroke;

#[macro_use]
mod model;

mod action;
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
            Escape         => { "Cancel" cancel }
            Ctrl + g       => { "Cancel" cancel }
            Cmd + question => { "Toggle Help" toggle help }
        }
        root {
            Command => { "Application" => application }
        }
    ));

    ceramic::extend_model(&mut model);

    model
}
