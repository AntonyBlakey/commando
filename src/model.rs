use super::{connection::connection, keystroke::Keystroke};
use std::{collections::HashMap, rc::Rc, sync::Arc};

pub struct Context {}

impl Context {
    pub fn instance(&self) -> String {
        "".into()
    }

    pub fn class(&self) -> String {
        "".into()
    }

    pub fn connection(&self) -> &xcb::Connection {
        connection()
    }
}

pub struct Model {
    bindings: HashMap<&'static str, Vec<Binding>>,
}

impl Model {
    pub fn new() -> Model {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn extend_with(&mut self, factory: &Fn(&mut Self)) {
        factory(self);
    }

    pub fn add_binding(
        &mut self,
        set: &'static str,
        keystrokes: Vec<Keystroke>,
        label: &'static str,
        group: Option<&'static str>,
        guard: Option<Arc<Guard>>,
        action: Action,
    ) {
        self.bindings
            .entry(set)
            .or_default()
            .extend(keystrokes.iter().map(|&keystroke| {
                Binding::new(keystroke, label, group, guard.clone(), action.clone())
            }));
    }

    pub fn get_applicable_bindings(&self, name: &str, context: &Context) -> Vec<Binding> {
        self.bindings
            .get("@global")
            .iter()
            .chain(self.bindings.get(name).iter())
            .flat_map(|&bs| bs)
            .filter(|b| b.apply_guard(context))
            .cloned()
            .collect()
    }

    pub fn get_binding(
        &self,
        set: &str,
        context: &Context,
        keystroke: Keystroke,
    ) -> Option<Binding> {
        self.bindings
            .get("@global")
            .iter()
            .chain(self.bindings.get(set).iter())
            .flat_map(|&bs| bs)
            .find(|b| b.keystroke() == keystroke && b.apply_guard(context))
            .cloned()
    }

    pub fn get_root_grab_keys(&self) -> Vec<Keystroke> {
        self.get_applicable_bindings("@root", &Context {})
            .iter()
            .filter_map(|b| match b.action {
                Action::Cancel => None,
                _ => Some(b.keystroke),
            })
            .collect()
    }
}

pub trait Guard = Fn(&Context) -> bool;

pub fn new_guard<F>(f: F) -> Arc<Guard>
where
    F: Guard + 'static,
{
    Arc::new(f)
}

pub trait ActionFn = Fn(&Context);
pub fn new_actionfn<F>(f: F) -> Arc<ActionFn>
where
    F: ActionFn + 'static,
{
    Arc::new(f)
}

#[derive(Clone)]
pub enum Action {
    Cancel,
    ToggleHelp,
    Mode(&'static str),
    Call(Arc<ActionFn>),
    Exec(Arc<ActionFn>),
}

#[derive(Clone)]
pub struct Binding {
    keystroke: Keystroke,
    label: &'static str,
    group: Option<&'static str>,
    guard: Option<Arc<Guard>>,
    action: Action,
}

impl Binding {
    pub fn new(
        keystroke: Keystroke,
        label: &'static str,
        group: Option<&'static str>,
        guard: Option<Arc<Guard>>,
        action: Action,
    ) -> Binding {
        Self {
            keystroke,
            label,
            group,
            guard,
            action,
        }
    }

    pub fn keystroke(&self) -> Keystroke {
        self.keystroke
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    pub fn group(&self) -> Option<&'static str> {
        self.group
    }

    pub fn apply_guard(&self, context: &Context) -> bool {
        match &self.guard {
            Some(f) => f(context),
            None => true,
        }
    }

    pub fn action<'a>(&'a self) -> &'a Action {
        &self.action
    }
}

