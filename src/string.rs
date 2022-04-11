//! String utilities
//!
//! This module offers further utilities base on [`ProgMem`] to make working
//! with strings in progmem more convenient.
//!
//! The difficulty with strings is that normally they are either heap allocated
//! (as with the std Rust `String`) or dynamically sized (as with `str`).
//! To store a string in progmem, one needs first of all a fixed-sized storage
//! variant of a `str`.
//! One option is to use byte string literals (e.g. `b"foobar"`), however,
//! for some reason those only accept ASCII and no Unicode.
//! At some day, one might be able to to convert arbitrary string literals to
//! byte arrays like this:
//!
//! ```ignore
//! # use std::convert::TryInto;
//! // Dose not compile as of 1.51, because `try_into` is not a const fn, yet
//! static WIKIPEDIA: [u8; 12] = "维基百科".as_bytes().try_into().unwrap();
//! ```
//!
//! However, for the time being, this module offers as a convenient workaround:
//! * [`LoadedString`] a simple UTF-8 encoded sized byte array
//! * [`PmString`] a UTF-8 encoded sized byte array in progmem similar to [`ProgMem`].
//!
//!
//! # Working with Strings
//!
//! To work with strings in progmem, this crate offers two APIs and two modes
//! of operation, each with their own little tradeoff.
//!
//! ## Operation Modes
//!
//! When you want to use strings from progmem you have two options:
//!
//! * you can load them as whole from progmem into RAM and work with them
//!   essentially like stack-allocated `&str`s
//! * or, you choose to load the strings `char` after `char` from progmem and work
//!   with them using a `char`-iterator.
//!
//! The first mode of operation obviously allows you to use them everywhere,
//! where you can use a `&str`, giving you high compatibility with other APIs.
//! On the other hand, this comes at the cost of high RAM usage.
//! So you must leave enough free RAM to fit all your string, thus the bigger,
//! your biggest string is, the less RAM you must use statically.
//! So, you might have to split your strings somehow to make them manageable.
//!
//! The alternative is to only load just one `char` at a time.
//! This obviously limits the amount of RAM that you need, independently of
//! how big your strings are, allowing you to work with really huge strings.
//! However, you no longer get a `&str`, any you have make do with a `char`
//! iterator.
//!
//! However, if you only need your strings to be printed in some way,
//! the [`Display`](fmt::Display) and [`ufmt::uDisplay`] traits implementations
//! (the latter only if the `ufmt` crate feature is enabled) of [`PmString`],
//! might become very handy.
//! These trait implementations only need the `char`-iterator so they are very
//! economic with respect to RAM usage.
//!
//! ## APIs
//!
//! API-wise you can either:
//!
//! * define progmem `static`s via the [`progmem`](crate::progmem) macro
//!   and use them all over your program,
//! * or, you create single-use progmem strings via the
//!   [`progmem_str`](crate::progmem_str) and
//!   [`progmem_display`](crate::progmem_display) macro
//!
//! The single-use macros are the most concise option, but also a rather
//! special-case solution.
//! `progmem_str` gives you are very temporary `&str` to an ad-hoc loaded
//! progmem string, so you can only pass it to a function call and you need
//! enough RAM to store it.
//! On the other hand, [`progmem_display`](crate::progmem_display) gives you
//! just something that is `impl Display + uDisplay`, so you can just print it,
//! but it has minimal RAM usage.
//!
//! If need anything more flexible or fancy, you are probably best served
//! creating a `static` via [`progmem`](crate::progmem) macro.
//!
//!
//! # Examples
//!
//! Using [`PmString`] directly via the [`progmem`] macro:
//!
//! ```rust
//! #![feature(const_option)]
//!
//! # use std::iter::FromIterator;
//! use avr_progmem::progmem;
//! use avr_progmem::string::LoadedString;
//!
//! progmem! {
//!     // A simple Unicode string in progmem, internally stored as fix-sized
//!     // byte array, i.e. a `PmString<18>`.
//!     static progmem string TEXT = "Hello 大賢者";
//!     // text too large to fit in the RAM of a microcontroller
//!     static progmem string LOVECRAFT = include_str!("../examples/test_text.txt");
//! }
//!
//! // You can load it all at once (like a `ProgMem`)
//! let buffer: LoadedString<15> = TEXT.load();
//! // and use that as `&str`
//! assert_eq!("Hello 大賢者", &*buffer);
//!
//! // Or you load it one char at a time (limits RAM usage) via the
//! // chars-iterator
//! let chars_iter = TEXT.chars(); // impl Iterator<Item=char>
//! let exp = ['H', 'e', 'l', 'l', 'o', ' ', '大', '賢', '者'];
//! assert_eq!(&exp, &*Vec::from_iter(chars_iter));
//!
//! // Or you use directly the `Display`/`uDisplay` impl on `PmString`
//! // which uses the chars-iterator internally
//! use ufmt::uWrite;
//! #
//! # struct MyWriter;
//! # impl uWrite for MyWriter {
//! #     type Error = ();
//! #     fn write_str(&mut self, _s: &str) -> Result<(),()> {
//! #         Ok(()) // ignore input
//! #     }
//! # }
//! let mut writer =
//! #   MyWriter
//!     /* SNIP */;
//! #[cfg(feature = "ufmt")] // requires the `ufmt` crate feature
//! ufmt::uwrite!(&mut writer, "{}", TEXT);
//! ```
//!
//! Using the special literal in-line string macros [`progmem_str`] and
//! [`progmem_display`]:
//!
//! ```rust
//! #![feature(const_option)]
//!
//! use avr_progmem::progmem_str as F;
//! use avr_progmem::progmem_display as D;
//!
//! // Or you use directly the `Display`/`uDisplay` impl on `PmString`
//! // which uses the chars-iterator internally
//! use ufmt::uWrite;
//! # struct MyWriter;
//! # impl uWrite for MyWriter {
//! #     type Error = ();
//! #     fn write_str(&mut self, _s: &str) -> Result<(),()> {
//! #         Ok(()) // ignore input
//! #     }
//! # }
//! let mut writer =
//! #   MyWriter
//!     /* SNIP */;
//!
//! // In-line string as temporary `&str`
//! writer.write_str(F!("Hello 大賢者"));
//!
//! // In-line string as some `impl Display + uDisplay`
//! #[cfg(feature = "ufmt")] // requires the `ufmt` crate feature
//! ufmt::uwrite!(&mut writer, "{}", D!("Hello 大賢者"));
//! ```
//!


