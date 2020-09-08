
// We don't need anything from std, and on AVR there is no std anyway.
#![no_std]

// We need inline assembly for the `lpm` instruction.
#![feature(llvm_asm)]

// We need const generics, however the `const_generics` feature is reported as
// incomplete, thus we actually use the `min_const_generics` feature, which is
// sufficient for us. However, min_const_generics in turn fails to work with
// `cargo doc`, thus when documenting we fallback to the incomplete
// `const_generics` feature, because it has actual doc support.
#![cfg_attr(doc, feature(const_generics))]
#![cfg_attr(not(doc), feature(min_const_generics))]


//!
//! Progmem utilities for the AVR architectures.
//!
//! This crate provides unsafe utilities for working with data stored in
//! the program memory of an AVR micro-controller. Additionally, it defines a
//! 'best-effort' safe wrapper struct [`ProgMem`] to simplify working with it.
//!
//! This crate is implemented only in Rust and some short assembly, it does NOT
//! depend on the [`avr-libc`] or any other C-library. However, due to the use
//! of inline assembly, this crate may only be compiled using a **nightly Rust**
//! compiler.
//!
//!
//! # AVR Memory
//!
//! This crate is specifically for [AVR-base micro-controllers][avr] such as
//! the Arduino Uno (and some other Arduino boards, but not all), which have a
//! modified Harvard architecture which implies the strict separation of program
//! code and data while having special instructions to read and write to
//! program memory.
//!
//! While, of course, all ordinary data is stored in the data domain where it is
//! perfectly usable, the harsh constraints of most AVR processors make it very
//! appealing to use the program memory (also referred to as _progmem_) for
//! storing constant values. However, due to the Harvard design, those values
//! are not usable with normal instructions (i.e. those emitted from normal
//! Rust code). Instead, special instructions are required to load data from
//! the program code domain, i.e. the `lpm` (load _from_ program memory)
//! instruction. And because there is no way to emit it from Rust code, this
//! crate uses inline assembly to emit that instruction.
//!
//! However, since a pointer into program code cannot be differentiated from a
//! normal data pointer, it is entirely up to the programmer to ensure that
//! these different 'pointer-types' are not accidentally mixed. In other words,
//! this is `unsafe` in the context of Rust.
//!
//!
//! # Loading Data from Program Memory
//!
//! The first part of this crate simply provides a few functions (e.g.
//! [`read_byte`]) to load constant data (i.e. a Rust `static` that is
//! immutable) from the program memory into the data domain, so that
//! sub-sequentially it is normal usable data, i.e. as owned data on the stack.
//!
//! Because, as aforementioned, a simple `*const u8` in Rust does not specify
//! whether is lives in the program code domain or the data domain, all
//! functions which simply load a given pointer from the program memory are
//! inherently `unsafe`.
//!
//! Notice that using a `&u8` reference might make things rather worse than
//! safe. Because keeping a pointer/reference/address into the program memory
//! as Rust reference might easily cause it to be dereferenced, even in safe
//! code. But since that address is only valid in the program code domain (and
//! Rust doesn't know about it) it would illegally load the address from the
//! data memory, causing **undefined behavior**!
//!
//! ## Example
//!
//! ```
//! use avr_progmem::read_byte;
//!
//! // This `static` must never be directly dereferenced/accessed!
//! // So a `let data: u8 = P_BYTE;` is **undefined behavior**!!!
//! /// Static byte stored in progmem!
//! #[link_section = ".progmem"]
//! static P_BYTE: u8 = b'A';
//!
//! // Load the byte from progmem
//! // Here, it is sound, because due to the link_section it is indeed in the
//! // program code memory.
//! let data: u8 = unsafe { read_byte(&P_BYTE) };
//! assert_eq!(b'A', data);
//! ```
//!
//!
//! # The best-effort Wrapper
//!
//! Since working with progmem data is inherently unsafe and rather
//! difficult to do correctly, this crate introduces the best-effort 'safe'
//! wrapper [`ProgMem`], that is supposed to only wrap data in progmem, thus
//! offering only functions to load its content using the progmem loading
//! function.
//! The latter are fine and safe, given that the wrapper type really contains
//! data in the program memory. Therefore, to keep that invariant up, the
//! constructor is `unsafe`.
//!
//! Yet to make that also easier, this crate provides the [`progmem!`] macro
//! (it has to be a macro), which will create a static variable in program
//! memory for you and wrap it in the `ProgMem` struct. It will ensure that the
//! `static` will be stored in the program memory by defining the
//! `#[link_section = ".progmem"]` attribute on it. This makes the load
//! functions on that struct sound and additionally prevents users to
//! accidentally access that `static` directly, which, since it is in progmem,
//! would be fundamentally unsound.
//!
//! ## Example
//!
//! ```
//! use avr_progmem::progmem;
//!
//! // It will be wrapped in the ProgMem struct and expand to:
//! // ```
//! // #[link_section = ".progmem"]
//! // static P_BYTE: ProgMem<u8> = unsafe { ProgMem::new(b'A') };
//! // ```
//! // Thus it is impossible for safe Rust to directly dereference/access it!
//! progmem! {
//!     /// Static byte stored in progmem!
//!     static progmem P_BYTE: u8 = b'A';
//! }
//!
//! // Load the byte from progmem
//! // It is still sound, because the `ProgMem` guarantees us that it comes
//! // from the program code memory.
//! let data: u8 = P_BYTE.load();
//! assert_eq!(b'A', data);
//! ```
//!
//!
//! # Other Architectures
//!
//! As mentioned before, this crate is specifically designed to be use with
//! AVR-base micro-controllers. But since most of us don't write their programs
//! on an AVR system but e.g. on x86 systems, and might want to test them
//! there (well as far as it is possible), this crate also has a fallback
//! implementation for all other architectures that are not AVR, falling back
//! to a simple Rust `static` in the default data segment. And all the data
//! loading functions will just dereference the pointed-to data, assuming that
//! they just live in the default location.
//!
//! This fallback is perfectly safe on x86 and friend, and should also be fine
//! on all further architectures, otherwise normal Rust `static`s would be
//! broken. However, it is an important point to know when for instance writing
//! a library that is not limited to AVR.
//!
//!
//! # Implementation Limitations
//!
//! Aside from what has been already been covered, the current implementation
//! has two further limitations.
//!
//! First, since this crate uses an inline assembly loop on a 8-bit
//! architecture, the loop counter only allows values up to 255. This means
//! that not more that 255 bytes can be loaded at once with any of the methods
//! of this crate. However, this only applies to a single continuous load
//! operation, so for instance `ProgMem<[u8;1024]>::load()` will panic, but
//! accessing such a big type in smaller chunks e.g.
//! `ProgMem<[u8;1024]>::load_sub_array::<[u8;128]>(512)` is perfectly fine
//! because the to be loaded type `[u8;128]` is only 128 bytes in size.
//!
//! Second, since this crate only uses the `lpm` instruction, which is limited
//! by a 16-bit pointer, this crate may only be used with data stored in the
//! lower 64 kiB of program memory. Since this property has not be tested it is
//! unclear whether it will cause a panic or right-up undefined behavior, so be
//! very wary when working with AVR chips having more then 64 kiB of program
//! memory.
//! This second restriction, of course, dose not apply to non-AVR architectures.
//!
//!
//! [`ProgMem`]: https://docs.rs/avr-progmem/latest/avr_progmem/struct.ProgMem.html
//! [`read_byte`]: https://docs.rs/avr-progmem/latest/avr_progmem/fn.read_byte.html
//! [`progmem!`]: https://docs.rs/avr-progmem/latest/avr_progmem/macro.progmem.html
//! [`avr-libc`]: https://crates.io/crates/avr-libc
//! [avr]: https://en.wikipedia.org/wiki/AVR_microcontrollers
//!


