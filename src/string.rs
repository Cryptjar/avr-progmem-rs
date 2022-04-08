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
/// # Example
///
/// ```rust
/// #![feature(const_option)]
///
/// use avr_progmem::progmem;
/// use avr_progmem::string::LoadedString;
///
/// progmem! {
///     // Stores a string as a byte array, i.e. `[u8;19]`, but makes it usable
///     // as `&str` (via `Deref`)
///     static progmem TEXT: LoadedString<19> = LoadedString::new(
///         "dai 大賢者 kenja"
///     ).unwrap();
/// }
///
/// // usage:
/// let text_buffer = TEXT.load(); // The temporary DRAM buffer for `TEXT`
/// let text: &str = &text_buffer; // Just derefs to `str`
/// assert_eq!(text, "dai 大賢者 kenja")
/// ```
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
				unsafe {
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
/// A `LoadedString` is just a wrapper around a byte array (`[u8;N]`) that can
/// be put into a [`ProgMem`].
/// A `PmString` on the other hand, is a wrapper around a
/// `ProgMem<[u8;N]>`.
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
				unsafe {
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

		let pm = unsafe {
			// SAFETY: the caller ensures that this value is in progmem
			ProgMem::new(array)
		};

		// SAFETY: the caller ensures that the bytes are valid UTF-8
		Self {
			pm_utf8_array: pm,
		}
	}

	/// Loads the entire string into RAM
	pub fn load(&self) -> LoadedString<N> {
		let array = self.load_bytes();

		let bs_opt = unsafe {
			// SAFETY: The contract on `Self` guarantees us that we have UTF-8
			LoadedString::from_bytes(&array)
		};

		bs_opt.unwrap()
	}

	/// Loads the entire string as byte array into RAM
	pub fn load_bytes(&self) -> [u8; N] {
		self.as_bytes().load()
	}

	/// Returns the underlying progmem byte array.
	pub fn as_bytes(&self) -> &ProgMem<[u8; N]> {
		&self.pm_utf8_array
	}

	/// Lazily iterate over the `char`s of the string.
	///
	/// This function is analog to [`ProgMem::iter`], except it is over the
	/// `char`s of this string.
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



/// Define a string in progmem
///
/// This is a short-cut macro to create an ad-hoc static storing the given
/// string literal as by [`LoadedString`] and load it here from progmem into a
/// temporary and return it as `&str`.
///
/// This macro allows to conveniently put literal string into progmem exactly,
/// where they are used. However, since they are directly loaded into a
/// temporary you don't get a `&'static str` back, and must use the `&str`
/// immediately (i.e. pass it as a function parameter).
/// You can't even store the returned `&str` in a local `let` assignment.
///
/// # Example
///
/// ```rust
/// #![feature(const_option)]
///
/// use avr_progmem::progmem_str as S;
///
/// fn print(s: &str) {
///     // -- snip --
///     # assert_eq!(s, "dai 大賢者 kenja")
/// }
///
/// // Put the literal as byte array into progmem and load it here as `&str`
/// print(S!("dai 大賢者 kenja"));
/// ```
#[macro_export]
macro_rules! progmem_str {
	($text:literal) => {{
		const TEXT_LEN: usize = <str>::as_bytes($text).len();
		$crate::progmem! {
			// TODO: use PmString
			static progmem TEXT: $crate::string::LoadedString<TEXT_LEN> = $crate::string::LoadedString::new(
				$text
			).unwrap();
		}
		&*TEXT.load()
	}};
}
