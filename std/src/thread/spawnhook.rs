use crate::cell::Cell;
use crate::sync::Arc;
use crate::thread::Thread;

// A thread local linked list of spawn hooks.
crate::thread_local! {
    static SPAWN_HOOKS: Cell<SpawnHooks> = const { Cell::new(SpawnHooks { first: None }) };
}

#[derive(Default, Clone)]
struct SpawnHooks {
    first: Option<Arc<SpawnHook>>,
}

// Manually implement drop to prevent deep recursion when dropping linked Arc list.
impl Drop for SpawnHooks {
    fn drop(&mut self) {
        let mut next = self.first.take();
        while let Some(SpawnHook { hook, next: n }) = next.and_then(|n| Arc::into_inner(n)) {
            drop(hook);
            next = n;
        }
    }
}

struct SpawnHook {
    hook: Box<dyn Sync + Fn(&Thread) -> Box<dyn Send + FnOnce()>>,
    next: Option<Arc<SpawnHook>>,
}

/// Registers a function to run for every newly thread spawned.
///
/// The hook is executed in the parent thread, and returns a function
/// that will be executed in the new thread.
///
/// The hook is called with the `Thread` handle for the new thread.
///
/// The hook will only be added for the current thread and is inherited by the threads it spawns.
/// In other words, adding a hook has no effect on already running threads (other than the current
/// thread) and the threads they might spawn in the future.
///
/// Hooks can only be added, not removed.
///
/// The hooks will run in order, starting with the most recently added.
///
/// # Usage
///
/// ```
/// #![feature(thread_spawn_hook)]
///
/// std::thread::add_spawn_hook(|_| {
///     ..; // This will run in the parent (spawning) thread.
///     move || {
///         ..; // This will run it the child (spawned) thread.
///     }
/// });
/// ```
///
/// # Example
///
/// A spawn hook can be used to "inherit" a thread local from the parent thread:
///
/// ```
/// #![feature(thread_spawn_hook)]
///
/// use std::cell::Cell;
///
/// thread_local! {
///     static X: Cell<u32> = Cell::new(0);
/// }
///
/// // This needs to be done once in the main thread before spawning any threads.
/// std::thread::add_spawn_hook(|_| {
///     // Get the value of X in the spawning thread.
///     let value = X.get();
///     // Set the value of X in the newly spawned thread.
///     move || X.set(value)
/// });
///
/// X.set(123);
///
/// std::thread::spawn(|| {
///     assert_eq!(X.get(), 123);
/// }).join().unwrap();
/// ```
#[unstable(feature = "thread_spawn_hook", issue = "none")]
pub fn add_spawn_hook<F, G>(hook: F)
where
    F: 'static + Sync + Fn(&Thread) -> G,
    G: 'static + Send + FnOnce(),
{
    SPAWN_HOOKS.with(|h| {
        let mut hooks = h.take();
        hooks.first = Some(Arc::new(SpawnHook {
            hook: Box::new(move |thread| Box::new(hook(thread))),
            next: hooks.first.take(),
        }));
        h.set(hooks);
    });
}

/// Runs all the spawn hooks.
///
/// Called on the parent thread.
///
/// Returns the functions to be called on the newly spawned thread.
pub(super) fn run_spawn_hooks(thread: &Thread) -> SpawnHookResults {
    // Get a snapshot of the spawn hooks.
    // (Increments the refcount to the first node.)
    let hooks = SPAWN_HOOKS.with(|hooks| {
        let snapshot = hooks.take();
        hooks.set(snapshot.clone());
        snapshot
    });
    // Iterate over the hooks, run them, and collect the results in a vector.
    let mut next: &Option<Arc<SpawnHook>> = &hooks.first;
    let mut to_run = Vec::new();
    while let Some(hook) = next {
        to_run.push((hook.hook)(thread));
        next = &hook.next;
    }
    // Pass on the snapshot of the hooks and the results to the new thread,
    // which will then run SpawnHookResults::run().
    SpawnHookResults { hooks, to_run }
}

/// The results of running the spawn hooks.
///
/// This struct is sent to the new thread.
/// It contains the inherited hooks and the closures to be run.
pub(super) struct SpawnHookResults {
    hooks: SpawnHooks,
    to_run: Vec<Box<dyn FnOnce() + Send>>,
}

impl SpawnHookResults {
    // This is run on the newly spawned thread, directly at the start.
    pub(super) fn run(self) {
        SPAWN_HOOKS.set(self.hooks);
        for run in self.to_run {
            run();
        }
    }
}
