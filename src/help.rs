use super::model::{Binding, Event, Model};
use horrorshow::{append_html, helper::doctype, html, Raw};
use itertools::{enumerate, Itertools};
use lazy_static::lazy_static;
use std::{cmp::Ordering, collections::HashMap, io::Write, path::PathBuf};

#[derive(Eq, PartialEq)]
struct SortKey {
    prefix: String,
    category: u8,
    name: String,
}

impl SortKey {
    fn category(str: &str) -> u8 {
        match KEYSYM_SORT_ORDER.get(str) {
            Some(order) => 2 + order,
            None => {
                if str.len() > 1 {
                    255
                } else {
                    match str.chars().next() {
                        None => 0,
                        Some(c) => {
                            if c.is_numeric() {
                                0
                            } else if c.is_alphabetic() {
                                1
                            } else {
                                254
                            }
                        }
                    }
                }
            }
        }
    }
}

impl From<&String> for SortKey {
    fn from(str: &String) -> SortKey {
        match str.rfind('-') {
            Some(index) => {
                let (prefix, name) = str.split_at(index);
                SortKey {
                    prefix: String::from(prefix),
                    category: Self::category(name),
                    name: String::from(name),
                }
            }
            None => SortKey {
                prefix: String::from(""),
                category: Self::category(str.as_str()),
                name: str.clone(),
            },
        }
    }
}

impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &SortKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SortKey {
    fn cmp(&self, other: &SortKey) -> Ordering {
        let prefix_len_cmp = self.prefix.len().cmp(&other.prefix.len());
        if prefix_len_cmp != Ordering::Equal {
            prefix_len_cmp
        } else {
            let prefix_cmp = self.prefix.cmp(&other.prefix);
            if prefix_cmp != Ordering::Equal {
                prefix_cmp
            } else {
                let category_cmp = self.category.cmp(&other.category);
                if category_cmp != Ordering::Equal {
                    category_cmp
                } else {
                    self.name.cmp(&other.name)
                }
            }
        }
    }
}

#[derive(Default)]
pub struct HelpEngine {
    is_showing: bool,
    current_position: u8,
    start_position: u8,
}

impl HelpEngine {
    pub fn is_showing(&self) -> bool {
        self.is_showing
    }

