use core::ops::Deref;



/// Our own `core::array::TryFromSliceError`
///
/// Used in [`array_ref_try_from_slice`].
// We need a local copy of this type, because we need to instantiate it, but it
// has a private field.
struct TryFromSliceError(());

/// Const version of `<&[T; N]>::try_from(&[T])`
///
/// Original Source:
/// https://github.com/rust-lang/rust/blob/eb82facb1626166188d49599a3313fc95201f556/library/core/src/array/mod.rs#L203-L215
const fn array_ref_try_from_slice<'a, T, const N: usize>(
	slice: &[T],
) -> Result<&[T; N], TryFromSliceError> {
	if slice.len() == N {
		let ptr = slice.as_ptr() as *const [T; N];
		// SAFETY: ok because we just checked that the length fits
		unsafe { Ok(&*ptr) }
	} else {
		Err(TryFromSliceError(()))
	}
}

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
pub struct ByteString<const N: usize>(pub [u8; N]);

impl<const N: usize> ByteString<N> {
	/// Creates a new byte array from the given string
	pub const fn new(s: &str) -> Option<Self> {
		Self::from_bytes(s.as_bytes())
	}

	/// Wraps the given byte slice
	pub const fn from_bytes(bytes: &[u8]) -> Option<Self> {
		let res = array_ref_try_from_slice(bytes);

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
