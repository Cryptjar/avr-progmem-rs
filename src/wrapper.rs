//! Best-effort safe wrapper for progmem.
//!
//! This module offers the [`ProgMem`] struct that wraps pointers into progmem,
//! and only gives access to that value via methods that first load the value
//! into the normal data memory domain.
//! This is also the reason why the value must be `Copy` and is always returned
//! by-value instead of by-reference (since the value is not in the data memory
//! where it could be referenced).
//!
//! Since the `ProgMem` struct loads the value using special instructions,
//! it really must be in progmem, otherwise it would be **undefined behavior**
//! to use any of its methods.
//! Therefore, its constructor is `unsafe` where the
//! caller must guarantee that the given pointer truly points to a valid value
//! stored in progmem.
//!
//! As convenience, the [`progmem!`] macro is offered that will create
//! a `static` in progmem with the given value and wrap a pointer to it in the
//! [`ProgMem`] struct for you.



use crate::raw::read_value;



/// Best-effort safe wrapper around a value in program memory.
///
/// This type wraps a pointer to a value that is stored in program memory,
/// and offers safe functions to [`load`](ProgMem::load) that value from
/// program memory into the data memory domain from where it can be normally
/// used.
///
/// Since its constructor is the single most critical point in its API,
/// it is `unsafe`, despite it is supposed to be a safe wrapper (hence the
/// 'best-effort' notation).
/// The caller of the constructor therefore must ensure that the supplied
/// pointer points to a valid value stored in program memory.
///
/// Consequently, the only way to use this struct soundly is to define a
/// `static` with the `#[link_section = ".progmem.data"]` attribute on it and
/// pass a pointer to that `static` to `ProgMem::new`.
/// However, having an accessible `static` around that is stored in progmem
/// is a very dangerous endeavor.
///
/// In order to make working with progmem safer and more convenient,
/// consider using the [`progmem!`] macro, that will put the given data
/// into a hidden `static` in progmem and provide you with an accessible static
/// containing the pointer to it wrapped in `ProgMem`.
///
///
/// # Safety
///
/// The `target` pointer in this struct must point to a valid object of type
/// `T` that is stored in the program memory domain.
/// The object must be initialized, readable, and immutable (i.e. it must not
/// be changed).
/// Also the `target` pointer must be valid for the `'static` lifetime.
///
/// However, the above requirement only applies to the AVR architecture
/// (`#[cfg(target_arch = "avr")]`), because otherwise normal data access
/// primitives are used. This means that the value must be stored in the
/// regular data memory domain for ALL OTHER architectures! This still
/// holds, even if such other architecture is of the Harvard architecture,
/// because this is an AVR-only crate, not a general Harvard architecture
/// crate!
///
#[non_exhaustive] // SAFETY: Must not be publicly creatable
pub struct ProgMem<T> {
	/// Points to some `T` in progmem.
	///
	/// # Safety
	///
	/// See the struct doc.
	target: *const T,
}

unsafe impl<T> Send for ProgMem<T> {
	// SAFETY: pointers per-se are sound to send & share.
	// Further more, we will never mutate the underling value, thus `ProgMem`
	// can be considered as some sort of sharable `'static` "reference".
	// Thus it can be shared and transferred between threads.
}

unsafe impl<T> Sync for ProgMem<T> {
	// SAFETY: pointers per-se are sound to send & share.
	// Further more, we will never mutate the underling value, thus `ProgMem`
	// can be considered as some sort of sharable `'static` "reference".
	// Thus it can be shared and transferred between threads.
}

impl<T> ProgMem<T> {
	/// Construct a new instance of this type.
	///
	/// This struct is a pointer wrapper for data in the program memory domain.
	/// Therefore when constructing this struct, it must be guaranteed
	/// that the pointed data is stored in progmem!
	/// This contract is expressed by the fact that this function is `unsafe`.
	/// See the Safety section for details.
	///
	/// You should not need to call this function directly.
	/// It is recommended to use the [`progmem!`] macro instead (which calls
	/// this constructor for you, while enforcing its contract.
	///
	///
	/// # Safety
	///
	/// The `ProgMem` wrapper is build around the invariant that the wrapped
	/// pointer is stored in the program code memory domain (on the AVR
	/// architecture).
	///
	/// That means that this function is only sound to call, if the value to
	/// which `target` points is stored in a `static` that is stored in progmem,
	/// e.g. by using the attribute `#[link_section = ".progmem.data"]`.
	///
	/// However, the above requirement only applies to the AVR architecture
	/// (`#[cfg(target_arch = "avr")]`), because otherwise normal data access
	/// primitives are used. This means that the value must be stored in the
	/// regular data memory domain for ALL OTHER architectures! This still
	/// holds, even if such other architecture is of the Harvard architecture,
	/// because this is an AVR-only crate, not a general Harvard architecture
	/// crate!
	///
	pub const unsafe fn new(target: *const T) -> Self {
		ProgMem {
			target,
		}
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
	/// close to a stack overflow. Thus it might be better to restructure your
	/// data, so you can store it as an array of something, than you can use
	/// the [`load_at`] and [`load_sub_array`] methods instead.
	///
	/// [`load_at`]: struct.ProgMem.html#method.load_at
	/// [`load_sub_array`]: struct.ProgMem.html#method.load_sub_array
	///
	pub fn load(&self) -> T {
		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		unsafe { read_value(self.target) }
	}

	/// Return the raw pointer to the inner value.
	///
	/// Notice that the returned pointer is indeed a pointer into the progmem
	/// domain! It may never be dereferenced via the default Rust operations.
	/// That means a `unsafe{*pm.get_inner_ptr()}` is **undefined behavior**!
	///
	/// Instead, if you want to use the pointer, you may want to use one of
	/// the "raw" functions, see the [raw](crate::raw) module.
	///
	pub fn as_ptr(&self) -> *const T {
		self.target
	}
}

/// Utilities to work with an array in progmem.
impl<T: Copy, const N: usize> ProgMem<[T; N]> {
	/// Load a single element from the inner array.
	///
	/// This method is analog to a slice indexing `self.load()[idx]`, so the
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
		// SAFETY: check that `idx` is in bounds
		assert!(idx < N, "Given index is out of bounds");

