use super::model::*;
use std::process::Command;

fn ceramic_do(cmd: &str) {
    Command::new("sh")
            .arg("-c")
            .arg(format!("xprop -root -f CERAMIC_COMMAND 8u -set CERAMIC_COMMAND '{}'", cmd).as_str())
            .output()
            .expect("failed to execute process");
}

pub fn extend_model(model: &mut Model) {
    model.extend_with(&bindings! {
        root {
            Cmd + space => { "Cycle Layout" ceramic_do("switch_to_next_layout") }
            Cmd + Opt + q => { "Exit" {} }
            Cmd + Backspace => { "Close Window" {} }
            Cmd + minus => { "Decrease Ratio" {} }
            Cmd + plus => { "Increase Ratio" {} }
            Cmd + Opt + minus => { "Decrease Count" {} }
            Cmd + Opt + plus => { "Increase Count" {} }
            Cmd + r => { "Launch" => window_manager::launch }
            group "Focus" {
                Cmd + Tab => { "Next" ceramic_do("focus_on_next_window") }
                Cmd + Shift + Tab => { "Previous" ceramic_do("focus_on_previous_window") }
                Cmd + j => { "Next" ceramic_do("focus_on_next_window") }
                Cmd + k => { "Previous" ceramic_do("focus_on_previous_window") }
                Cmd + 1 => { "Space 1" ceramic_do("focus_on_workspace_named: 1") }
                Cmd + 1 => { "Space 2" ceramic_do("focus_on_workspace_named: 2") }
                Cmd + 1 => { "Space 3" ceramic_do("focus_on_workspace_named: 3") }
                Cmd + 1 => { "Space 4" ceramic_do("focus_on_workspace_named: 4") }
                Cmd + 1 => { "Space 5" ceramic_do("focus_on_workspace_named: 5") }
                Cmd + 1 => { "Space 6" ceramic_do("focus_on_workspace_named: 6") }
                Cmd + 1 => { "Space 7" ceramic_do("focus_on_workspace_named: 7") }
                Cmd + 1 => { "Space 8" ceramic_do("focus_on_workspace_named: 8") }
                Cmd + 1 => { "Space 9" ceramic_do("focus_on_workspace_named: 9") }
                Cmd + 0 => { "Select …" ceramic_do("focus_on_window: {window}") }
            }
            group "Move" {
                Cmd + Shift + j => { "Forward" {} }
                Cmd + Shift + k => { "Backward" {} }
                Cmd + Shift + 1 => { "To Space 1" {} }
                Cmd + Shift + 2 => { "To Space 2" {} }
                Cmd + Shift + 3 => { "To Space 3" {} }
                Cmd + Shift + 4 => { "To Space 4" {} }
                Cmd + Shift + 5 => { "To Space 5" {} }
                Cmd + Shift + 6 => { "To Space 6" {} }
                Cmd + Shift + 7 => { "To Space 7" {} }
                Cmd + Shift + 8 => { "To Space 8" {} }
                Cmd + Shift + 9 => { "To Space 9" {} }
                Cmd + Shift + 0 => { "To …" {} }
                Cmd + Shift + Opt + 0 => { "Swap With …" {} }
                Cmd + Opt + 0 => { "Pull To Head …" {} }
        }
     }
    })
}
