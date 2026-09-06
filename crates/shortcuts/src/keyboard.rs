//! The [`Keyboard`] orchestrator: context stack, interceptor chain, and the
//! stateful `fire` dispatch loop.
//!
//! Interior-mutable (`&self` methods, `RefCell`/`Cell` inside) so it can live
//! in `Rc<Keyboard>` and be driven from a host event loop.

use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::context::{CommandOptions, ContextOptions, KeymapOptions, ShortcutContext};
use crate::error::ShortcutsError;
use crate::event::{KeyEventType, ShortcutKeyboardEvent};
use crate::interceptor::{Interceptor, ShortcutEvent};
use crate::registry::{default_macro_registry, MacroRegistryImpl};
use crate::shortcut::Shortcut;

/// A resolved command (after parsing the shortcut string).
struct ParsedCommand {
    shortcut: Rc<Shortcut>,
    event: Vec<KeyEventType>,
    prevent_default: bool,
    interceptors: Vec<Interceptor>,
}

/// A resolved context.
#[derive(Clone)]
struct FullContext {
    commands: Vec<String>,
    #[allow(dead_code)]
    abstract_ctx: bool,
    fallbacks: Vec<String>,
}

/// An ad-hoc `on_shortcut_key_match` listener.
struct AdhocListener {
    shortcut: Rc<Shortcut>,
    handler: Box<dyn Fn(&ShortcutEvent<'_>) + 'static>,
    once: bool,
    event_type: KeyEventType,
}

/// A matched command returned by [`Keyboard::fire`].
pub struct FireMatch<'a> {
    pub command: String,
    pub event: ShortcutEvent<'a>,
    /// Whether this command requested default-action suppression (maps to
    /// `stop_propagation` in vgui).
    pub prevent_default: bool,
}

/// Guard that truncates the context stack back to its prior depth on drop.
pub struct ContextGuard {
    stack: Rc<RefCell<Vec<String>>>,
    index: usize,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        if self.index < self.stack.borrow().len() {
            self.stack.borrow_mut().truncate(self.index);
        }
    }
}
/// Guard that removes a registered interceptor on drop.
pub struct InterceptorGuard {
    interceptors: Rc<RefCell<Vec<Interceptor>>>,
    interceptor: Interceptor,
}

impl Drop for InterceptorGuard {
    fn drop(&mut self) {
        self.interceptors
            .borrow_mut()
            .retain(|i| !Rc::ptr_eq(i, &self.interceptor));
    }
}

/// The shortcuts engine.
pub struct Keyboard {
    commands: RefCell<HashMap<String, ParsedCommand>>,
    contexts: RefCell<HashMap<String, FullContext>>,
    context_stack: Rc<RefCell<Vec<String>>>,
    interceptors: Rc<RefCell<Vec<Interceptor>>>,
    registry: MacroRegistryImpl,
    partial_matches: RefCell<HashSet<String>>,
    paused: Cell<bool>,
    command_listeners:
        RefCell<HashMap<String, Vec<Rc<dyn Fn(&ShortcutEvent<'_>) + 'static>>>>,
    adhoc_listeners: RefCell<Vec<AdhocListener>>,
    partial_change_listeners: RefCell<Vec<Box<dyn Fn(&[String]) + 'static>>>,
}

impl Keyboard {
    /// New engine with an empty keymap and a child registry whose parent is
    /// the built-in default locale.
    pub fn new() -> Self {
        Self {
            commands: RefCell::new(HashMap::new()),
            contexts: RefCell::new(HashMap::new()),
            context_stack: Rc::new(RefCell::new(Vec::new())),
            interceptors: Rc::new(RefCell::new(Vec::new())),
            registry: MacroRegistryImpl::with_parent(default_macro_registry()),
            partial_matches: RefCell::new(HashSet::new()),
            paused: Cell::new(false),
            command_listeners: RefCell::new(HashMap::new()),
            adhoc_listeners: RefCell::new(Vec::new()),
            partial_change_listeners: RefCell::new(Vec::new()),
        }
    }