		let first_element_ptr: *const T = self.target.cast();

		// Get a point to the selected element
		let element_ptr = first_element_ptr.wrapping_add(idx);

		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		//
		// Also notice that the slice-indexing above gives us a bounds check.
		//
		unsafe { read_value(element_ptr) }
	}

	/// Loads a sub array from the inner array.
	///
	/// This method is analog to a sub-slicing `self.load()[idx..(idx+M)]` but
	/// returning an owned array instead of a slice, simply because it has to
	/// copy the data anyway from the progmem into the data domain (i.e. the
	/// stack).
	///
	/// Also notice, that since this crate is intended for AVR
	/// micro-controllers, static arrays are generally preferred over
	/// dynamically allocated types such as a `Vec`.
	///
	///
	/// # Panics
	///
	/// This method panics, if the given index `idx` is grater or equal to the
	/// length `N` of the inner array, or the end index `idx+M` is grater than
	/// the length `N` of the inner array.
	///
	/// This method also panics, if the size of the value
	/// (i.e. `size_of::<[T;M]>()`) is beyond 255 bytes.
	/// However, this is currently just a implementation limitation, which may
	/// be lifted in the future.
	///
	pub fn load_sub_array<const M: usize>(&self, start_idx: usize) -> [T; M] {
		// Just a check to give a nicer panic message
		assert!(
			M <= N,
			"The sub array can not be grater than the source array"
		);

		// SAFETY: bounds check, the last element of the sub array must
		// still be within the source array (i.e. self)
		assert!(
			start_idx + M <= N,
			"The sub array goes beyond the end of the source array"
		);

		let first_source_element_ptr: *const T = self.target.cast();

		// Get a point to the selected element
		let first_output_element_ptr = first_source_element_ptr.wrapping_add(start_idx);

		// Pointer into as sub array into the source
		let sub_array_ptr: *const [T; M] = first_output_element_ptr.cast();

		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		//
		// Also notice that the sub-slicing above gives us a bounds check.
		//
		unsafe { read_value(sub_array_ptr) }
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
/// use avr_progmem::progmem;
///
/// progmem! {
///     /// A static string stored in program memory.
///     static progmem string TEXT = "Unicode text: 大賢者";
/// }
///
/// let text = TEXT.load();
/// assert_eq!("Unicode text: 大賢者", &*text);
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
			#[allow(non_camel_case_types)]
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
			#[allow(non_camel_case_types)]
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


#[doc(hidden)]
pub const fn array_from_str<const N: usize>(s: &str) -> [u8; N] {
	let array_ref = crate::string::from_slice::array_ref_try_from_slice(s.as_bytes());
	match array_ref {
		Ok(r) => *r,
		Err(_) => panic!("Invalid array size"),
	}
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
		// User attributes
		$(#[$attr])*
		// The facade static definition, this only contains a pointer and thus
		// is NOT in progmem, which in turn makes it safe & sound to access this
		// facade.
		$vis static $name: $crate::string::PmString<{
			// This bit runs at compile-time
			let s: &str = $value;
			s.len()
		}> = {
			// This inner hidden static contains the actual real raw value.
			//
			// SAFETY: it must be stored in the progmem or text section!
			// The `link_section` lets us define that:
			#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
			static VALUE: [u8; {
				// This bit runs at compile-time
				let s: &str = $value;
				s.len()
			}] = $crate::wrapper::array_from_str( $value );

			let pm = unsafe {
				// SAFETY: This call is sound because we ensure with the above
				// `link_section` attribute on `VALUE` that it is indeed
				// in the progmem section.
				$crate::wrapper::ProgMem::new(
					// TODO: use the `addr_of` macro here!!!
					& VALUE
				)
			};

			// Just return the PmString wrapper around the local static
			unsafe {
				// SAFETY: This call is sound, because we started out with a
				// `&str` thus the conent of `VALUE` must be valid UTF-8
				$crate::string::PmString::new(
					pm
				)
			}
		};
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
		// User attributes
		$(#[$attr])*
		// The facade static definition, this only contains a pointer and thus
		// is NOT in progmem, which in turn makes it safe & sound to access this
		// facade.
		$vis static $name: $crate::wrapper::ProgMem<$ty> = {
			// This inner hidden static contains the actual real raw value.
			//
			// SAFETY: it must be stored in the progmem or text section!
			// The `link_section` lets us define that:
			#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
			static VALUE: $ty = $value;

			unsafe {
				// SAFETY: This call is sound because we ensure with the above
				// `link_section` attribute on `VALUE` that it is indeed
				// in the progmem section.
				$crate::wrapper::ProgMem::new(
					// TODO: use the `addr_of` macro here!!!
					& VALUE
				)
			}
		};
	};
}
