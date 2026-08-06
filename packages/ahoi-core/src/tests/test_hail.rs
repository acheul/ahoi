use super::*;
use std::any::Any;
use std::sync::{Arc, Mutex};

// ── Shared test scaffolding ────────────────────────────────────────────────

/// Generic hail converter used across the hail tests: the hail value is the
/// raw value wrapped in `Option` (so "absent" maps to `None`).
struct Opt;
impl<T: Clone + 'static> HailConverter<T> for Opt {
    type HailValue = Option<T>;
    const NONE: Option<T> = None;
    fn from_raw_value(raw_value: &T) -> Option<T> {
        Some(raw_value.clone())
    }
    fn into_raw_value(hail_value: Option<T>) -> T {
        hail_value.expect("write_hail(None)")
    }
}

/// Records dispatched hail values. Values within a single dispatch are sorted
/// so multi-hail assertions are order-independent; separate dispatches stay
/// in chronological order.
struct LogDispatcher<V>(Arc<Mutex<Vec<V>>>);
impl<V: Clone + Ord + 'static> HailDispatcher for LogDispatcher<V> {
    fn dispatch_hails(&self, hails: IntMap<SphereId, Box<dyn Any>>) {
        let mut vals: Vec<V> = hails
            .into_iter()
            .map(|(_, v)| *v.downcast::<V>().unwrap())
            .collect();
        vals.sort();
        self.0.lock().unwrap().extend(vals);
    }
}

/// Install a logging dispatcher of value type `V` and return the shared log.
fn logging<V: Clone + Ord + 'static>() -> Arc<Mutex<Vec<V>>> {
    let log = Arc::new(Mutex::new(Vec::new()));
    set_local_hail_dispatcher(LogDispatcher(log.clone()));
    log
}

#[derive(Stock, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Duo {
    a: u32,
    b: u32,
}

// ── Structural ref-hail on a Vec item ─────────────────────────────────────

#[test]
fn test_hail_structural() {
    let log = logging::<Option<u32>>();

    make_sphere(None, || {
        let items = Stock::new(vec![0u32, 1u32, 2u32]);
        let item1 = items.get(1usize).pool();
        let item2 = items.get(2usize).pool();

        let initial = item1.set_read_hail::<Opt>();
        assert_eq!(initial, Some(1u32));

        // Structural shift: item1 ends up at value 0, hail fires once.
        batch(|| {
            items.write().push(4u32);
            *item1.write().unwrap() = 10;
            items.write().insert(0, 100u32);
        });

        // item2 write does not touch item1's path; item1 → 20.
        batch(|| {
            *item2.write().unwrap() = 10;
            *item1.write().unwrap() = 20;
        });
        assert_eq!(*log.lock().unwrap(), vec![Some(0u32), Some(20u32)]);

        // Clearing the vec makes item1 OOB → None.
        batch(|| items.write().clear());
        assert_eq!(*log.lock().unwrap(), vec![Some(0u32), Some(20u32), None]);
    });
}

// ── Memo dedup suppresses unchanged hails ─────────────────────────────────

#[test]
fn test_hail_equality_suppresses() {
    let log = logging::<Option<bool>>();

    make_sphere(None, || {
        let stock = Stock::new(5i32);
        let memo = Memo::new(move || *stock.read() > 0);
        let value = memo.set_read_hail::<Opt>();
        assert_eq!(value, Some(true));

        batch(|| *stock.write() = 10); // memo stays true → suppressed
        assert!(log.lock().unwrap().is_empty());

        batch(|| *stock.write() = -1); // memo flips → fires
        assert_eq!(*log.lock().unwrap(), vec![Some(false)]);
    });
}

// ── Deep-path hail: direct fires, sibling silent, root fires ──────────────

