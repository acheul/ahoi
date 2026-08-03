use super::*;
use futures_util::stream::{AbortHandle, Abortable};

#[derive(Debug, Clone)]
pub struct TaskHandle(AbortHandle);

impl TaskHandle {
    pub fn abort(&self) {
        self.0.abort();
    }

    /// Detaches the task so it keeps running after this handle is dropped.
    ///
    /// Use this for fire-and-forget spawns from a synchronous handler, where the
    /// handle would otherwise drop (and abort the task) before the executor ever
    /// polls it. After detaching the task can no longer be aborted.
    pub fn detach(self) {
        // Skip the abort-on-drop in `Drop` while letting the `Abortable` future
        // (and its registration) live on inside the executor. We must NOT
        // `mem::forget(self)`: that would also leak the `AbortHandle`'s `Arc`,
        // keeping its shared abort-state allocation alive forever. Instead, take
        // the `AbortHandle` out and drop it normally (it has no custom `Drop`, so
        // this only decrements the `Arc` — no abort is triggered).
        let md = std::mem::ManuallyDrop::new(self);
        // SAFETY: `md` is never used or dropped again; we move the field out and
        // let it drop, bypassing `TaskHandle`'s abort-on-drop.
        let abort_handle = unsafe { std::ptr::read(&md.0) };
        drop(abort_handle);
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

// Enable batch wrapper for a Future
struct BatchedFuture<F> {
    fut: F,
    sphere: Option<SphereId>,
}

impl<F: Future> Future for BatchedFuture<F> {
    type Output = F::Output;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        runtime::propagation::batch_with_sphere(self.sphere, || {
            unsafe { self.map_unchecked_mut(|s| &mut s.fut) }.poll(cx)
        })
    }
}

/// Spawns a future as a detached task and returns a [`TaskHandle`] that can abort it.
/// * The internal future callback will be wrapped in batch automatically
///
/// # Important: the returned [`TaskHandle`] aborts the task on `Drop`
/// If the handle drops before the task has a chance to run, the task is aborted
/// and **never runs**. Note that binding to a local variable does *not* help when
/// spawning from a synchronous handler: the variable drops when the handler
/// returns, which is *before* the executor first polls the spawned task.
/// ```ignore
/// spawn_batch(async { ... });         // dropped at once -> aborted, never runs
/// let _ = spawn_batch(async { ... });  // same: `_` drops immediately
/// let _h = spawn_batch(async { ... }); // dropped when the handler returns -> aborted
/// ```
/// To keep abort control, store the handle somewhere that outlives the task
/// (e.g. a signal/state, as the action & resource runners do). For fire-and-forget,
/// [`TaskHandle::detach`] lets the task run without keeping the handle:
/// ```ignore
/// spawn_batch(async { ... }).detach(); // runs to completion; no longer abortable
/// ```
///
/// # Platform notes
/// - **wasm32**: uses `spawn_local`, always safe.
/// - **non-wasm**: uses `tokio::task::spawn_local`; requires a single-threaded Tokio
///   runtime with an active [`tokio::task::LocalSet`]
///   (e.g. `LocalSet::new().block_on(&rt, async { ... })`).
///   All state is stored in `thread_local!`, so `Send` is intentionally not required.
pub fn spawn_batch(fut_task: impl Future<Output = ()> + 'static) -> TaskHandle {
    let cur_sphere = current_sphere_id();

    let (abort_handle, abort_registration) = AbortHandle::new_pair();
    let abortable_fut = Abortable::new(fut_task, abort_registration);

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(BatchedFuture {
        sphere: cur_sphere,
        fut: async move {
            let _ = abortable_fut.await;
        },
    });

    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::spawn_local(BatchedFuture {
        sphere: cur_sphere,
        fut: async move {
            let _ = abortable_fut.await;
        },
    });

    return TaskHandle(abort_handle);
}

/// # Helper to run a tokio single thread
// Runs `f` inside a single-threaded tokio runtime with a LocalSet active.
// spawn_local (used by ahoi::spawn on non-wasm) requires a LocalSet context.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_local<F: std::future::Future<Output = ()>>(f: F) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    tokio::task::LocalSet::new().block_on(&rt, f);
}
