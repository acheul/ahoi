use crate::runtime::propagation::batch;
use crate::*;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

mod test_cbs;
mod test_hail;
mod test_location;
mod test_propagation;

// ── Pool borrow semantics ──────────────────────────────────────────────────

#[test]
fn test_pool_independent_slots() {
    // Mut-borrowing two different slots at once must not panic.
    make_sphere(None, || {
        let a = Stock::new("abc".to_owned());
        let b = Stock::new("ABC".to_owned());
        batch(|| {
            let mut x = a.write();
            let mut y = b.write();
            x.push_str("def");
            y.push_str("DEF");
        });
        assert_eq!(&*a.peek(), "abcdef");
        assert_eq!(&*b.peek(), "ABCDEF");
    });
}

#[test]
#[should_panic]
#[cfg_attr(miri, ignore)]
fn test_pool_double_borrow_panics() {
    // A second borrow of the same slot while a RefMut is alive must panic.
    make_sphere(None, || {
        let a = Stock::new("abc".to_owned());
        batch(|| {
            let _m = a.write();
            let _r = a.peek();
        });
    });
}

// ── Stock ──────────────────────────────────────────────────────────────────

#[test]
fn test_stock_read_write() {
    make_sphere(None, || {
        let stock = Stock::new(10i32);
        assert_eq!(*stock.peek(), 10);
        batch(|| *stock.write() = 42);
        assert_eq!(*stock.peek(), 42);
    });
}

// ── Memo ──────────────────────────────────────────────────────────────────

#[test]
fn test_memo_initial_value() {
    make_sphere(None, || {
        let stock = Stock::new(21i32);
        let memo = Memo::new(move || *stock.read() * 2);
        assert_eq!(*memo.peek(), 42);
    });
}

#[test]
fn test_memo_reactive_to_stock() {
    make_sphere(None, || {
        let stock = Stock::new(1i32);
        let memo = Memo::new(move || *stock.read() + 100);
        assert_eq!(*memo.peek(), 101);
        batch(|| *stock.write() = 9);
        assert_eq!(*memo.peek(), 109);
    });
}

#[test]
fn test_memo_equality_short_circuit() {
    // When the computed value does not change, downstream is not re-triggered.
    let count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(5i32);
        let memo = Memo::new(move || *stock.read() > 0);
        let c = count.clone();
        let _effect = Effect::new(move || {
            let _ = memo.read();
            c.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);

        batch(|| *stock.write() = 10); // stays true → no re-run
        assert_eq!(count.load(Ordering::SeqCst), 1);

        batch(|| *stock.write() = -1); // flips → re-run
        assert_eq!(count.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn test_chained_memos() {
    make_sphere(None, || {
        let stock = Stock::new(3i32);
        let memo1 = Memo::new(move || *stock.read() * 10);
        let memo2 = Memo::new(move || *memo1.read() + 1);
        assert_eq!(*memo2.peek(), 31);
        batch(|| *stock.write() = 5);
        assert_eq!(*memo2.peek(), 51);
    });
}

// ── Effect ──────────────────────────────────────────────────────────────────

#[test]
fn test_effect_runs_initially_and_on_change() {
    let count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(0i32);
        let c = count.clone();
        let _effect = Effect::new(move || {
            let _ = stock.read();
            c.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);
        batch(|| *stock.write() = 7);
        assert_eq!(count.load(Ordering::SeqCst), 2);
        batch(|| *stock.write() = 8);
        assert_eq!(count.load(Ordering::SeqCst), 3);
    });
}

// ── Raw derive (GetNext) ────────────────────────────────────────────────────

#[test]
fn test_derived_named_fields() {
    struct Point {
        x: i32,
        y: i32,
    }
    make_sphere(None, || {
        let stock = Stock::new(Point { x: 10, y: 20 });
        let dx = stock
            .derive(0u64, GetNext::new(|p: &Point| &p.x, |p| &mut p.x))
            .pool();
        let dy = stock
            .derive(1u64, GetNext::new(|p: &Point| &p.y, |p| &mut p.y))
            .pool();

        assert_eq!(*dx.peek(), 10);
        assert_eq!(*dy.peek(), 20);
        batch(|| *dx.write() = 99);
        assert_eq!(*dx.peek(), 99);
        assert_eq!(*dy.peek(), 20);
    });
}

#[test]
fn test_derived_selective_reactivity() {
    // Mutating one derived field must not re-trigger subscribers of another.
    struct AB {
        a: i32,
        b: i32,
    }
    let count_a = Arc::new(AtomicU32::new(0));
    let count_b = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(AB { a: 1, b: 2 });
        let da = stock
            .derive(0u64, GetNext::new(|s: &AB| &s.a, |s| &mut s.a))
            .pool();
        let db = stock
            .derive(1u64, GetNext::new(|s: &AB| &s.b, |s| &mut s.b))
            .pool();

        let ca = count_a.clone();
        let _ea = Effect::new(move || {
            let _ = da.read();
            ca.fetch_add(1, Ordering::SeqCst);
        });
        let cb = count_b.clone();
        let _eb = Effect::new(move || {
            let _ = db.read();
            cb.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);

        batch(|| *da.write() = 99); // dirties path [0] only
        assert_eq!(count_a.load(Ordering::SeqCst), 2);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);
    });
}