    pub fn show(&mut self, model: &Model, definition_id: &Option<String>) {
        let path = PathBuf::from("/tmp/commando.help.html");
        let mut file = std::fs::File::create(&path).unwrap();

        let mut all_group_labels = Vec::new();
        let mut all_group_keys = Vec::new();

        let keys: Vec<(&String, &Binding)> = match definition_id {
            None => model.keys.iter().collect(),
            // TODO: check guards on each definition
            Some(id) => match model.definitions.get(id) {
                Some(def) => def
                .iter()
                .flat_map(|d| d.keys.iter())
                .collect(),
                None => Vec::new()
            }
        };

        let mut unique_keys: Vec<&(&String, &Binding)> = keys.iter().unique_by(|a| a.0).collect();
        unique_keys.sort_unstable_by_key(|a| a.1.group_label());
        for (group_label, group_keys_iterator) in
            &unique_keys.iter().group_by(|a| a.1.group_label())
        {
            let mut group_keys: Vec<(String, Binding)> = group_keys_iterator
                .map(|(s, b)| ((*s).clone(), (*b).clone()))
                .collect();
            group_keys.sort_by_cached_key(|a| SortKey::from(&a.0));
            let mut i = 0;
            while i + 8 < group_keys.len() {
                let (key_1, binding_1) = group_keys[i].clone();
                if key_1 == "1" || key_1.ends_with("-1") {
                    let label_1 = binding_1.label();
                    let mut have_full_sequence = true;
                    for c in ["2", "3", "4", "5", "6", "7", "8", "9"].iter() {
                        i += 1;
                        let (key_n, binding_n) = &group_keys[i];
                        let key_n_translated = key_n.replace(c, "1");
                        let label_n_translated = binding_n.label().replace(c, "1");
                        if key_n_translated.as_str() != key_1
                            || label_n_translated.as_str() != label_1
                        {
                            have_full_sequence = false;
                            break;
                        }
                    }
                    if have_full_sequence {
                        i -= 8;
                        for _ in 1..=9 {
                            group_keys.remove(i);
                        }
                        let key = key_1.replace("1", "1..9");
                        let binding = binding_1.clone_with_label(label_1.replace("1", "1..9"));
                        group_keys.insert(i, (key, binding));
                    }
                } else {
                    i += 1;
                }
            }
            all_group_labels.push(group_label.clone());
            all_group_keys.push(group_keys);
        }

        // TODO: Add cancel/help generic instructions

        write!(
            file,
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        style(type="text/css") {
                            @ for f in model.files.iter().filter(|f| f.file_name().unwrap() == "help.css") {
                                : Raw(std::fs::read_to_string(f).unwrap());
                            }
                        }
                    }
                    body {
                        div(id="body") {
                            div(id="content") {
                                @ for (group_number, group_label) in enumerate(all_group_labels.iter()) {
                                    @ if group_label.is_some() {
                                        div(class="group-label", id=format!("group-label-{}", group_number)) {
                                            : group_label.clone().unwrap();
                                        }
                                        div(class="group-background", id=format!("group-background-{}", group_number)) {
                                        }
                                    }
                                    @ for (group_key_number, group_key) in enumerate(all_group_keys[group_number].iter()) {
                                        div(class="key", id=format!("key-{}-{}", group_number, group_key_number)) {
                                            : Raw(self.html_for_key(&group_key.0));
                                        }
                                        div(class=format!("label {}", group_key.1.label_class()), id=format!("label-{}-{}", group_number, group_key_number)) {
                                            : group_key.1.label();
                                        }
                                    }
                                }
                            }
                        }
                        script(type="text/javascript") {
                            : "const bindings = [";
                            @ for group_keys in &all_group_keys {
                                : format!("{},", group_keys.len());
                            }
                            : "];";
                            @ for f in model.files.iter().filter(|f| f.file_name().unwrap() == "help.js") {
                                : Raw(std::fs::read_to_string(f).unwrap());
                            }
                        }
                    }
                }
            }
        )
        .unwrap();

        let path_str = path.to_str().unwrap();

        if !self.is_showing {
            self.current_position = self.start_position;
        }
        if let Some(command_line) = model.handlers.get(&Event::ShowHelp) {
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!(
                    "{} {} {}",
                    command_line, path_str, self.current_position
                ))
                .status();
            if let Err(err) = status {
                eprintln!("command {} failed with {:?}", command_line, err);
            }
        }

        self.is_showing = true;
    }

    pub fn hide(&mut self, model: &Model) {
        if self.is_showing {
            if let Some(command_line) = model.handlers.get(&Event::HideHelp) {
                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command_line)
                    .status();
                if let Err(err) = status {
                    eprintln!("command {} failed with {:?}", command_line, err);
                }
            }

            self.is_showing = false;
        }
    }

    pub fn toggle(&mut self, model: &Model, definition_id: &Option<String>) {
        if self.is_showing {
            if self.current_position == self.start_position {
                self.hide(model);
                self.start_position = 1 - self.start_position;
                self.show(model, definition_id);
                self.start_position = 1 - self.start_position;
            } else {
                self.hide(model);
                self.start_position = self.current_position;
            }
        } else {
            self.show(model, definition_id)
        }
    }

    fn html_for_keysym(&self, string: &str) -> String {
        match KEYSYM_HTML.get(string) {
            Some(html) => html.to_string(),
            _ => string.to_string(),
        }
    }

    fn html_for_key(&self, string: &String) -> String {
        let tokens: Vec<&str> = string.split('-').collect();
        if let Some((keysym_name, raw_modifiers)) = tokens.split_last() {
            let mut shift = false;
            let mut control = false;
            let mut alt = false;
            let mut hyper = false;
            let mut supr = false;
            for raw in raw_modifiers {
                match raw.to_lowercase().as_str() {
                    "shift" | "s" => shift = true,
                    "control" | "ctrl" | "c" => control = true,
                    "alt" | "a" | "opt" | "o" | "meta" | "m" => alt = true,
                    "super" | "windows" | "win" | "w" | "command" | "cmd" => supr = true,
                    "hyper" | "h" => hyper = true,
                    _ => (),
                }
            }

            format!(
                "<span class='prefix'>{}{}{}{}{}</span>{}",
                if hyper {
                    self.html_for_keysym("Hyper")
                } else {
                    "".to_string()
                },
                if supr {
                    self.html_for_keysym("Super")
                } else {
                    "".to_string()
                },
                if control {
                    self.html_for_keysym("Control")
                } else {
                    "".to_string()
                },
                if alt {
                    self.html_for_keysym("Alt")
                } else {
                    "".to_string()
                },
                if shift {
                    self.html_for_keysym("Shift")
                } else {
                    "".to_string()
                },
                self.html_for_keysym(keysym_name)
            )
        } else {
            self.html_for_keysym(string.as_str())
        }
    }
}

