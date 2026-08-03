use super::*;

pub struct Effect {
    citer_id: StateId,
}

impl Clone for Effect {
    fn clone(&self) -> Self {
        Self {
            citer_id: self.citer_id,
        }
    }
}
impl Copy for Effect {}

impl Effect {
    pub fn new(runner: impl Fn() -> () + 'static) -> Self {
        let citer_id = runtime::insert::insert_citer_runner_state(runner);
        let effect = Self { citer_id };

        // initial run
        runtime::run_runner::run_citer(effect.citer_id);
        return effect;
    }
}
