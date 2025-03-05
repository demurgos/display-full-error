# Display Full Error - Minimal display formatter for Rust error chains

[![GitHub](https://img.shields.io/badge/GitHub-demurgos%2Fdisplay--full--error-informational.svg?maxAge=86400)](https://github.com/demurgos/display-full-error)
[![crates.io](https://img.shields.io/crates/v/display_full_error.svg?maxAge=86400)](https://crates.io/crates/display_full_error)
[![CI status](https://img.shields.io/github/actions/workflow/status/demurgos/display-full-error/check-rs.yml.svg?branch=main&maxAge=86400)](https://github.com/demurgos/display-full-error/actions/workflows/check-rs.yml?query=branch%3Amain)
[![docs.rs/display_full_error](https://img.shields.io/docsrs/display_full_error.svg?maxAge=86400)](https://docs.rs/display_full_error)
[![license MIT](https://img.shields.io/badge/license-MIT-green)](./LICENSE.md)

This library provides the [`DisplayFullError`] wrapper type to format
[errors](https://doc.rust-lang.org/nightly/core/error/trait.Error.html) with their chain of
[sources](https://doc.rust-lang.org/nightly/core/error/trait.Error.html#method.source).

Error messages are formatted on a single line, separated with `: `; up to
1024 messages per chain are printed, after which a single `: ...` is printed.

That's all there is to it, there is no extra configuration or advanced
features. This is intended as the most minimal formatter supporting error
sources, to address the fact that there's no helper in the standard library
so far as of Rust 1.83 (2024-11). If a standard formatter supporting error
sources is added, this crate will be deprecated (but remain available).
As a convenience, this library also exposes the [`DisplayFullErrorExt`][DisplayFullErrorExt]
trait. It adds the [`display_full`][DisplayFullErrorExt::display_full]
method to errors which returns the error in the formatting wrapper, as well
as the [`to_string_full`][DisplayFullErrorExt::to_string_full] method as
a shorthand for `.display_full().to_string()`.

```rust
use ::core::{error, fmt};

use ::display_full_error::{DisplayFullError, DisplayFullErrorExt};

// main error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum UploadError {
  Permission(PermissionError),
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

// first source error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PermissionError;
impl fmt::Display for PermissionError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("permission denied")
  }
}
impl error::Error for PermissionError {}

// second source error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct LimitError;
impl fmt::Display for LimitError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str("upload exceeds max limit")
  }
}
impl error::Error for LimitError {}

// usage example
let err = UploadError::Permission(PermissionError);

// You can use the wrapper directly, e.g. in a `format!`
assert_eq!(format!("the app crashed: {}", DisplayFullError(&err)), String::from("the app crashed: upload failed: permission denied"));
// Or you can use `to_string`
assert_eq!(DisplayFullError(&err).to_string(), String::from("upload failed: permission denied"));
// You can also use the convenience methods from the extension trait
assert_eq!(format!("the app crashed: {}", err.display_full()), String::from("the app crashed: upload failed: permission denied"));
// `to_string_full` requires the `alloc` feature to be enabled
#[cfg(feature = "alloc")]
assert_eq!(err.to_string_full(), String::from("upload failed: permission denied"));
```

This library requires Rust 1.81.0 or later as it depends on the Rust
feature `error_in_core`. This library is compatible with `no_std`. There
are no dependencies or optional features. This library does not introduce
any runtime panics. It is recommended to use this library as an internal
helper and to avoid leaking it into your public APIs. The output is
guaranteed to be stable, any change would cause a major version bump.

# License

[MIT](./LICENSE.md)

[DisplayFullErrorExt]: https://docs.rs/display_full_error/latest/display_full_error/trait.DisplayFullErrorExt.html
[DisplayFullErrorExt::display_full]: https://docs.rs/display_full_error/latest/display_full_error/trait.DisplayFullErrorExt.html#method.display_full
[DisplayFullErrorExt::to_string_full]: https://docs.rs/display_full_error/latest/display_full_error/trait.DisplayFullErrorExt.html#method.to_string_full
