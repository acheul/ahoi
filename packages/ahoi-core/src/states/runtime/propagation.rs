//! Propagate Reactive Context
use super::*;
use hashbrown::hash_map::Entry;

/// Running Batch Cache
#[derive(Default)]
pub(super) struct RunningBatch {
    is_batching: bool,
    /// true while the top-level batch is draining the citer queue.
    /// Pulls ([ensure_citer_fresh]) only act during this phase; reads in the
    /// batch body see pre-batch values by design (see pull-on-read-design.md).
    is_propagating: bool,
    /// Collect values' paths who get "mutated"
    mutated_values: IntMap<StateId, MutatedNode>,
    hails_to_dispatch: IntMap<SphereId, Box<dyn Any>>,
    citer_queue: CiterQueue,
}

#[derive(Default)]
pub(crate) struct MutatedNode {
    pub(crate) mutated: bool,
    pub(crate) subs: IntMap<u64, MutatedNode>,
}

impl MutatedNode {
    /// If given `path`'s ancestor, or oneself, or subs is mutated, return true
    pub(crate) fn is_propagatable_to(&self, path: &Path) -> bool {
        let mut node = self;
        for path_key in path.as_slice() {
            // anc is mutated -> return true
            if node.mutated {
                return true;
            }
            match node.subs.get(path_key) {
                Some(sub_node) => {
                    node = sub_node;
                }
                // the path is not included: return false
                None => {
                    return false;
                }
            }
        }
        // the target node exists: it means that any of this one or subs is mutated,
        // thus return true!
        return true;
    }
}

pub(crate) fn mark_dirty(value_id: StateId, mutated_path: Path) {
    RUNTIME.with_borrow_mut(|runtime| {
        if !runtime.running_batch.is_batching {
            panic!("Use batch when mutate state");
        }

        let mut node = runtime
            .running_batch
            .mutated_values
            .entry(value_id)
            .or_default();

        for path_key in mutated_path.as_slice() {
            if node.mutated {
                return;
            }
            node = match node.subs.entry(*path_key) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => entry.insert(MutatedNode::default()),
            };
        }

        node.mutated = true;
    });
}

pub(crate) fn mark_hail<U: 'static>(sphere_id: SphereId, hail_value: U) {
    RUNTIME.with_borrow_mut(|runtime| {
        runtime
            .running_batch
            .hails_to_dispatch
            .insert(sphere_id, Box::new(hail_value));
    });
}

pub fn batch<R>(run: impl FnOnce() -> R) -> R {
    let is_top_batch = RUNTIME.with_borrow_mut(|runtime| {
        let is_top = !runtime.running_batch.is_batching;
        runtime.running_batch.is_batching = true;
        is_top
    });

    // run
    let res = run();

    if is_top_batch {
        RUNTIME.with_borrow_mut(|runtime| {
            runtime.running_batch.is_propagating = true;
        });

        loop {
            let next_citer = RUNTIME.with_borrow_mut(|runtime| {
                flush_marks(runtime);
                runtime.running_batch.citer_queue.pop_marked()
            });
            let Some(citer_id) = next_citer else {
                break;
            };
            settle_citer(citer_id);
        }

        // disptach hails
        let hails_to_dispatch = RUNTIME.with_borrow_mut(|runtime| {
            let running_batch = &mut runtime.running_batch;
            running_batch.is_propagating = false;
            running_batch.is_batching = false;
            running_batch.citer_queue.clear();
            std::mem::take(&mut running_batch.hails_to_dispatch)
        });
        // dispatch hails
        if !hails_to_dispatch.is_empty() {
            let _ = utils::hail_utils::dispatch_hails(hails_to_dispatch);
        }
    }

    return res;
}