use core::convert::TryFrom;
use core::fmt;
use core::ops::Deref;

use crate::wrapper::PmIter;
use crate::ProgMem;


mod from_slice;
mod validations;



/// Indicates that static type size does not match the dynamic str length.
#[derive(Debug, Clone)]
pub struct InvalidLengthError;


/// A string stored as byte array.
///
/// This type is a simple wrapper around a byte array `[u8;N]` and therefore,
/// is stored as such.
/// However, this type primarily is created from `&str` and derefs to `&str`,
/// thus it can be used similar to `String` except that it is not mutable.
///
/// This type is particularly useful to store string literals in progmem.
///
///
/// # Safety
///
/// The wrapped byte array must contain valid UTF-8.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive] // SAFETY: must not be publicly constructible
pub struct LoadedString<const N: usize> {
	/// The inner UTF-8 string as byte array
	///
	/// # Safety
	///
	/// Must be valid UTF-8.
	utf8_array: [u8; N],
}

impl<const N: usize> LoadedString<N> {
	/// Creates a new byte array from the given string
	///
	/// # Error
	///
	/// If the byte size of `str` is not exactly `N`, `None` is returned.
	///
	pub const fn new(s: &str) -> Option<Self> {
		let bytes: &[u8] = s.as_bytes();
		unsafe {
			// SAFETY: we got a `str` thus it must already contain valid UTF-8
			Self::from_bytes(bytes)
		}
	}

	/// Wraps the given byte slice
	///
	/// # Safety
	///
	/// The give byte slice must contain valid UTF-8.
	///
	/// # Error
	///
	/// If the size of the given byte slice is not exactly `N`, `None` is
	/// returned.
	///
	pub const unsafe fn from_bytes(bytes: &[u8]) -> Option<Self> {
		// Cast slice into an array
		let res = from_slice::array_ref_try_from_slice(bytes);

		match res {
			Ok(array) => {
				let array = *array;
				{
					// SAFETY: the caller ensures that the bytes are valid
					// UTF-8
					Some(Self::from_array(array))
				}
			},
			Err(_e) => None,
		}
	}