// ── Batch ─────────────────────────────────────────────────────────────────

#[test]
fn test_batch_coalesces_mutations() {
    // Multiple writes inside one batch yield a single propagation cycle.
    let count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(0i32);
        let memo = Memo::new(move || *stock.read());
        let c = count.clone();
        let _effect = Effect::new(move || {
            let _ = memo.read();
            c.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);

        batch(|| {
            *stock.write() = 1;
            *stock.write() = 2;
            *stock.write() = 3;
        });
        assert_eq!(count.load(Ordering::SeqCst), 2);
        assert_eq!(*stock.peek(), 3);
    });
}

// ── Sphere lifecycle ────────────────────────────────────────────────────────

#[test]
fn test_sphere_clear_removes_states() {
    let (sid, stock) = make_sphere(None, || Stock::new(7u32));
    assert_eq!(*stock.peek(), 7);
    clear_sphere(sid);
    assert!(stock.try_peek().is_none());
}

#[test]
fn test_sphere_clear_cascades_to_children() {
    let baseline = crate::states::pool::slots_count();

    let (pid, parent_stock) = make_sphere(None, || Stock::new(1u32));
    let (_cid, child_stock) = make_sphere(Some(pid), || Stock::new(2u32));
    assert_eq!(*parent_stock.peek(), 1);
    assert_eq!(*child_stock.peek(), 2);

    // Clearing only the parent also tears down the child sphere.
    clear_sphere(pid);

    assert!(parent_stock.try_peek().is_none());
    assert!(child_stock.try_peek().is_none());
    // No leak: every slot created by parent + child is freed.
    assert_eq!(crate::states::pool::slots_count(), baseline);
}

#[test]
fn test_sphere_clear_cascades_deeply() {
    let baseline = crate::states::pool::slots_count();

    let (gid, g) = make_sphere(None, || Stock::new(1u32));
    let (pid, p) = make_sphere(Some(gid), || Stock::new(2u32));
    let (_cid, c) = make_sphere(Some(pid), || Stock::new(3u32));

    // Clearing the root frees the whole subtree.
    clear_sphere(gid);

    assert!(g.try_peek().is_none());
    assert!(p.try_peek().is_none());
    assert!(c.try_peek().is_none());
    assert_eq!(crate::states::pool::slots_count(), baseline);
}

#[test]
fn test_sphere_clear_is_idempotent_any_order() {
    let baseline = crate::states::pool::slots_count();

    let (pid, p) = make_sphere(None, || Stock::new(1u32));
    let (cid, c) = make_sphere(Some(pid), || Stock::new(2u32));

    // Child clears itself first; the parent's cascade then finds nothing to do
    // (and must not panic on the already-detached child).
    clear_sphere(cid);
    clear_sphere(pid);
    assert!(p.try_peek().is_none());
    assert!(c.try_peek().is_none());

    // Clearing an already-removed sphere is a no-op.
    clear_sphere(pid);
    clear_sphere(cid);
    assert_eq!(crate::states::pool::slots_count(), baseline);
}

#[test]
#[should_panic(expected = "par-sphere not found")]
fn test_sphere_with_missing_parent_panics() {
    // Creating a sphere under a non-existent parent fails fast.
    let _ = make_sphere(Some(99_999u32), || Stock::new(0u32));
}

// ── Vec extension ──────────────────────────────────────────────────────────

