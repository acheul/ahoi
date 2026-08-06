use super::*;

mod opt;
pub use opt::{OptReadStock, OptStock};

mod pipe;
pub use pipe::*;

mod derive;
pub use derive::*;

// basic stock structs

/// The read-only, always-present stock handle: the borrow methods of [`Stock`]
/// minus the writing ones. See [`Stock`] for the full type table.
pub struct ReadStock<T, Pipe = PooledPipe<T>>(OptStock<T, Pipe>);

impl<T, Pipe: Clone> Clone for ReadStock<T, Pipe> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, Pipe: Copy> Copy for ReadStock<T, Pipe> {}

/// A handle to a reactive value — or a value derived from one — living in the
/// state pool.
///
/// Four concrete types cover the capability × presence matrix:
///
/// | type             | writable | may be absent |
/// |------------------|:--------:|:-------------:|
/// | [`Stock`]        | yes      | no            |
/// | [`ReadStock`]    | no       | no            |
/// | [`OptStock`]     | yes      | yes           |
/// | [`OptReadStock`] | no       | yes           |
///
/// - "May be absent" marks values that can be missing — a `Vec` index, a map
///   key, an optional derive, or an enum-variant field. The `Opt*` types wrap
///   their borrow results in `Option`: `None` means the value is genuinely
///   absent, which is not an error.
/// - Every borrow method (`read`, `peek`, `write`, `set`) has a `try_*` twin
///   returning `Result<_, BorrowError>`: `Err(Disposed)` when the state has
///   been cleared from the pool (e.g. an async callback outliving its sphere),
///   `Err(BorrowConflict)` when a live guard conflicts. The non-`try` methods
///   are the `try_*` twins with the error unwrapped — they panic instead.
/// - `Pipe` is the getter pipeline mapping the root value to this stock's
///   value. It defaults to [`PooledPipe`] once the stock is materialized via
///   `.pool()`.
///
/// A stock handle is cheap to copy: it is [`Copy`] when `Pipe` is, and
/// [`Clone`] otherwise.
///
/// #### Pooled vs. Chained
/// - A Pooled stock has [`PooledPipe`] as its pipeline; a Chained one has
///   [`ChainedPipe`].
/// - An initially created stock is always pooled, while a derived stock created
///   by the [`Derivable`] methods (`derive`, `derive_opt`) is chained.
/// - A chained stock can be pooled by the [`Poolable::pool`] method.
///
/// ##### What to use?
/// - A pooled stock handle is always `Copy`, and its `Pipe` generic can be
///   omitted (it defaults to [`PooledPipe`]).
/// - A chained stock's pipeline is usually very light — zero or near-zero cost.
/// - It's fine (and better) to use a chained stock without pooling. However,
///   for Context ([provide_context], [use_context]), a pooled stock is more
///   convenient because it can omit the `Pipe` generic.
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

/// Panic message for the non-optional invariant: a non-`Opt` stock's pipeline
/// must never miss. It can only trip when a hand-written derive accessor
/// declared as non-optional actually returns `None`.
const NON_OPT_INVARIANT: &str =
    "non-optional stock resolved to an absent value — inconsistent derive accessor?";

// Borrow methods (ReadStock)
impl<T, Pipe: Pipeline<T>> ReadStock<T, Pipe> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_peek(&self) -> Result<Ref<'static, T>, BorrowError> {
        Ok(self.0.0.try_peek()?.expect(NON_OPT_INVARIANT))
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn try_read(&self) -> Result<Ref<'static, T>, BorrowError> {
        Ok(self.0.0.try_read()?.expect(NON_OPT_INVARIANT))
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
        Ok(self.0.0.try_write()?.expect(NON_OPT_INVARIANT))
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
        Ok(self.0.0.try_set(value)?.expect(NON_OPT_INVARIANT))
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
