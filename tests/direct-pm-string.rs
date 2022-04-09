//
// This file tests using `PmString` directly, this is not recommended,
// use the `progmem` macro instead.
//

// Our generated warnings need a nightly feature
#![feature(extended_key_value_attributes)]
// Using unwrap in const context
#![feature(const_option)]

use avr_progmem::progmem;
use avr_progmem::string::LoadedString;
use avr_progmem::string::PmString;

/// A string directly in progmem
#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
static UNICODE_TEXT: PmString<137> = unsafe {
	PmString::new(
		"dai 大賢者 kenja, Völlerei lässt grüßen, le garçon de théâtre, Ελληνική Δημοκρατία, \
		 Слава Україні",
	)
	.unwrap()
};

/// A string directly in progmem
#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
static LONG_TEXT: PmString<240> = unsafe {
	PmString::new(
		"
A long test string literal, that is stored in progmem instead of DRAM.
Of course, it needs to be temporarily load into DRAM.
However, unlike a `LoadedString`, it will be only read a char at a time,
thus a `PmString` can never be too long.
",
	)
	.unwrap()
};

/// A single string that is over 2 KiB is size
#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
static MUCH_LONGER_TEXT: PmString<2177> =
	unsafe { PmString::new(include_str!("../examples/test_text.txt")).unwrap() };

#[test]
fn read_by_chars() {
	// Read string from progmem char-by-char
	for _c in LONG_TEXT.chars() {
		//printer.print(c);
	}

	/*
	printer.println("");

	// Or just use the `ufmt::uDisplay` impl
	#[cfg(feature = "ufmt")]
	ufmt::uwrite!(&mut printer, "{}", &UNICODE_TEXT).unwrap();

	printer.println("");

	// Thus loading 2 KiB with ease
	#[cfg(feature = "ufmt")]
	ufmt::uwrite!(&mut printer, "{}", MUCH_LONGER_TEXT).unwrap();

	// Print some final lines
	printer.println("");
	printer.println("--------------------------");
	printer.println("");
	printer.println("DONE");

	// It is very convenient to just exit on non-AVR platforms, otherwise users
	// might get the impression that the program hangs, whereas it already
	// succeeded.
	#[cfg(not(target_arch = "avr"))]
	std::process::exit(0);

	// Otherwise, that is on AVR, just go into an infinite loop, because on AVR
	// we just can't exit!
	loop {
		// Done, just do nothing
	}
	*/
}


#[test]
fn test_direct_loaded_string() {
	progmem! {
		// Stores a string as a byte array, i.e. `[u8;19]`, but makes it usable
		// as `&str` (via `Deref`)
		static progmem TEXT: LoadedString<19> = LoadedString::new(
			"dai 大賢者 kenja"
		).unwrap();
	}
	// usage:
	let text_buffer = TEXT.load(); // The temporary DRAM buffer for `TEXT`
	let text: &str = &text_buffer; // Just derefs to `str`
	assert_eq!(text, "dai 大賢者 kenja");
}
