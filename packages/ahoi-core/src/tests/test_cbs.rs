use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

// ── Callback ──────────────────────────────────────────────────────────────

#[test]
fn test_callback_call() {
    make_sphere(None, || {
        let cb = Callback::new(|x: i32| x * 2);
        assert_eq!(cb.call(5), 10);
        assert_eq!(cb.call(-3), -6);
    });
}

#[test]
fn test_callback_is_copy() {
    make_sphere(None, || {
        let cb = Callback::new(|s: String| s.len());
        let cb2 = cb; // Copy
        assert_eq!(cb.call("hello".to_string()), 5);
        assert_eq!(cb2.call("world!".to_string()), 6);
    });
}

#[test]
fn test_callback_captures_reactive_state() {
    make_sphere(None, || {
        let stock = Stock::new(0u32);
        let cb = Callback::new(move |_: ()| *stock.peek());
        assert_eq!(cb.call(()), 0);
        batch(|| stock.set(42));
        assert_eq!(cb.call(()), 42);
    });
}

#[test]
fn test_callback_runner_is_batched() {
    // The runner executes inside a batch(), so mutations propagate reactively.
    let run_count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(0i32);
        let rc = run_count.clone();
        let _effect = Effect::new(move || {
            let _ = *stock.read();
            rc.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(run_count.load(Ordering::SeqCst), 1);

        let cb = Callback::new(move |val: i32| stock.set(val));
        cb.call(99);

        assert_eq!(*stock.peek(), 99);
        assert_eq!(run_count.load(Ordering::SeqCst), 2);
    });
}

#[test]
#[should_panic(expected = "Callback is called after sphere cleared")]
fn test_callback_panics_after_sphere_cleared() {
    let (sid, cb) = make_sphere(None, || Callback::new(|x: i32| x));
    clear_sphere(sid);
    cb.call(1);
}

// ── Action ────────────────────────────────────────────────────────────────

#[test]
fn test_action_is_copy() {
    make_sphere(None, || {
        let action: Action<i32, i32> = Action::new(|x| async move { x * 2 });
        let action2 = action; // Copy
        assert!(!action.pending());
        assert!(!action2.pending());
    });
}