#[test]
fn test_vec_derive_index() {
    make_sphere(None, || {
        let stock: Stock<Vec<i32>> = Stock::new(vec![10, 20, 30]);
        let d0 = stock.get(0).pool();
        let d1 = stock.get(1).pool();
        let d2 = stock.get(2).pool();

        assert_eq!(*d0.try_peek().unwrap(), 10);
        assert_eq!(*d1.try_peek().unwrap(), 20);
        assert_eq!(*d2.try_peek().unwrap(), 30);

        batch(|| *d1.try_write().unwrap() = 99);

        assert_eq!(*d0.try_peek().unwrap(), 10);
        assert_eq!(*d1.try_peek().unwrap(), 99);
        assert_eq!(*d2.try_peek().unwrap(), 30);

        // Out-of-bounds → None.
        let oob = stock.get(10).pool();
        assert!(oob.try_peek().is_none());
    });
}

// ── HashMap extension (std + hashbrown) ────────────────────────────────────

#[test]
fn test_hashmap_derive_key() {
    use std::collections::HashMap;

    // Mutating one key must not notify a subscriber of a different key.
    let count_a = Arc::new(AtomicU32::new(0));
    let count_b = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let mut m = HashMap::new();
        m.insert("a".to_string(), 1i32);
        m.insert("b".to_string(), 2i32);
        let map = Stock::new(m);

        let da = map.get("a".to_string()).pool();
        let db = map.get("b".to_string()).pool();
        let missing = map.get("z".to_string()).pool();

        assert_eq!(*da.try_peek().unwrap(), 1);
        assert_eq!(*db.try_peek().unwrap(), 2);
        assert!(missing.try_peek().is_none()); // absent key → None

        let ca = count_a.clone();
        let _ea = Effect::new(move || {
            let _ = da.try_read();
            ca.fetch_add(1, Ordering::SeqCst);
        });
        let cb = count_b.clone();
        let _eb = Effect::new(move || {
            let _ = db.try_read();
            cb.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count_a.load(Ordering::SeqCst), 1);
        assert_eq!(count_b.load(Ordering::SeqCst), 1);

        // Mutate value at key "a" only — key "b" subscriber must not re-run.
        batch(|| *da.try_write().unwrap() = 99);
        assert_eq!(*da.try_peek().unwrap(), 99);
        assert_eq!(count_a.load(Ordering::SeqCst), 2);
        assert_eq!(count_b.load(Ordering::SeqCst), 1); // different key, untouched

        // Inserting the previously-absent key makes its derived value appear.
        batch(|| {
            map.write().insert("z".to_string(), 7);
        });
        assert_eq!(*missing.try_peek().unwrap(), 7);
    });
}

#[test]
fn test_hashbrown_map_derive_key() {
    // Same `.get(key)` extension, but over a hashbrown::HashMap.
    make_sphere(None, || {
        let mut m = hashbrown::HashMap::new();
        m.insert(1u32, "one".to_string());
        let map = Stock::new(m);

        let one = map.get(1u32).pool();
        let missing = map.get(9u32).pool();

        assert_eq!(&*one.try_peek().unwrap(), "one");
        assert!(missing.try_peek().is_none());

        batch(|| one.try_write().unwrap().push_str("-edited"));
        assert_eq!(&*one.try_peek().unwrap(), "one-edited");
    });
}

// ── Context ───────────────────────────────────────────────────────────────

#[test]
fn test_provide_and_use_context() {
    #[derive(Clone)]
    struct Ctx(u32);

    // Parent provides; child reads through its parent link.
    let (par, _) = make_sphere(None, || provide_context(Ctx(42)));
    make_sphere(Some(par), || {
        assert_eq!(use_context::<Ctx>().unwrap().0, 42);
        assert!(use_context::<String>().is_none());
    });
}

// ── #[derive(Stock)] — structs ─────────────────────────────────────────────

#[derive(Stock)]
struct World {
    slots: Vec<Loadable<i32>>,
    registry: std::collections::HashMap<String, Config>,
}

#[derive(Stock)]
struct Config {
    width: u32,
    height: u32,
}

#[derive(Stock)]
struct Rgb(u8, u8, u8);

