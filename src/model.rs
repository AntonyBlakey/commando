use super::{config, event_source::EventSource, key_description::KeyDescription};
use regex::Regex;
use std::{collections::HashMap, path::PathBuf};

pub type DefinitionId = String;
pub type KeySpec = String;
pub type CommandLine = String;
pub type DisplayLabel = String;

pub type Command = config::Command;
pub type Event = config::Event;

#[derive(Default, Debug, Clone)]
pub struct Model {
    // Preserved for producing help
    pub keys: HashMap<KeySpec, Binding>,
    pub commands: HashMap<Command, Vec<KeySpec>>,
    pub files: Vec<PathBuf>,

    pub definitions: HashMap<DefinitionId, Vec<Definition>>,
    pub handlers: HashMap<Event, String>,
    pub command_bindings: HashMap<KeyDescription, Command>,

    pub bindings: HashMap<KeyDescription, Binding>,
}

impl Model {
    pub fn new(files: Vec<PathBuf>, event_source: &EventSource) -> Model {
        let mut model = Model {
            files,
            ..Default::default()
        };

        let keysyms = event_source.key_symbols();

        let definitions = model
            .files
            .iter()
            .filter(|f| {
                let ext = f.extension().unwrap();
                ext == "json5" || ext == "json"
            })
            .map(|f| std::fs::read_to_string(f).unwrap())
            .map(|source| config::ConfigFile::from_string(&source).unwrap())
            .flat_map(|config| config.definitions);

        for def in definitions {
            match def {
                config::Definition::Root {
                    keys,
                    commands,
                    handlers,
                } => {
                    model.handlers = handlers;
                    model.commands = commands;
                    for (c, ks) in &model.commands {
                        for k in ks {
                            for d in KeyDescription::from_string(k, keysyms) {
                                model.command_bindings.insert(d, *c);
                            }
                        }
                    }
                    // keys has to be converted to a map to bindings
                    for (k, v) in &keys {
                        let binding = Binding::new(v, None);
                        for d in KeyDescription::from_string(k, keysyms) {
                            model.bindings.insert(d, binding.clone());
                        }
                        model.keys.insert(k.clone(), binding);
                    }
                }
                config::Definition::Linear {
                    path,
                    guard,
                    keys,
                    groups,
                } => {
                    let def = Definition::new(&guard, &keys, &groups, event_source);
                    match model.definitions.get_mut(&path) {
                        Some(vec) => {
                            vec.push(def);
                        }
                        None => {
                            model.definitions.insert(path.clone(), vec![def]);
                        }
                    };
                }
            }
        }

        model
    }
}

#[derive(Default, Debug, Clone)]
pub struct Definition {
    pub guard: Guard,
    pub keys: HashMap<KeySpec, Binding>,
    pub bindings: HashMap<KeyDescription, Binding>,
}

impl Definition {
    fn new(
        from_guard: &config::Guard,
        from_keys: &config::KeyMap,
        from_groups: &Vec<config::GroupDefinition>,
        event_source: &EventSource,
    ) -> Definition {
        let keysyms = event_source.key_symbols();

        let mut definition: Definition = Default::default();

        definition.guard.class = from_guard.class.clone();
        definition.guard.instance = from_guard.instance.clone();
        definition.guard.command = from_guard.command.clone();

        for (k, v) in from_keys {
            let binding = Binding::new(v, None);
            for d in KeyDescription::from_string(k, keysyms) {
                definition.bindings.insert(d, binding.clone());
            }
            definition.keys.insert(k.clone(), binding);
        }
        for g in from_groups {
            for (k, v) in &g.keys {
                let binding = Binding::new(&v, Some(g.label.clone()));
                for d in KeyDescription::from_string(&k, keysyms) {
                    definition.bindings.insert(d, binding.clone());
                }
                definition.keys.insert(k.clone(), binding);
            }
        }

        definition
    }
}

#[derive(Default, Debug, Clone)]
pub struct Guard {
    pub class: Option<Regex>,
    pub instance: Option<Regex>,
    pub command: Option<CommandLine>,
}

#[derive(Clone, Debug)]
pub enum Binding {
    Exec {
        group_label: Option<DisplayLabel>,
        label: DisplayLabel,
        exec: CommandLine,
    },
    Call {
        group_label: Option<DisplayLabel>,
        label: DisplayLabel,
        call: CommandLine,
    },
    Mode {
        group_label: Option<DisplayLabel>,
        label: DisplayLabel,
        mode: DefinitionId,
    },
}

impl Binding {
    fn new(from: &config::Binding, group_label: Option<DisplayLabel>) -> Binding {
        match from {
            config::Binding::Command {
                label,
                command,
                r#loop: false,
                select_window: _,
            } => Binding::Exec {
                group_label,
                label: label.clone(),
                exec: command.clone(),
            },
            config::Binding::Command {
                label,
                command,
                r#loop: true,
                select_window: _,
            } => Binding::Call {
                group_label,
                label: label.clone(),
                call: command.clone(),
            },
            config::Binding::Mode { label, mode } => Binding::Mode {
                group_label,
                label: label.clone(),
                mode: mode.clone(),
            },
        }
    }

    pub fn clone_with_label(&self, label: String) -> Binding {
        match self {
            Binding::Exec {
                group_label, exec, ..
            } => Binding::Exec {
                label,
                group_label: group_label.clone(),
                exec: exec.clone(),
            },
            Binding::Call {
                group_label, call, ..
            } => Binding::Call {
                label,
                group_label: group_label.clone(),
                call: call.clone(),
            },
            Binding::Mode {
                group_label, mode, ..
            } => Binding::Mode {
                label,
                group_label: group_label.clone(),
                mode: mode.clone(),
            },
        }
    }

    pub fn group_label(&self) -> &Option<DisplayLabel> {
        match self {
            Binding::Exec { group_label, .. } => group_label,
            Binding::Call { group_label, .. } => group_label,
            Binding::Mode { group_label, .. } => group_label,
        }
    }

    pub fn label(&self) -> &DisplayLabel {
        match self {
            Binding::Exec { label, .. } => label,
            Binding::Call { label, .. } => label,
            Binding::Mode { label, .. } => label,
        }
    }
}
