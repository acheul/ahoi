//! Guards for the debug-only creation-site registry (`documents/todos.md`:
//! panic path debug).
//!
//! Every state records where user code created it, so panics raised deep in the
//! runtime can blame a real source line. That only works if *every* function
//! between the user's call and `Location::caller()` carries
//! `#[cfg_attr(debug_assertions, track_caller)]` — miss one and the recorded
//! location silently becomes an ahoi-core file instead. These tests catch that.
//!
//! Debug-only: the registry does not exist in release builds.
#![cfg(debug_assertions)]

use super::*;
use crate::states::runtime::{location_files, locations_count};

/// Assert every location recorded so far points at this test file.
///
/// Checking the whole registry (rather than one id per constructor) means a
/// broken chain is caught no matter which state it belongs to.
#[track_caller]
fn assert_all_origins_are_local() {
    let files = location_files();
    assert!(!files.is_empty(), "no locations recorded — nothing tested");
    for f in files {
        assert_eq!(
            f,
            file!(),
            "a creation site was recorded inside ahoi-core; a \
             `#[cfg_attr(debug_assertions, track_caller)]` is missing along that \
             constructor's chain"
        );
    }
}

#[test]
fn test_origin_of_sync_constructors_is_user_code() {
    let (sid, _) = make_sphere(None, || {
        // value states
        let stock = Stock::new(vec![0u32, 1u32, 2u32]);

        // mapper state, via the inherent `.pool()`
        let item = stock.get(1usize).pool();

        // mapper state, via `Into` (best-effort: `Into::into` is not
        // `#[track_caller]` upstream — see `hooks::stock::pipe`)
        let _pooled: OptStock<u32> = stock.get(2usize).into();

        // citer runner + its associated backing stock
        let _memo = Memo::new(move || *item.read().unwrap());

        // the combinator form, which reaches `Memo::new` one hop deeper
        let _memo2 = stock.memo(|v| v.len());
        let _memo3 = item.memo(|v| v.copied());

        // executer runner
        let _cb = Callback::new(|x: i32| x * 2);

        // citer runner, run immediately
        let _effect = Effect::new(move || {
            let _ = stock.try_read();
        });
    });

    assert_all_origins_are_local();
    clear_sphere(sid);
}

#[test]
fn test_origin_of_hail_constructors_is_user_code() {
    struct Opt;
    impl<T: Clone + 'static> HailConverter<T> for Opt {
        type HailValue = Option<T>;
        const NONE: Option<T> = None;
        fn from_raw_value(raw_value: &T) -> Option<T> {
            Some(raw_value.clone())
        }
        fn into_raw_value(hail_value: Option<T>) -> T {
            hail_value.unwrap()
        }
    }

    struct NoopDispatcher;
    impl HailDispatcher for NoopDispatcher {
        fn dispatch_hails(&self, _hails: HailsMap) {}
    }
    set_local_hail_dispatcher(NoopDispatcher);

    let (sid, _) = make_sphere(None, || {
        let stock = Stock::new(1u32);
        // hail citer runner (+ a write-callback executer for the mutable form)
        let _ = stock.set_hail::<Opt>();
    });

    let (sid2, _) = make_sphere(None, || {
        let stock = Stock::new(2u32);
        let _ = stock.set_read_hail::<Opt>();
    });

    assert_all_origins_are_local();
    clear_sphere(sid);
    clear_sphere(sid2);
}

#[test]
fn test_locations_are_freed_with_their_states() {
    let baseline_slots = crate::states::pool::slots_count();
    let baseline_locations = locations_count();

    let (pid, _) = make_sphere(None, || {
        let stock = Stock::new(vec![0u32, 1u32]);
        let pooled = stock.get(0usize).pool();
        let _memo = Memo::new(move || *pooled.read().unwrap());
        let _cb = Callback::new(|x: u32| x);
    });
    let (_cid, _) = make_sphere(Some(pid), || Stock::new(9u32));

    assert!(locations_count() > baseline_locations);

    clear_sphere(pid);

    // The registry must shrink exactly like the pool does — otherwise it grows
    // without bound over a long-running debug session.
    assert_eq!(crate::states::pool::slots_count(), baseline_slots);
    assert_eq!(locations_count(), baseline_locations);
}
