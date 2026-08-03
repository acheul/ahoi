use super::*;
use std::any::TypeId;

pub type SphereId = u32;

#[derive(Default)]
pub(super) struct Sphere {
    par_sphere: Option<SphereId>,
    child_spheres: IntSet<SphereId>,
    values: IntSet<StateId>,
    runners: IntSet<StateId>,
    /// Option<Option<hail-writer-executer-id>>
    hail: Option<Option<StateId>>,
    mappers: IntSet<StateId>,
    contexts: HashMap<TypeId, Box<dyn Any>>,
}

impl Sphere {
    fn new(par_sphere_id: Option<SphereId>) -> Self {
        Self {
            par_sphere: par_sphere_id,
            child_spheres: Default::default(),
            values: Default::default(),
            runners: Default::default(),
            hail: None,
            mappers: Default::default(),
            contexts: Default::default(),
        }
    }
}

/// Current building or working sphere
pub fn current_sphere_id() -> Option<SphereId> {
    RUNTIME.with_borrow(|runtime| match runtime.building_sphere.as_ref() {
        Some((id, _)) => Some(*id),
        None => runtime.working_spheres.last().copied(),
    })
}

/// Register created states
fn register_state_to_current_sphere(register: impl FnOnce(&mut Sphere)) {
    RUNTIME
        .with_borrow_mut(|runtime| runtime.update_current_sphere(register))
        .expect("create state out of sphere");
}

pub(crate) fn register_value_to_current_sphere(value_id: StateId) {
    register_state_to_current_sphere(|sphere| {
        sphere.values.insert(value_id);
    });
}

pub(crate) fn register_runner_to_current_sphere(runner_id: StateId) {
    register_state_to_current_sphere(|sphere| {
        sphere.runners.insert(runner_id);
    });
}

pub(crate) fn register_mapper_to_current_sphere(getter: StateId) {
    register_state_to_current_sphere(|sphere| {
        sphere.mappers.insert(getter);
    });
}

pub(crate) fn register_hail_to_current_sphere(write_callback_id: Option<StateId>) {
    register_state_to_current_sphere(|sphere| {
        if sphere.hail.is_some() {
            panic!("Already has hail")
        }
        let _ = sphere.hail.replace(write_callback_id);
    });
}

pub(crate) fn get_hail_writer_id(sphere_id: SphereId) -> Option<StateId> {
    let writer_id = RUNTIME.with_borrow(|runtime| {
        let sphere = runtime.spheres.get(&sphere_id)?;
        sphere.hail
    })??;
    Some(writer_id)
}

/// Run a closure as a new sphere and return its id alongside the closure's result.
///
/// A sphere is the unit of reactive lifetime: every state created inside `run`
/// (stocks, mappers, runners, contexts) is owned by the returned [`SphereId`]
/// and lives until [`clear_sphere`] is called with that id.
///
/// `par_sphere_id` links this sphere to a parent: [`use_context`] resolves up
/// the chain, and clearing the parent cascades into this sphere (see
/// [`clear_sphere`]). The parent must already exist when the child is created.
pub fn make_sphere<R>(par_sphere_id: Option<SphereId>, run: impl FnOnce() -> R) -> (SphereId, R) {
    // set building_sphere
    RUNTIME.with_borrow_mut(|runtime| {
        // get par sphere
        let par_sphere = match par_sphere_id {
            Some(par_sphere_id) => Some(
                runtime
                    .spheres
                    .get_mut(&par_sphere_id)
                    .expect("par-sphere not found"),
            ),
            None => None,
        };

        // set building sphere
        if runtime.building_sphere.is_some() {
            panic!("sphere cannot be built nested");
        }

        // get sphere id
        runtime.next_sphere_id += 1;
        let sphere_id = runtime.next_sphere_id;

        let _ = runtime
            .building_sphere
            .replace((sphere_id, Sphere::new(par_sphere_id)));

        // enrol to par shpere
        if let Some(par_sphere) = par_sphere {
            par_sphere.child_spheres.insert(sphere_id);
        }
    });

    // run
    let res = run();

    // enrol the new sphere
    let new_sphere_id = RUNTIME.with_borrow_mut(|runtime| {
        let (new_sphere_id, new_sphere) = runtime.building_sphere.take().unwrap();
        runtime.spheres.insert(new_sphere_id, new_sphere);
        new_sphere_id
    });

    return (new_sphere_id, res);
}

/// Make an empty top sphere
pub fn make_top_sphere() -> SphereId {
    make_sphere(None, || {}).0
}

