//! Debug-only source locations for blaming panics.
//!
//! In debug builds a [`Location`] wraps a `&'static core::panic::Location`; in
//! release it is a ZST and `Location::caller()` is *not* `#[track_caller]`, so
//! no call site ever has to materialize a `Location` constant. That is what
//! keeps source paths out of the release binary — see `documents/todos.md`.
//!
//! Rules for callers:
//! - Every function between user code and `Location::caller()` must carry
//!   `#[cfg_attr(debug_assertions, track_caller)]`, or the recorded location
//!   points at ahoi-core itself.
//! - `#[track_caller]` does **not** cross closure boundaries, and a bare
//!   `panic!` always reports its own position. For a panic inside a closure,
//!   capture the location *outside* the closure and raise it with [`panic_at`].

#[cfg(debug_assertions)]
#[derive(Clone, Copy)]
pub(crate) struct Location(&'static core::panic::Location<'static>);

#[cfg(not(debug_assertions))]
#[derive(Clone, Copy)]
pub(crate) struct Location; // ZST

impl Location {
    #[cfg(debug_assertions)]
    #[track_caller]
    #[inline]
    pub(crate) fn caller() -> Self {
        Self(core::panic::Location::caller())
    }

    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub(crate) fn caller() -> Self {
        Self
    }

    /// Source file this location points at.
    /// * Test-only: used to assert a recorded location lands in user code
    ///   rather than inside ahoi-core.
    #[cfg(all(test, debug_assertions))]
    pub(crate) fn file(&self) -> &'static str {
        self.0.file()
    }
}

// Debug only: in release there is nothing to print, and keeping a Display impl
// alive would drag formatting code (and a placeholder literal) into the binary.
#[cfg(debug_assertions)]
impl core::fmt::Display for Location {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}:{}", self.0.file(), self.0.line(), self.0.column())
    }
}

/// Panic with `msg`, appending `origin` in debug builds.
/// * Prefer the [`panic_at!`](crate::panic_at) macro over calling this directly.
#[cfg(debug_assertions)]
#[cold]
#[inline(never)]
pub(crate) fn do_panic_at(origin: Option<Location>, msg: core::fmt::Arguments<'_>) -> ! {
    match origin {
        Some(origin) => panic!("{} (at {})", msg, origin),
        None => panic!("{}", msg),
    }
}

/// Release counterpart: the origin is a ZST and is discarded, so only the
/// message literal reaches the binary.
#[cfg(not(debug_assertions))]
#[cold]
#[inline(never)]
pub(crate) fn do_panic_at(_origin: Option<Location>, msg: core::fmt::Arguments<'_>) -> ! {
    panic!("{}", msg)
}

/// Panic with a message, appending a recorded [`Location`] when one is known.
///
/// `#[track_caller]` does not cross closure boundaries, and a bare `panic!`
/// always reports its own source position — so any panic raised inside a
/// `RUNTIME.with_borrow*(|runtime| ...)` closure has to carry its origin
/// explicitly. Capture it *outside* the closure with `Location::caller()`, or
/// look up a state's creation site with `Runtime::location_of`, then pass it
/// here.
///
/// Takes `Option<Location>`: in release the option is always `None` (locations
/// are not tracked) and the whole formatting branch is compiled out.
///
/// * Declared before the `states`/`hooks` modules so textual macro scope covers
///   them.
macro_rules! panic_at {
    ($origin:expr, $($msg:tt)*) => {
        $crate::utils::location::do_panic_at($origin, format_args!($($msg)*))
    };
}

pub(crate) use panic_at;
