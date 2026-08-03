use super::*;

/// Callback (executer)
pub struct Callback<A, R> {
    pub(crate) executer_id: StateId,
    ty: PhantomData<(A, R)>,
}

impl<A, R> Clone for Callback<A, R> {
    fn clone(&self) -> Self {
        Self {
            executer_id: self.executer_id,
            ty: self.ty,
        }
    }
}

impl<A, R> Copy for Callback<A, R> {}

impl<A: 'static, R: 'static> Callback<A, R> {
    pub fn new(runner: impl Fn(A) -> R + 'static) -> Self {
        let executer_id = runtime::insert::insert_executer_runner_state(runner);
        return Self {
            executer_id,
            ty: PhantomData,
        };
    }

    pub fn call(&self, args: A) -> R {
        let res = runtime::run_runner::run_executer::<A, R>(self.executer_id, args)
            .expect("Callback is called after sphere cleared");
        return res;
    }
}
