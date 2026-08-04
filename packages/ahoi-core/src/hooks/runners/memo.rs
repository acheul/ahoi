use super::*;

pub struct Memo<T> {
    /// Citer runner Id
    citer_id: StateId,
    /// Backing stock. Always holds a value (seeded by the constructor's first
    /// run), so it is a plain `ReadStock<T>` rather than `ReadStock<Option<T>>`.
    stock: ReadStock<T>,
}

impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        Self {
            citer_id: self.citer_id,
            stock: self.stock,
        }
    }
}

impl<T> Copy for Memo<T> {}

impl<T: 'static> Memo<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new(runner: impl Fn() -> T + 'static) -> Self
    where
        T: PartialEq,
    {
        Self::new_with(runner, |x, y| x.eq(y))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new_with(
        runner: impl Fn() -> T + 'static,
        eq_checker: impl Fn(&T, &T) -> bool + 'static,
    ) -> Self {
        // 1. Reserve the citer id up front, so the first run can register its
        //    cite-rels under it.
        let citer_id = runtime::insert::insert_citer_runner_state(|| {});

        // 2. First run: compute the initial value AND register cite-rels in a
        //    single pass.
        let initial = runtime::run_runner::run_citer_with(citer_id, || runner());

        // 3. Back the memo with a plain `Stock<T>` seeded with that value.
        let stock = Stock::new_citer_associated_stock(initial, citer_id);

        // 4. Install the steady-state runner: recompute and write only on change.
        //    (`stock` is `Copy`, so the move closure copies it.)
        runtime::insert::replace_citer_runner_state(citer_id, move || {
            let res = runner();
            let unchanged = eq_checker(&*stock.peek(), &res);
            if !unchanged {
                *stock.try_write().unwrap() = res;
            }
        });

        return Self {
            citer_id,
            stock: *stock,
        };
    }

    pub fn peek(&self) -> Ref<'static, T> {
        self.stock.peek()
    }

    pub fn read(&self) -> Ref<'static, T> {
        self.stock.read()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_read_hail<X: HailConverter<T> + 'static>(self) -> X::HailValue {
        self.stock.set_read_hail::<X>()
    }
}

// Into ReadStock<T>
impl<T> Memo<T> {
    pub fn into_ref_stock(self) -> ReadStock<T> {
        self.stock
    }
}

impl<T> Into<ReadStock<T>> for Memo<T> {
    fn into(self) -> ReadStock<T> {
        self.into_ref_stock()
    }
}
