use super::keystroke::Keystroke;
use std::{collections::HashMap, rc::Rc};

pub struct Context {}

impl Context {
    pub fn instance(&self) -> String {
        "".into()
    }

    pub fn class(&self) -> String {
        "".into()
    }

    pub fn cancel(&self) {}

    pub fn toggle_help(&self) {}

    pub fn enter_mode(&self, _name: &str) {}
}


pub struct KeyBindings {
    binding_sets: HashMap<&'static str, Vec<BindingSet>>,
}

impl KeyBindings {
    pub fn new() -> KeyBindings {
        Self {
            binding_sets: HashMap::new(),
        }
    }

    pub fn extend_with(&mut self, factory: &Fn(&mut Self)) {
        factory(self);
    }

    pub fn add_binding_set(&mut self, binding_set: BindingSet) {
        self.binding_sets
            .entry(binding_set.name())
            .or_insert(Default::default())
            .push(binding_set);
    }
}

pub type Guard = Fn(&Context) -> bool;

pub struct BindingSet {
    name: &'static str,
    binding_groups: HashMap<Option<&'static str>, Vec<BindingGroup>>,
    guard: Option<Rc<Guard>>,
}

impl BindingSet {
    pub fn new(name: &'static str, guard: Option<Rc<Guard>>) -> BindingSet {
        Self {
            name,
            binding_groups: HashMap::new(),
            guard,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn add_binding_group(&mut self, binding_group: BindingGroup) {
        self.binding_groups
            .entry(binding_group.name())
            .or_insert(Default::default())
            .push(binding_group);
    }
}

pub struct BindingGroup {
    name: Option<&'static str>,
    bindings: std::collections::HashMap<Keystroke, Vec<Binding>>,
    guard: Option<Rc<Guard>>,
}

impl BindingGroup {
    pub fn new(name: Option<&'static str>, guard: Option<Rc<Guard>>) -> BindingGroup {
        Self {
            name,
            bindings: HashMap::new(),
            guard,
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        self.name
    }

    pub fn add_binding(&mut self, keystrokes: Vec<Keystroke>, binding: Binding) {
        for keystroke in keystrokes {
            self.bindings
                .entry(keystroke)
                .or_insert(Default::default())
                .push(binding.clone());
        }
    }
}

pub type Action = Fn(&Context);

#[derive(Clone)]
pub struct Binding {
    label: &'static str,
    action: Rc<Action>,
    is_hydra: bool,
}

impl Binding {
    pub fn new(label: &'static str, is_hydra: bool, action: Rc<Action>) -> Binding {
        Binding {
            label,
            is_hydra,
            action,
        }
    }
}

#[macro_export]
macro_rules! bindings {

    (@clone None) => { None };
    (@clone $x:tt) => { $x.clone() };

    (@with_context | $($x:tt)+) => { ::std::rc::Rc::new(| $($x)+) };
    (@with_context $($x:tt)+) => { ::std::rc::Rc::new(|_ctx:&Context| $($x)+) };


    (
        @binding
        $label:literal => $mode:path
    ) => {
         $crate::root::Binding::new($label, false, ::std::rc::Rc::new(|ctx:&Context| ctx.enter_mode(stringify!($mode))))
    };

    (
        @binding
        $label:literal cancel
    ) => {
         $crate::root::Binding::new($label, false, ::std::rc::Rc::new(|ctx:&Context| ctx.cancel()))
    };

    (
        @binding
        $label:literal toggle help
    ) => {
         $crate::root::Binding::new($label, false, ::std::rc::Rc::new(|ctx:&Context| ctx.toggle_help()))
    };

    (
        @binding
        $label:literal hydra $($expr:tt)+
    ) => {
         $crate::root::Binding::new($label, true, bindings!(@with_context $($expr)+))
    };

    (
        @binding
        $label:literal $($expr:tt)+
    ) => {
         $crate::root::Binding::new($label, false, bindings!(@with_context $($expr)+))
    };


    (@named_group $group:ident) => {};

    (
        @named_group $group:ident
        $head:tt $(+ $tail:tt)* => { $($body:tt)+ } $($rest:tt)*
    ) => {
        $group.add_binding(key!($head $(+ $tail)*), bindings!(@binding $($body)+));
        bindings!(@named_group $group $($rest)*)
    };


    (@default_group $set:ident $group:ident $guard:ident) => {};

    (
        @default_group $set:ident $group:ident $guard:ident
        group $name:literal = { $($body:tt)+ } $($rest:tt)*
    ) => {
        let mut group = $crate::root::BindingGroup::new(Some($name), bindings!(@clone $guard));
        bindings!(@named_group group $($body)+);
        $set.add_binding_group(group);
        bindings!(@default_group $set $group $guard $($rest)*);
    };

    (
        @default_group $set:ident $group:ident $guard:ident
        guard ( $($new_guard:tt)+ ) { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let guard = Some(bindings!(@with_context $($new_guard)+) as ::std::rc::Rc<Guard>);
            bindings!(@default_group $set None guard $($body)+);
        }
        bindings!(@default_group $set $group $guard $($rest)*);
    };

    (
        @default_group $set:ident None $guard:ident
        $head:tt $(+ $tail:tt)* => { $($body:tt)+ } $($rest:tt)*
    ) => {
        let mut group = $crate::root::BindingGroup::new(None, bindings!(@clone $guard));
        group.add_binding(key!($head $(+ $tail)*), bindings!(@binding $($body)+));
        bindings!(@default_group $set group None $($rest)*);
        $set.add_binding_group(group);
    };

    (
        @default_group $set:ident $group:ident $guard:ident
        $head:tt $(+ $tail:tt)* => { $($body:tt)+ } $($rest:tt)*
    ) => {
        $group.add_binding(key!($head $(+ $tail)*), bindings!(@binding $($body)+));
        bindings!(@default_group $set $group $guard $($rest)*);
    };


    (@binding_set $v:ident $guard:ident) => {};

    (
        @binding_set $v:ident $guard:ident
        guard ( $($new_guard:tt)+ ) { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let guard = Some(bindings!(@with_context $($new_guard)+) as ::std::rc::Rc<Guard>);
            bindings!(@binding_set $v guard $($body)+);
        }
        bindings!(@binding_set $v $guard $($rest)*);
    };

    (
        @binding_set $v:ident $guard:ident
        $id:path = { $($body:tt)+ } $($rest:tt)*
    ) => {
        {
            let mut set = $crate::root::BindingSet::new(stringify!($id), bindings!(@clone $guard));
            bindings!(@default_group set None None $($body)+);
            $v.add_binding_set(set);
        }
        bindings!(@binding_set $v $guard $($rest)*);
    };


    (
        $($body:tt)*
    ) => {
        |key_bindings:&mut $crate::root::KeyBindings| { bindings!(@binding_set key_bindings None $($body)*); }
    };

}
