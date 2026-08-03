use super::*;

/// State of AsyncCallback
#[derive(Debug, Clone, Copy)]
pub enum AsyncCallbackState {
    Unset,
    Pending,
    Ready,
}

impl AsyncCallbackState {
    pub const fn is_pending(&self) -> bool {
        match self {
            Self::Pending => true,
            _ => false,
        }
    }

    pub const fn is_ready(&self) -> bool {
        match self {
            Self::Ready => true,
            _ => false,
        }
    }
}

/// Core Data of AsyncCallback, managed as Stocks
pub struct AsyncCallbackData<T> {
    pub(crate) value: Stock<Option<T>>,
    pub(crate) state: Stock<AsyncCallbackState>,
    pub(crate) task: Stock<Option<TaskHandle>>,
}

impl<T> Clone for AsyncCallbackData<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            state: self.state,
            task: self.task,
        }
    }
}

impl<T> Copy for AsyncCallbackData<T> {}

// Deref -> ReadStock<Option<T>>
impl<T> Deref for AsyncCallbackData<T> {
    type Target = ReadStock<Option<T>>;
    fn deref(&self) -> &Self::Target {
        self.value.deref()
    }
}

impl<T: 'static> AsyncCallbackData<T> {
    /// Is pending?
    /// * An AsyncCallback can have one of the three states: Unset, Pending, or Ready
    pub fn pending(&self) -> bool {
        match *self.state.read() {
            AsyncCallbackState::Pending => true,
            _ => false,
        }
    }

    /// Is ready?
    /// * An AsyncCallback can have one of the three states: Unset, Pending, or Ready
    pub fn ready(&self) -> bool {
        match *self.state.read() {
            AsyncCallbackState::Ready => true,
            _ => false,
        }
    }

    /// Abort in-flight task without clearing recent result
    pub fn cancel(&self) {
        runtime::propagation::batch(|| {
            self.task.write().take(); // drop will abort internal handler
            self.state.set(AsyncCallbackState::Unset);
        });
    }

    /// Abort in-flight task and clear recent result
    pub fn reset(&self) {
        runtime::propagation::batch(|| {
            self.task.write().take(); // drop will abort internal handler
            self.state.set(AsyncCallbackState::Unset);
            self.value.set(None);
        });
    }
}