    /// Register commands and contexts. Re-registering a command merges new
    /// options over prior over defaults (trinomial merge, mirroring the TS
    /// library).
    pub fn keymap(&self, opts: KeymapOptions) {
        self.record_commands(opts.commands);
        self.record_contexts(opts.contexts);
    }

    fn record_commands(&self, commands: HashMap<String, CommandOptions>) {
        let mut cmds = self.commands.borrow_mut();
        for (name, opts) in commands {
            let prev = cmds.get(&name);
            let event = opts
                .event
                .or_else(|| prev.map(|p| p.event.clone()))
                .unwrap_or_else(|| vec![KeyEventType::KeyDown]);
            let prevent_default = opts
                .prevent_default
                .or_else(|| prev.map(|p| p.prevent_default))
                .unwrap_or(true);
            let interceptors = opts
                .interceptors
                .or_else(|| prev.map(|p| p.interceptors.clone()))
                .unwrap_or_default();
            let shortcut = match Shortcut::from(&opts.shortcut, &self.registry) {
                Ok(s) => Rc::new(s),
                Err(e) => {
                    panic!("Failed to parse shortcut for command {name}: {e}");
                }
            };
            cmds.insert(
                name,
                ParsedCommand {
                    shortcut,
                    event,
                    prevent_default,
                    interceptors,
                },
            );
        }
    }

    fn record_contexts(&self, contexts: HashMap<String, ContextOptions>) {
        let mut ctxs = self.contexts.borrow_mut();
        for (name, opts) in contexts {
            ctxs.insert(
                name,
                FullContext {
                    commands: opts.commands,
                    abstract_ctx: opts.abstract_ctx.unwrap_or(false),
                    fallbacks: opts.fallbacks.unwrap_or_default(),
                },
            );
        }
    }

    /// Push `name` onto the context stack with the TS reactivation rule and
    /// return a guard that pops back on drop.
    pub fn switch_context(&self, name: &str) -> Result<ContextGuard, ShortcutsError> {
        if !self.contexts.borrow().contains_key(name) {
            return Err(ShortcutsError::ContextNotRegistered(name.to_string()));
        }
        let mut stack = self.context_stack.borrow_mut();
        let activated_index = stack.iter().position(|c| c == name);
        let index = stack.len();
        match activated_index {
            None => stack.push(name.to_string()),
            Some(ai) => {
                let current = stack.last().cloned();
                if current.as_deref() == Some(name) {
                    stack.push(name.to_string());
                } else {
                    stack.truncate(ai + 1);
                    stack.push(name.to_string());
                }
            }
        }
        drop(stack);
        Ok(ContextGuard {
            stack: self.context_stack.clone(),
            index,
        })
    }

    pub fn current_context(&self) -> Option<String> {
        self.context_stack.borrow().last().cloned()
    }


    /// Replace the entire context stack with a single entry. Unlike
    /// [`switch_context`](Self::switch_context), this does not push or
    /// return a guard — the stack is cleared first.
    pub fn set_context(&self, name: &str) -> Result<(), ShortcutsError> {
        if !self.contexts.borrow().contains_key(name) {
            return Err(ShortcutsError::ContextNotRegistered(name.to_string()));
        }
        let mut stack = self.context_stack.borrow_mut();
        stack.clear();
        stack.push(name.to_string());
        Ok(())
    }

    /// Whether any commands have been registered via `keymap`.
    pub fn has_keymap(&self) -> bool {
        !self.commands.borrow().is_empty()
    }

    /// Current partial-match command names, sorted alphabetically.
    pub fn partial_matches(&self) -> Vec<String> {
        let mut names: Vec<String> =
            self.partial_matches.borrow().iter().cloned().collect();
        names.sort();
        names
    }

    /// Add a global interceptor. `front` inserts at the head. Returns a guard
    /// that removes it on drop. Deduplicates by pointer identity.
    pub fn add_interceptor(&self, interceptor: Interceptor, front: bool) -> InterceptorGuard {
        let mut interceptors = self.interceptors.borrow_mut();
        if !interceptors.iter().any(|i| Rc::ptr_eq(i, &interceptor)) {
            if front {
                interceptors.insert(0, interceptor.clone());
            } else {
                interceptors.push(interceptor.clone());
            }
        }
        InterceptorGuard {
            interceptors: self.interceptors.clone(),
            interceptor,
        }
    }

