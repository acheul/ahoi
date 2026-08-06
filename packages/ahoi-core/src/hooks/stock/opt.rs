use super::*;

/// The read-only, may-be-absent stock handle: borrow methods return `Option` —
/// `None` means the value is genuinely absent, which is not an error. See
/// [`Stock`] for the full type table.
pub struct OptReadStock<T, Pipe = PooledPipe<T>> {
    pub(crate) value_id: StateId,
    pub(crate) path: Path,
    pub(super) pipeline: Pipe,
    pub(super) ty: PhantomData<T>,
    pub(super) associated_citer_id: Option<StateId>,
}

impl<T, Pipe: Clone> Clone for OptReadStock<T, Pipe> {
    fn clone(&self) -> Self {
        Self {
            value_id: self.value_id,
            path: self.path,
            pipeline: self.pipeline.clone(),
            ty: PhantomData,
            associated_citer_id: self.associated_citer_id,
        }
    }
}

impl<T, Pipe: Copy> Copy for OptReadStock<T, Pipe> {}

/// The writable, may-be-absent stock handle. See [`OptReadStock`] for the
/// `Option` semantics of the borrow methods, and [`Stock`] for the full type
/// table.
pub struct OptStock<T, Pipe = PooledPipe<T>>(pub(crate) OptReadStock<T, Pipe>);

impl<T, Pipe: Clone> Clone for OptStock<T, Pipe> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, Pipe: Copy> Copy for OptStock<T, Pipe> {}

// Deref OptStock -> ReadOptStock
impl<T, Pipe> Deref for OptStock<T, Pipe> {
    type Target = OptReadStock<T, Pipe>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Manually mark dirty
impl<T, Pipe> OptStock<T, Pipe> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn mark_dirty(&self) {
        runtime::propagation::mark_dirty(self.value_id, self.path)
    }
}

// OptReadStock
impl<T, Pipe: Pipeline<T>> OptReadStock<T, Pipe> {
    /// `Ok(None)` = value absent (optional derive missed); `Err` = state
    /// disposed or borrow conflict — from the value state itself, or from a
    /// pooled mapper down the pipeline.
    pub fn try_peek(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
        let state = states::pool::get_state(self.value_id)?;
        // `filter_map`'s closure can only signal None, so a pipeline error is
        // carried out through this capture slot.
        let mut pipe_err = None;
        let value = Ref::filter_map(state, |state: &'_ states::State| {
            let src_value = state.as_value().unwrap().as_ref();
            match self.pipeline.map_ref(src_value) {
                Ok(value) => value,
                Err(err) => {
                    pipe_err = Some(err);
                    None
                }
            }
        })
        .ok();
        match pipe_err {
            Some(err) => Err(err),
            None => Ok(value),
        }
    }

    pub fn try_read(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
        // 1. pull: settle the producing citer first so the value is fresh.
        // * Must run before `try_peek` — the producer's runner writes this very
        //   value slot, so no borrow of it may be held while it runs.
        if let Some(associated_citer_id) = self.associated_citer_id {
            states::runtime::propagation::ensure_citer_fresh(associated_citer_id);
        }
        // 2. peek value
        let value = self.try_peek();
        // 3. mark cited — always, even when value is None.
        states::runtime::citation::mark_cited(self.value_id, self.path, self.associated_citer_id);
        return value;
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn peek(&self) -> Option<Ref<'static, T>> {
        self.try_peek().unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn read(&self) -> Option<Ref<'static, T>> {
        self.try_read().unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn memo<U: PartialEq + 'static>(self, runner: impl Fn(Option<&T>) -> U + 'static) -> Memo<U>
    where
        T: 'static,
        Pipe: 'static,
    {
        let runner = move || runner(self.read().as_deref());
        Memo::new(runner)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_read_hail<X: HailConverter<T> + 'static>(self) -> X::HailValue
    where
        T: 'static,
        Pipe: 'static,
    {
        hail::set_read_hail::<X, T, Pipe>(self, true)
    }
}

// OptStock
impl<T, Pipe: Pipeline<T>> OptStock<T, Pipe> {
    fn try_write_silent(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
        let state = states::pool::get_mut_state(self.value_id)?;
        // See `try_peek`: the capture slot carries a pipeline error out of the
        // `filter_map` closure.
        let mut pipe_err = None;
        let value = RefMut::filter_map(state, |state: &'_ mut states::State| {
            let src_value = state.as_mut_value().unwrap().as_mut();
            match self.pipeline.map_mut(src_value) {
                Ok(value) => value,
                Err(err) => {
                    pipe_err = Some(err);
                    None
                }
            }
        })
        .ok();
        match pipe_err {
            Some(err) => Err(err),
            None => Ok(value),
        }
    }

    /// `track_caller` here is not for this function's own result — it is so
    /// `propagation::mark_dirty`'s "Use batch" panic blames the user's write.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_write(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
        // 1. peek_mut value
        let value = match self.try_write_silent() {
            Ok(Some(v)) => v,
            x @ _ => return x,
        };
        // 2. mark dirty
        // * Unlike reads, do not mark dirty if try_write_silent returns Ok(None).
        // (To prevent spurious propagation)
        states::runtime::propagation::mark_dirty(self.value_id, self.path);
        return Ok(Some(value));
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn write(&self) -> Option<RefMut<'static, T>> {
        self.try_write().unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_set(&self, value: T) -> Result<Option<()>, BorrowError>
    where
        T: 'static,
    {
        let Some(mut value_) = self.try_write()? else {
            return Ok(None);
        };
        *value_ = value;
        Ok(Some(()))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(&self, value: T) -> Option<()>
    where
        T: 'static,
    {
        self.try_set(value).unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_hail<X: HailConverter<T> + 'static>(self) -> X::HailValue
    where
        T: 'static,
        Pipe: 'static,
    {
        hail::set_hail::<X, T, Pipe>(self, true)
    }
}
