use core::fmt;
use core::ops::Deref;



mod from_slice;



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
/// use avr_progmem::string::ByteString;
///
/// progmem! {
///     // Stores a string as a byte array, i.e. `[u8;19]`, but makes it usable
///     // as `&str` (via `Deref`)
///     static progmem TEXT: ByteString<19> = ByteString::new(
///         "dai 大賢者 kenja"
///     ).unwrap();
/// }
///
/// // usage:
/// let text_buffer = TEXT.load(); // The temporary DRAM buffer for `TEXT`
/// let text: &str = &text_buffer; // Just derefs to `str`
/// assert_eq!(text, "dai 大賢者 kenja")
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ByteString<const N: usize>(pub [u8; N]);

impl<const N: usize> ByteString<N> {
	/// Creates a new byte array from the given string
	pub const fn new(s: &str) -> Option<Self> {
		Self::from_bytes(s.as_bytes())
	}

	/// Wraps the given byte slice
	pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
		let res = from_slice::array_ref_try_from_slice(bytes);

		match res {
			Ok(array) => Some(Self(*array)),
			Err(_e) => None,
		}
	}
}

impl<const N: usize> Deref for ByteString<N> {
	type Target = str;

	fn deref(&self) -> &str {
		core::str::from_utf8(&self.0).unwrap()
	}
}

impl<const N: usize> fmt::Display for ByteString<N> {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		write!(fmt, "{}", self.deref())
	}
}

#[cfg(any(feature = "ufmt", doc))]
#[doc(cfg(feature = "ufmt"))]
impl<const N: usize> ufmt::uDisplay for ByteString<N> {
	fn fmt<W: ?Sized>(&self, fmt: &mut ufmt::Formatter<W>) -> Result<(), W::Error>
	where
		W: ufmt::uWrite,
	{
		ufmt::uwrite!(fmt, "{}", self.deref())
	}
}


/// Define a string in progmem
///
/// This is a short-cut macro to create an ad-hoc static storing the given
/// string literal as by [`ByteString`] and load it here from progmem into a
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
			static progmem TEXT: $crate::string::ByteString<TEXT_LEN> = $crate::string::ByteString::new(
				$text
			).unwrap();
		}
		&*TEXT.load()
	}};
}
