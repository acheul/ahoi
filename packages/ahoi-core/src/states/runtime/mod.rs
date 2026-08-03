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
}
