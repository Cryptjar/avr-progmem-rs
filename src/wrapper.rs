//! Best-effort safe wrapper for progmem.
//!
//! This module offers the [`ProgMem`] struct that wraps a value in progmem,
//! and only gives access to this value via methods that first load the value
//! into the normal data memory domain.
//! This is also the reason why the value must be `Copy` and is always returned
//! by-value instead of by-reference (since the value is not in the data memory
//! where it could be referenced).
//!
//! Since the `ProgMem` struct loads the value using special instructions,
//! it must actually be in progmem, otherwise it would be
//! **undefined behavior**, therefore, its constructor is `unsafe` where the
//! caller must guarantee that it is indeed stored in progmem.
//!
//! As further convenience, the [`progmem`] macro is offered that will create
//! a `static` in progmem and wrap the given value in the [`ProgMem`] struct
//! for you.


use core::convert::TryInto;

use crate::raw::read_value;



/// Best-effort safe wrapper around a value in program memory.
///
/// This type wraps a value that is stored in program memory, and offers safe
/// functions to load those values from program memory into the data memory (or
/// at least some registers).
///
/// Since its constructer is the single most critical point in its API, it is
/// `unsafe`, despite it is supposed to be a safe wrapper (hence the
/// 'best-effort' notation).
///
/// However, there is a rather simple way to make is sound, and that is defining
/// the `#[link_section = ".progmem.data"]` (or `".text"`) on a static that contains
/// this struct. And since its that simple, a macro `progmem!` is provided that
/// will ensure this and should be always used to obtain a `ProgMem` instance
/// in the first place.
///
///
/// # Safety
///
/// This type is a best-effort safe, thus it interface with unsafe Rust given
/// some invariants (like any other safe wrapper).
///
/// The important and obvious invariant is that all values of the struct
/// (instances) must be stored in the program memory. Since that is a property
/// that the compiler (as of now) can not determine or assert or anything, it
/// can't even be asserted, so far, the constructor is the central most unsafe
/// point of this type.
/// But once established it can't change (for statics at least),
/// thus the only unsafe part of this type is the constructor where the user
/// has to guarantee that it is indeed stored in a `static` in progmem.
///
/// Notice that if you got a `static mut` it is unsafe from a start so such a
/// safe wrapper is of little use, and still then has the problem, that it is
/// totally unsound to move it out of the static (e.g. using `std::mem::swap`
/// on it).
///
/// Therefore, only a immutable `static` in the correct memory segment can be
/// considered to be a correct location for it.
///
#[repr(transparent)]
pub struct ProgMem<T>(T);

impl<T> ProgMem<T> {
	/// Construct a new instance of this type.
	///
	/// This struct is a wrapper type for data in the program code memory
	/// domain. Therefore when constructing this struct, it must be guaranteed
	/// to uphold this requirement! This contract is expressed by the fact that
	/// this function is `unsafe`. Also see the Safety section for details.
	///
	/// To simplify, there is a macro `progmem!` which creates a static and
	/// ensures that it is stored indeed in the program code memory domain, and
	/// then makes a call to this function to wrap that static. A user of this
	/// crate should always prefer using the `progmem!` macro to obtain a
	/// `ProgMem` value!
	///
	/// # Safety
	///
	/// The `ProgMem` wrapper is build around the invariant that itself an thus
	/// its inner value are stored in the program code memory domain (on the
	/// AVR architecture).
	///
	/// That means that this function is only sound to call, if the value is
	/// stored in a static that is for instance attributed with
	/// `#[link_section = ".progmem.data"]`.
	///
	/// However, the above requirement only applies to the AVR architecture
	/// (`#[cfg(target_arch = "avr")]`), because otherwise normal data access
	/// primitives are used. This means that the value must be stored in the
	/// regular data memory domain for ALL OTHER architectures! This still
	/// holds, even if such other architecture is of the Harvard architecture,
	/// because this is an AVR-only crate, not a general Harvard architecture
	/// crate!
	///
	pub const unsafe fn new(t: T) -> Self {
		ProgMem(t)
	}
}

impl<T: Copy> ProgMem<T> {
	/// Read the inner value from progmem and return a regular value.
	///
	/// # Panics
	///
	/// This method panics, if the size of the value (i.e. `size_of::<T>()`)
	/// is beyond 255 bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	/// Also notice, if you really hit this limit, you would need 256+ bytes on
	/// your stack, on the Arduino Uno (at least) that means that you might be
	/// close to stack overflow. Thus it might be better to restructure your
	/// data, so you can store it as an array of something, than you can use
	/// the [`load_at`] and [`load_sub_array`] methods instead.
	///
	/// [`load_at`]: struct.ProgMem.html#method.load_at
	/// [`load_sub_array`]: struct.ProgMem.html#method.load_sub_array
	///
	pub fn load(&self) -> T {
		// Get the actual address of the value to load
		let p_addr = &self.0;

		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		unsafe { read_value(p_addr) }
	}