    /// Register a handler for `command`. Panics if the command is not in the
    /// keymap (mirrors the TS library throwing).
    pub fn on(&self, command: &str, handler: impl Fn(&ShortcutEvent<'_>) + 'static) {
        if !self.commands.borrow().contains_key(command) {
            panic!("{}", ShortcutsError::CommandNotRegistered(command.to_string()));
        }
        self.command_listeners
            .borrow_mut()
            .entry(command.to_string())
            .or_default()
            .push(Rc::new(handler));
    }

    /// Register an ad-hoc listener that fires when `shortcut` fully matches,
    /// bypassing contexts/commands. If `once`, the listener auto-removes after
    /// firing.
    pub fn on_shortcut_key_match(
        &self,
        shortcut: &str,
        handler: impl Fn(&ShortcutEvent<'_>) + 'static,
        event_type: KeyEventType,
        once: bool,
    ) -> Result<(), ShortcutsError> {
        let s = Rc::new(Shortcut::from(shortcut, &self.registry)?);
        self.adhoc_listeners.borrow_mut().push(AdhocListener {
            shortcut: s,
            handler: Box::new(handler),
            once,
            event_type,
        });
        Ok(())
    }

    /// Register a handler fired when the partial-match command-name set
    /// changes. Receives the sorted command names.
    pub fn on_partial_change(&self, handler: impl Fn(&[String]) + 'static) {
        self.partial_change_listeners
            .borrow_mut()
            .push(Box::new(handler));
    }

    pub fn pause(&self) {
        self.paused.set(true);
    }
    pub fn resume(&self) {
        self.paused.set(false);
    }
    pub fn is_paused(&self) -> bool {
        self.paused.get()
    }

    /// Depth-first walk of `context`'s fallbacks (cycle-guarded), concatenating
    /// each context's commands, deduped keeping first occurrence.
    pub fn commands_of(&self, context: &str) -> Vec<String> {
        let ctxs = self.contexts.borrow();
        let mut visited = HashSet::new();
        let mut ordered = Vec::new();
        fn walk(
            name: &str,
            ctxs: &HashMap<String, FullContext>,
            visited: &mut HashSet<String>,
            ordered: &mut Vec<String>,
        ) {
            if !visited.insert(name.to_string()) {
                return;
            }
            if let Some(ctx) = ctxs.get(name) {
                for cmd in &ctx.commands {
                    if !ordered.contains(cmd) {
                        ordered.push(cmd.clone());
                    }
                }
                for fb in &ctx.fallbacks {
                    walk(fb, ctxs, visited, ordered);
                }
            }
        }
        walk(context, &ctxs, &mut visited, &mut ordered);
        ordered
    }

    /// Whether `shortcut` is already bound in `context` (or all commands if
    /// `context` is `None`).
    pub fn is_shortcut_occupied(
        &self,
        shortcut: &str,
        context: Option<&str>,
    ) -> Result<bool, ShortcutsError> {
        let target = Shortcut::from(shortcut, &self.registry)?;
        let commands = match context {
            Some(c) => self.commands_of(c),
            None => self.commands.borrow().keys().cloned().collect(),
        };
        let cmds = self.commands.borrow();
        Ok(commands.iter().any(|name| {
            cmds.get(name)
                .map(|c| c.shortcut.equals(&target))
                .unwrap_or(false)
        }))
    }

    /// Introspect a registered context.
    pub fn context(&self, name: &str) -> Option<ShortcutContext> {
        let ctxs = self.contexts.borrow();
        ctxs.get(name).map(|c| ShortcutContext {
            name: name.to_string(),
            fallbacks: c.fallbacks.clone(),
            commands: c.commands.clone(),
        })
    }

    /// The core dispatch. Returns fully-matched `(command, event, prevent_default)`
    /// triples; also runs the interceptor chain and emits to core-library
    /// command listeners registered via [`Keyboard::on`].
    pub fn fire<'a>(&'a self, event: &'a ShortcutKeyboardEvent<'a>) -> Vec<FireMatch<'a>> {
        if self.paused.get() {
            return Vec::new();
        }

        // 1. Ad-hoc listeners (bypass contexts/commands).
        self.fire_adhoc(event);

        // 2. Current context.
        let current = match self.context_stack.borrow().last().cloned() {
            Some(c) => c,
            None => return Vec::new(),
        };
        let commands = self.commands_of(&current);

        if commands.is_empty() {
            return Vec::new();
        }

        let mut partial = self.partial_matches.borrow_mut();
        let size_before = partial.len();
        let mut results: Vec<FireMatch<'a>> = Vec::new();
        let cmds = self.commands.borrow();

        for name in &commands {
            let cmd = match cmds.get(name) {
                Some(c) => c,
                None => continue,
            };
            let seg_match = cmd.shortcut.matches(event);
            if seg_match {
                if cmd.shortcut.is_full_match() {
                    partial.remove(name);
                    if cmd.event.contains(&event.event_type) {
                        let sev = ShortcutEvent::new(cmd.shortcut.clone(), event);
                        self.execute_command(&sev, name, cmd);
                        results.push(FireMatch {
                            command: name.clone(),
                            event: sev,
                            prevent_default: cmd.prevent_default,
                        });
                    }
                } else {
                    partial.insert(name.clone());
                }
            } else {
                partial.remove(name);
            }
        }
        drop(cmds);

        let changed = partial.len() != size_before;
        drop(partial);

        if changed {
            self.notify_partial_change();
        }

        results
    }

