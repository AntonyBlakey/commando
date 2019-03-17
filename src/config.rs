use regex::Regex;
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct ConfigFile {
    pub definitions: Vec<Definition>,
}

impl ConfigFile {
    pub fn from_string(source: &String) -> json5::Result<ConfigFile> {
        json5::from_str(source.as_str())
    }
}

#[derive(Deserialize, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Cancel,
    ToggleHelp,
}

#[derive(Deserialize, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    ShowHelp,
    HideHelp,
    EnterModal,
    ExitModal,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Definition {
    Root {
        keys: KeyMap,
        commands: HashMap<Command, Vec<String>>,
        handlers: HashMap<Event, String>,
    },
    Linear {
        path: String,
        #[serde(default)]
        guard: Guard,
        #[serde(default)]
        keys: KeyMap,
        #[serde(default)]
        groups: Vec<GroupDefinition>,
    },
}

#[derive(Deserialize, Debug, Default)]
pub struct Guard {
    #[serde(default, deserialize_with = "optional_regex")]
    pub class: Option<Regex>,
    #[serde(default, deserialize_with = "optional_regex")]
    pub instance: Option<Regex>,
    #[serde(default)]
    pub command: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct GroupDefinition {
    pub label: String,
    pub keys: KeyMap,
}

pub type KeyMap = HashMap<String, Binding>;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Binding {
    Command {
        label: String,
        command: String,
        #[serde(default)]
        r#loop: bool,
        #[serde(default)]
        select_window: bool,
    },
    Mode {
        label: String,
        mode: String,
    },
}

fn optional_regex<'de, D>(deserializer: D) -> Result<Option<Regex>, D::Error>
where
    D: Deserializer<'de>,
{
    struct RegexVisitor;

    impl<'de> Visitor<'de> for RegexVisitor {
        type Value = Option<Regex>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a regular exression")
        }

        fn visit_str<E>(self, value: &str) -> Result<Option<Regex>, E>
        where
            E: de::Error,
        {
            match Regex::new(value) {
                Ok(x) => Ok(Some(x)),
                Err(e) => Err(E::custom(e)),
            }
        }
    }

    deserializer.deserialize_str(RegexVisitor)
}
