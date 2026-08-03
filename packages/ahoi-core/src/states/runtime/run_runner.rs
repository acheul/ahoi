use super::*;

pub(crate) fn run_citer_with<R>(citer_id: StateId, run: impl FnOnce() -> R) -> R {
    runtime::propagation::batch(|| runtime::citation::cite(citer_id, run))
}

/// This will panic if the runner has recursive dependency.
pub(crate) fn run_citer(id: StateId) -> Option<()> {
    let state = pool::get_state(id)?;
    match state.as_runner()? {
        Runner::Citer {
            runner: run,
            is_hail_sender,
        } => {
            let res = if *is_hail_sender {
                // not using cite for hail-citer
                runtime::propagation::batch(|| run())
            } else {
                // run in batch & cite
                run_citer_with(id, || run())
            };
            Some(res)
        }
        _ => None,
    }
}

/// This will panic if the runner has recursive dependency.
pub(crate) fn run_executer<A: 'static, R: 'static>(id: StateId, args: A) -> Option<R> {
    let state = pool::get_state(id)?;
    match state.as_runner()? {
        Runner::Executer(run) => {
            // run in batch
            let res = runtime::propagation::batch(|| run(Box::new(args)));
            Some(*res.downcast::<R>().unwrap())
        }
        _ => None,
    }
}