#[test]
fn test_macro_named_struct() {
    // Path-selective mutation: changing width must not notify height subscribers.
    let count1 = Arc::new(AtomicU32::new(0));
    let count2 = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(Config {
            width: 1920,
            height: 1080,
        });
        let w = stock.width().pool();
        let h = stock.height().pool();
        let m_width = Memo::new(move || *w.read());
        let m_height = Memo::new(move || *h.read());

        let c1 = count1.clone();
        let _ = Effect::new(move || {
            let _ = m_width.read();
            c1.fetch_add(1, Ordering::SeqCst);
        });
        let c2 = count2.clone();
        let _ = Effect::new(move || {
            let _ = m_height.read();
            c2.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(*m_width.peek(), 1920);
        assert_eq!(*m_height.peek(), 1080);

        batch(|| *w.write() = 800);

        assert_eq!(*m_width.peek(), 800);
        assert_eq!(*m_height.peek(), 1080);
        assert_eq!(count1.load(Ordering::SeqCst), 2);
        assert_eq!(count2.load(Ordering::SeqCst), 1); // untouched sibling
    });
}

#[test]
fn test_macro_tuple_struct() {
    make_sphere(None, || {
        let stock = Stock::new(Rgb(255, 128, 0));
        let d0 = stock.f0().pool();
        let d1 = stock.f1().pool();
        let d2 = stock.f2().pool();

        assert_eq!(*d0.peek(), 255u8);
        assert_eq!(*d1.peek(), 128u8);
        assert_eq!(*d2.peek(), 0u8);

        batch(|| *d1.write() = 64u8);

        assert_eq!(*d0.peek(), 255);
        assert_eq!(*d1.peek(), 64);
        assert_eq!(*d2.peek(), 0);
    });
}

// ── #[derive(Stock)] — enums ───────────────────────────────────────────────

#[derive(Stock)]
#[allow(dead_code)]
enum Shape {
    Circle(f64),
    Rect { w: f64, h: f64 },
    Dot,
}

#[derive(Stock)]
#[allow(dead_code)]
enum Loadable<T> {
    Loading,
    Loaded(T),
    Failed(String),
}

#[test]
fn test_macro_enum_accessor() {
    make_sphere(None, || {
        let stock = Stock::<Shape>::new(Shape::Circle(2.5));

        assert_eq!(*stock.circle().try_peek().unwrap(), 2.5);

        batch(|| *stock.write() = Shape::Dot);

        assert!(stock.circle().try_peek().is_none());
    });
}

#[test]
fn test_macro_enum_generic() {
    make_sphere(None, || {
        let stock = Stock::<Loadable<i32>>::new(Loadable::Loading);

        assert!(stock.loaded().try_peek().is_none()); // wrong variant

        batch(|| *stock.write() = Loadable::Loaded(42));

        assert_eq!(*stock.loaded().try_peek().unwrap(), 42);
        assert!(stock.failed().try_peek().is_none());
    });
}

// ── Generic bounds on derived trait ────────────────────────────────────────

#[test]
fn test_macro_generic_bounds_inline() {
    #[derive(Stock)]
    struct BndStruct<T: Clone> {
        item: T,
    }
    make_sphere(None, || {
        let stock = Stock::new(BndStruct {
            item: vec![1u32, 2, 3],
        });
        let item_derived = stock.item().pool();
        assert_eq!(*item_derived.peek(), vec![1u32, 2, 3]);
        batch(|| item_derived.write().push(4));
        assert_eq!(*item_derived.peek(), vec![1u32, 2, 3, 4]);
    });
}

#[test]
fn test_macro_generic_bounds_where_clause() {
    #[derive(Stock)]
    struct WhereBnd<T>
    where
        T: Clone,
    {
        item: T,
    }
    make_sphere(None, || {
        let stock = Stock::new(WhereBnd {
            item: "hello".to_string(),
        });
        let item_stock = stock.item().pool();
        assert_eq!(*item_stock.peek(), "hello");
        batch(|| item_stock.write().push_str(" world"));
        assert_eq!(*item_stock.peek(), "hello world");
    });
}

// ── Deep derive chain (struct macro + Vec/HashMap ext + enum macro) ─────────

