use super::*;

/// Resource
pub struct Resource<T> {
    data: AsyncCallbackData<T>,
    citer_id: StateId,
}

impl<T> Clone for Resource<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            citer_id: self.citer_id,
        }
    }
}

impl<T> Copy for Resource<T> {}

// Deref: Resource -> AsyncCallbackData
impl<T> Deref for Resource<T> {
    type Target = AsyncCallbackData<T>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// Enable cite wrapper for an async callback
struct CitingFuture<F>(StateId, F);

impl<F: Future> Future for CitingFuture<F> {
    type Output = F::Output;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        runtime::citation::cite_accumulate(self.0, || {
            unsafe { self.map_unchecked_mut(|s| &mut s.1) }.poll(cx)
        })
    }
}

impl<T: 'static> Resource<T> {
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn new<Func, Fut>(runner: Func) -> Self
    where
        Func: Fn() -> Fut + 'static,
        Fut: Future<Output = T> + 'static,
    {
        // 1. Reserve the citer id up front
        let citer_id = runtime::insert::insert_citer_runner_state(|| {});

        // 2. stocks
        let value = Stock::new_citer_associated_stock(None::<T>, citer_id);
        let state = Stock::new_citer_associated_stock(AsyncCallbackState::Unset, citer_id);
        let task = Stock::new(None::<TaskHandle>);

        // 3. Replace runner
        runtime::insert::replace_citer_runner_state(citer_id, move || {
            // Cancel any existing task
            task.try_write().unwrap().take(); // drop will abort internal handler.

            // Set the state to pending
            state.try_set(AsyncCallbackState::Pending).unwrap();

            // Spawn a new task, and *then* fire off the async
            let result = runner();

            // Create a new task
            let new_task = spawn_batch(CitingFuture(citer_id, async move {
                let result = result.await;
                value.try_set(Some(result)).unwrap();
                state.try_set(AsyncCallbackState::Ready).unwrap();
            }));

            task.try_set(Some(new_task)).unwrap();
        });

        let resource = Self {
            data: AsyncCallbackData { value, state, task },
            citer_id,
        };

        // initial run
        runtime::run_runner::run_citer(resource.citer_id);
        return resource;
    }

    pub fn peek(&self) -> Ref<'static, Option<T>> {
        self.data.value.peek()
    }

    pub fn read(&self) -> Ref<'static, Option<T>> {
        self.data.value.read()
    }

    #[cfg_attr(debug_assertions, track_caller)]
    pub fn set_read_hail<X: HailConverter<Option<T>> + 'static>(self) -> X::HailValue {
        self.data.value.set_read_hail::<X>()
    }
}
