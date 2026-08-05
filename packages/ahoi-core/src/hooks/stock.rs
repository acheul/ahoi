use super::*;

mod opt;
pub use opt::{OptReadStock, OptStock};

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
pub struct ReadStock<T, Pipe = PooledPipe<T>>(OptStock<T, Pipe>);

impl<T, Pipe: Clone> Clone for ReadStock<T, Pipe> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, Pipe: Copy> Copy for ReadStock<T, Pipe> {}

/// Stock
pub struct Stock<T, Pipe = PooledPipe<T>>(pub(crate) ReadStock<T, Pipe>);

impl<T, Pipe: Clone> Clone for Stock<T, Pipe> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, Pipe: Copy> Copy for Stock<T, Pipe> {}

// Deref Stock -> ReadStock
impl<T, Pipe> Deref for Stock<T, Pipe> {
    type Target = ReadStock<T, Pipe>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// New ReadStock
impl<T: 'static> ReadStock<T, PooledPipe<T>> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new(value: T) -> Self {
        let value_id = runtime::insert::insert_value_state(value);
        Self(OptStock(OptReadStock {
            value_id,
            path: Path::new_empty(),
            pipeline: PooledPipe::initial(),
            ty: PhantomData,
            associated_citer_id: None,
        }))
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
        stock.0.0.associated_citer_id.replace(citer_id);
        runtime::insert::register_citer_output(citer_id, stock.0.0.value_id);
        Self(stock)
    }
}

// Manually mark dirty
impl<T, Pipe> Stock<T, Pipe> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn mark_dirty(&self) {
        self.0.0.mark_dirty()
    }
}

// Borrow methods (ReadStock)
impl<T, Pipe: Pipeline<T>> ReadStock<T, Pipe> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_peek(&self) -> Result<Ref<'static, T>, BorrowError> {
        Ok(self.0.0.try_peek()?.unwrap())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_read(&self) -> Result<Ref<'static, T>, BorrowError> {
        Ok(self.0.0.try_read()?.unwrap())
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

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_read_hail<X: HailConverter<T> + 'static>(self) -> X::HailValue
    where
        T: 'static,
        Pipe: 'static,
    {
        hail::set_read_hail::<X, T, Pipe>(self.0.0, false)
    }
}

// BorrowMut methods (Stock)
impl<T, Pipe: Pipeline<T>> Stock<T, Pipe> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_write(&self) -> Result<RefMut<'static, T>, BorrowError> {
        Ok(self.0.0.try_write()?.unwrap())
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
        Ok(self.0.0.try_set(value)?.unwrap())
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set(&self, value: T) -> ()
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
        hail::set_hail::<X, T, Pipe>(self.0.0, false)
    }
}