#[test]
fn test_action_initial_state() {
    make_sphere(None, || {
        let action: Action<i32, i32> = Action::new(|x| async move { x * 2 });
        assert!(!action.pending());
        assert!(!action.ready());
        assert!(action.value().read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_pending_immediately_after_call() {
    run_local(async {
        let action: Action<i32, i32> =
            make_sphere(None, || Action::new(|x| async move { x * 2 })).1;

        let _rx = action.call(5);

        // state.set(Pending) runs synchronously inside the callback's batch.
        assert!(action.pending());
        assert!(!action.ready());
        assert!(action.value().read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_completes() {
    run_local(async {
        let action: Action<i32, i32> =
            make_sphere(None, || Action::new(|x| async move { x * 2 })).1;

        let rx = action.call(7);
        rx.await.ok();

        assert!(!action.pending());
        assert!(action.ready());
        assert_eq!(*action.value().read(), Some(14));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_multiple_sequential_calls() {
    run_local(async {
        let action: Action<i32, i32> =
            make_sphere(None, || Action::new(|x| async move { x * 2 })).1;

        let rx1 = action.call(1);
        rx1.await.ok();
        assert_eq!(*action.value().read(), Some(2));

        let rx2 = action.call(5);
        rx2.await.ok();
        assert_eq!(*action.value().read(), Some(10));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_reactive_state_updates() {
    // An Effect reading pending() re-runs on Unset → Pending (sync, on call)
    // and Pending → Ready (async, on complete).
    let run_count = Arc::new(AtomicU32::new(0));
    run_local(async {
        let rc = run_count.clone();
        let action: Action<i32, i32> = make_sphere(None, || {
            let action = Action::new(|x| async move { x * 2 });
            let _effect = Effect::new(move || {
                let _ = action.pending();
                rc.fetch_add(1, Ordering::SeqCst);
            });
            action
        })
        .1;
        assert_eq!(run_count.load(Ordering::SeqCst), 1); // initial

        let rx = action.call(5);
        assert_eq!(run_count.load(Ordering::SeqCst), 2); // Pending

        rx.await.ok();
        assert_eq!(run_count.load(Ordering::SeqCst), 3); // Ready
        assert_eq!(*action.value().read(), Some(10));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_value_memo() {
    run_local(async {
        let value_memo = make_sphere(None, || {
            let action: Action<i32, i32> = Action::new(|x| async move { x * 2 });
            let action_value = action.value();
            let value_memo = Memo::new(move || action_value.read().as_ref().copied());
            (action, value_memo)
        });
        let (action, value_memo) = value_memo.1;

        assert_eq!(*value_memo.peek(), None);

        let rx = action.call(5);
        rx.await.ok();

        assert_eq!(*value_memo.peek(), Some(10));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_new_call_aborts_previous() {
    run_local(async {
        let action: Action<i32, i32> =
            make_sphere(None, || Action::new(|x| async move { x * 2 })).1;

        let _rx1 = action.call(5); // aborted before it runs
        let rx2 = action.call(10);
        rx2.await.ok();

        assert_eq!(*action.value().read(), Some(20));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_cancel() {
    run_local(async {
        let action: Action<i32, i32> =
            make_sphere(None, || Action::new(|x| async move { x * 2 })).1;

        let _rx = action.call(5);
        assert!(action.pending());

        action.cancel();

        assert!(!action.pending());
        assert!(!action.ready());
        assert!(action.value().read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_reset() {
    run_local(async {
        let action: Action<i32, i32> =
            make_sphere(None, || Action::new(|x| async move { x * 2 })).1;

        let rx = action.call(5);
        rx.await.ok();
        assert_eq!(*action.value().read(), Some(10));

        action.reset();

        assert!(!action.pending());
        assert!(!action.ready());
        assert!(action.value().read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_action_sphere_clear_while_pending_no_panic() {
    // Clearing the sphere while a task is pending must not panic.
    run_local(async {
        let (sid, action) = make_sphere(None, || Action::<i32, i32>::new(|x| async move { x * 2 }));

        let _rx = action.call(5);
        assert!(action.pending());

        clear_sphere(sid);

        tokio::task::yield_now().await;
    });
}

// ── Resource ──────────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_initial_state() {
    run_local(async {
        let resource: Resource<i32> = make_sphere(None, || Resource::new(|| async { 42 })).1;
        assert!(resource.pending());
        assert!(!resource.ready());
        assert!(resource.read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_completes() {
    run_local(async {
        let resource: Resource<i32> = make_sphere(None, || Resource::new(|| async { 42 })).1;
        tokio::task::yield_now().await;
        assert!(resource.ready());
        assert_eq!(*resource.read(), Some(42));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_reactive() {
    // Resource re-fetches automatically when a reactive dependency changes.
    run_local(async {
        let (dep, resource) = make_sphere(None, || {
            let dep = Stock::new(1i32);
            let resource: Resource<i32> = Resource::new(move || async move {
                let v = *dep.read();
                v * 10
            });
            (dep, resource)
        })
        .1;

        tokio::task::yield_now().await;
        assert_eq!(*resource.read(), Some(10));

        batch(|| dep.set(3));
        tokio::task::yield_now().await;
        assert_eq!(*resource.read(), Some(30));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_sync_dep_reactive() {
    // dep.read() in the sync runner phase must still be tracked.
    run_local(async {
        let (dep, resource) = make_sphere(None, || {
            let dep = Stock::new(1i32);
            let resource: Resource<i32> = Resource::new(move || {
                let v = *dep.read();
                async move { v * 10 }
            });
            (dep, resource)
        })
        .1;

        tokio::task::yield_now().await;
        assert_eq!(*resource.read(), Some(10));

        batch(|| dep.set(3));
        tokio::task::yield_now().await;
        assert_eq!(*resource.read(), Some(30));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_value_memo() {
    run_local(async {
        let (dep, value_memo) = make_sphere(None, || {
            let dep = Stock::new(1i32);
            let resource: Resource<i32> = Resource::new(move || async move {
                let v = *dep.read();
                v * 10
            });
            let value_memo = Memo::new(move || resource.read().as_ref().copied());
            (dep, value_memo)
        })
        .1;

        assert_eq!(*value_memo.peek(), None);

        tokio::task::yield_now().await;
        assert_eq!(*value_memo.peek(), Some(10));

        batch(|| dep.set(5));
        tokio::task::yield_now().await;
        assert_eq!(*value_memo.peek(), Some(50));
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_cancel() {
    run_local(async {
        let resource: Resource<i32> = make_sphere(None, || Resource::new(|| async { 42 })).1;
        assert!(resource.pending());

        resource.cancel();

        assert!(!resource.pending());
        assert!(!resource.ready());
        assert!(resource.read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_reset() {
    run_local(async {
        let resource: Resource<i32> = make_sphere(None, || Resource::new(|| async { 42 })).1;
        tokio::task::yield_now().await;
        assert_eq!(*resource.read(), Some(42));

        resource.reset();

        assert!(!resource.pending());
        assert!(!resource.ready());
        assert!(resource.read().is_none());
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_resource_sphere_clear_while_pending_no_panic() {
    run_local(async {
        let (sid, resource) = make_sphere(None, || Resource::<i32>::new(|| async { 42 }));
        assert!(resource.pending());

        clear_sphere(sid);

        tokio::task::yield_now().await;
    });
}

// ── spawn_batch sphere context ──────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn test_spawn_batch_in_work_preserves_sphere_context() {
    // Regression: a future spawned by spawn_batch inside work() is first polled on
    // a *later* tick, after work()'s synchronous span has ended and the working
    // sphere was popped. spawn_batch captures the current sphere id, and the
    // spawned body re-establishes work(), so use_context() still resolves.
    // Previously use_context() here panicked ("use context in sphere or work").
    use std::sync::atomic::AtomicI32;

    #[derive(Clone, Copy)]
    struct Ctx(i32);

    run_local(async {
        let captured = Arc::new(AtomicI32::new(-1));

        // Build a sphere that provides Ctx(7).
        let sid = make_sphere(None, || provide_context(Ctx(7))).0;

        // Inside work(), fire-and-forget a task that reads the context. The task
        // is only polled after work() returns (next tick), so without the captured
        // sphere it would see an empty working stack and panic in use_context().
        let cap = captured.clone();
        batch_with_sphere(sid, || {
            spawn_batch(async move {
                let Ctx(v) = use_context::<Ctx>().expect("Ctx must be in scope");
                cap.store(v, Ordering::SeqCst);
            })
            .detach();
        });

        tokio::task::yield_now().await;

        assert_eq!(captured.load(Ordering::SeqCst), 7);
    });
}
