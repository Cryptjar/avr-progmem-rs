// This file as a copy of the `<&[T; N]>::try_from(&[T])` impl of the Rust core
// lib.
//
// This copy is needed, because as of Rust 1.51 `impl const` has not progressed
// far enough to make that impl work in const context, instead, it is copied
// here as a stand-alone `const fn`.
//
// Source:
// https://github.com/rust-lang/rust/blob/eb82facb1626166188d49599a3313fc95201f556/library/core/src/array/mod.rs#L203-L215


/// Our own `core::array::TryFromSliceError`
///
/// Used in [`array_ref_try_from_slice`].
// We need a local copy of this type, because we need to instantiate it, but it
// has a private field.
pub(crate) struct TryFromSliceError(());

/// Const version of `<&[T; N]>::try_from(&[T])`
///
/// Original Source:
/// https://github.com/rust-lang/rust/blob/eb82facb1626166188d49599a3313fc95201f556/library/core/src/array/mod.rs#L203-L215
pub(crate) const fn array_ref_try_from_slice<'a, T, const N: usize>(
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
