use super::*;
use futures_channel::oneshot::Receiver;
use futures_util::{FutureExt, future::Shared};

/* # NOTE
 * Action 도 Stock value 를 가지고 있긴 함. 다만 Resource 와 달리, value 를 반환하고 tracking 하는 것이 Action의 주 목적은 아님.
*/

/// Action (call an async runner)
pub struct Action<A, R> {
    data: AsyncCallbackData<R>,
    callback: Callback<A, Shared<Receiver<()>>>,
}

impl<A, R> Clone for Action<A, R> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            callback: self.callback,
        }
    }
}

impl<A, R> Copy for Action<A, R> {}

// Deref: Action -> AsyncCallbackData
impl<A, R> Deref for Action<A, R> {
    type Target = AsyncCallbackData<R>;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<A: 'static, R: 'static> Action<A, R> {
    pub fn new<Func, Fut>(runner: Func) -> Self
    where
        Func: Fn(A) -> Fut + 'static,
        Fut: Future<Output = R> + 'static,
    {
        let value = Stock::new(None::<R>);
        let state = Stock::new(AsyncCallbackState::Unset);
        let task = Stock::new(None::<TaskHandle>);

        let callback: Callback<A, Shared<Receiver<()>>> = Callback::new(move |args: A| {
            // Cancel any existing task
            task.try_write().unwrap().take(); // drop will abort internal handler.

            // Set the state to pending
            state.try_set(AsyncCallbackState::Pending).unwrap();

            let (tx, rx) = futures_channel::oneshot::channel();
            let rx = rx.shared();

            // Spawn a new task, and *then* fire off the async
            let result = runner(args);

            // Create a new task
            let new_task = spawn_batch(async move {
                let result = result.await;
                value.try_set(Some(result)).unwrap();
                state.try_set(AsyncCallbackState::Ready).unwrap();
                tx.send(()).ok();
            });

            task.try_set(Some(new_task)).unwrap();

            rx
        });

        Self {
            data: AsyncCallbackData { value, state, task },
            callback,
        }
    }

    /// use `.await` or not: internal runner will be executed in any case.
    pub fn call(&self, args: A) -> Shared<Receiver<()>> {
        return self.callback.call(args);
    }

    /// To use the recent result as a reactive ReadStock
    pub fn value(&self) -> ReadStock<Option<R>> {
        *self.value
    }
}
