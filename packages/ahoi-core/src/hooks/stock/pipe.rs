use super::*;

// Common trait for Pipes
pub trait Pipeline<Out>: Clone {
    fn map_ref<'a>(&self, value: &'a dyn Any) -> Option<&'a Out>;
    fn map_mut<'a>(&self, value: &'a mut dyn Any) -> Option<&'a mut Out>;
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
    fn map_ref<'a>(&self, value: &'a dyn Any) -> Option<&'a T> {
        match self.pooled_id {
            None => value.downcast_ref::<T>(),
            Some(id) => {
                let state = states::pool::get_state(id)?;
                let value = state.as_mapper()?.map_ref(value)?.downcast_ref::<T>()?;
                return Some(value);
            }
        }
    }
    fn map_mut<'a>(&self, value: &'a mut dyn Any) -> Option<&'a mut T> {
        match self.pooled_id {
            None => value.downcast_mut::<T>(),
            Some(id) => {
                let state = states::pool::get_state(id)?;
                let value = state.as_mapper()?.map_mut(value)?.downcast_mut::<T>()?;
                return Some(value);
            }
        }
    }
}

// Helper of Chained Pipe

pub trait MapNext<T, S>: Clone {
    fn as_ref<'a>(&self, value: &'a T) -> Option<&'a S>;
    fn as_mut<'a>(&self, value: &'a mut T) -> Option<&'a mut S>;
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
    Next: MapNext<T, S>,
{
    fn map_ref<'a>(&self, value: &'a dyn Any) -> Option<&'a S> {
        let t_ref = self.prev.map_ref(value)?;
        self.next.as_ref(t_ref)
    }

    fn map_mut<'a>(&self, value: &'a mut dyn Any) -> Option<&'a mut S> {
        let t_mut = self.prev.map_mut(value)?;
        self.next.as_mut(t_mut)
    }
}

// impl Abstract Mapper Trait for ChainedPipe (for pooling into PooledPipe)
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static> states::Mapper
    for ChainedPipe<Prev, Next, T, U>
where
    Prev: Pipeline<T>,
    Next: MapNext<T, U>,
{
    fn map_ref<'a>(&self, source: &'a dyn Any) -> Option<&'a dyn Any> {
        <Self as Pipeline<U>>::map_ref(self, source).map(|e| e as &dyn Any)
    }

    fn map_mut<'a>(&self, source: &'a mut dyn Any) -> Option<&'a mut dyn Any> {
        <Self as Pipeline<U>>::map_mut(self, source).map(|e| e as &mut dyn Any)
    }
}

// Chained Pipe Into Pooled Pipe
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static> ChainedPipe<Prev, Next, T, U>
where
    Prev: Pipeline<T>,
    Next: MapNext<T, U>,
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
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static, const OPT: bool>
    ReadStock<U, ChainedPipe<Prev, Next, T, U>, OPT>
where
    Prev: Pipeline<T>,
    Next: MapNext<T, U>,
{
    /// Materialize this chained read-stock into a pooled one: its pipeline is
    /// stored as a state, making the handle `Copy`. See [`Stock::pool`].
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn pool(self) -> ReadStock<U, PooledPipe<U>, OPT> {
        let pipeline = self.pipeline.pool();
        return ReadStock {
            value_id: self.value_id,
            path: self.path,
            pipeline,
            ty: PhantomData,
            associated_citer_id: None,
        };
    }
}

impl<T: 'static, U: 'static, Prev: 'static, Next: 'static, const OPT: bool>
    Stock<U, ChainedPipe<Prev, Next, T, U>, OPT>
where
    Prev: Pipeline<T>,
    Next: MapNext<T, U>,
{
    /// Materialize this chained stock into a pooled one: its pipeline is stored
    /// as a state, so the handle becomes `Copy` and the `Pipe` generic collapses
    /// to `PooledPipe<U>` (handy for context values). Otherwise prefer the
    /// chained form — it costs zero pool state.
    ///
    /// Note: pooling allocates a mapper state that lives until the sphere is
    /// cleared, so do NOT call `.pool()` inside a reactive closure (it would leak
    /// one mapper per run) — derive inline there instead.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn pool(self) -> Stock<U, PooledPipe<U>, OPT> {
        Stock(self.0.pool())
    }
}

// -----

// Pooled ReadStock From Chained ReadStock Into
//
// NOTE: `From::from` is not declared `#[track_caller]` upstream, so the
// attribute on these impls only helps for statically-resolved calls. When the
// recorded creation site matters, prefer `.pool()` — it is an inherent method
// and carries the caller reliably.
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static, const OPT: bool>
    From<ReadStock<U, ChainedPipe<Prev, Next, T, U>, OPT>> for ReadStock<U, PooledPipe<U>, OPT>
where
    Prev: Pipeline<T>,
    Next: MapNext<T, U>,
{
    #[cfg_attr(debug_assertions, track_caller)]
    fn from(stock: ReadStock<U, ChainedPipe<Prev, Next, T, U>, OPT>) -> Self {
        stock.pool()
    }
}

// Pooled Stock From Chained Stock
impl<T: 'static, U: 'static, Prev: 'static, Next: 'static, const OPT: bool>
    From<Stock<U, ChainedPipe<Prev, Next, T, U>, OPT>> for Stock<U, PooledPipe<U>, OPT>
where
    Prev: Pipeline<T>,
    Next: MapNext<T, U>,
{
    #[cfg_attr(debug_assertions, track_caller)]
    fn from(stock: Stock<U, ChainedPipe<Prev, Next, T, U>, OPT>) -> Self {
        stock.pool()
    }
}