use core::mem::size_of;
use core::mem::MaybeUninit;
use core::convert::TryInto;

use cfg_if::cfg_if;


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
/// the `#[link_section = ".progmem"]` (or `".text"`) on a static that contains
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
	/// `#[link_section = ".progmem"]`.
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
		unsafe {
			read_value(p_addr)
		}
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
impl<T: Copy, const N: usize> ProgMem<[T;N]> {

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
		unsafe {
			read_value(addr)
		}
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
	pub fn load_sub_array<const M: usize>(&self, start_idx: usize) -> [T;M] {
		assert!(M <= N);

		// Make sure that we convert from &[T] to &[T;M] without constructing
		// an actual [T;M], because we MAY NOT LOAD THE DATA YET!
		// Also notice, that this sub-slicing dose ensure that the bound are
		// correct.
		let slice: &[T] = &self.0[start_idx..(start_idx+M)];
		let array: &[T;M] = slice.try_into().unwrap();

		// This is safe, because the invariant of this struct demands that
		// this value (i.e. self and thus also its inner value) are stored
		// in the progmem domain, which is what `read_value` requires from us.
		//
		// Also notice that the sub-slicing above gives us a bounds check.
		//
		unsafe {
			read_value(array)
		}
	}
}

/// Only for internal use. Use the `progmem!` macro instead.
#[doc(hidden)]
#[macro_export]
macro_rules! progmem_internal {
	{
		$(#[$attr:meta])*
		($($vis:tt)*) static $name:ident : $ty:ty = $value:expr ;
	} => {
		// ProgMem must be stored in the progmem or text section!
		// The link_section lets us define it.
		#[cfg_attr(target_arch = "avr", link_section = ".progmem")]
		// User attributes
		$(#[$attr])*
		// The actual static definition
		$($vis)* static $name : $crate::ProgMem<$ty> =
			unsafe {
				// This call is safe, be cause we ensure with the above
				// link_section attribute that this value is indeed in the
				// progmem section.
				$crate::ProgMem::new( $value )
			};
	};
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
	// Match private (not pub) definitions.
	($(#[$attr:meta])* static progmem $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
		// use `()` to explicitly forward the information about private items
		$crate::progmem_internal!($(#[$attr])* () static $N : $T = $e;);
		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!($($t)*);
	};
	// Match simple public (just pub) definitions.
	($(#[$attr:meta])* pub static progmem $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
		$crate::progmem_internal!($(#[$attr])* (pub) static $N : $T = $e;);
		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!($($t)*);
	};
	// Match public path (pub with path) definitions.
	($(#[$attr:meta])* pub ($($vis:tt)+) static progmem $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
		$crate::progmem_internal!($(#[$attr])* (pub ($($vis)+)) static $N : $T = $e;);
		// Recursive call to allow multiple items in macro invocation
		$crate::progmem!($($t)*);
	};
	() => ()
}


/// Read a single byte from the progmem.
///
/// This function reads just a single byte from the program code memory domain.
/// Thus this is essentially a Rust function around the AVR `lpm` instruction.
///
/// If you need to read from an array you might use [`read_slice`] or
/// just generally for any value (including arrays) [`read_value`].
///
/// ## Example
///
/// ```
/// use avr_progmem::read_byte;
///
/// // This static must never be directly dereferenced/accessed!
/// // So a `let data: u8 = P_BYTE;` is Undefined Behavior!!!
/// /// Static byte stored in progmem!
/// #[link_section = ".progmem"]
/// static P_BYTE: u8 = b'A';
///
/// // Load the byte from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// let data: u8 = unsafe { read_byte(&P_BYTE) };
/// assert_eq!(b'A', data);
/// ```
///
///
/// # Safety
///
/// The given point must be valid in the program domain which in AVR normal
/// pointers (to data) are NOT, because they point into the data domain.
///
/// Typically only function pointers (which make no sense here) and pointer to
/// or into statics that are defined to be stored into progmem are valid.
/// For instance, a valid progmem statics would be one, that is attributed with
/// `#[link_section = ".progmem"]`.
///
/// Also general Rust pointer dereferencing constraints apply, i.e. it must not
/// be dangling.
///
/// [`read_slice`]: fn.read_slice.html
/// [`read_value`]: fn.read_value.html
///
pub unsafe fn read_byte(p_addr: *const u8) -> u8 {
	cfg_if! {
		if #[cfg(target_arch = "avr")] {
			// Only addresses below the 64 KiB limit are supported!
			// Apparently this is of no concern for architectures with true
			// 16-bit pointers.
			// TODO: switch to use the extended lpm instruction if >64k
			assert!(p_addr as usize <= u16::MAX as usize);

			// Allocate a byte for the output (actually a single register r0
			// will be used).
			let res: u8;

			// The inline assembly to read a single byte from given address
			llvm_asm!(
				// Just issue the single `lpm` assembly instruction, which reads
				// implicitly indirectly the address from the Z register, and
				// stores implicitly the read value in the register 0.
				"lpm"
				// Output is in the register 0
				: "={r0}"(res)
				// Input the program memory address to read from
				: "z"(p_addr)
				// No clobber list.
			);

			// Just output the read value
			res

		} else {
			// This is the non-AVR dummy.
			// We have to assume that otherwise a normal data or text segment
			// would be used, and thus that it is actually save to access it
			// directly!

			// Notice the above assumption fails and results in UB for any other
			// Harvard architecture other than AVR.

			*p_addr
		}
	}
}

/// Read an array of type `T` from progmem into data array.
///
/// This function uses the above byte-wise `read_byte` function instead
/// of the looped assembly of `read_asm_loop_raw`.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy(p_addr, out, len as usize)` thus it
/// has the same basic requirements such as both pointers must be valid for
/// dereferencing i.e. not dangling and both pointers must
/// be valid to read or write, respectively, of `len` many elements of type `T`,
/// i.e. `len * size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain. And `out` must be valid point to a writable location in the data
/// memory.
///
/// However alignment is not strictly required for AVR, since the read/write is
/// done byte-wise.
///
unsafe fn read_byte_loop_raw<T>(p_addr: *const T, out: *mut T, len: u8)
		where T: Sized + Copy {

	// Convert to byte pointers
	let p_addr_bytes = p_addr as *const u8;
	let out_bytes = out as *mut u8;

	// Get size in bytes of T
	let size_type = size_of::<T>();
	// Must not exceed 256 byte
	assert!(size_type <= u8::MAX as usize);

	// Multiply with the given length
	let size_bytes = size_type * len as usize;
	// Must still not exceed 256 byte
	assert!(size_bytes <= u8::MAX as usize);
	// Now its fine to cast down to u8
	let size_bytes = size_bytes as u8;

	for i in 0..size_bytes {
		let i: isize = i.into();

		let value = read_byte(p_addr_bytes.offset(i));
		out_bytes.offset(i).write(value);
	}
}

/// Read an array of type `T` from progmem into data array.
///
/// This function uses the optimized `read_asm_loop_raw` with a looped
/// assembly instead of byte-wise `read_byte` function.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy(p_addr, out, len as usize)` thus it
/// has the same basic requirements such as both pointers must be valid for
/// dereferencing i.e. not dangling and both pointers must
/// be valid to read or write, respectively, of `len` many elements of type `T`,
/// i.e. `len * size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain. And `out` must be valid point to a writable location in the data
/// memory.
///
/// However alignment is not strictly required for AVR, since the read/write is
/// done byte-wise, but the non-AVR fallback dose actually use
/// `core::ptr::copy` and therefore the pointers must be aligned.
///
unsafe fn read_asm_loop_raw<T>(p_addr: *const T, out: *mut T, len: u8) {

	// Here are the general requirements essentially required by the AVR-impl
	// However, assume, the non-AVR version is only used in tests, it makes a
	// lot of sens to ensure the AVR requirements are held up.

	// Loop head check, just return for zero iterations
	if len == 0 || size_of::<T>() == 0 {
		return
	}

	// Get size in bytes of T
	let size_type = size_of::<T>();
	// Must not exceed 256 byte
	assert!(size_type <= u8::MAX as usize);

	// Multiply with the given length
	let size_bytes = size_type * len as usize;
	// Must still not exceed 256 byte
	assert!(size_bytes <= u8::MAX as usize);
	// Now its fine to cast down to u8
	let size_bytes = size_bytes as u8;


	cfg_if!{
		if #[cfg(target_arch = "avr")] {
			// Only addresses below the 64 KiB limit are supported
			// Apparently this is of no concern for architectures with true
			// 16-bit pointers.
			// TODO: switch to use the extended lpm instruction if >64k
			assert!(p_addr as usize <= u16::MAX as usize);

			// A loop to read a slice of T from prog memory
			// The prog memory address (addr) is stored in the 16-bit address
			// register Z (since this is the default register for the `lpm`
			// instruction).
			// The output data memory address (out) is stored in the 16-bit
			// address register X, because Z is already used and Y seams to be
			// used other wise or is callee-save, whatever, it emits more
			// instructions by llvm.
			//
			// This loop appears in the assembly, because it allows to exploit
			// `lpm 0, Z+` instruction that simultaneously increments the
			// pointer.
			llvm_asm!(
				"
					// load value from program memory at indirect Z into register 0
					// and increment Z by one
					lpm 0, Z+
					// write register 0 to data memory at indirect X
					// and increment X by one
					st X+, 0
					// Decrement the loop counter in register $0 (size_bytes).
					// If zero has been reached the equality flag is set.
					subi $0, 1
					// Check whether the end has not been reached and if so jump back.
					// The end is reached if $0 (size_bytes) == 0, i.e. equality flag
					// is set.
					// Thus if equality flag is NOT set (brNE) jump back by 4
					// instruction, that are all instructions in this assembly.
					// Notice: 4 instructions = 8 Byte
					brne -8
				"
				// No direct outputs
				:
				// Input the iteration count, input program memory address,
				// and output data memory address
				: "r"(size_bytes), "z"(p_addr), "x"(out)
				// The register 0 is clobbered
				: "0"
			);

		} else {
			// This is a non-AVR dummy.
			// We have to assume that otherwise a normal data or text segment
			// would be used, and thus that it is actually save to access it
			// directly!

			// Notice the above assumption fails and results in UB for any other
			// Harvard architecture other than AVR.

			// Now, just copy the bytes from p_addr to out
			// It is save by the way, because we require the user to give use
			// pointer valid for exactly that case.
			core::ptr::copy(p_addr, out, len as usize);
		}
	}
}


/// Read an array of type `T` from progmem into data array.
///
/// This function uses either the optimized `read_asm_loop_raw` with a
/// looped assembly instead of byte-wise `read_byte` function depending
/// whether the `lpm-asm-loop` crate feature is set.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy(p_addr, out, len as usize)` thus it
/// has the same basic requirements such as both pointers must be valid for
/// dereferencing i.e. not dangling and both pointers must
/// be valid to read or write, respectively, of `len` many elements of type `T`,
/// i.e. `len * size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain. And `out` must be valid point to a writable location in the data
/// memory.
///
/// While the alignment is not strictly required for AVR, the non-AVR fallback
/// might be done actually use `core::ptr::copy` and therefore the pointers
/// must be aligned.
///
unsafe fn read_value_raw<T>(p_addr: *const T, out: *mut T, len: u8)
		where T: Sized + Copy {

	cfg_if!{
		if #[cfg(feature = "lpm-asm-loop")] {
			read_asm_loop_raw(p_addr, out, len)
		} else {
			read_byte_loop_raw(p_addr, out, len)
		}
	}
}


/// Read a slice of type `T` from progmem into given slice in data memory.
///
/// This function uses either a optimized assembly with loop or just a
/// byte-wise assembly function which is looped outside depending on
/// whether the `lpm-asm-loop` crate feature is set or not.
///
/// If you need to read just a single byte you might use [`read_byte`] or
/// just generally for any value (including arrays) [`read_value`].
///
/// ## Example
///
/// ```
/// use avr_progmem::read_slice;
///
/// // This static must never be directly dereferenced/accessed!
/// // So a `let data: [u8;11] = P_ARRAY;` is Undefined Behavior!!!
/// // Also notice the `*` in front of the string, because we want to store the
/// // data, not just a reference!
/// /// Static bytes stored in progmem!
/// #[link_section = ".progmem"]
/// static P_ARRAY: [u8;11] = *b"Hello World";
///
/// // Notice since we use a sub-slice the data better is pre-initialized even
/// // tho we will override it.
/// let mut data = [0u8; 5];
///
/// // Load the bytes from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// unsafe { read_slice(&P_ARRAY[0..5], &mut data[..]) };
/// assert_eq!(b"Hello", &data);
/// ```
///
///
/// # Panics
///
/// This function panics if the given slices `p` and `out` have a different
/// lengths.
///
/// This function also panics, if the size of the value (i.e. `p.len() * size_of::<T>()`)
/// is beyond 255 bytes.
/// However, this is currently just a implementation limitation, which may
/// be lifted in the future.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy(p_addr, out, len as usize)` thus it
/// has the same basic requirements such as both pointers must be valid for
/// dereferencing i.e. not dangling and both pointers must
/// be valid to read or write, respectively, of `len` many elements of type `T`,
/// i.e. `len * size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain. And `out` must be valid point to a writable location in the data
/// memory.
///
/// While the alignment is not strictly required for AVR, the non-AVR fallback
/// might be done actually use `core::ptr::copy` and therefore the pointers
/// must be aligned.
///
/// Also notice, that the output slice must be correctly initialized, it would
/// be UB if not. If you don't want to initialize the data upfront, the
/// `read_value` might be a good alternative.
///
/// [`read_byte`]: fn.read_byte.html
/// [`read_value`]: fn.read_value.html
///
#[cfg_attr(feature = "dev", inline(never))]
pub unsafe fn read_slice(p: &[u8], out: &mut [u8]) {
	assert_eq!(p.len(), out.len());
	assert!(p.len() <= u8::MAX as usize);

	let p_addr: *const u8 = &p[0];
	let out_bytes: *mut u8 = &mut out[0];
	let len: u8 = out.len() as u8;

	read_value_raw(p_addr, out_bytes, len);
}


/// Read a single `T` from progmem and return it by value.
///
/// This function uses either a optimized assembly with loop or just a
/// byte-wise assembly function which is looped outside depending on
/// whether the `lpm-asm-loop` crate feature is set or not.
///
/// Notice that `T` might be also something like `[T, N]` so that in fact
/// entire arrays can be loaded using this function. Alternatively if the the
/// size of an array can not be known at compile time (i.e. a slice) there is
/// also the [`read_slice`] function, but it requires proper
/// initialization upfront.
///
/// If you need to read just a single byte you might use [`read_byte`].
///
/// ## Example
///
/// ```
/// use avr_progmem::read_value;
///
/// // This static must never be directly dereferenced/accessed!
/// // So a `let data: [u8;11] = P_ARRAY;` is Undefined Behavior!!!
/// // Also notice the `*` in front of the string, because we want to store the
/// // data, not just a reference!
/// /// Static bytes stored in progmem!
/// #[link_section = ".progmem"]
/// static P_ARRAY: [u8;11] = *b"Hello World";
///
/// // Load the bytes from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// let data: [u8;11] = unsafe { read_value(&P_ARRAY) };
/// assert_eq!(b"Hello World", &data);
/// ```
///
/// Also statically sized sub-arrays can be loaded using this function:
///
/// ```
/// use std::convert::TryInto;
/// use avr_progmem::read_value;
///
/// /// Static bytes stored in progmem!
/// #[link_section = ".progmem"]
/// static P_ARRAY: [u8;11] = *b"Hello World";
///
/// // Get a sub-array reference without dereferencing it
///
/// // Make sure that we convert from &[T] directly to &[T;M] without
/// // constructing an actual [T;M], because we MAY NOT LOAD THE DATA!
/// // Also notice, that this sub-slicing does ensure that the bound are
/// // correct.
/// let slice: &[u8] = &P_ARRAY[6..];
/// let array: &[u8;5] = slice.try_into().unwrap();
///
/// // Load the bytes from progmem
/// // Here, it is sound, because due to the link_section it is indeed in the
/// // program code memory.
/// let data: [u8;5] = unsafe { read_value(array) };
/// assert_eq!(b"World", &data);
/// ```
///
/// # Panics
///
/// This function panics, if the size of the value (i.e. `size_of::<T>()`)
/// is beyond 255 bytes.
/// However, this is currently just a implementation limitation, which may
/// be lifted in the future.
///
///
/// # Safety
///
/// This call is analog to `core::ptr::copy` thus it
/// has the same basic requirements such as the pointer must be valid for
/// dereferencing i.e. not dangling and the pointer must
/// be valid to read one entire value of type `T`,
/// i.e. `size_of::<T>()` bytes.
///
/// Additionally, `p_addr` must be a valid pointer into the program memory
/// domain.
///
/// While the alignment is not strictly required for AVR, the non-AVR fallback
/// might be done actually use `core::ptr::copy` and therefore the pointers
/// must be aligned.
///
/// Also notice, that the output slice must be correctly initialized, it would
/// be UB if not. If you don't want to initialize the data upfront, the
/// `read_value` might be a good alternative.
///
/// [`read_byte`]: fn.read_byte.html
/// [`read_slice`]: fn.read_slice.html
///
#[cfg_attr(feature = "dev", inline(never))]
pub unsafe fn read_value<T>(p_addr: *const T) -> T
		where T: Sized + Copy {

	// The use of an MaybeUninit allows us to correctly allocate the space
	// required to hold one `T`, whereas we correctly comunicate that it is
	// uninitialized to the compiler.
	//
	// The alternative of using a [0u8; size_of::<T>()] is actually much more
	// cumbersome as it also removes the type inference of `read_value_raw` and
	// still requires a `transmute` in the end.
	let mut buffer = MaybeUninit::<T>::uninit();

	let size = size_of::<T>();
	// TODO add a local loop to process bigger chunks in 256 Byte blocks
	assert!(size <= u8::MAX as usize);

	let res: *mut T = buffer.as_mut_ptr();

	// The soundness of this call is directly derived from the prerequisite as
	// defined by the Safety section of this function.
	//
	// Additionally, the use of the MaybeUninit there is also sound, because it
	// only written to and never read and not even a Rust reference is created
	// to it.
	read_value_raw(p_addr, res, 1);

	// After `read_value_raw` returned, it wrote an entire `T` into the `res`
	// pointer, which is baked by this `buffer`. Thus it is now properly
	// initialized, and this call is sound.
	buffer.assume_init()
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
