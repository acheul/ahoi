use super::*;

mod collections;

#[cfg(test)]
#[allow(unused)]
mod extensions;

// Helper of Chained Pipe

pub trait MapNext<T, S>: MapNextOpt<T, S> {
    fn as_ref<'a>(&self, value: &'a T) -> &'a S;
    fn as_mut<'a>(&self, value: &'a mut T) -> &'a mut S;
}

pub trait MapNextOpt<T, S>: Clone {
    fn as_ref<'a>(&self, value: &'a T) -> Option<&'a S>;
    fn as_mut<'a>(&self, value: &'a mut T) -> Option<&'a mut S>;
}

// derive methods

pub trait Derivable<T, Pipe> {
    /// Derived-stock type of a non-optional accessor: keeps `Self`'s
    /// capability row (Stock → Stock, ReadStock → ReadStock, Opt* → Opt*).
    type DeriveType<U, ChainedPipe>;

    /// Derived-stock type of an optional accessor: absence is absorbing, so
    /// this is always the `Opt*` counterpart of `Self`.
    type DeriveOptType<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> Self::DeriveType<U, ChainedPipe<Pipe, Next, T, U>>;

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> Self::DeriveOptType<U, ChainedPipe<Pipe, Next, T, U>>;
}

// OptReadStock — the base builder; the other three impls wrap this one.
impl<T, Pipe> Derivable<T, Pipe> for OptReadStock<T, Pipe> {
    type DeriveType<U, ChainedPipe> = OptReadStock<U, ChainedPipe>;
    type DeriveOptType<U, ChainedPipe> = OptReadStock<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> OptReadStock<U, ChainedPipe<Pipe, Next, T, U>> {
        self.derive_opt(path_key, next)
    }

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptReadStock<U, ChainedPipe<Pipe, Next, T, U>> {
        let mut path = self.path;
        path.push(path_key);

        OptReadStock {
            value_id: self.value_id,
            path,
            pipeline: ChainedPipe {
                prev: self.pipeline,
                next: opt_next,
                phantom: PhantomData,
            },
            ty: PhantomData,
            // Carried over: a stock derived from a citer-associated stock
            // (e.g. a memo's backing stock) must keep the pull link, or its
            // reads would skip `ensure_citer_fresh` (stale-value glitches) and
            // lose `update_depth` ordering.
            associated_citer_id: self.associated_citer_id,
        }
    }
}

// OptStock
impl<T, Pipe> Derivable<T, Pipe> for OptStock<T, Pipe> {
    type DeriveType<U, ChainedPipe> = OptStock<U, ChainedPipe>;
    type DeriveOptType<U, ChainedPipe> = OptStock<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>> {
        OptStock(self.0.derive_opt(path_key, next))
    }

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>> {
        OptStock(self.0.derive_opt(path_key, opt_next))
    }
}

// ReadStock
impl<T, Pipe> Derivable<T, Pipe> for ReadStock<T, Pipe> {
    type DeriveType<U, ChainedPipe> = ReadStock<U, ChainedPipe>;
    type DeriveOptType<U, ChainedPipe> = OptReadStock<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> ReadStock<U, ChainedPipe<Pipe, Next, T, U>> {
        ReadStock(OptStock(self.0.0.derive_opt(path_key, next)))
    }

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptReadStock<U, ChainedPipe<Pipe, Next, T, U>> {
        self.0.0.derive_opt(path_key, opt_next)
    }
}

// Stock
impl<T, Pipe> Derivable<T, Pipe> for Stock<T, Pipe> {
    type DeriveType<U, ChainedPipe> = Stock<U, ChainedPipe>;
    type DeriveOptType<U, ChainedPipe> = OptStock<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> Stock<U, ChainedPipe<Pipe, Next, T, U>> {
        Stock(ReadStock(OptStock(self.0.0.0.derive_opt(path_key, next))))
    }

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>> {
        OptStock(self.0.0.0.derive_opt(path_key, opt_next))
    }
}

// flatten *Stock<Option<T>> => Opt*Stock<T>

macro_rules! impl_flatten {
    ($ty:ident => $out:ident) => {
        impl<T: 'static, Pipe> $ty<Option<T>, Pipe> {
            /// Collapse a stock of `Option<T>` into an optional stock of `T`.
            ///
            /// `flatten` uses `u64::MAX` as its path key. Real path keys are
            /// small, sequential field/variant indices assigned by the derive
            /// macro, so `u64::MAX` is unreachable in practice, making this key
            /// safe from collision.
            pub fn flatten(
                self,
            ) -> $out<T, ChainedPipe<Pipe, GetNextOpt<Option<T>, T>, Option<T>, T>> {
                self.derive_opt(u64::MAX, GetNextOpt::new(|x| x.as_ref(), |x| x.as_mut()))
            }
        }
    };
}

impl_flatten!(Stock => OptStock);
impl_flatten!(OptStock => OptStock);
impl_flatten!(ReadStock => OptReadStock);
impl_flatten!(OptReadStock => OptReadStock);

/// A simple MapNext impl struct
pub struct GetNext<T, S> {
    next_ref: fn(&T) -> &S,
    next_mut: fn(&mut T) -> &mut S,
}

impl<T, S> GetNext<T, S> {
    pub fn new(next_ref: fn(&T) -> &S, next_mut: fn(&mut T) -> &mut S) -> Self {
        Self { next_ref, next_mut }
    }
}

impl<T, S> Clone for GetNext<T, S> {
    fn clone(&self) -> Self {
        Self {
            next_ref: self.next_ref,
            next_mut: self.next_mut,
        }
    }
}

impl<T, S> Copy for GetNext<T, S> {}

impl<T, S> MapNext<T, S> for GetNext<T, S> {
    fn as_ref<'a>(&self, value: &'a T) -> &'a S {
        (self.next_ref)(value)
    }
    fn as_mut<'a>(&self, value: &'a mut T) -> &'a mut S {
        (self.next_mut)(value)
    }
}

impl<T, S> MapNextOpt<T, S> for GetNext<T, S> {
    fn as_ref<'a>(&self, value: &'a T) -> Option<&'a S> {
        Some(<Self as MapNext<T, S>>::as_ref(self, value))
    }
    fn as_mut<'a>(&self, value: &'a mut T) -> Option<&'a mut S> {
        Some(<Self as MapNext<T, S>>::as_mut(self, value))
    }
}

/// optional GetNext
pub struct GetNextOpt<T, S> {
    next_ref: fn(&T) -> Option<&S>,
    next_mut: fn(&mut T) -> Option<&mut S>,
}

impl<T, S> GetNextOpt<T, S> {
    pub fn new(next_ref: fn(&T) -> Option<&S>, next_mut: fn(&mut T) -> Option<&mut S>) -> Self {
        Self { next_ref, next_mut }
    }
}

impl<T, S> Clone for GetNextOpt<T, S> {
    fn clone(&self) -> Self {
        Self {
            next_ref: self.next_ref,
            next_mut: self.next_mut,
        }
    }
}

impl<T, S> Copy for GetNextOpt<T, S> {}

impl<T, S> MapNextOpt<T, S> for GetNextOpt<T, S> {
    fn as_ref<'a>(&self, value: &'a T) -> Option<&'a S> {
        (self.next_ref)(value)
    }
    fn as_mut<'a>(&self, value: &'a mut T) -> Option<&'a mut S> {
        (self.next_mut)(value)
    }
}