/// run batch within a working sphere wrapper when cur_sphere is some
pub fn batch_with_sphere<R>(cur_sphere: impl Into<Option<SphereId>>, run: impl FnOnce() -> R) -> R {
    let Some(cur_sphere) = cur_sphere.into() else {
        return batch(run);
    };

    // register to working_spheres
    RUNTIME.with_borrow_mut(|runtime| {
        runtime.working_spheres.push(cur_sphere);
    });

    // run in batch
    let res = batch(run);

    // unregister from working_spheres
    RUNTIME.with_borrow_mut(|runtime| runtime.working_spheres.pop().unwrap());

    return res;
}

/// Drain `mutated_values` into the citer queue (the "mark" phase):
/// direct citers of a changed value become `dirty`, and every citer
/// transitively downstream of their associated stocks becomes `check`
/// (possibly stale). Cheap no-op when nothing was mutated.
fn flush_marks(runtime: &mut Runtime) {
    let Runtime {
        running_batch,
        cite_rels,
        running_cites,
        ..
    } = runtime;

    if running_batch.mutated_values.is_empty() {
        return;
    }
    let mutated_values = std::mem::take(&mut running_batch.mutated_values);

    for (mutated_value_id, mutated_node) in mutated_values {
        if let Some(path_to_citers) = cite_rels.value_rels.get(&mutated_value_id) {
            for (path, citer_ids) in path_to_citers {
                if mutated_node.is_propagatable_to(path) {
                    for citer_id in citer_ids {
                        // A citer currently mid-run observes fresh values through
                        // its own pulls — don't (re)mark it for this write.
                        if running_cites.0.contains_key(citer_id) {
                            continue;
                        }
                        running_batch.citer_queue.mark_dirty(*citer_id, cite_rels);
                    }
                }
            }
        }
    }
}

/// Make `citer_id`'s associated stocks fresh: if it is marked in the running
/// propagation, settle it (recursively settling its producers first) before
/// the caller reads its value.
/// * No-op outside the propagation phase — reads in the batch body see
///   pre-batch values by design.
pub(crate) fn ensure_citer_fresh(citer_id: StateId) {
    let marked = RUNTIME.with_borrow_mut(|runtime| {
        if !runtime.running_batch.is_propagating {
            return false;
        }
        // make freshly mutated values visible before the check
        flush_marks(runtime);
        let queue = &runtime.running_batch.citer_queue;
        queue.dirty.contains(&citer_id) || queue.check.contains(&citer_id)
    });
    if marked {
        settle_citer(citer_id);
    }
}

enum SettleAction {
    Run,
    CheckProducers(smallvec::SmallVec<[StateId; 2]>),
    Skip,
}

/// Settle one marked citer (the "settle" phase):
/// * `dirty` — a cited value actually changed: run.
/// * `check` — possibly stale: settle its producers first; if one of them
///   actually wrote, this citer got re-marked `dirty` — run then. Otherwise
///   nothing changed: skip without running.
fn settle_citer(citer_id: StateId) {
    let action = RUNTIME.with_borrow_mut(|runtime| {
        let queue = &mut runtime.running_batch.citer_queue;
        if queue.dirty.remove(&citer_id) {
            queue.ran.insert(citer_id);
            return SettleAction::Run;
        }
        if !queue.check.remove(&citer_id) {
            // already settled through another path (e.g. a pull)
            return SettleAction::Skip;
        }
        SettleAction::CheckProducers(runtime.cite_rels.producers_of(&citer_id))
    });

    match action {
        SettleAction::Run => {
            let _ = run_runner::run_citer(citer_id);
            // Flush this run's writes immediately: pulled producers' writes must
            // be consumed before the pulling consumer re-registers its rels,
            // or a late flush would misread an already-observed write as a
            // re-dirty of a ran citer (a false "cite cycle").
            RUNTIME.with_borrow_mut(flush_marks);
        }
        SettleAction::CheckProducers(producer_ids) => {
            for producer_id in producer_ids {
                ensure_citer_fresh(producer_id);
            }
            // If a producer actually wrote, its mark_dirty re-marked this citer
            // as `dirty` (it directly cites the changed stock).
            let run_now = RUNTIME.with_borrow_mut(|runtime| {
                flush_marks(runtime);
                let queue = &mut runtime.running_batch.citer_queue;
                if queue.dirty.remove(&citer_id) {
                    queue.ran.insert(citer_id);
                    true
                } else {
                    false
                }
            });
            if run_now {
                let _ = run_runner::run_citer(citer_id);
                // see the Run branch: flush immediately after the run
                RUNTIME.with_borrow_mut(flush_marks);
            }
        }
        SettleAction::Skip => {}
    }
}