#[test]
fn test_hail_deep_path() {
    let log = logging::<Option<u32>>();

    make_sphere(None, || {
        let stock = Stock::new(Duo { a: 0, b: 0 });
        stock.a().set_read_hail::<Opt>();

        batch(|| *stock.a().write() = 1); // direct → fires
        assert_eq!(*log.lock().unwrap(), vec![Some(1u32)]);

        batch(|| *stock.b().write() = 99); // sibling → silent
        assert_eq!(*log.lock().unwrap(), vec![Some(1u32)]);

        batch(|| *stock.write() = Duo { a: 2, b: 0 }); // root → fires
        assert_eq!(*log.lock().unwrap(), vec![Some(1u32), Some(2u32)]);
    });
}

// ── Root hail fires on child-path mutation ────────────────────────────────

#[test]
fn test_hail_root_fires_on_child_mutation() {
    let log = logging::<Option<Duo>>();

    make_sphere(None, || {
        let stock = Stock::new(Duo { a: 0, b: 0 });
        stock.set_read_hail::<Opt>();

        batch(|| *stock.a().write() = 7); // child of root → fires
        assert_eq!(*log.lock().unwrap(), vec![Some(Duo { a: 7, b: 0 })]);

        batch(|| *stock.b().write() = 9); // also a child of root → fires
        assert_eq!(log.lock().unwrap().len(), 2);
    });
}

// ── Two hails in one batch are coalesced into a single dispatch ───────────

#[test]
fn test_hail_two_hails_coalesced() {
    // Regression for batch hail coalescing: two hails (in two spheres) on the
    // same root stock must be delivered in ONE dispatch, so the dispatcher can
    // wrap them in a single JS batch. The sort inside LogDispatcher only yields
    // a deterministic order if both arrive together.
    let log = logging::<Option<u32>>();

    let (_, stock) = make_sphere(None, || {
        let stock = Stock::new(Duo { a: 0, b: 0 });
        stock.a().set_read_hail::<Opt>();
        stock
    });
    make_sphere(None, || {
        stock.b().set_read_hail::<Opt>();
    });

    batch(|| {
        *stock.a().write() = 5; // deep dirty [a]
        *stock.write() = Duo { a: 10, b: 20 }; // then whole root
    });

    assert_eq!(*log.lock().unwrap(), vec![Some(10u32), Some(20u32)]);
}

// ── Clearing the sphere removes its hail ──────────────────────────────────

#[test]
fn test_hail_cleared_with_sphere() {
    let log = logging::<Option<u32>>();

    let (sid, stock) = make_sphere(None, || {
        let stock = Stock::new(0u32);
        stock.set_read_hail::<Opt>();
        stock
    });

    batch(|| stock.set(1));
    assert_eq!(*log.lock().unwrap(), vec![Some(1u32)]);

    clear_sphere(sid);

    // Stock is gone with the sphere; further writes are no-ops, no hail.
    batch(|| {
        let _ = stock.try_set(2);
    });
    assert_eq!(*log.lock().unwrap(), vec![Some(1u32)]);
}

// ── Hail on a non-pooled derived stock ────────────────────────────────────

#[test]
fn test_hail_on_non_pooled_derive() {
    // set_hail / set_read_hail no longer require a pooled stock: a derived
    // stock carrying a `ChainedPipe` pipeline can register an hail directly,
    // and both the hail-citer and the write callback read/write through it.
    let log = logging::<Option<u32>>();

    let (sid, stock) = make_sphere(None, || {
        let stock = Stock::new(Duo { a: 0, b: 0 });
        // No pooling — hail registered straight on the derived `a` stock.
        stock.a().set_hail::<Opt>();
        stock
    });

    batch(|| *stock.a().write() = 1); // mutation at [a] → fires
    assert_eq!(*log.lock().unwrap(), vec![Some(1u32)]);

    batch(|| *stock.b().write() = 9); // sibling → silent
    assert_eq!(*log.lock().unwrap(), vec![Some(1u32)]);

    // write_hail drives the value through the non-pooled getter;
    write_hail(sid, Some(42u32));
    assert_eq!(*stock.a().peek(), 42);
    assert_eq!(*log.lock().unwrap(), vec![Some(1u32), Some(42u32)]);
}