	/// Return the raw pointer to the inner value.
	///
	/// Notice that the returned pointer is indeed a pointer into the progmem
	/// domain! It may never be dereferenced via the default Rust operations.
	/// That means a `unsafe{*pm.get_inner_ptr()}` is **undefined behavior**!
	///
	pub fn ptr(&self) -> *const T {
		&self.0
	}
}

/// Utilities to work with an array in progmem.
impl<T: Copy, const N: usize> ProgMem<[T; N]> {
	/// Load a single element from the inner array.
	///
	/// This method is analog to a slice indexing `self.inner[idx]`, so the
	/// same requirements apply, like the index `idx` should be less then the
	/// length `N` of the array, otherwise a panic will be risen.
	///
	///
	/// # Panics
	///
	/// This method panics, if the given index `idx` is grater or equal to the
	/// length `N` of the inner type.
	///
	/// This method also panics, if the size of the value (i.e. `size_of::<T>()`)
	/// is beyond 255 bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	/// Notice, that here `T` is the type of the elements not the entire array
	/// as it would be with [`load`](Self::load).
	///
	pub fn load_at(&self, idx: usize) -> T {
		// Just take a reference to the selected element.
		// Notice that this will execute a bounds check.
		let addr: &T = &self.0[idx];

		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		//
		// Also notice that the slice-indexing above gives us a bounds check.
		//
		unsafe { read_value(addr) }
	}

	/// Loads a sub array from the inner array.
	///
	/// This method is analog to a sub-slicing `self.inner[idx..(idx+M)]` but
	/// returning an owned array instead of a slice, simply because it has to
	/// copy the data anyway from the progmem into the data domain (i.e. the
	/// stack).
	///
	/// Also notice, that since this crate is intended for AVR
	/// micro-controllers, static arrays are generally preferred over
	/// dynamically allocated types such as a `Vec` (as of now (mid-2020) there
	/// isn't even a good way to get a `Vec` on AVR in Rust).
	///
	///
	/// # Panics
	///
	/// This method panics, if the given index `idx` is grater or equal to the
	/// length `N` of the inner array, or the end index `idx+M` is grater than
	/// the length `N` of the inner array.
	///
	/// This method also panics, if the size of the value (i.e. `size_of::<[T;M]>()`)
	/// is beyond 255 bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	pub fn load_sub_array<const M: usize>(&self, start_idx: usize) -> [T; M] {
		assert!(M <= N);

		// Make sure that we convert from &[T] to &[T;M] without constructing
		// an actual [T;M], because we MAY NOT LOAD THE DATA YET!
		// Also notice, that this sub-slicing dose ensure that the bound are
		// correct.
		let slice: &[T] = &self.0[start_idx..(start_idx + M)];
		let array: &[T; M] = slice.try_into().unwrap();

		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		//
		// Also notice that the sub-slicing above gives us a bounds check.
		//
		unsafe { read_value(array) }
	}

	/// Lazily iterate over all elements
	///
	/// Returns an iterator which lazily loads the elements one at a time
	/// from progmem.
	/// This means this iterator can be used to access huge arrays while
	/// only requiring `size_of::<T>()` amount of stack memory.
	///
	/// # Panics
	///
	/// This method panics, if the size of an element (i.e. `size_of::<T>()`)
	/// is beyond 255 bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	/// Notice, that here `T` is the type of the elements not the entire array
	/// as it would be with [`load`](Self::load).
	///
	pub fn iter(&self) -> PmIter<T, N> {
		PmIter::new(self)
	}
}


/// An iterator over an array in progmem.
pub struct PmIter<'a, T, const N: usize> {
	progmem: &'a ProgMem<[T; N]>,
	current_idx: usize,
}

impl<'a, T, const N: usize> PmIter<'a, T, N> {
	/// Creates a new iterator over the given progmem array.
	pub const fn new(pm: &'a ProgMem<[T; N]>) -> Self {
		Self {
			progmem: pm,
			current_idx: 0,
		}
	}
}

impl<'a, T: Copy, const N: usize> Iterator for PmIter<'a, T, N> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		// Check for iterator end
		if self.current_idx < N {
			// Load next item from progmem
			let b = self.progmem.load_at(self.current_idx);
			self.current_idx += 1;

			Some(b)
		} else {
			None
		}
	}
}