/// Clear a sphere, dropping every state it owns (stocks, mappers, runners,
/// contexts) and removing its cite-relations from the reactive graph.
///
/// ## Cascades to children; order-independent
///
/// Clearing a sphere also clears its child spheres recursively, so the whole
/// subtree is freed in one call — the host does **not** need to clear children
/// before parents. Each sphere is tracked under its parent (`child_spheres`)
/// at creation, which is what makes the cascade possible.
///
/// Clearing is idempotent: clearing an already-removed sphere is a no-op. So a
/// host can register a `clear_sphere` per component (e.g. in each SolidJS
/// `onCleanup`) and stay correct regardless of order — whether a child clears
/// itself first, or a parent's cascade reaches it first, the later call simply
/// finds nothing to do.
pub fn clear_sphere(sphere_id: SphereId) {
    let _ = RUNTIME.with_borrow_mut(|runtime| runtime.clear_sphere(false, sphere_id));
}

impl Runtime {
    /// Recursive worker for [`clear_sphere`].
    ///
    /// `par_is_cleared` says whether this sphere's parent has already been removed
    /// (true when reached via a parent's cascade). When false, the sphere detaches
    /// itself from its still-present parent's `child_spheres`; when true that step
    /// is skipped, since the parent — and its `child_spheres` set — is already gone.
    fn clear_sphere(&mut self, par_is_cleared: bool, sphere_id: SphereId) -> Option<()> {
        let Sphere {
            par_sphere,
            child_spheres,
            values,
            runners,
            mappers,
            // and these will be dropped automatically
            hail: _,
            contexts: _,
        } = self.spheres.remove(&sphere_id)?;

        // clear from par_sphere
        if !par_is_cleared {
            if let Some(par_sphere_id) = par_sphere {
                let _ = self
                    .spheres
                    .get_mut(&par_sphere_id)
                    .unwrap()
                    .child_spheres
                    .remove(&sphere_id);
            }
        }

        // Clear values
        // * remove from cite-rels
        for value_id in values.iter() {
            self.cite_rels.value_rels.remove(value_id);
            self.cite_rels.value_owners.remove(value_id);
        }
        // * remove from pool
        for value_id in values {
            pool::remove_state(value_id).unwrap();
        }

        // Clear runners
        for runner_id in runners {
            // remove from pool
            let runner = pool::remove_state(runner_id).unwrap();
            let is_citer = runner.as_runner().unwrap().is_citer();

            // remove from cite-rels if it's citer
            if is_citer {
                self.cite_rels.remove_rels_by_citer(&runner_id);
            }
        }

        // Clear mappers
        for getter_id in mappers {
            // remove from pool
            pool::remove_state(getter_id).unwrap();
        }

        // Clear child_spheres
        for child_sphere_id in child_spheres {
            let _ = self.clear_sphere(true, child_sphere_id);
        }

        Some(())
    }
}

pub fn provide_context<T: 'static>(value: T) -> () {
    let type_id = TypeId::of::<T>();

    // register to current building or working sphere
    RUNTIME
        .with_borrow_mut(|runtime| {
            runtime.update_current_sphere(|sphere| {
                sphere.contexts.insert(type_id, Box::new(value));
            })
        })
        .expect("provide_context: out of sphere");
}

/// Look up a context value of type `T`, walking up the parent-sphere chain.
/// Returns `None` if no ancestor provided a `T`.
/// Panics if called outside any sphere or work.
pub fn use_context<T: Clone + 'static>() -> Option<T> {
    let type_id = TypeId::of::<T>();

    RUNTIME.with_borrow(|runtime| {
        // Resolves starting from the parent of the currently running sphere.
        // Or, when called from within [`work`], from the running work sphere.

        // 1. from running sphere
        let start_sphere_id = match runtime.building_sphere.as_ref() {
            Some((_, sphere)) => sphere.par_sphere,
            None => {
                // 2. from running work
                match runtime.working_spheres.last() {
                    Some(sphere_id) => Some(*sphere_id),
                    None => {
                        panic!("use_context: out of sphere")
                    }
                }
            }
        };

        let sphere_id = start_sphere_id?;
        runtime.get_context::<T>(sphere_id, type_id)
    })
}

impl Runtime {
    fn get_context<T: Clone + 'static>(&self, sphere_id: SphereId, type_id: TypeId) -> Option<T> {
        let sphere = self.spheres.get(&sphere_id)?;
        let Some(value) = sphere.contexts.get(&type_id) else {
            let par_sphere_id = sphere.par_sphere?;
            return self.get_context(par_sphere_id, type_id);
        };
        value.as_ref().downcast_ref::<T>().cloned()
    }
}
