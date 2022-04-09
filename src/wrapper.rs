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
///
/// # Examples
///
/// ```
/// use avr_progmem::progmem;
///
/// progmem!{
///     /// Static string stored in progmem!
///     pub static progmem WORDS: [u8; 4] = *b"abcd";
/// }
///
/// let data: [u8; 4] = WORDS.load();
/// assert_eq!(b"abcd", &data);
/// ```
///
/// ```
/// use avr_progmem::progmem;
///
/// progmem!{
///     /// 4kB string stored in progmem!
///     pub static progmem WORDS: [u8; 4096] = [b'X'; 4096];
/// }
/// let first_bytes: [u8; 16] = WORDS.load_sub_array(0);
/// assert_eq!([b'X'; 16], first_bytes);
/// ```
///
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
