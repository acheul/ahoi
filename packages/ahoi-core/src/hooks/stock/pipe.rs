use super::*;

// Common trait for Pipes
/// * `Ok(None)` means the mapped value is absent (an optional derive missed).
/// * `Err` carries a `BorrowError` from a *pooled* mapper state's own access:
///   a pooled mapper can be disposed earlier than the value state it maps
///   (e.g. pooled in a child sphere over a parent's value).
pub trait Pipeline<Out>: Clone {
    fn map_ref<'a>(&self, value: &'a dyn Any) -> Result<Option<&'a Out>, BorrowError>;

    fn map_mut<'a>(&self, value: &'a mut dyn Any) -> Result<Option<&'a mut Out>, BorrowError>;
}

/// A Pooled Pipe. Copy implemented.
/// * If it's on a top stock, it's just a blank marker.
/// * If it's on a derived one, its getter is pooled as a state, and `pooled_id` is the getter's state id.
pub struct PooledPipe<T> {
    pub(super) pooled_id: Option<StateId>,
    phantom: PhantomData<T>,
}

impl<T> PooledPipe<T> {
    pub(super) fn initial() -> Self {
        Self {
            pooled_id: None,
            phantom: PhantomData,
        }
    }
}

impl<T> Clone for PooledPipe<T> {
    fn clone(&self) -> Self {
        Self {
            pooled_id: self.pooled_id,
            phantom: PhantomData,
        }
    }
}

impl<T> Copy for PooledPipe<T> {}

impl<T: 'static> Pipeline<T> for PooledPipe<T> {
    /// * `Err(Disposed)` is reachable: the pooled mapper state may be cleared
    ///   before the value state it maps (see [`Pipeline`]).
    fn map_ref<'a>(&self, value: &'a dyn Any) -> Result<Option<&'a T>, BorrowError> {
        // * downcast / `as_mapper` failures are type invariants of macro-generated
        //   code — those stay panics.
        match self.pooled_id {
            None => Ok(Some(value.downcast_ref::<T>().unwrap())),
            Some(id) => {
                let state = states::pool::get_state(id)?;
                let Some(mapped) = state.as_mapper().unwrap().map_ref(value)? else {
                    return Ok(None);
                };
                Ok(Some(mapped.downcast_ref::<T>().unwrap()))
            }
        }
    }
    fn map_mut<'a>(&self, value: &'a mut dyn Any) -> Result<Option<&'a mut T>, BorrowError> {
        match self.pooled_id {
            None => Ok(Some(value.downcast_mut::<T>().unwrap())),
            Some(id) => {
                let state = states::pool::get_state(id)?;
                let Some(mapped) = state.as_mapper().unwrap().map_mut(value)? else {
                    return Ok(None);
                };
                Ok(Some(mapped.downcast_mut::<T>().unwrap()))
            }
        }
    }
}

/// A Chained Pipeline (Merge Prev Pipeline and Next Pipeline).
///
/// Conditional Copy implemented.
pub struct ChainedPipe<Prev, Next, T, S> {
    pub(super) prev: Prev,
    pub(super) next: Next,
    pub(super) phantom: PhantomData<(T, S)>,
}

impl<Prev: Clone, Next: Clone, T, S> Clone for ChainedPipe<Prev, Next, T, S> {
    fn clone(&self) -> Self {
        Self {
            prev: self.prev.clone(),
            next: self.next.clone(),
            phantom: PhantomData,
        }
    }
}

impl<Prev: Copy, Next: Copy, T, S> Copy for ChainedPipe<Prev, Next, T, S> {}

impl<T: 'static, S: 'static, Prev: 'static, Next: 'static> Pipeline<S>
    for ChainedPipe<Prev, Next, T, S>
where
    Prev: Pipeline<T>,
    Next: MapNextOpt<T, S>,
{
    fn map_ref<'a>(&self, value: &'a dyn Any) -> Result<Option<&'a S>, BorrowError> {
        let Some(t_ref) = self.prev.map_ref(value)? else {
            return Ok(None);
        };
        Ok(self.next.as_ref(t_ref))
    }

    fn map_mut<'a>(&self, value: &'a mut dyn Any) -> Result<Option<&'a mut S>, BorrowError> {
        let Some(t_mut) = self.prev.map_mut(value)? else {
            return Ok(None);
        };
        Ok(self.next.as_mut(t_mut))
    }
}

// impl Abstract Mapper Trait for ChainedPipe (for pooling into PooledPipe)
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static> states::Mapper
    for ChainedPipe<Prev, Next, T, U>