// -- citers queue for batch
use std::collections::BinaryHeap;

#[derive(Eq, PartialEq)]
struct Item {
    citer_id: StateId,
    depth: usize,
}

// make smaller depth have higher priority
impl Ord for Item {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.depth.cmp(&self.depth)
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default)]
struct CiterQueue {
    /// depth-ordered schedule. Purely a heuristic: popping shallow citers first
    /// keeps pull recursion rare/shallow, but freshness never depends on it.
    heap: BinaryHeap<Item>,
    /// a cited value actually changed — must run
    dirty: IntSet<StateId>,
    /// transitively downstream of a dirty citer — possibly stale
    check: IntSet<StateId>,
    /// citers already run in this top-level batch (cycle guard)
    ran: IntSet<StateId>,
}

impl CiterQueue {
    fn mark_dirty(&mut self, citer_id: StateId, cite_rels: &CiteRels) {
        // Cycle guard: each citer runs at most once per top-level batch. With
        // pull-on-read, ordering violations self-heal, so a citer re-dirtied
        // after it already ran is a genuine cite cycle — drop it so the batch
        // terminates.
        if self.ran.contains(&citer_id) {
            debug_assert!(false, "cite cycle detected");
            return;
        }
        if self.dirty.contains(&citer_id) {
            return;
        }
        if self.check.remove(&citer_id) {
            // promote check -> dirty: heap entry & downstream marks already exist
            self.dirty.insert(citer_id);
            return;
        }
        self.dirty.insert(citer_id);
        self.push_heap(citer_id, cite_rels);
        self.mark_check_downstream(citer_id, cite_rels);
    }

    /// Mark every citer transitively downstream of `from`'s associated stocks
    /// as `check`. Stops at already-marked (or already-run) citers.
    fn mark_check_downstream(&mut self, from: StateId, cite_rels: &CiteRels) {
        let mut walk_stack = vec![from];
        while let Some(citer_id) = walk_stack.pop() {
            let Some(out_values) = cite_rels.citer_outputs.get(&citer_id) else {
                continue;
            };
            for out_value in out_values {
                let Some(path_to_citers) = cite_rels.value_rels.get(out_value) else {
                    continue;
                };
                for citer_ids in path_to_citers.values() {
                    for down in citer_ids {
                        if self.ran.contains(down)
                            || self.dirty.contains(down)
                            || self.check.contains(down)
                        {
                            continue;
                        }
                        self.check.insert(*down);
                        self.push_heap(*down, cite_rels);
                        walk_stack.push(*down);
                    }
                }
            }
        }
    }

    fn push_heap(&mut self, citer_id: StateId, cite_rels: &CiteRels) {
        let depth = cite_rels
            .citer_depths
            .get(&citer_id)
            .copied()
            .unwrap_or_default();
        self.heap.push(Item { citer_id, depth });
    }

    /// Pop the shallowest still-marked citer. Entries whose citer was already
    /// settled through a pull are skipped lazily.
    fn pop_marked(&mut self) -> Option<StateId> {
        while let Some(item) = self.heap.pop() {
            if self.dirty.contains(&item.citer_id) || self.check.contains(&item.citer_id) {
                return Some(item.citer_id);
            }
        }
        None
    }

    /// Reset at the end of a top-level batch (notably `ran`, which must not
    /// leak into the next batch).
    fn clear(&mut self) {
        self.heap.clear();
        self.dirty.clear();
        self.check.clear();
        self.ran.clear();
    }
}