	/// Wraps the given byte array
	///
	/// # Safety
	///
	/// The give byte array must contain valid UTF-8.
	///
	pub const unsafe fn from_array(array: [u8; N]) -> Self {
		/* TODO: Use this once it becomes const fn
		match core::str::from_utf8(bytes) {
			Ok(_) => (),
			Err(_) => panic!("Not UTF-8"),
		};
		*/

		// SAFETY: The caller ensures that `array` is indeed UTF-8
		Self {
			utf8_array: array,
		}
	}

	/// Returns the underlying byte array.
	pub fn as_bytes(&self) -> &[u8; N] {
		&self.utf8_array
	}
}

impl<const N: usize> TryFrom<&str> for LoadedString<N> {
	type Error = InvalidLengthError;

	fn try_from(s: &str) -> Result<Self, Self::Error> {
		match LoadedString::new(s) {
			Some(bs) => Ok(bs),
			None => Err(InvalidLengthError),
		}
	}
}

impl<const N: usize> Deref for LoadedString<N> {
	type Target = str;

	fn deref(&self) -> &str {
		unsafe {
			// SAFETY: by the contract of this struct, `utf8_array` must be
			// valid UTF-8
			core::str::from_utf8_unchecked(&self.utf8_array)
		}
	}
}

impl<const N: usize> fmt::Display for LoadedString<N> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.deref())
	}
}

#[cfg(any(feature = "ufmt", doc))]
#[doc(cfg(feature = "ufmt"))]
impl<const N: usize> ufmt::uDisplay for LoadedString<N> {
	fn fmt<W: ?Sized>(&self, fmt: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
	where
		W: ufmt::uWrite,
	{
		ufmt::uwrite!(fmt, "{}", self.deref())
	}
}


/// A byte string in progmem
///
/// Not to be confused with a [`LoadedString`].
/// A `LoadedString` is a simple wrapper around a byte array (`[u8;N]`) that
/// derefs to `str`, and should be used in RAM.
/// A `PmString` on the other hand, is a wrapper around a byte array in progmem
/// aka around a `ProgMem<[u8;N]>`, and thus must always be progmem.
/// Similar to `ProgMem`, `PmString` offers a [`load`](PmString::load) method to
/// load its entire content into RAM.
/// The loaded content will be a `LoadedString`, hence the name.
///
/// Besides loading the entire string at once into RAM, `PmString` also offers
/// a lazy [`chars`](PmString::chars) iterator method, that will load just one
/// char at a time.
/// This allows `chars` to be used on very large strings that do not fit into
/// the RAM as whole.
///
///
/// # Safety
///
/// This type is a wrapper around [`ProgMem`], thus it any value of this type
/// must be placed in program memory.
/// See the [`ProgMem`] safety section for more details.
///
/// Additionally to the [`ProgMem`] contract, the byte array wrapped by this
/// struct must be valid UTF-8.
///
///
/// # Example
///
/// ```rust
/// #![feature(const_option)]
///
/// use avr_progmem::progmem;
/// use avr_progmem::string::PmString;
/// use avr_progmem::string::LoadedString;
///
/// progmem! {
///     // Stores a string as a byte array, i.e. `[u8;19]`, but makes it usable
///     // as `&str` (via `Deref`)
///     static progmem string TEXT = "dai 大賢者 kenja";
/// }
///
/// // The static has type `PmString`
/// let text: &PmString<19> = &TEXT;
/// // The loaded RAM string has type `LoadedString`
/// let loaded: LoadedString<19> = text.load();
/// // Which derefs to `&str`
/// assert_eq!("dai 大賢者 kenja", &*loaded)
/// ```
///
#[repr(transparent)]
#[non_exhaustive] // SAFETY: this struct must not be publicly constructible
pub struct PmString<const N: usize> {
	/// The inner UTF-8 string as byte array in progmem.
	///
	/// # Safety
	///
	/// Must be valid UTF-8.
	pm_utf8_array: ProgMem<[u8; N]>,
}

impl<const N: usize> PmString<N> {
	/// Creates a new byte array from the given string
	///
	/// # Safety
	///
	/// This function is only sound to call, if the value is
	/// stored in a static that is for instance attributed with
	/// `#[link_section = ".progmem.data"]`.
	///
	/// You are encouraged to use the [`progmem`] macro instead.
	pub const unsafe fn new(s: &str) -> Option<Self> {
		Self::from_bytes(s.as_bytes())
	}

