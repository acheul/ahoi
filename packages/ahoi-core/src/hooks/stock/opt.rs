use super::*;

/// OptReadStock
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

/// OptStock
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
    /// `track_caller` here is not about the `Option` result — it is so a
    /// `BorrowError` raised down in `pool::get_state` names the user's read.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_peek(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
        let state = states::pool::get_state(self.value_id)?;
        let value = Ref::filter_map(state, |state: &'_ states::State| {
            let src_value = state.as_value().unwrap().as_ref();
            let value = self.pipeline.map_ref(src_value)?;
            Some(value)
        })
        .ok();
        Ok(value)
    }

    #[cfg_attr(debug_assertions, track_caller)]
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
    #[cfg_attr(debug_assertions, track_caller)]
    fn try_write_silent(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
        let state = states::pool::get_mut_state(self.value_id)?;
        let value = RefMut::filter_map(state, |state: &'_ mut states::State| {
            let src_value = state.as_mut_value().unwrap().as_mut();
            let value = self.pipeline.map_mut(src_value)?;
            Some(value)
        })
        .ok();
        Ok(value)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_write(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
        // 1. peek_mut value
        let value = match self.try_write_silent() {
            Ok(Some(v)) => v,
            x @ _ => return x,
        };
        // 2. mark dirty
        // * Unlike "read_option", do not mark dirty if raw_write_silent returns None.
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
    pub fn set(self, value: T) -> Option<()>
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
