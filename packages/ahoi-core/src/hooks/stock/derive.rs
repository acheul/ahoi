use super::*;

mod collections;

impl<T, Pipe, const OPT: bool> Stock<T, Pipe, OPT> {
    pub fn try_derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> Stock<U, ChainedPipe<Pipe, Next, T, U>, true> {
        let mut path = self.path;
        path.push(path_key);

        Stock(ReadStock {
            value_id: self.value_id,
            path,
            pipeline: ChainedPipe {
                prev: self.0.pipeline,
                next,
                phantom: PhantomData,
            },
            ty: PhantomData,
            associated_citer_id: None,
        })
    }

    pub fn derive<U: 'static, Next: MapNext<T, U>>(
        self,
        path_key: u64,
        next: Next,
    ) -> Stock<U, ChainedPipe<Pipe, Next, T, U>, OPT> {
        let mut path = self.path;
        path.push(path_key);

        Stock(ReadStock {
            value_id: self.value_id,
            path,
            pipeline: ChainedPipe {
                prev: self.0.pipeline,
                next,
                phantom: PhantomData,
            },
            ty: PhantomData,
            associated_citer_id: None,
        })
    }
}

// flatten Stock<Option<T>, OPT> => OptionStock<T>

impl<T: 'static, Pipe, const OPT: bool> Stock<Option<T>, Pipe, OPT> {
    pub fn flatten(
        self,
    ) -> Stock<T, ChainedPipe<Pipe, GetNextOpt<Option<T>, T>, Option<T>, T>, true> {
        // `flatten` uses u64::MAX as its path key. Real path keys are small,
        // sequential field/variant indices assigned by the derive macro, so
        // u64::MAX is unreachable in practice, making this key safe from collision.
        self.try_derive(u64::MAX, GetNextOpt::new(|x| x.as_ref(), |x| x.as_mut()))
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
    fn as_ref<'a>(&self, value: &'a T) -> Option<&'a S> {
        Some((self.next_ref)(value))
    }
    fn as_mut<'a>(&self, value: &'a mut T) -> Option<&'a mut S> {
        Some((self.next_mut)(value))
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

impl<T, S> MapNext<T, S> for GetNextOpt<T, S> {
    fn as_ref<'a>(&self, value: &'a T) -> Option<&'a S> {
        (self.next_ref)(value)
    }
    fn as_mut<'a>(&self, value: &'a mut T) -> Option<&'a mut S> {
        (self.next_mut)(value)
    }
}