	/// Wraps the given byte slice
	///
	/// # Safety
	///
	/// This function is only sound to call, if the value is
	/// stored in a static that is for instance attributed with
	/// `#[link_section = ".progmem.data"]`.
	///
	/// Additionally, the given byte slice must contain valid UTF-8.
	pub const unsafe fn from_bytes(bytes: &[u8]) -> Option<Self> {
		let res = from_slice::array_ref_try_from_slice(bytes);

		match res {
			Ok(array) => {
				let array = *array;
				{
					// SAFETY: the caller ensures that this value is in progmem
					// and the bytes are valid UTF-8
					Some(Self::from_array(array))
				}
			},
			Err(_e) => None,
		}
	}

	/// Wraps the given byte array
	///
	/// # Safety
	///
	/// This function is only sound to call, if the value is
	/// stored in a static that is for instance attributed with
	/// `#[link_section = ".progmem.data"]`.
	///
	/// The give byte array must contain valid UTF-8.
	///
	pub const unsafe fn from_array(array: [u8; N]) -> Self {
		/* TODO: Use this once it becomes const fn
		match core::str::from_utf8(&array) {
			Ok(_) => (),
			Err(_) => panic!("Not UTF-8"),
		};
		*/

		let pm = {
			// SAFETY: the caller ensures that this value is in progmem
			ProgMem::new(array)
		};

		// SAFETY: the caller ensures that the bytes are valid UTF-8
		Self {
			pm_utf8_array: pm,
		}
	}

	/// Loads the entire string into RAM
	///
	/// # Panics
	///
	/// This method panics, if the size of the value (i.e. `N`) is beyond 255
	/// bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	/// If you have a very large string, consider using the lazy
	/// [`chars`](Self::chars) iterator that accesses the string by one char at
	/// a time and thus does not have such a limitation.
	///
	pub fn load(&self) -> LoadedString<N> {
		let array = self.load_bytes();

		let bs_opt = unsafe {
			// SAFETY: The contract on `Self` guarantees us that we have UTF-8
			LoadedString::from_bytes(&array)
		};

		bs_opt.unwrap()
	}

	/// Loads the entire string as byte array into RAM
	///
	/// # Panics
	///
	/// This method panics, if the size of the value (i.e. `[u8; N]`) is beyond
	/// 255 bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	/// If you have a very large string, consider using the lazy
	/// [`chars`](Self::chars) iterator or the respective byte iterator
	/// (via `as_bytes().iter()`).
	pub fn load_bytes(&self) -> [u8; N] {
		self.as_bytes().load()
	}

	/// Returns the underlying progmem byte array.
	pub fn as_bytes(&self) -> &ProgMem<[u8; N]> {
		&self.pm_utf8_array
	}

	/// Lazily iterate over the `char`s of the string.
	///
	/// This function is analog to [`ProgMem::iter`], except it performs UTF-8
	/// parsing and returns the `char`s of this string, thus it is more similar
	/// to [`str::chars`].
	pub fn chars(&self) -> PmChars<N> {
		PmChars::new(self)
	}
}

impl<const N: usize> fmt::Display for PmString<N> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		for c in self.chars() {
			write!(fmt, "{}", c)?
		}
		Ok(())
	}
}

#[cfg(any(feature = "ufmt", doc))]
#[doc(cfg(feature = "ufmt"))]
impl<const N: usize> ufmt::uDisplay for PmString<N> {
	fn fmt<W: ?Sized>(&self, fmt: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
	where
		W: ufmt::uWrite,
	{
		for c in self.chars() {
			ufmt::uwrite!(fmt, "{}", c)?
		}
		Ok(())
	}
}


/// An iterator over a [`PmString`]
///
/// # Safety
///
/// The inner byte iterator of this struct must yield valid UTF-8 sequence.
#[non_exhaustive] // SAFETY: this struct must not be publicly constructible
pub struct PmChars<'a, const N: usize> {
	/// The inner byte iterator
	///
	/// # Safety
	///
	/// Must yield valid UTF-8 sequences.
	bytes: PmIter<'a, u8, N>,
}

impl<'a, const N: usize> PmChars<'a, N> {
	pub fn new(pm: &'a PmString<N>) -> Self {
		// SAFETY: the contract on PmString guarantees us that it wraps
		// valid UTF-8, thus its byte iterator will yield valid UTF-8
		PmChars {
			bytes: pm.pm_utf8_array.iter(),
		}
	}
}

