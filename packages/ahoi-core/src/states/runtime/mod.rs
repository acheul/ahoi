use super::*;

pub(crate) mod citation;
pub(crate) mod propagation;
pub(crate) mod sphere;

pub(crate) mod insert;
pub(crate) mod run_runner;

use citation::{CiteRels, RunningCites};
use propagation::RunningBatch;
use sphere::Sphere;

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Default::default());
}

#[derive(Default)]
struct Runtime {
    running_cites: RunningCites,
    running_batch: RunningBatch,
    working_spheres: Vec<SphereId>, // spawn_batch 로 인해, Option 대신 Stack 사용해야 함.
    building_sphere: Option<(SphereId, Sphere)>,
    next_sphere_id: SphereId,
    spheres: IntMap<SphereId, Sphere>,
    cite_rels: CiteRels,
    #[cfg(debug_assertions)]
    locations: IntMap<StateId, Location>,
}

impl Runtime {
    /// Update current building or running sphere
    fn update_current_sphere<R>(&mut self, update: impl FnOnce(&mut Sphere) -> R) -> Option<R> {
        match self.building_sphere.as_mut() {
            Some((_, sphere)) => Some(update(sphere)),
            None => match self.working_spheres.last_mut() {
                Some(sphere_id) => {
                    let sphere = self.spheres.get_mut(sphere_id)?;
                    return Some(update(sphere));
                }
                None => None,
            },
        }
    }

    /// Creation site of a state, when one was recorded.
    /// * Always `None` in release: locations are not tracked there.
    #[cfg(debug_assertions)]
    fn location_of(&self, id: &StateId) -> Option<Location> {
        self.locations.get(id).copied()
    }

    #[cfg(not(debug_assertions))]
    fn location_of(&self, _id: &StateId) -> Option<Location> {
        None
    }

    /// Drop a state's recorded location. Must be called wherever the state
    /// itself leaves the pool, or `locations` grows without bound in debug.
    #[cfg(debug_assertions)]
    fn unregister_location(&mut self, id: &StateId) {
        let _ = self.locations.remove(id);
    }

    #[cfg(not(debug_assertions))]
    fn unregister_location(&mut self, _id: &StateId) {}
}

#[cfg(debug_assertions)]
fn register_location(id: StateId, location: Location) {
    RUNTIME.with_borrow_mut(|runtime| {
        runtime.locations.insert(id, location);
    });
}

/// Number of recorded locations (test-only; used to assert the debug-only
/// location registry is freed alongside the states it describes).
#[cfg(all(test, debug_assertions))]
pub(crate) fn locations_count() -> usize {
    RUNTIME.with_borrow(|runtime| runtime.locations.len())
}

/// Source files of every recorded location (test-only).
/// * Used to assert that no creation site was recorded as ahoi-core itself,
///   which is what happens when a `#[cfg_attr(debug_assertions, track_caller)]`
///   is missing somewhere along a constructor chain.
#[cfg(all(test, debug_assertions))]
pub(crate) fn location_files() -> Vec<&'static str> {
    RUNTIME.with_borrow(|runtime| runtime.locations.values().map(|l| l.file()).collect())
}
