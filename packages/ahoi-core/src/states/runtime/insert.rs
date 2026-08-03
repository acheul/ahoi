use super::*;

pub(crate) fn insert_value_state<T: 'static>(value: T) -> StateId {
    let id = pool::insert_state(State::Value(Box::new(value)));
    // register to building sphere
    runtime::sphere::register_value_to_current_sphere(id);
    id
}

pub(crate) fn insert_mapper_state(getter: Box<dyn Mapper>) -> StateId {
    let id = pool::insert_state(State::Mapper(getter));
    // register to building sphere
    runtime::sphere::register_mapper_to_current_sphere(id);
    id
}

fn insert_runner_state(runner: Runner) -> StateId {
    let id = pool::insert_state(State::Runner(runner));
    // register to building sphere
    runtime::sphere::register_runner_to_current_sphere(id);
    return id;
}

pub(crate) fn insert_citer_runner_state(f: impl Fn() + 'static) -> StateId {
    return insert_runner_state(Runner::Citer {
        runner: Box::new(f),
        is_hail_sender: false,
    });
}

/// Replace the closure of an already-inserted citer runner.
/// * Used when the runner can only be built after its id (and other state) exist.
pub(crate) fn replace_citer_runner_state(id: StateId, f: impl Fn() + 'static) {
    let mut state = pool::get_mut_state(id).unwrap();
    *state = State::Runner(Runner::Citer {
        runner: Box::new(f),
        is_hail_sender: false,
    });
}

/// Register `value_id` as an associated stock (output) of `citer_id`.
/// * Needed by propagation's mark/pull phases to walk downstream and to find
///   a citer's producers.
pub(crate) fn register_citer_output(citer_id: StateId, value_id: StateId) {
    RUNTIME.with_borrow_mut(|runtime| {
        runtime.cite_rels.register_citer_output(citer_id, value_id);
    });
}

pub(crate) fn insert_hail_citer_runner_state(
    hail_src_value: (StateId, Path),
    f: impl Fn() + 'static,
) -> StateId {
    // 1. insert
    let citer_id = insert_runner_state(Runner::Citer {
        runner: Box::new(f),
        is_hail_sender: true,
    });

    // 2. Set a fixed cite-rel (for hail citer, "cite" will not update rels)
    RUNTIME.with_borrow_mut(|runtime| {
        runtime
            .cite_rels
            .set_hail_cite_rel(citer_id, hail_src_value)
    });

    return citer_id;
}

pub(crate) fn insert_executer_runner_state<A: 'static, R: 'static>(
    f: impl Fn(A) -> R + 'static,
) -> StateId {
    let run_fn = move |args: Box<dyn Any>| {
        let args = *args.downcast::<A>().unwrap();
        let res = f(args);
        let res: Box<dyn Any> = Box::new(res);
        res
    };

    return insert_runner_state(Runner::Executer(Box::new(run_fn)));
}
