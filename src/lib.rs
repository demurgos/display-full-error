//! Display Full Error - Minimal display formatter for error chains
//!
//! This library provides the [`DisplayFullError`] wrapper type to format
//! [errors](::core::error::Error) with their chain of
//! [sources](core::error::Error::source).
//!
//! Error messages are formatted on a single line, separated with `: `; up to
//! 1024 messages per chain are printed, after which a single `: ...` is printed.
//!
//! That's all there is to it, there is no extra configuration or advanced
//! features. This is intended as the most minimal formatter supporting error
//! sources, to address the fact that there's no helper in the standard library
//! so far as of Rust 1.85 (2025-03). If a standard formatter supporting error
//! sources is added, this crate will be deprecated (but remain available).
//!
//! As a convenience, this library also exposes the [`DisplayErrorChainExt`]
//! trait. It adds the [`display_full`](DisplayErrorChainExt::display_full)
//! method to errors which returns the error in the formatting wrapper, as well
//! as the [`to_string_full`](DisplayErrorChainExt::to_string_full) method as
//! a convenience for `.display_full().to_string()`.
//!
//! ```rust
//! use ::core::{error, fmt};
//!
//! use ::display_full_error::{DisplayFullError, DisplayFullErrorExt};
//!
//! // main error
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//! enum UploadError {
//!   Permission(PermissionError),
//!   Limit(LimitError),
//! }
//! impl fmt::Display for UploadError {
//!   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!     f.write_str("upload failed")
//!   }
//! }
//! impl error::Error for UploadError {
//!   fn source(&self) -> Option<&(dyn error::Error + 'static)> {
//!     Some(match self {
//!       UploadError::Permission(e) => e,
//!       UploadError::Limit(e) => e,
//!     })
//!   }
//! }
//!
//! // first source error
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//! struct PermissionError;
//! impl fmt::Display for PermissionError {
//!   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!     f.write_str("permission denied")
//!   }
//! }
//! impl error::Error for PermissionError {}
//!
//! // second source error
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
//! struct LimitError;
//! impl fmt::Display for LimitError {
//!   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!     f.write_str("upload exceeds max limit")
//!   }
//! }
//! impl error::Error for LimitError {}
//!
//! // usage example
//! let err = UploadError::Permission(PermissionError);
//!
//! // You can use the wrapper directly, e.g. in a `format!`
//! assert_eq!(format!("the app crashed: {}", DisplayFullError(&err)), String::from("the app crashed: upload failed: permission denied"));
//! // Or you can use `to_string`
//! assert_eq!(DisplayFullError(&err).to_string(), String::from("upload failed: permission denied"));
//! // You can also use the convenience methods from the extension trait
//! assert_eq!(format!("the app crashed: {}", err.display_full()), String::from("the app crashed: upload failed: permission denied"));
//! // `to_string_full` requires the `alloc` feature to be enabled
//! #[cfg(feature = "alloc")]
//! assert_eq!(err.to_string_full(), String::from("upload failed: permission denied"));
//! ```
//!
//! This library requires Rust 1.81.0 or later as it depends on the Rust
//! feature `error_in_core`. This library is compatible with `no_std`. There
//! are no dependencies or optional features. This library does not introduce
//! any runtime panics. It is recommended to use this library as an internal
//! helper and to avoid leaking it into your public APIs. The output is
//! guaranteed to be stable, any change would cause a major version bump.
//!
//! The formatting uses `: ` as it follows existing conventions and allows to
//! keep the formatted error on a single line if the error messages don't
//! include newlines. Keeping the error on a single line increases compatibility
//! with tools handling error output.
//!
//! The maximum number of messages could have been a const parameter, but making
//! it so currently harms ergonomics quite a lot as there is no support for
//! default const values as of Rust 1.83. See the following Rust issues:
//! [#27336](https://github.com/rust-lang/rust/issues/27336),
//! [#85077](https://github.com/rust-lang/rust/issues/85077).
#![deny(missing_docs)]
#![no_std]
#[cfg(any(test, feature = "alloc"))]
extern crate alloc;