#[test]
fn test_deep_derive_chain_selective_propagation() {
    use std::collections::HashMap;

    let cnt_slots = Arc::new(AtomicU32::new(0));
    let cnt_slot0 = Arc::new(AtomicU32::new(0));
    let cnt_loaded = Arc::new(AtomicU32::new(0));
    let cnt_reg = Arc::new(AtomicU32::new(0));
    let cnt_alice = Arc::new(AtomicU32::new(0));
    let cnt_width = Arc::new(AtomicU32::new(0));

    let mut registry = HashMap::new();
    registry.insert(
        "alice".to_string(),
        Config {
            width: 100,
            height: 200,
        },
    );

    make_sphere(None, || {
        let stock = Stock::new(World {
            slots: vec![Loadable::Loaded(42)],
            registry,
        });

        let da = stock.slots().pool();
        let c = cnt_slots.clone();
        let _ea = Effect::new(move || {
            let _ = da.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });

        let db = stock.slots().get(0usize).pool();
        let c = cnt_slot0.clone();
        let _eb = Effect::new(move || {
            let _ = db.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });

        let dc = stock.slots().get(0usize).loaded().pool();
        let c = cnt_loaded.clone();
        let _ec = Effect::new(move || {
            let _ = dc.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });

        let dd = stock.registry().pool();
        let c = cnt_reg.clone();
        let _ed = Effect::new(move || {
            let _ = dd.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });

        let de = stock.registry().get("alice".to_string()).pool();
        let c = cnt_alice.clone();
        let _ee = Effect::new(move || {
            let _ = de.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });

        let df = stock.registry().get("alice".to_string()).width().pool();
        let c = cnt_width.clone();
        let _ef = Effect::new(move || {
            let _ = df.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(cnt_slots.load(Ordering::SeqCst), 1);
        assert_eq!(cnt_slot0.load(Ordering::SeqCst), 1);
        assert_eq!(cnt_loaded.load(Ordering::SeqCst), 1);
        assert_eq!(cnt_reg.load(Ordering::SeqCst), 1);
        assert_eq!(cnt_alice.load(Ordering::SeqCst), 1);
        assert_eq!(cnt_width.load(Ordering::SeqCst), 1);

        // L4a: mutate slots[0].loaded → ancestors L2a/L3a + self fire.
        batch(|| {
            *stock.slots().get(0usize).loaded().try_write().unwrap() = 99;
        });
        assert_eq!(cnt_slots.load(Ordering::SeqCst), 2);
        assert_eq!(cnt_slot0.load(Ordering::SeqCst), 2);
        assert_eq!(cnt_loaded.load(Ordering::SeqCst), 2);
        assert_eq!(cnt_reg.load(Ordering::SeqCst), 1);

        // L4b: mutate registry["alice"].width → L2b/L3b + self.
        batch(|| {
            *stock
                .registry()
                .get("alice".to_string())
                .width()
                .try_write()
                .unwrap() = 800;
        });
        assert_eq!(cnt_reg.load(Ordering::SeqCst), 2);
        assert_eq!(cnt_alice.load(Ordering::SeqCst), 2);
        assert_eq!(cnt_width.load(Ordering::SeqCst), 2);
        assert_eq!(cnt_slots.load(Ordering::SeqCst), 2); // unrelated subtree

        // L2a: mutate whole slots → descendants L3a/L4a fire, registry subtree not.
        batch(|| stock.slots().write().push(Loadable::Loading));
        assert_eq!(cnt_slots.load(Ordering::SeqCst), 3);
        assert_eq!(cnt_slot0.load(Ordering::SeqCst), 3);
        assert_eq!(cnt_loaded.load(Ordering::SeqCst), 3);
        assert_eq!(cnt_reg.load(Ordering::SeqCst), 2);
    });
}

// ── Regression: subscribe even when peek returns None ─────────────────────

#[test]
fn test_optional_derive_subscribes_when_none() {
    // mark_cited must run even when the value is currently None (OOB index),
    // so the effect is notified once the value later appears.
    let count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let items: Stock<Vec<u32>> = Stock::new(vec![0u32, 1u32, 2u32]);
        let item5 = items.get(5usize).pool(); // OOB → None

        let c = count.clone();
        let _effect = Effect::new(move || {
            let _ = item5.try_read();
            c.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);

        batch(|| {
            for v in 3u32..=5 {
                items.write().push(v);
            }
        });
        assert_eq!(count.load(Ordering::SeqCst), 2);
    });
}

// ── Derived stock via prefix path (built on a pooled intermediate) ─────────

#[test]
fn test_derived_stock_reactive_via_prefix_path() {
    #[derive(Stock)]
    struct PfxX {
        y: PfxY,
    }
    #[derive(Stock)]
    struct PfxY {
        z: u32,
    }
    let run_count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(PfxX { y: PfxY { z: 0 } });
        let y_stock = stock.y().pool();
        let rc = run_count.clone();
        let _memo = Memo::new(move || {
            let v = *y_stock.z().read();
            rc.fetch_add(1, Ordering::SeqCst);
            v
        });
        assert_eq!(run_count.load(Ordering::SeqCst), 1);
        batch(|| *y_stock.z().write() = 7);
        assert_eq!(run_count.load(Ordering::SeqCst), 2);
        assert_eq!(*y_stock.z().peek(), 7);
    });
}

