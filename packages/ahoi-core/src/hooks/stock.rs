use super::*;

mod pipe;
pub use pipe::*;

mod derive;
pub use derive::*;

// basic stock structs

/// A handle to a reactive value — or a value derived from one — living in the
/// state pool.
///
/// The two const flags are specialized by the type aliases below; in practice
/// you work with those names rather than `Stock` directly:
///
/// | alias            | `MUT` | `OPT` | meaning                   |
/// |------------------|:-----:|:-----:|---------------------------|
/// | [`Stock`](type@Stock) | true  | false | writable, always present  |
/// | [`RefStock`]     | false | false | read-only, always present |
/// | [`OptStock`]     | true  | true  | writable, may be absent   |
/// | [`RefOptStock`]  | false | true  | read-only, may be absent  |
///
/// - `MUT` gates the mutating API (`read_mut`, `peek_mut`, `set`, …).
/// - `OPT` marks values that can be missing — a `Vec` index, a map key, an
///   optional derive, or an enum-variant field. `OPT` stocks expose the `try_*`
///   accessors instead of the panicking `peek`/`read`.
/// - `P` is the getter pipeline mapping the root value to this stock's value.
///   It defaults to [`PooledPipe`] once the stock is materialized via `.pool()`.
///
/// A `Stock` is cheap to copy: it is [`Copy`] when `P` is, and [`Clone`]
/// otherwise.
///
/// #### Pooled vs. Chained
/// - Pooled Stock has [PooledPipe] as its pipeline, while Chained has [ChainedMap].
/// - An initially created Stock is always a pooled stock, while a derived stock
///   created by [Stock::derive] or [Stock::derive_option] methods is a chained.
/// - A chained stock can be pooled by [Stock::pool] method.
///
/// ##### What to use?
/// - Pooled Stock hook is always Copy-able, and the generic `P` can be omitted
///   for [PooledPipe]
/// - Chained Stock's pipeline is usually very light and requires zero or very
///   little cost.
/// - It's ok (and better) to use Chained Stock without pooling.
///   - However, for Context ([provide_context], [use_context]), Pooled Stock will
///     be more convenient to use because it can omit `P` generic.

// ReadStock
pub struct ReadStock<T, Pipe = PooledPipe<T>, const OPT: bool = false> {
    pub(super) value_id: StateId,
    pub(super) path: Path,
    pipeline: Pipe,
    ty: PhantomData<T>,
    associated_citer_id: Option<StateId>,
}

pub type OptReadStock<T, Pipe = PooledPipe<T>> = ReadStock<T, Pipe, true>;

impl<T, Pipe: Clone, const OPT: bool> Clone for ReadStock<T, Pipe, OPT> {
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

impl<T, Pipe: Copy, const OPT: bool> Copy for ReadStock<T, Pipe, OPT> {}

// Stock
pub struct Stock<T, Pipe = PooledPipe<T>, const OPT: bool = false>(
    pub(crate) ReadStock<T, Pipe, OPT>,
);

pub type OptStock<T, Pipe = PooledPipe<T>> = Stock<T, Pipe, true>;

impl<T, Pipe: Clone, const OPT: bool> Clone for Stock<T, Pipe, OPT> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, Pipe: Copy, const OPT: bool> Copy for Stock<T, Pipe, OPT> {}

// Deref Stock -> ReadStock
impl<T, Pipe, const OPT: bool> Deref for Stock<T, Pipe, OPT> {
    type Target = ReadStock<T, Pipe, OPT>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// New ReadStock
impl<T: 'static> ReadStock<T, PooledPipe<T>, false> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new(value: T) -> Self {
        let value_id = runtime::insert::insert_value_state(value);
        Self {
            value_id,
            path: Path::new_empty(),
            pipeline: PooledPipe::initial(),
            ty: PhantomData,
            associated_citer_id: None,
        }
    }
}

/// New Stock
impl<T: 'static> Stock<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new(value: T) -> Self {
        Self(ReadStock::new(value))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub(super) fn new_citer_associated_stock(value: T, citer_id: StateId) -> Self {
        let mut stock = ReadStock::new(value);
        stock.associated_citer_id.replace(citer_id);
        runtime::insert::register_citer_output(citer_id, stock.value_id);
        Self(stock)
    }
}