impl<'a, const N: usize> Iterator for PmChars<'a, N> {
	type Item = char;

	fn next(&mut self) -> Option<Self::Item> {
		unsafe {
			// SAFETY: the contract on `Self` struct guarantees us that we only
			// get valid UTF-8 sequences
			validations::next_code_point(&mut self.bytes)
		}
		.map(|u| core::char::from_u32(u).unwrap())
	}
}



/// Define a string in progmem usable as temporary `&str`
///
/// This is a short-cut macro to create an ad-hoc static storing the given
/// string literal as by [`LoadedString`] and load it here from progmem into a
/// temporary and return it as `&str`.
/// This is similar to the `F` macro available in Arduino.
///
/// Similar to the C marco, this will load the full string into RAM at once
/// and thus the string should be of limited size, to not exceed the space
/// available in RAM.
/// Also see the [progmem_display] macro which does not have this limitation.
///
/// This macro allows to conveniently put a literal string into progmem
/// right where it is used.
/// However, since they are directly loaded into a temporary you don't get a
/// `&'static str` back, and must use the `&str` immediately (i.e. pass it as a
/// function parameter).
/// You can't even store the returned `&str` in a local `let` assignment.
///
///
/// # Example
///
/// ```rust
/// #![feature(const_option)]
/// use avr_progmem::progmem_str as F;
/// use ufmt::uWrite;
///
/// # struct MyWriter;
/// # impl uWrite for MyWriter {
/// #     type Error = ();
/// #     fn write_str(&mut self, _s: &str) -> Result<(),()> {
/// #         Ok(()) // ignore input
/// #     }
/// # }
/// #
/// let mut writer = // impl uWrite
/// #    MyWriter;
///     /* SNIP */;
///
/// // Put the literal `str` into progmem and load it here as `&str`
/// writer.write_str(F!("dai 大賢者 kenja"));
/// ```
///
#[macro_export]
macro_rules! progmem_str {
	($text:expr) => {{
		$crate::progmem! {
			static progmem string TEXT = $text;
		}
		&*TEXT.load()
	}};
}


/// Define a string in progmem usable as `impl Display + uDisplay`
///
/// This is a short-cut macro to create an ad-hoc static storing the given
/// string literal as a [`PmString`] and return it.
/// This is somewhat similar to the `F` macro available in Arduino IDE, but
/// different.
/// For a macro more in line with the `F` macro, see [progmem_str].
///
/// Unlike the `F` macro, this macro neither loads the string here, nor, can
/// it be use as a `&str`.
/// However, the returned value implements [Display](fmt::Display) as well as
/// [ufmt::uDisplay] (if the `ufmt` crate feature is enabled).
///
/// This macro allows to conveniently put a literal string into progmem
/// right where it is used.
/// However, since it is not loaded (yet) into RAM it is not a `&str`, it only
/// exposes a [Display](fmt::Display) and [ufmt::uDisplay] (if the `ufmt` crate
/// feature is enabled) implementation,
/// which will load it char-by-char when used, thus limiting the RAM usage,
/// and allowing arbitrarily large strings to be wrapped.
///
///
/// # Example
///
/// ```rust
/// #![feature(const_option)]
/// use avr_progmem::progmem_display as D;
/// use ufmt::uWrite;
///
/// # struct MyWriter;
/// # impl uWrite for MyWriter {
/// #     type Error = ();
/// #     fn write_str(&mut self, _s: &str) -> Result<(),()> {
/// #         Ok(()) // ignore input
/// #     }
/// # }
/// #
/// let mut writer = // impl uWrite
/// #    MyWriter;
///     /* SNIP */;
///
/// // Put the literal `str` into progmem and use it as `impl uDisplay`
/// #[cfg(feature = "ufmt")] // requires the `ufmt` crate feature
/// ufmt::uwrite!(&mut writer, "{}", D!("dai 大賢者 kenja"));
///
/// // Huge strings are fine
/// #[cfg(feature = "ufmt")] // requires the `ufmt` crate feature
/// ufmt::uwrite!(&mut writer, "{}", D!(include_str!("../examples/test_text.txt")));
/// ```
///
#[macro_export]
macro_rules! progmem_display {
	($text:expr) => {{
		$crate::progmem! {
			static progmem string TEXT = $text;
		}
		&TEXT
	}};
}