// ── Deriving inside a reactive closure (no pooling) ────────────────────────

#[test]
fn test_derive_inside_closure_no_leak() {
    // Idiom: capture only the (Copy) root stock and derive *inside* the reactive
    // closure — no pooling. `derive` allocates no pool state, so re-running the
    // closure many times keeps reactivity correct WITHOUT growing the pool
    // (whereas `.pool()` inside a closure would leak a getter state per run).
    #[derive(Stock)]
    struct Wrap {
        inner: Vec<i32>,
    }

    let runs = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(Wrap {
            inner: vec![10, 20, 30],
        });

        let rc = runs.clone();
        let _effect = Effect::new(move || {
            // 2-level derive built fresh each run; nothing is pooled.
            let _ = stock.inner().get(1).try_read();
            rc.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(runs.load(Ordering::SeqCst), 1);

        // Baseline pool size after setup (root value + effect citer).
        let baseline = crate::states::pool::slots_count();

        for i in 0..20 {
            batch(|| *stock.inner().get(1).try_write().unwrap() = i);
        }

        // Reactivity held across all re-runs ...
        assert_eq!(runs.load(Ordering::SeqCst), 21);
        assert_eq!(*stock.inner().get(1).try_read().unwrap(), 19);
        // ... and not a single pool slot was added.
        assert_eq!(crate::states::pool::slots_count(), baseline);
    });
}

// ── Ancestor-path propagation ─────────────────────────────────────────────

