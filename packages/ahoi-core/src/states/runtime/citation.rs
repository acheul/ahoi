//! Reactive Context of Ahoi: Citation
use super::*;

/// <citer-id, collect <cited-value-id, {path}>>
#[derive(Default)]
pub(super) struct RunningCites(pub(super) IntIndexMap<StateId, IntMap<StateId, HashSet<Path>>>);

pub(crate) fn mark_cited(value_id: StateId, path: Path, associated_citer_id: Option<StateId>) {
    debug_assert!(if associated_citer_id.is_some() {
        path.is_empty()
    } else {
        true
    });

    RUNTIME.with_borrow_mut(|runtime| {
        if let Some((citer_id, citeds)) = runtime.running_cites.0.last_mut() {
            let _ = citeds.entry(value_id).or_default().insert(path);

            // update depth
            if let Some(associated_citer_id) = associated_citer_id {
                let _ = runtime
                    .cite_rels
                    .update_depth(*citer_id, associated_citer_id);
            }
            return;
        }
        // TODO: warnning for out-of-cite-read?
    });
}

fn raw_cite<R>(citer_id: StateId, run: impl FnOnce() -> R, replace_or_accumulate_rels: bool) -> R {
    // 1. Run
    // 1) Set running Cite
    RUNTIME.with_borrow_mut(|runtime| {
        // A cycle is detected deep inside propagation, so the caller here is
        // runtime code, not user code — blame the citer's creation site instead.
        let origin = runtime.location_of(&citer_id);

        match runtime.running_cites.0.entry(citer_id) {
            indexmap::map::Entry::Occupied(_) => {
                panic_at!(origin, "cite cycle detected");
            }
            indexmap::map::Entry::Vacant(v) => {
                v.insert(Default::default());
            }
        }
    });

    // 2) run
    // Re-entrant runs of the same citer are blocked by the entry guard above.
    let res = run();

    // 3) update citer rels
    RUNTIME.with_borrow_mut(|runtime| {
        let (popped_citer_id, cited_value_to_paths) = runtime.running_cites.0.pop().unwrap();
        debug_assert_eq!(popped_citer_id, citer_id);
        runtime.cite_rels.update_rels_by_citer(
            citer_id,
            cited_value_to_paths,
            replace_or_accumulate_rels,
        );
    });

    return res;
}

pub(crate) fn cite<R>(citer_id: StateId, run: impl FnOnce() -> R) -> R {
    raw_cite(citer_id, run, true)
}

pub(crate) fn cite_accumulate<R>(citer_id: StateId, run: impl FnOnce() -> R) -> R {
    raw_cite(citer_id, run, false)
}

#[derive(Default)]
pub(super) struct CiteRels {
    /// <value-id, <path, {citer-id}>>
    pub(super) value_rels: IntMap<StateId, HashMap<Path, IntSet<StateId>>>,
    /// <citer-id, <cited-value-id, {path}>>
    citer_rels: IntMap<StateId, IntMap<StateId, HashSet<Path>>>,
    /// When "an associated stock" of a citer(Producer) is cited by other citer(Consumer),
    /// allocate Procuder's depth plus one to Consumer's depth.
    /// * Since pull-on-read, depth is a scheduling heuristic (it keeps pull
    ///   recursion shallow), not a correctness requirement.
    pub(crate) citer_depths: IntMap<StateId, usize>,
    /// <citer-id, [value-ids of its associated stocks]> (memo: 1, resource: 2)
    pub(super) citer_outputs: IntMap<StateId, Vec<StateId>>,
    /// inverse of `citer_outputs`: <value-id, associated citer-id>
    pub(super) value_owners: IntMap<StateId, StateId>,
}

impl CiteRels {
    pub(crate) fn remove_rels_by_citer(&mut self, citer_id: &StateId) {
        if let Some(value_to_paths) = self.citer_rels.remove(citer_id) {
            let _ = Self::help_remove_value_rels_by(
                &mut self.value_rels,
                citer_id,
                value_to_paths,
                true,
            );
        }
        let _ = self.citer_depths.remove(citer_id);
        if let Some(out_values) = self.citer_outputs.remove(citer_id) {
            for out_value in out_values {
                let _ = self.value_owners.remove(&out_value);
            }
        }
    }

    pub(crate) fn register_citer_output(&mut self, citer_id: StateId, value_id: StateId) {
        self.citer_outputs
            .entry(citer_id)
            .or_default()
            .push(value_id);
        self.value_owners.insert(value_id, citer_id);
    }

    /// Producers of `citer_id`: the associated citers of the values it cited on
    /// its last run. A citer's staleness can only flow in through those values.
    pub(crate) fn producers_of(&self, citer_id: &StateId) -> smallvec::SmallVec<[StateId; 2]> {
        let Some(cited_value_to_paths) = self.citer_rels.get(citer_id) else {
            return smallvec::smallvec![];
        };
        cited_value_to_paths
            .keys()
            .filter_map(|value_id| self.value_owners.get(value_id))
            .filter(|owner| *owner != citer_id)
            .copied()
            .collect()
    }

    fn help_remove_value_rels_by(
        value_rels: &mut IntMap<StateId, HashMap<Path, IntSet<StateId>>>,
        citer_id: &StateId,
        cited_value_to_paths: IntMap<StateId, HashSet<Path>>,
        remove_empty_value_id: bool,
    ) {
        for (value_id, paths) in cited_value_to_paths {
            if let Some(path_to_citers) = value_rels.get_mut(&value_id) {
                for path in paths {
                    if let Some(citers) = path_to_citers.get_mut(&path) {
                        citers.remove(citer_id);
                        if citers.is_empty() {
                            // not using "remove_entry" option here: as "path" can be replaced frequently.
                            path_to_citers.remove(&path);
                        }
                    }
                }
                if remove_empty_value_id && path_to_citers.is_empty() {
                    value_rels.remove(&value_id);
                }
            }
        }
    }

    fn update_depth(&mut self, current_citer: StateId, associated_citer_of_cited_value: StateId) {
        if associated_citer_of_cited_value != current_citer {
            let depth0 = self
                .citer_depths
                .get(&associated_citer_of_cited_value)
                .copied()
                .unwrap_or_default();
            let depth = self.citer_depths.entry(current_citer).or_default();
            *depth = (*depth).max(depth0 + 1);
        }
    }

    fn update_rels_by_citer(
        &mut self,
        citer_id: StateId,
        cited_value_to_paths: IntMap<StateId, HashSet<Path>>,
        replace_old: bool,
    ) {
        // On citer_rels:
        let cur = self.citer_rels.entry(citer_id).or_default();
        let old = std::mem::replace(cur, cited_value_to_paths);

        // On value_rels:

        // replace old
        if replace_old {
            // `remove_emtpy` = false (for efficiency of following "add new" process)
            let _ = Self::help_remove_value_rels_by(&mut self.value_rels, &citer_id, old, false);
        }
        // add new
        for (value_id, paths) in cur.iter() {
            let path_to_citers = self.value_rels.entry(*value_id).or_default();
            for path in paths {
                let _ = path_to_citers.entry(*path).or_default().insert(citer_id);
            }
        }
    }

    pub(crate) fn set_hail_cite_rel(
        &mut self,
        hail_citer_id: StateId,
        hail_src_value: (StateId, Path),
    ) {
        let (value_id, path) = hail_src_value;
        self.value_rels
            .entry(value_id)
            .or_default()
            .entry(path)
            .or_default()
            .insert(hail_citer_id);
        self.citer_rels
            .entry(hail_citer_id)
            .or_default()
            .entry(value_id)
            .or_default()
            .insert(path);
    }
}