where
    Prev: Pipeline<T>,
    Next: MapNextOpt<T, U>,
{
    fn map_ref<'a>(&self, source: &'a dyn Any) -> Result<Option<&'a dyn Any>, BorrowError> {
        Ok(<Self as Pipeline<U>>::map_ref(self, source)?.map(|e| e as &dyn Any))
    }

    fn map_mut<'a>(&self, source: &'a mut dyn Any) -> Result<Option<&'a mut dyn Any>, BorrowError> {
        Ok(<Self as Pipeline<U>>::map_mut(self, source)?.map(|e| e as &mut dyn Any))
    }
}

// Chained Pipe Into Pooled Pipe
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static> ChainedPipe<Prev, Next, T, U>
where
    Prev: Pipeline<T>,
    Next: MapNextOpt<T, U>,
{
    #[cfg_attr(debug_assertions, track_caller)]
    fn pool(self) -> PooledPipe<U> {
        let pooled_id = runtime::insert::insert_mapper_state(Box::new(self));
        PooledPipe {
            pooled_id: Some(pooled_id),
            phantom: PhantomData,
        }
    }
}

// Named pooling helpers — same as the `Into` impls above, but the method name
// makes the intent explicit and pins the target type, so callers don't need a
// type annotation to disambiguate from the reflexive `Into` identity.
pub trait Poolable {
    type PoolType;

    /// Materialize this chained read-stock into a pooled one: its pipeline is
    /// stored as a state, making the handle `Copy`. See [`Stock::pool`].
    fn pool(self) -> Self::PoolType;
}

impl<T: 'static, U: 'static, Prev: 'static, Next: 'static> Poolable
    for OptReadStock<U, ChainedPipe<Prev, Next, T, U>>
where
    Prev: Pipeline<T>,
    Next: MapNextOpt<T, U>,
{
    type PoolType = OptReadStock<U, PooledPipe<U>>;

    /// Materialize this chained stock into a pooled one: its pipeline is stored
    /// as a state, so the handle becomes `Copy` and the `Pipe` generic collapses
    /// to `PooledPipe<U>` (handy for context values). Otherwise prefer the
    /// chained form — it costs zero pool state.
    ///
    /// Note: pooling allocates a mapper state that lives until the sphere is
    /// cleared, so do NOT call `.pool()` inside a reactive closure (it would leak
    /// one mapper per run) — derive inline there instead.
    #[cfg_attr(debug_assertions, track_caller)]
    fn pool(self) -> OptReadStock<U, PooledPipe<U>> {
        let pipeline = self.pipeline.pool();
        return OptReadStock {
            value_id: self.value_id,
            path: self.path,
            pipeline,
            ty: PhantomData,
            // Carried over — see `Derivable::derive_opt`: dropping the pull
            // link here would silently break freshness of a pooled
            // derived-from-memo stock.
            associated_citer_id: self.associated_citer_id,
        };
    }
}

macro_rules! impl_poolable {
    ($ty:ident) => {
        impl<T: 'static, U: 'static, Prev: 'static, Next: 'static> Poolable
            for $ty<U, ChainedPipe<Prev, Next, T, U>>
        where
            Prev: Pipeline<T>,
            Next: MapNextOpt<T, U>,
        {
            type PoolType = $ty<U, PooledPipe<U>>;

            #[cfg_attr(debug_assertions, track_caller)]
            fn pool(self) -> $ty<U, PooledPipe<U>> {
                $ty(self.0.pool())
            }
        }
    };
}

impl_poolable!(OptStock);
impl_poolable!(ReadStock);
impl_poolable!(Stock);

// Pooled ReadStock From Chained ReadStock Into
//
// NOTE: `From::from` is not declared `#[track_caller]` upstream, so the
// attribute on these impls only helps for statically-resolved calls. When the
// recorded creation site matters, prefer `.pool()` — it is an inherent method
// and carries the caller reliably.
macro_rules! impl_pool_from {
    ($ty:ident) => {
        impl<T: 'static, U: 'static, Prev: 'static, Next: 'static>
            From<$ty<U, ChainedPipe<Prev, Next, T, U>>> for $ty<U, PooledPipe<U>>
        where
            Prev: Pipeline<T>,
            Next: MapNextOpt<T, U>,
        {
            #[cfg_attr(debug_assertions, track_caller)]
            fn from(stock: $ty<U, ChainedPipe<Prev, Next, T, U>>) -> Self {
                stock.pool()
            }
        }
    };
}

impl_pool_from!(OptReadStock);
impl_pool_from!(OptStock);
impl_pool_from!(ReadStock);
impl_pool_from!(Stock);