impl Binding {
    pub fn label_class(&self) -> &str {
        match self {
            Binding::Exec { .. } => "exec",
            Binding::Call { .. } => "call",
            Binding::Mode { .. } => "mode",
        }
    }
}

lazy_static! {
    static ref KEYSYM_SORT_ORDER: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        let symbols = [
            "1..9",
            "parenleft",
            "parenright",
            "bracketleft",
            "bracketright",
            "braceleft",
            "braceright",
            "less",
            "greater",
            "plus",
            "minus",
            "equal",
            "slash",
            "backslash",
            "underscore",
            "bar",
            "semicolon",
            "colon",
            "apostrophe",
            "quotedbl",
            "grave",
            "asciitilde",
            "comma",
            "period",
            "question",
            "numbersign",
            "exclam",
            "at",
            "dollar",
            "percent",
            "asciicircum",
            "ampersand",
            "asterisk",
            "Up",
            "Down",
            "Left",
            "Right",
            "BackSpace",
            "Delete",
            "PageUp",
            "PageDown",
            "Home",
            "End",
            "Tab",
            "Return",
            "space",
            "Escape",
        ];
        for i in 0..symbols.len() {
            m.insert(symbols[i], i as u8);
        }
        m
    };
}

lazy_static! {
    static ref KEYSYM_HTML: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("Tab", "<span class='symbol'>&#8677;</span>");
        m.insert("Return", "<span class='symbol'>&crarr;</span>");
        m.insert("Escape", "<span class='symbol'>&#9099;</span>");
        m.insert("BackSpace", "<span class='symbol'>&#9003;</span>");
        m.insert("Delete", "<span class='symbol'>&#8998;</span>");
        m.insert("Up", "<span class='symbol'>&uarr;</span>");
        m.insert("Down", "<span class='symbol'>&darr;</span>");
        m.insert("Left", "<span class='symbol'>&larr;</span>");
        m.insert("Right", "<span class='symbol'>&rarr;</span>");
        m.insert("PageUp", "<span class='symbol'>&#8670;</span>");
        m.insert("PageDown", "<span class='symbol'>&#8671;</span>");
        m.insert("Home", "<span class='symbol'>&#8598;</span>");
        m.insert("End", "<span class='symbol'>&#8600;</span>");
        m.insert("space", "<span class='symbol'>&#9251;</span>");
        m.insert("plus", "+");
        m.insert("minus", "-");
        m.insert("less", "&lt;");
        m.insert("greater", "&gt;");
        m.insert("equal", "=");
        m.insert("semicolon", ";");
        m.insert("apostrophe", "'");
        m.insert("grave", "`");
        m.insert("backslash", "\\");
        m.insert("comma", ",");
        m.insert("period", ".");
        m.insert("question", "?");
        m.insert("bar", "|");
        m.insert("asciitilde", "~");
        m.insert("quotedbl", "\"");
        m.insert("colon", ":");
        m.insert("underscore", "_");
        m.insert("asterisk", "*");
        m.insert("ampersand", "&");
        m.insert("asciicircum", "^");
        m.insert("percent", "%");
        m.insert("dollar", "$");
        m.insert("numbersign", "#");
        m.insert("at", "@");
        m.insert("exclam", "!");
        m.insert("bracketleft", "[");
        m.insert("bracketright", "]");
        m.insert("braceleft", "{");
        m.insert("braceright", "}");
        m.insert("parenleft", "(");
        m.insert("parenright", ")");
        m.insert("Hyper", "hyper-");
        m.insert("Super", "<span class='symbol'>&#8984;</span>");
        m.insert("Control", "<span class='symbol'>&#8963;</span>");
        m.insert("Alt", "<span class='symbol'>&#8997;</span>");
        m.insert("Shift", "<span class='symbol'>&#8679;</span>");
        m
    };
}
