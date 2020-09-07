#![feature(llvm_asm)]
#![feature(min_const_generics)]


use core::mem::size_of;
use core::mem::MaybeUninit;
use core::convert::TryInto;


#[repr(transparent)]
pub struct ProgMem<T>(T);

impl<T: Sized> ProgMem<T> {
	pub const unsafe fn new(t: T) -> Self {
		ProgMem(t)
	}
}

impl<T: Sized + Copy> ProgMem<T> {
	pub fn read(&self) -> T {
		let addr = &self.0;

		unsafe {
			read_value(addr)
		}
	}
	pub unsafe fn get_inner_ref(&self) -> &T {
		&self.0
	}
}

impl<T: Sized + Copy, const N: usize> ProgMem<[T;N]> {
	pub fn get(&self, idx: usize) -> T {
		let addr = &self.0[idx];

		unsafe {
			read_value(addr)
		}
	}

	pub fn get_range<const M: usize>(&self, idx: usize) -> [T;M] {
		assert!(M <= N);

		// Make sure that we convert from &[T] to &[T;M] without constructing
		// an actual [T;M], because we MAY NOT LOAD THE DATA YET!
		let array: &[T;M] = self.0[idx..(idx+M)].try_into().unwrap();

		unsafe {
			read_value(array)
		}
	}
}

#[doc(hidden)]
#[macro_export]
macro_rules! progmem_internal {
	{
		$(#[$attr:meta])*
		($($vis:tt)*) static $name:ident : $ty:ty = $value:expr ;
	} => {
		// ProgMem must be stored in the progmem or text section!
		// The link_section lets us define it.
		#[link_section = ".progmem"]
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

#[macro_export]
macro_rules! progmem {
    ($(#[$attr:meta])* static progmem $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        // use `()` to explicitly forward the information about private items
        progmem_internal!($(#[$attr])* () static ref $N : $T = $e;);
		// Recursive call to allow multiple items in macro invocation
		progmem!($($t)*);
    };
    ($(#[$attr:meta])* pub static progmem $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        progmem_internal!($(#[$attr])* (pub) static $N : $T = $e;);
		// Recursive call to allow multiple items in macro invocation
		progmem!($($t)*);
    };
    ($(#[$attr:meta])* pub ($($vis:tt)+) static progmem $N:ident : $T:ty = $e:expr; $($t:tt)*) => {
        progmem_internal!($(#[$attr])* (pub ($($vis)+)) static $N : $T = $e;);
		// Recursive call to allow multiple items in macro invocation
		progmem!($($t)*);
    };
    () => ()
}



pub unsafe fn read_progmem(addr: *const u8) -> u8 {
	let res: u8;
	llvm_asm!(
		"lpm"
		: "={r0}"(res)
		: "z"(addr)
	);
	res
}

pub unsafe fn read_progmem_slice_fallback(p: &[u8], out: &mut [u8]) {
	assert_eq!(p.len(), out.len());

	if p.len() == 0 {
		return
	}

	for i in 0..p.len() {
		out[i] = read_progmem(&p[i]);
	}
}

pub unsafe fn read_progmem_slice(p: &[u8], out: &mut [u8]) {
	assert_eq!(p.len(), out.len());
	assert!(p.len() <= u8::MAX as usize);

	let addr: *const u8 = &p[0];
	let mut res: *mut u8 = &mut out[0];
	let len: u8 = out.len() as u8;

	read_progmem_loop_raw(addr, res, len);
}

/// Only for internals
unsafe fn read_progmem_loop_raw<T>(addr: *const T, out: *mut T, len: u8)
		where T: Sized + Copy {

	// Loop head check, just return for no iterations
	if len == 0 {
		return
	}

	// A loop to read a slice of T from prog memory
	// The prog memory address (addr) is stored in the 16-bit address register Z
	// (since this is the default register for the `lpm` instruction).
	// The output data memory address (out) is stored in the 16-bit address
	// register X, because Z is already used and Y seams to be used other wise
	// or is callee-save.
	//
	// This loop appears in the assembly, because it allows to exploit
	// `lpm 0, Z+` instruction that simultaneously increments the pointer.
	llvm_asm!(
		"
			// load value from program memory at indirect Z into register 0
			// and increment Z by one
			lpm 0, Z+
			// write register 0 to data memory at indirect X
			// and increment X by one
			st X+, 0
			// Decrement the loop counter in register $0 (len).
			// If zero has been reached the equality flag is set.
			subi $0, 1
			// Check whether the end has not been reached and if so jump back.
			// The end is reached if $0 (len) == 0, i.e. equality flag is set.
			// Thus if equality flag is NOT set (brNE) jump back by 4
			// instruction, that are all instructions in this assembly.
			// Notice: 4 instructions = 8B
			brne -8
		"
		// No direct outputs
		:
		// Input the iteration count, input program memory address,
		// and output data memory address
		: "r"(len), "z"(addr), "x"(out)
		// The register 0 is clobbered
		: "0"
	);

}

#[cfg_attr(feature = "dev", inline(never))]
pub unsafe fn read_value<T>(p: &T) -> T
		where T: Sized + Copy {

	let mut buffer = MaybeUninit::<T>::uninit();
	let size = size_of::<T>();
	assert!(size <= u8::MAX as usize);

	let addr: *const T = p;
	let res: *mut T = buffer.as_mut_ptr();
	let len: u8 = size as u8;

	read_progmem_loop_raw(addr, res, len);

	buffer.assume_init()
}




#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
