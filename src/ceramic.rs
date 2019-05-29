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
            Cmd + Opt + q => { "Exit" ceramic_do("quit") }
            Cmd + Backspace => { "Close Window" ceramic_do("close_focused_window") }
            Cmd + minus => { "Decrease Ratio" ceramic_do("layout/decrease_ratio") }
            Cmd + plus => { "Increase Ratio" ceramic_do("layout/increase_ratio") }
            Cmd + Opt + minus => { "Decrease Count" ceramic_do("layout/decrease_count") }
            Cmd + Opt + plus => { "Increase Count" ceramic_do("layout/increase_count") }
            Cmd + r => { "Launch" => window_manager::launch }
            group "Focus" {
                Cmd + Tab => { "Next" ceramic_do("focus_on_next_window") }
                Cmd + Shift + Tab => { "Previous" ceramic_do("focus_on_previous_window") }
                Cmd + j => { "Next" ceramic_do("focus_on_next_window") }
                Cmd + k => { "Previous" ceramic_do("focus_on_previous_window") }
                Cmd + 1 => { "Space 1" ceramic_do("switch_to_workspace_named: 1") }
                Cmd + 2 => { "Space 2" ceramic_do("switch_to_workspace_named: 2") }
                Cmd + 3 => { "Space 3" ceramic_do("switch_to_workspace_named: 3") }
                Cmd + 4 => { "Space 4" ceramic_do("switch_to_workspace_named: 4") }
                Cmd + 5 => { "Space 5" ceramic_do("switch_to_workspace_named: 5") }
                Cmd + 6 => { "Space 6" ceramic_do("switch_to_workspace_named: 6") }
                Cmd + 7 => { "Space 7" ceramic_do("switch_to_workspace_named: 7") }
                Cmd + 8 => { "Space 8" ceramic_do("switch_to_workspace_named: 8") }
                Cmd + 9 => { "Space 9" ceramic_do("switch_to_workspace_named: 9") }
                Cmd + 0 => { "Select …" ceramic_do("focus_on_window: {window}") }
            }
            group "Move" {
                Cmd + Shift + j => { "Forward" ceramic_do("move_focused_window_forward") }
                Cmd + Shift + k => { "Backward" ceramic_do("move_focused_window_backward") }
                Cmd + Shift + 1 => { "To Space 1" ceramic_do("move_focused_window_to_workspace_named: 1") }
                Cmd + Shift + 2 => { "To Space 2" ceramic_do("move_focused_window_to_workspace_named: 2") }
                Cmd + Shift + 3 => { "To Space 3" ceramic_do("move_focused_window_to_workspace_named: 3") }
                Cmd + Shift + 4 => { "To Space 4" ceramic_do("move_focused_window_to_workspace_named: 4") }
                Cmd + Shift + 5 => { "To Space 5" ceramic_do("move_focused_window_to_workspace_named: 5") }
                Cmd + Shift + 6 => { "To Space 6" ceramic_do("move_focused_window_to_workspace_named: 6") }
                Cmd + Shift + 7 => { "To Space 7" ceramic_do("move_focused_window_to_workspace_named: 7") }
                Cmd + Shift + 8 => { "To Space 8" ceramic_do("move_focused_window_to_workspace_named: 8") }
                Cmd + Shift + 9 => { "To Space 9" ceramic_do("move_focused_window_to_workspace_named: 9") }
                // Cmd + Shift + 0 => { "To …" {} }
                // Cmd + Shift + Opt + 0 => { "Swap With …" {} }
                Cmd + Opt + 0 => { "Pull To Head …" ceramic_do("move_focused_window_to_head") }
        }
     }
    })
}
