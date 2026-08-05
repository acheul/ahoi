use super::*;

mod collections;

#[cfg(test)]
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
    type DeriveType<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> Self::DeriveType<U, ChainedPipe<Pipe, Next, T, U>>;

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>>;
}

// OptStock
impl<T, Pipe> Derivable<T, Pipe> for OptStock<T, Pipe> {
    type DeriveType<U, ChainedPipe> = OptStock<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>> {
        self.derive_opt(path_key, next)
    }

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>> {
        let mut path = self.path;
        path.push(path_key);

        OptStock(OptReadStock {
            value_id: self.value_id,
            path,
            pipeline: ChainedPipe {
                prev: self.0.pipeline,
                next: opt_next,
                phantom: PhantomData,
            },
            ty: PhantomData,
            associated_citer_id: None,
        })
    }
}

// Stock
impl<T, Pipe> Derivable<T, Pipe> for Stock<T, Pipe> {
    type DeriveType<U, ChainedPipe> = Stock<U, ChainedPipe>;

    fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> Stock<U, ChainedPipe<Pipe, Next, T, U>> {
        Stock(ReadStock(self.0.0.derive(path_key, next)))
    }

    fn derive_opt<U: 'static, Next: MapNextOpt<T, U>>(
        self,
        path_key: u64,
        opt_next: Next,
    ) -> OptStock<U, ChainedPipe<Pipe, Next, T, U>> {
        self.0.0.derive_opt(path_key, opt_next)
    }
}

// flatten Stock<Option<T>> => OptStock<T>

impl<T: 'static, Pipe> Stock<Option<T>, Pipe> {
    pub fn flatten(self) -> OptStock<T, ChainedPipe<Pipe, GetNextOpt<Option<T>, T>, Option<T>, T>> {
        // `flatten` uses u64::MAX as its path key. Real path keys are small,
        // sequential field/variant indices assigned by the derive macro, so
        // u64::MAX is unreachable in practice, making this key safe from collision.
        self.derive_opt(u64::MAX, GetNextOpt::new(|x| x.as_ref(), |x| x.as_mut()))
    }
}

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