/// Maximum number of messages to print in a single full error.
///
/// This value includes the initial error. If there are more errors left, the
/// next error will be printed as `...` and formatting will end.
pub const MESSAGE_LIMIT: u16 = 1024;

/// Formatting wrapper to display errors, including their sources.
///
/// Error messages are formatted on a single line, separated with `: `; up to
/// 1024 messages per chain are printed, after which a single `: ...` is printed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayFullError<'e, E>(pub &'e E)
where
  E: ::core::error::Error + ?Sized;

impl<E> ::core::fmt::Display for DisplayFullError<'_, E>
where
  E: ::core::error::Error + ?Sized,
{
  fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
    core::fmt::Display::fmt(&self.0, f)?;
    let mut printed: u16 = 1;
    for e in ::core::iter::successors(self.0.source(), |e| e.source()) {
      if printed >= MESSAGE_LIMIT {
        f.write_str(": ...")?;
        return Ok(());
      }
      f.write_str(": ")?;
      ::core::fmt::Display::fmt(e, f)?;
      printed = printed.saturating_add(1);
    }
    Ok(())
  }
}

/// Private module, to implement the trait sealing pattern.
mod private {
  /// To restrict `DisplayFullErrorExt` implementations to this crate.
  pub trait Sealed {}
}

/// Extension trait providing convenience methods on [errors](::core::error::Error).
///
/// This trait provides a blanket implementation for all types implementing [the standard `Error` trait](::core::error::Error).
pub trait DisplayFullErrorExt: ::core::error::Error + private::Sealed {
  /// Get a reference to this error wrapped in a [`DisplayFullError`] formatter, to display the error with all its sources.
  fn display_full(&self) -> DisplayFullError<'_, Self> {
    DisplayFullError(self)
  }

  /// Shorthand for `.display_full().to_string()`
  ///
  /// Requires the `alloc` feature.
  #[cfg(feature = "alloc")]
  fn to_string_full(&self) -> alloc::string::String {
    use crate::alloc::string::ToString;

    self.display_full().to_string()
  }
}

impl<E> private::Sealed for E where E: ::core::error::Error + ?Sized {}

impl<E> DisplayFullErrorExt for E where E: ::core::error::Error + ?Sized {}

#[cfg(test)]
mod tests {
  use super::*;
  use ::alloc::format;
  use ::alloc::string::{String, ToString};
  use ::core::{error, fmt};

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  enum UploadError {
    Permission(PermissionError),
    #[allow(dead_code)]
    Limit(LimitError),
  }

  impl fmt::Display for UploadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("upload failed")
    }
  }

  impl error::Error for UploadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
      Some(match self {
        UploadError::Permission(e) => e,
        UploadError::Limit(e) => e,
      })
    }
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct PermissionError;

  impl fmt::Display for PermissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("permission denied")
    }
  }

  impl error::Error for PermissionError {}

  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct LimitError;

  impl fmt::Display for LimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
      f.write_str("upload exceeds max limit")
    }
  }

  impl error::Error for LimitError {}

  #[test]
  fn error_without_source() {
    let input = PermissionError;
    let actual: String = input.display_full().to_string();
    let expected = String::from("permission denied");
    assert_eq!(actual, expected);
  }

  #[test]
  fn error_with_source() {
    let input = UploadError::Permission(PermissionError);
    let actual: String = input.display_full().to_string();
    let expected = String::from("upload failed: permission denied");
    assert_eq!(actual, expected);
  }

  #[test]
  fn error_with_cyclic_source_chain() {
    #[derive(Debug)]
    struct CyclicError;

    impl fmt::Display for CyclicError {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("cycle detected")
      }
    }

    impl error::Error for CyclicError {
      fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self as &dyn error::Error)
      }
    }

    let input = CyclicError;
    let actual: String = input.display_full().to_string();
    let expected = format!("{}...", ["cycle detected: "; 1024].join(""));
    assert_eq!(actual, expected);
  }
}