    fn fire_adhoc(&self, event: &ShortcutKeyboardEvent<'_>) {
        let mut listeners = self.adhoc_listeners.borrow_mut();
        let mut to_remove = Vec::new();
        for (i, l) in listeners.iter().enumerate() {
            if l.event_type != event.event_type {
                continue;
            }
            if l.shortcut.matches(event) && l.shortcut.is_full_match() {
                let sev = ShortcutEvent::new(l.shortcut.clone(), event);
                (l.handler)(&sev);
                if l.once {
                    to_remove.push(i);
                }
            }
        }
        for i in to_remove.into_iter().rev() {
            listeners.remove(i);
        }
    }

    fn notify_partial_change(&self) {
        let mut names: Vec<String> = self.partial_matches.borrow().iter().cloned().collect();
        names.sort();
        let listeners = self.partial_change_listeners.borrow();
        for l in listeners.iter() {
            l(&names);
        }
    }

    fn execute_command(&self, event: &ShortcutEvent<'_>, name: &str, cmd: &ParsedCommand) {
        let listeners = self.command_listeners.borrow();
        let handlers = listeners.get(name).cloned().unwrap_or_default();
        drop(listeners);

        // Build the interceptor chain: [global..., per-command...] right-folded,
        // terminal handler at the bottom emits to command listeners.
        let global = self.interceptors.borrow().clone();
        let chain: Vec<Interceptor> = global
            .into_iter()
            .chain(cmd.interceptors.iter().cloned())
            .collect();

        let terminal = move |sev: &ShortcutEvent<'_>| {
            for h in &handlers {
                h(sev);
            }
        };

        // Right-fold into nested closures. Each level captures `next` and calls
        // the interceptor with a continuation that invokes `next`.
        let runner = chain.iter().rev().fold(
            Box::new(terminal) as Box<dyn Fn(&ShortcutEvent<'_>) + '_>,
            |next, cur| {
                let next: Box<dyn Fn(&ShortcutEvent<'_>) + '_> = next;
                Box::new(move |sev: &ShortcutEvent<'_>| {
                    cur(sev, &|e: &ShortcutEvent<'_>| next(e));
                })
            },
        );

        runner(event);
    }

    /// Reset all in-progress sequences and clear partial tracking.
    pub fn reset_all(&self) {
        let cmds = self.commands.borrow();
        for (_, c) in cmds.iter() {
            c.shortcut.reset();
        }
        self.partial_matches.borrow_mut().clear();
        self.notify_partial_change();
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new()
    }
}
