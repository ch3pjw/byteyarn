//! `byteyarn` - Space-efficient byte strings.
//!
//! A [`Yarn`] is a highly optimized string type that provides a number of
//! useful properties over [`String`]:
//!
//! * Always two pointers wide, so it is always passed into and out of functions
//!   in registers.
//! * Small string optimization (SSO) up to 15 bytes on 64-bit architectures.
//! * Can be either an owned buffer or a borrowed buffer (like [`Cow<str>`]).
//! * Can be upcast to `'static` lifetime if it was constructed from a
//!   known-static string.
//! 
//! The main caveat is that [`Yarn`]s cannot be easily appended to, since they
//! do not track an internal capacity, and the slice returned by
//! [`Yarn::as_slice()`] does not have the same pointer stability properties as
//! [`String`] (these are rarely needed, though).
//! 
//! ---
//!
//! Yarns are useful for situations in which a copy-on-write string is necessary
//! and most of the strings are relatively small. Although [`Yarn`] itself is
//! not [`Copy`], there is a separate [`YarnRef`] type that is. These types
//! have equivalent representations, and can be cheaply cast between each other.
//!
//! The easiest way to create a yarn is with the [`yarn!()`] and [`byarn!()`]
//! macros, which are similar to [`format!()`].
//! 
//! ```
//! # use byteyarn::*;
//! // Create a new yarn via `fmt`ing.
//! let yarn = yarn!("Answer: {}", 42);
//!
//! // Convert that yarn into a reference.
//! let ry = yarn.as_ref();
//!
//! // Try up-casting the yarn into an "immortal yarn" without copying.
//! let copy: YarnRef<'static, _> = ry.immortalize().unwrap();
//!
//! assert_eq!(yarn, copy);
//! ```
//!
//! Yarns are intended for storing text, either as UTF-8 or as
//! probably-UTF-8 bytes; [`Yarn<str>`] and [`Yarn<[u8]>`] serve these purposes,
//! and can be inter-converted with each other. The [`Yarn::utf8_chunks()`]
//! function can be used to iterate over definitely-valid-UTF-8 chunks within
//! a string.
//! 
//! Both kinds of yarns can be `Debug`ed and `Display`ed, and will print out as
//! strings would. In particular, invalid UTF-8 is converted into either `\xNN`
//! escapes or replacement characters (for `Debug` and `Display` respectively).
//! 
//! ```
//! # use byteyarn::*;
//! let invalid = Yarn::from_byte(0xff);
//! assert_eq!(format!("{invalid:?}"), r#""\xFF""#);
//! assert_eq!(format!("{invalid}"), "�");
//! ```

use std::hash::Hash;

#[cfg(doc)]
use std::borrow::Cow;

mod convert;
mod raw;
mod utf8;
mod yarn;
mod yarn_ref;

pub use utf8::Utf8Chunks;
pub use yarn::Yarn;
pub use yarn_ref::YarnRef;

mod z {
  pub trait Sealed {}
}

/// A trait for abstracting over `str` and `[u8]`.
pub trait Buf: z::Sealed + Eq + Ord + Hash {}
impl z::Sealed for [u8] {}
impl z::Sealed for str {}
impl Buf for [u8] {}
impl Buf for str {}

/// Similar to [`format!()`], but returns a [`Yarn<'static, str>`], instead.
///
/// This macro calls out to [`Yarn::from_fmt()`] internally, although when
/// passed a single string (e.g. `yarn!("foo")`), it will be `const`-evaluable.
#[macro_export]
macro_rules! yarn {
  ($str:literal) => {
    $crate::Yarn::<'static, str>::from_static_buf($str)
  };
  ($($args:tt)*) => {
    $crate::Yarn::<'static, str>::from_fmt(core::format_args!($($args)*))
  };
}

/// Similar to [`format!()`], but returns a [`Yarn<'static, [u8]>`], instead.
///
/// This macro calls out to [`Yarn::from_fmt()`] internally, although when
/// passed a single string (e.g. `yarn!(b"foo")`), it will be `const`-evaluable.
#[macro_export]
macro_rules! byarn {
  ($str:expr) => {
    $crate::Yarn::<'static, [u8]>::from_static_buf($str)
  };
  ($($args:tt)*) => {
    $crate::Yarn::<'static, str>::from_fmt(core::format_args!($($args)*))
    .into_bytes()
  };
}