/// Define a static in progmem.
///
/// This is a helper macro to simplify the definition of statics that are valid
/// to be wrapped in the `ProgMem` struct thus providing a safe way to work
/// with data in progmem.
///
/// Thus this macro essentially takes a user static definition and emits a
/// definition that is defined to be stored in the progmem section and then is
/// wrap in the `ProgMem` wrapper for safe access.
///
/// There are essentially three types of statics that you can created:
///
/// * ordinary fixed-size data, e.g. a `u8`, `(u16,u32)`, or your own struct.
/// * "auto-sized" arrays, essentially any kind of array `[T; N]`
/// * strings, i.e. anything `str`-ish such as string literals
///
///
/// # Ordinary Data
///
/// You can store any `Copy + Sized` data in progmem and load it at your
/// leisure.
///
/// ## Example
///
/// ```
/// use avr_progmem::progmem;
///
/// #[derive(Copy, Clone)]
/// struct Foo {
///     a: u16,
///     b: u32,
/// }
///
/// progmem!{
///     /// Static data stored in progmem!
///     pub static progmem BYTE: u8 = b'a';
///
///     /// Anything that is `Copy + Sized`
///     pub static progmem FOO: Foo = Foo { a: 42, b: 42 * 42 };
/// }
///
/// // Loading the byte from progmem onto the stack
/// let data: u8 = BYTE.load();
/// assert_eq!(b'a', data);
///
/// // Loading the arbitrary data
/// let foo: Foo = FOO.load();
/// assert_eq!(42, foo.a);
/// assert_eq!(1764, foo.b);
/// ```
///
///
/// # Arrays
///
/// Notice, that to access ordinary data from the progmem you have to load it
/// as whole before you can do anything with it.
/// In other words you can't just load `foo.a`, you have to first load the
/// entire struct into RAM.
///
/// When we have arrays, stuff can get hugh quickly, therefore,
/// specifically for arrays, we have additionally accessors to access elements
/// individually, without the burden to load the entire array first.
///
/// ```
/// use avr_progmem::progmem;
///
/// progmem!{
///     /// A simple array using ordinary syntax
///     pub static progmem ARRAY: [u16; 4] = [1, 2, 3, 4];
/// }
///
/// // We can still load the entire array (but you shouldn't do this with
/// // big arrays)
/// let array: [u16; 4] = ARRAY.load();
/// assert_eq!([1,2,3,4], array);
///
/// // We can also load individual elements
/// let last_elem: u16 = ARRAY.load_at(3);
/// assert_eq!(4, last_elem);
///
/// // And even arbitrary sub-arrays (tho they need to be statically sized)
/// let middle_stuff: [u16; 2] = ARRAY.load_sub_array(1);
/// assert_eq!([2, 3], middle_stuff);
///
/// // Finally, we can iterate the array lazily loading one byte after another
/// // so we need only just enough RAM for to handle a single element
/// let mut elem_iter = ARRAY.iter();
/// assert_eq!(Some(1), elem_iter.next());
/// assert_eq!(Some(2), elem_iter.next());
/// assert_eq!(Some(3), elem_iter.next());
/// assert_eq!(Some(4), elem_iter.next());
/// assert_eq!(None, elem_iter.next());
/// ```
///
/// ## Auto-Sizing
///
/// While we could use arrays with the syntax from above, we get also use an
/// alternative syntax, where the array size is gets inferred which is
/// particularly useful if you include external data (e.g. form a file).
///
/// ```
/// use avr_progmem::progmem;
///
/// progmem!{
///     /// An "auto-sized" array (the size is inferred and made accessible by
///     /// a constant named `DATA_LEN`, tho any name would do)
///     pub static progmem<const DATA_LEN: usize> DATA: [u8; DATA_LEN] =
///         *include_bytes!("../examples/test_text.txt"); // assume it's binary
/// }
///
/// // "auto-sized" array can be accessed in the exactly same way as ordinary
/// // arrays, we just don't need to hardcode the size, and even get this nice
/// // constant at our disposal.
/// let middle: u8 = DATA.load_at(DATA_LEN / 2);
/// assert_eq!(32, middle);
/// ```
///
/// # Strings
///
/// Strings are complicated, partially, because in Rust strings such as `str`
/// are unsized making storing them a nightmare (normally the compiler somehow
/// manages to automagically put all your string literals into static memory,
/// but you can't have a static storing a `str` without the `&` specifically).
/// The next best thing is to store some fix-size array either of `char`s or
/// of UTF-8 encoded `u8`s.
/// However, due to this, this crate dedicated an entire
/// [module to strings](crate::string).
///
/// Still, this macro has some special syntax to make string literals,
/// which originally are `str`, into something more manageable (i.e. a
/// [`PmString`](crate::string::PmString)) and put it into a progmem static.
///
/// ## Examples
///
/// ```rust
/// #![feature(const_option)]
///
/// use avr_progmem::progmem;
///
/// progmem! {
///     /// A static string stored in program memory.
///     static progmem string TEXT = "Unicode text: 🦀";
/// }
///
/// let text = TEXT.load();
/// assert_eq!("Unicode text: 🦀", &*text);
/// ```
///
#[macro_export]
macro_rules! progmem {
	// Special string rule
	(
		$( #[ $attr:meta ] )*
		$vis:vis static progmem string $name:ident = $value:expr ;

		$($rest:tt)*
	) => {
		// Just forward to internal rule
		$crate::progmem_internal!{
			$(#[$attr])*
			$vis static progmem string $name = $value ;
		}

		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!{
			$($rest)*
		}
	};

	// Catch strings rule, better use the above special rule
	(
		$( #[ $attr:meta ] )*
		$vis:vis static progmem $name:ident : LoadedString < $ty:literal > = LoadedString :: new ( $value:expr ) $( . unwrap () $(@ $unwrapped:ident)? )? ;

		$($rest:tt)*
	) => {
		// Use an anonymous constant to scope the types used for the warning.
		const _ : () = {
			#[deprecated = concat!("Prefer using the special `PmString` rule. Try: ", stringify!($vis), " static progmem string ", stringify!($name), " = ", stringify!($value), ";")]
			struct $name;

			let _ = $name;
		};

		// Crate the progmem static via internal macro
		$crate::progmem_internal!{
			$(#[$attr])* $vis static progmem $name : LoadedString < $ty > = LoadedString :: new ( $value ) $( . unwrap() $($unwrapped)?)?;
		}

		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!{
			$($rest)*
		}
	};

	// Catch references rule, reference are evil!
	// (well actually they are not, but most likely using them *is* a mistake)
	(
		$( #[ $attr:meta ] )*
		$vis:vis static progmem $name:ident : & $ty:ty = $value:expr ;

		$($rest:tt)*
	) => {
		// Use an anonymous constant to scope the types used for the warning.
		const _ : () = {
			#[deprecated = "You should not use a reference type for progmem, because this way only the reference itself will be in progmem, whereas the underlying data will not be in progmem!"]
			struct $name;

			let _ = $name;
		};

		// Crate the progmem static via internal macro
		$crate::progmem_internal!{
			$(#[$attr])* $vis static progmem $name : & $ty = $value;
		}

		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!{
			$($rest)*
		}
	};

	// Standard rule
	(
		$( #[ $attr:meta ] )*
		$vis:vis static progmem $( < const $size_name:ident : usize > )? $name:ident : $ty:ty = $value:expr ;

		$($rest:tt)*
	) => {
		// Crate the progmem static via internal macro
		$crate::progmem_internal!{
			$(#[$attr])* $vis static progmem $( < const $size_name : usize > )? $name : $ty = $value;
		}

		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!{
			$($rest)*
		}
	};

	// Empty rule
	() => ()
}



/// Only for internal use. Use the `progmem!` macro instead.
#[doc(hidden)]
#[macro_export]
macro_rules! progmem_internal {
	// The string rule creating the progmem string static via `PmString`
	{
		$( #[ $attr:meta ] )*
		$vis:vis static progmem string $name:ident = $value:expr ;
	} => {

		// PmString must be stored in the progmem or text section!
		// The link_section lets us define it:
		#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]

		// User attributes
		$(#[$attr])*
		// The actual static definition
		$vis static $name : $crate::string::PmString<{
			// This bit runs at compile-time
			let s: &str = $value;
			s.len()
		}> =
			unsafe {
				// SAFETY: This call is sound, be cause we ensure with the above
				// link_section attribute that this value is indeed in the
				// progmem section.
				$crate::string::PmString::new( $value )
			}.unwrap();
	};

	// The rule creating an auto-sized progmem static via `ProgMem`
	{
		$( #[ $attr:meta ] )*
		$vis:vis static progmem < const $size_name:ident : usize > $name:ident : $ty:ty = $value:expr ;
	} => {
		// Create a constant with the size of the value, which is retrieved
		// via `SizedOwned` on the value, assuming it is an array of sorts.
		//#[doc = concat!("Size of [", stringify!( $name ))]
		$vis const $size_name : usize = {
			// This bit is a bit hacky, we just hope that the type of `$value`
			// has some `len` method.
			$value.len()
		};

		// Just a normal prgomem static, `$ty` may use the above constant
		$crate::progmem_internal!{
			$( #[ $attr ] )*
			$vis static progmem $name : $ty = $value ;
		}
	};

	// The normal rule creating a progmem static via `ProgMem`
	{
		$( #[ $attr:meta ] )*
		$vis:vis static progmem $name:ident : $ty:ty = $value:expr ;
	} => {
		// ProgMem must be stored in the progmem or text section!
		// The link_section lets us define it:
		#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]

		// User attributes
		$(#[$attr])*
		// The actual static definition
		$vis static $name : $crate::ProgMem<$ty> =
			unsafe {
				// SAFETY: This call is safe, be cause we ensure with the above
				// link_section attribute that this value is indeed in the
				// progmem section.
				$crate::ProgMem::new( $value )
			};
	};
}
