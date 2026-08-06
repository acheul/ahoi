use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

// ── Depth-ordered propagation ─────────────────────────────────────────────

#[test]
fn test_chain_propagation() {
    make_sphere(None, || {
        let s = Stock::new(1i32);
        let m1 = Memo::new(move || *s.read() + 1);
        let m2 = Memo::new(move || *m1.read() + 1);
        let m3 = Memo::new(move || *m2.read() + 1);

        batch(|| s.set(10));

        assert_eq!(*m3.peek(), 13);
    });
}

#[test]
fn test_diamond_propagation() {
    // B cites both S and A's stock. After S changes, A must run before B so
    // B sees A's fresh value — and B must run exactly once per batch.
    let b_runs = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let s = Stock::new(1i32);
        let a = Memo::new(move || *s.read() * 2);
        let rc = b_runs.clone();
        let b = Memo::new(move || {
            rc.fetch_add(1, Ordering::SeqCst);
            *s.read() + *a.read()
        });

        assert_eq!(*b.peek(), 3);
        assert_eq!(b_runs.load(Ordering::SeqCst), 1);

        batch(|| s.set(10));

        assert_eq!(*a.peek(), 20);
        assert_eq!(*b.peek(), 30);
        assert_eq!(b_runs.load(Ordering::SeqCst), 2);
    });
}

// ── Pull-on-read: dynamic dependencies ────────────────────────────────────

#[test]
fn test_dynamic_dep_branch_flip_sees_fresh_deep_chain() {
    // c starts reading only `flag` (depth 0). In one batch, flip the flag AND
    // mutate the deep chain's source: c may pop before the chain has run, but
    // the pull on `m3.read()` must settle m1 -> m2 -> m3 first.
    make_sphere(None, || {
        let s0 = Stock::new(1i32);
        let m1 = Memo::new(move || *s0.read() + 1);
        let m2 = Memo::new(move || *m1.read() + 1);
        let m3 = Memo::new(move || *m2.read() + 1);
        let flag = Stock::new(false);
        let c = Memo::new(move || if *flag.read() { *m3.read() } else { 0 });

        assert_eq!(*c.peek(), 0);

        batch(|| {
            flag.set(true);
            s0.set(10);
        });

        assert_eq!(*m3.peek(), 13);
        assert_eq!(*c.peek(), 13);
    });
}

#[test]
fn test_dynamic_dep_stale_downstream_depth() {
    // Batch 1 raises c's depth (it starts reading deep2) without changing c's
    // value, so d never re-runs and keeps a stale-low depth. In batch 2, d may
    // pop before the deep chain; the pull on `c.read()` must fix the order.
    make_sphere(None, || {
        let s = Stock::new(1i32);
        let deep1 = Memo::new(move || *s.read() + 1);
        let deep2 = Memo::new(move || *deep1.read() + 1);
        let flag = Stock::new(false);
        // else-branch constant equals deep2's current value, so the flip below
        // changes c's dependencies but not its value.
        let c = Memo::new(move || if *flag.read() { *deep2.read() } else { 3 });
        let d = Memo::new(move || *s.read() + *c.read());

        assert_eq!(*d.peek(), 4);

        batch(|| flag.set(true));
        assert_eq!(*d.peek(), 4);

        batch(|| s.set(10));

        assert_eq!(*c.peek(), 12);
        assert_eq!(*d.peek(), 22);
    });
}

#[test]
fn test_check_marked_citer_skips_run_when_producer_unchanged() {
    // parity recomputes but writes nothing (3 % 2 == 1 % 2), so the
    // check-marked downstream must be skipped without running.
    let down_runs = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let s = Stock::new(1i32);
        let parity = Memo::new(move || *s.read() % 2);
        let rc = down_runs.clone();
        let down = Memo::new(move || {
            rc.fetch_add(1, Ordering::SeqCst);
            *parity.read() * 10
        });

        assert_eq!(*down.peek(), 10);
        assert_eq!(down_runs.load(Ordering::SeqCst), 1);

        batch(|| s.set(3));

        assert_eq!(*down.peek(), 10);
        assert_eq!(down_runs.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn test_batch_body_read_sees_pre_batch_value() {
    // Reads in the batch body do not pull: they see pre-batch values by design
    // (propagation runs once, after the body).
    make_sphere(None, || {
        let s = Stock::new(1i32);
        let m = Memo::new(move || *s.read() * 2);

        batch(|| {
            s.set(10);
            assert_eq!(*m.read(), 2);
        });

        assert_eq!(*m.peek(), 20);
    });
}

#[test]
#[should_panic(expected = "cite cycle detected")]
fn test_cite_cycle_detected() {
    // effect writes `s`, which re-dirties `m`, whose write re-dirties the
    // already-run effect: a genuine cite cycle.
    make_sphere(None, || {
        let s = Stock::new(0i32);
        let m = Memo::new(move || *s.read() + 1);
        let _effect = Effect::new(move || {
            let v = *m.read();
            if v < 10 {
                s.set(v);
            }
        });
    });
}

#[test]
fn test_multi_hop_diamond_propagation() {
    // S -> X -> A, while B cites both S and A's stock. B is queued in the very
    // first collection (it cites S directly), but must still pop only after X
    // and A have run and written their stocks.
    make_sphere(None, || {
        let s = Stock::new(1i32);
        let x = Memo::new(move || *s.read() + 1);
        let a = Memo::new(move || *x.read() * 10);
        let b = Memo::new(move || *s.read() + *a.read());

        assert_eq!(*b.peek(), 21);

        batch(|| s.set(5));

        assert_eq!(*a.peek(), 60);
        assert_eq!(*b.peek(), 65);
    });
}

#[test]
fn test_derived_from_memo_carries_pull_link() {
    // A stock derived from a memo's backing stock must carry the memo's
    // `associated_citer_id`, so reading it mid-propagation pulls the memo
    // fresh first (and `update_depth` keeps the ordering).
    //
    // Setup forces the effect's cite-rel on `src` to register BEFORE the
    // memo's, so on a depth tie the effect would pop first: without the
    // carried pull link its read of `derived` would observe the stale
    // pre-batch memo value (a glitch: seen = [10, 50] instead of [50]).
    use std::{cell::RefCell, rc::Rc, sync::Mutex};

    let seen = Arc::new(Mutex::new(Vec::new()));
    make_sphere(None, || {
        let src = Stock::new(1i32);

        // The effect is created first; `derived` is injected afterwards.
        let slot: Rc<RefCell<Option<ReadStock<i32>>>> = Rc::new(RefCell::new(None));
        let slot_ = slot.clone();
        let seen_ = seen.clone();
        let _effect = Effect::new(move || {
            let _ = src.read(); // direct dep: marked as soon as `src` mutates
            if let Some(derived) = &*slot_.borrow() {
                seen_.lock().unwrap().push(*derived.read());
            }
        });

        let memo = Memo::new(move || (*src.read() * 10, ()));
        let derived = memo
            .into_read_stock()
            .derive(
                0,
                GetNext::new(|t: &(i32, ())| &t.0, |t: &mut (i32, ())| &mut t.0),
            )
            .pool();
        slot.borrow_mut().replace(derived);

        batch(|| *src.write() = 5);
    });
    assert_eq!(&*seen.lock().unwrap(), &[50]);
}