// Manually mark dirty
impl<T, Pipe, const OPT: bool> Stock<T, Pipe, OPT> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn mark_dirty(&self) {
        runtime::propagation::mark_dirty(self.value_id, self.path)
    }
}

// Raw Borrow methods (ReadStock)
impl<T, Pipe: Pipeline<T>, const OPT: bool> ReadStock<T, Pipe, OPT> {
    /// `track_caller` here is not about the `Option` result — it is so a
    /// `BorrowError` raised down in `pool::get_state` names the user's read.
    #[cfg_attr(debug_assertions, track_caller)]
    pub(crate) fn raw_peek(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
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
    pub(crate) fn raw_read(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
        // 1. pull: settle the producing citer first so the value is fresh.
        // * Must run before `try_peek` — the producer's runner writes this very
        //   value slot, so no borrow of it may be held while it runs.
        if let Some(associated_citer_id) = self.associated_citer_id {
            states::runtime::propagation::ensure_citer_fresh(associated_citer_id);
        }
        // 2. peek value
        let value = self.raw_peek();
        // 3. mark cited — always, even when value is None.
        states::runtime::citation::mark_cited(self.value_id, self.path, self.associated_citer_id);
        return value;
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_read_hail<X: HailConverter<T> + 'static>(self) -> X::HailValue
    where
        T: 'static,
        Pipe: 'static,
    {
        hail::set_read_hail::<X, T, Pipe, OPT>(self)
    }
}

// Raw BorrowMut methods (Stock)
impl<T, Pipe: Pipeline<T>, const OPT: bool> Stock<T, Pipe, OPT> {
    /// See [`ReadStock::try_peek`]: carries the caller down to the `RefCell`
    /// borrow so a `BorrowMutError` blames the user's write.
    #[cfg_attr(debug_assertions, track_caller)]
    fn raw_write_silent(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
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
    pub(crate) fn raw_write(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
        // 1. peek_mut value
        let value = match self.raw_write_silent() {
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
    fn raw_set(&self, value: T) -> Result<Option<()>, BorrowError>
    where
        T: 'static,
    {
        let Some(mut value_) = self.raw_write()? else {
            return Ok(None);
        };
        *value_ = value;
        Ok(Some(()))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_hail<X: HailConverter<T> + 'static>(self) -> X::HailValue
    where
        T: 'static,
        Pipe: 'static,
    {
        hail::set_hail::<X, T, Pipe, OPT>(self)
    }
}

// non-OPT ReadStock
impl<T, Pipe: Pipeline<T>> ReadStock<T, Pipe, false> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_peek(&self) -> Result<Ref<'static, T>, BorrowError> {
        Ok(self.raw_peek()?.unwrap())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_read(&self) -> Result<Ref<'static, T>, BorrowError> {
        Ok(self.raw_read()?.unwrap())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn peek(&self) -> Ref<'static, T> {
        self.try_peek().unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn read(&self) -> Ref<'static, T> {
        self.try_read().unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn memo<U: PartialEq + 'static>(self, runner: impl Fn(&T) -> U + 'static) -> Memo<U>
    where
        T: 'static,
        Pipe: 'static,
    {
        let runner = move || runner(&*self.read());
        Memo::new(runner)
    }
}

// non-OPT Stock
impl<T, Pipe: Pipeline<T>> Stock<T, Pipe, false> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_write(&self) -> Result<RefMut<'static, T>, BorrowError> {
        Ok(self.raw_write()?.unwrap())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn write(&self) -> RefMut<'static, T> {
        self.try_write().unwrap()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_set(&self, value: T) -> Result<(), BorrowError>
    where
        T: 'static,
    {
        Ok(self.raw_set(value)?.unwrap())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(&self, value: T) -> ()
    where
        T: 'static,
    {
        self.try_set(value).unwrap()
    }
}

// OPT ReadStock
impl<T, Pipe: Pipeline<T>> ReadStock<T, Pipe, true> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_peek(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
        self.raw_peek()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_read(&self) -> Result<Option<Ref<'static, T>>, BorrowError> {
        self.raw_read()
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
}

// OPT Stock
impl<T, Pipe: Pipeline<T>> Stock<T, Pipe, true> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_write(&self) -> Result<Option<RefMut<'static, T>>, BorrowError> {
        self.raw_write()
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
        self.raw_set(value)
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(self, value: T) -> Option<()>
    where
        T: 'static,
    {
        self.try_set(value).unwrap()
    }
}