#[macro_export]
macro_rules! bindings {

    (@new_guard None | $($args:tt)* | $($body:tt)+) => { new_guard(| $($args)* | $($body)+) };
    (@new_guard None $($body:tt)+) => { new_guard(|_ctx:&Context| $($body)+) };

    (@new_guard $old_guard:ident | $($args:tt)* | $($body:tt)+) => { new_guard(| $($args)* | $($body)+) };
    (@new_guard $old_guard:ident $($body:tt)+) => { new_guard(|_ctx:&Context| $($body)+) };

    (@new_actionfn | $($x:tt)+) => { new_actionfn(| $($x)+) };
    (@new_actionfn $($x:tt)+) => { new_actionfn(|_ctx:&Context| $($x)+) };


    (
        @in_binding $model:ident $mode:tt $group:tt $guard:tt ($($keystrokes:tt)+)
        $label:literal => $new_mode:path
    ) => {
         $model.add_binding($mode, $($keystrokes)+, $label, $group, $guard, Action::Mode(stringify!($new_mode)))
    };

    (
        @in_binding $model:ident $mode:tt $group:tt $guard:tt  ($($keystrokes:tt)+)
        $label:literal cancel
    ) => {
         $model.add_binding($mode, $($keystrokes)+, $label, $group, $guard, Action::Cancel)
    };

    (
        @in_binding $model:ident $mode:tt $group:tt $guard:tt  ($($keystrokes:tt)+)
        $label:literal toggle help
    ) => {
         $model.add_binding($mode, $($keystrokes)+, $label, $group, $guard, Action::ToggleHelp)
    };

    (
        @in_binding $model:ident $mode:tt $group:tt $guard:tt ($($keystrokes:tt)+)
        $label:literal hydra $($expr:tt)+
    ) => {
         $model.add_binding($mode, $(keystrokes)+, $label, $group, $guard, Action::Call(bindings!(@new_actionfn $($expr)+)))
    };

    (
        @in_binding $model:ident $mode:tt $group:tt $guard:tt ($($keystrokes:tt)+)
        $label:literal $($expr:tt)+
    ) => {
         $model.add_binding($mode, $($keystrokes)+, $label, $group, $guard, Action::Exec(bindings!(@new_actionfn $($expr)+)))
    };


    (@in_mode $model:ident $mode:tt $group:tt $guard:tt) => {};

    (
        @in_mode $model:ident $mode:tt None $guard:tt
        group $name:literal { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let group = Some($name);
            bindings!(@in_mode $model $mode group $guard $($body)+);
        }
        bindings!(@in_mode $model $mode None $guard $($rest)*);
    };

    (
        @in_mode $model:ident $mode:tt $group:tt $guard:tt
        guard ( $($new_guard:tt)+ ) { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let guard = Some(bindings!(@new_guard $guard $($new_guard)+));
            bindings!(@in_mode $model $mode $group guard $($body)+);
        }
        bindings!(@in_mode $model $mode $group $guard $($rest)*);
    };

    (
        @in_mode $model:ident $mode:tt $group:tt $guard:tt
        $head:tt $(+ $tail:tt)* => { $($body:tt)+ } $($rest:tt)*
    ) => {
        bindings!(@in_binding $model $mode $group $guard (key!($head $(+ $tail)*)) $($body)+);
        bindings!(@in_mode $model $mode $group $guard $($rest)*)
    };

    (@in_mode $($rest:tt)+) => {
        // Error
        $($rest)+
    };


    (@in_model $model:ident $guard:tt) => {};

    (
        @in_model $model:ident $guard:tt
        guard ( $($new_guard:tt)+ ) { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let guard = Some(bindings!(@new_guard $guard $($new_guard)+));
            bindings!(@in_model $model guard $($body)+);
        }
        bindings!(@in_model $model $guard $($rest)*);
    };

    (
        @in_model $model:ident $guard:tt
        global { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let mode = "@global";
            bindings!(@in_mode $model mode None $guard $($body)+);
        }
        bindings!(@in_model $model $guard $($rest)*);
    };

    (
        @in_model $model:ident $guard:tt
        root { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let mode = "@root";
            bindings!(@in_mode $model mode None $guard $($body)+);
        }
        bindings!(@in_model $model $guard $($rest)*);
    };

    (
        @in_model $model:ident $guard:tt
        mode $id:path { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let mode = stringify!($id);
            bindings!(@in_mode $model mode None $guard $($body)+);
        }
        bindings!(@in_model $model $guard $($rest)*);
    };

    (@in_model $($rest:tt)+) => {
        // Error
        $($rest)+
    };


    (
        $($body:tt)*
    ) => {
        |model:&mut $crate::model::Model| { bindings!(@in_model model None $($body)*); }
    };

}