#[test]
fn test_root_subscriber_notified_on_child_path_mutation() {
    // Effect citing root [] re-runs when a child path is mutated.
    let count = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(Config {
            width: 1920,
            height: 1080,
        });
        let c = count.clone();
        let _e = Effect::new(move || {
            let _ = stock.read();
            c.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);
        batch(|| *stock.width().write() = 800);
        assert_eq!(count.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn test_sibling_path_not_notified() {
    // Ancestor propagation must not leak into sibling paths.
    let count_w = Arc::new(AtomicU32::new(0));
    let count_h = Arc::new(AtomicU32::new(0));
    make_sphere(None, || {
        let stock = Stock::new(Config {
            width: 1,
            height: 2,
        });
        let cw = count_w.clone();
        let _ew = Effect::new(move || {
            let _ = stock.width().read();
            cw.fetch_add(1, Ordering::SeqCst);
        });
        let ch = count_h.clone();
        let _eh = Effect::new(move || {
            let _ = stock.height().read();
            ch.fetch_add(1, Ordering::SeqCst);
        });
        assert_eq!(count_w.load(Ordering::SeqCst), 1);
        assert_eq!(count_h.load(Ordering::SeqCst), 1);

        batch(|| *stock.height().write() = 999);
        assert_eq!(count_w.load(Ordering::SeqCst), 1); // sibling untouched
        assert_eq!(count_h.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn test_repetitively_derived_stock_reactivity() {
    // Reactivity must round-trip through a chain that alternates
    // derive (ChainedPipe) -> `.pool()` (PooledPipe) four times. Mutating the
    // leaf must notify every ancestor level (prefix paths) and the leaf's own
    // subscriber, but NOT a sibling leaf.
    #[derive(Stock, Debug, Clone, PartialEq)]
    struct Vector<T>(Vec<T>);

    let c_l1 = Arc::new(AtomicU32::new(0));
    let c_l2 = Arc::new(AtomicU32::new(0));
    let c_l3 = Arc::new(AtomicU32::new(0));
    let c_leaf0 = Arc::new(AtomicU32::new(0));
    let c_leaf1 = Arc::new(AtomicU32::new(0));

    make_sphere(None, || {
        let stock: Stock<Vector<Vector<i32>>> = Stock::new(Vector(vec![Vector(vec![0i32, 1i32])]));

        // Each level rebuilds a ChainedPipe on top of the previous PooledPipe.
        let l1 = stock.f0().pool(); //    path [0]
        let l2 = l1.get(0).pool(); //     path [0,0]
        let l3 = l2.f0().pool(); //       path [0,0,0]
        let leaf0 = l3.get(0).pool(); //  path [0,0,0,0]
        let leaf1 = l3.get(1).pool(); //  path [0,0,0,1] (sibling)

        macro_rules! sub {
            ($stock:ident, $cnt:ident) => {{
                let c = $cnt.clone();
                Effect::new(move || {
                    let _ = $stock.try_read();
                    c.fetch_add(1, Ordering::SeqCst);
                })
            }};
        }
        let _e1 = sub!(l1, c_l1);
        let _e2 = sub!(l2, c_l2);
        let _e3 = sub!(l3, c_l3);
        let _e_leaf0 = sub!(leaf0, c_leaf0);
        let _e_leaf1 = sub!(leaf1, c_leaf1);

        // All run once initially.
        for c in [&c_l1, &c_l2, &c_l3, &c_leaf0, &c_leaf1] {
            assert_eq!(c.load(Ordering::SeqCst), 1);
        }

        // Mutate the leaf through the fully-pooled chain (every level `.pool()`ed).
        batch(|| *leaf0.try_write().unwrap() += 100);
        assert_eq!(
            stock.peek().clone(),
            Vector(vec![Vector(vec![100i32, 1i32])])
        );

        // Ancestors + leaf0 re-run; the sibling leaf does not.
        assert_eq!(c_l1.load(Ordering::SeqCst), 2, "L1 (ancestor)");
        assert_eq!(c_l2.load(Ordering::SeqCst), 2, "L2 (ancestor)");
        assert_eq!(c_l3.load(Ordering::SeqCst), 2, "L3 (ancestor)");
        assert_eq!(c_leaf0.load(Ordering::SeqCst), 2, "leaf0 (self)");
        assert_eq!(c_leaf1.load(Ordering::SeqCst), 1, "leaf1 (sibling, silent)");

        // Mutating an intermediate level propagates down to both leaves.
        batch(|| l3.try_write().unwrap().push(2i32));
        assert_eq!(c_l1.load(Ordering::SeqCst), 3);
        assert_eq!(c_l3.load(Ordering::SeqCst), 3);
        assert_eq!(
            c_leaf0.load(Ordering::SeqCst),
            3,
            "leaf0 (descendant of L3)"
        );
        assert_eq!(
            c_leaf1.load(Ordering::SeqCst),
            2,
            "leaf1 (descendant of L3)"
        );
    });
}

// ── #[stock] attribute macro ───────────────────────────────────────────────

#[derive(Stock)]
struct Pair {
    x: u32,
    y: u32,
}

#[stock]
impl Stock<Pair> {
    fn sum(&self) -> u32 {
        let x = *self.x().peek();
        let y = *self.y().peek();
        x + y
    }

    fn set_x(&self, val: u32) {
        *self.x().write() = val;
    }

    fn swap(&self) {
        let sx = self.x();
        let sy = self.y();
        let x = *sx.peek();
        let y = *sy.peek();
        sx.set(y);
        sy.set(x);
    }
}

#[test]
fn test_stock_attr_macro() {
    make_sphere(None, || {
        let pair = Stock::new(Pair { x: 3, y: 7 });
        assert_eq!(pair.sum(), 10);
        batch(|| pair.set_x(20));
        assert_eq!(pair.sum(), 27);
        batch(|| pair.swap());
        assert_eq!(*pair.x().peek(), 7);
        assert_eq!(*pair.y().peek(), 20);
    });
}

#[test]
fn test_stock_attr_macro_custom_name() {
    #[derive(Stock)]
    struct Point {
        px: i32,
        py: i32,
    }

    #[stock(PointOps)]
    impl Stock<Point> {
        fn manhattan(&self) -> i32 {
            let px = *self.px().peek();
            let py = *self.py().peek();
            px + py
        }
    }

    make_sphere(None, || {
        let pt = Stock::new(Point { px: 3, py: 4 });
        assert_eq!(pt.manhattan(), 7);
    });
}
