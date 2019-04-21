use super::root::*;

pub fn config(key_bindings: &mut KeyBindings) {
    key_bindings.extend_with(&bindings! {
        root = {
            Cmd + space => { "Cycle Layout" {} }
            Cmd + Opt + q => { "Exit" {} }
            Cmd + Backspace => { "Close Window" {} }
            Cmd + h => { "Decrease Ratio" {} }
            Cmd + l => { "Increase Ratio" {} }
            Cmd + Shift + h => { "Decrease Count" {} }
            Cmd + Shift + plus => { "Increase Count" {} }
            Cmd + r => { "Launch" => window_manager::launch }
            group "Focus" = {
                Cmd + Tab => { "Next" {} }
                Cmd + Shift + Tab => { "Previous" {} }
                Cmd + j => { "Next" {} }
                Cmd + k => { "Previous" {} }
                Cmd + 1 => { "Space 1" {} }
                Cmd + 2 => { "Space 2" {} }
                Cmd + 3 => { "Space 3" {} }
                Cmd + 4 => { "Space 4" {} }
                Cmd + 5 => { "Space 5" {} }
                Cmd + 6 => { "Space 6" {} }
                Cmd + 7 => { "Space 7" {} }
                Cmd + 8 => { "Space 8" {} }
                Cmd + 9 => { "Space 9" {} }
                Cmd + 0 => { "Select …" {} }
            }
            group "Move" = {
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
