// This file as a partial copy of the str/validations.rs of the Rust core lib.
//
// A copy was needed, because the original `next_code_point` takes an iterator
// of `&u8`, which is not an option for as, because we only have `u8` by-value.
//
// Source:
// https://github.com/rust-lang/rust/blob/03b17b181af4945fa24e0df79676e89454546440/library/core/src/str/validations.rs


/// Mask of the value bits of a continuation byte.
const CONT_MASK: u8 = 0b0011_1111;

/// Returns the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
#[inline]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
	(byte & (0x7F >> width)) as u32
}

/// Returns the value of `ch` updated with continuation byte `byte`.
#[inline]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
	(ch << 6) | (byte & CONT_MASK) as u32
}


/// Reads the next code point out of a byte iterator (assuming a
/// UTF-8-like encoding).
///
/// # Safety
///
/// `bytes` must produce a valid UTF-8-like (UTF-8 or WTF-8) string
#[inline]
pub(super) unsafe fn next_code_point<I: Iterator<Item = u8>>(bytes: &mut I) -> Option<u32> {
	// Decode UTF-8
	let x = bytes.next()?;
	if x < 128 {
		return Some(x as u32);
	}

	// Multibyte case follows
	// Decode from a byte combination out of: [[[x y] z] w]
	// NOTE: Performance is sensitive to the exact formulation here
	let init = utf8_first_byte(x, 2);
	// SAFETY: `bytes` produces an UTF-8-like string,
	// so the iterator must produce a value here.
	let y = { bytes.next().unwrap() };
	let mut ch = utf8_acc_cont_byte(init, y);
	if x >= 0xE0 {
		// [[x y z] w] case
		// 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
		// SAFETY: `bytes` produces an UTF-8-like string,
		// so the iterator must produce a value here.
		let z = { bytes.next().unwrap() };
		let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
		ch = init << 12 | y_z;
		if x >= 0xF0 {
			// [x y z w] case
			// use only the lower 3 bits of `init`
			// SAFETY: `bytes` produces an UTF-8-like string,
			// so the iterator must produce a value here.
			let w = { bytes.next().unwrap() };
			ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
		}
	}

	Some(ch)
}
