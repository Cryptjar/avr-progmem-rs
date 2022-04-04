//
// This file provides a example on how to use strings on an Arduino Uno.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![no_main]
//
// To unwrap the Option in const context
#![feature(const_option)]


use avr_progmem::string::PmByteString; // A progmem wrapper for strings
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR


/// A string directly in progmem
#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
static UNICODE_TEXT: PmByteString<137> = unsafe {
	PmByteString::new(
		"dai 大賢者 kenja, Völlerei lässt grüßen, le garçon de théâtre, Ελληνική Δημοκρατία, \
		 Слава Україні",
	)
	.unwrap()
};

/// A string directly in progmem
#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
static LONG_TEXT: PmByteString<242> = unsafe {
	PmByteString::new(
		"
A long test string literal, that is stored in progmem instead of DRAM.
Of course, it needs to be temporarily load into DRAM.
However, unlike a `ByteString`, it will be only read a char at a time,
thus a `PmByteString` can never be too long.
",
	)
	.unwrap()
};

/// A single string that is over 2 KiB is size
#[cfg_attr(target_arch = "avr", link_section = ".progmem.data")]
static MUCH_LONGER_TEXT: PmByteString<2177> =
	unsafe { PmByteString::new(include_str!("./test_text.txt")).unwrap() };


// Include a fancy printer supporting Arduino Uno's USB-Serial output as well
// as stdout on non-AVR targets.
mod printer;
use printer::Printer;

#[no_mangle]
fn main() -> ! {
	let mut printer = {
		#[cfg(target_arch = "avr")]
		{
			// Initialize the USB-Serial output on the Arduino Uno

			let dp = arduino_hal::Peripherals::take().unwrap();
			let pins = arduino_hal::pins!(dp);
			let serial = arduino_hal::default_serial!(dp, pins, 9600);

			Printer(serial)
		}
		#[cfg(not(target_arch = "avr"))]
		{
			// Just use stdout for non-AVR targets
			Printer
		}
	};

	// Print some introduction text
	printer.println("Hello from Arduino!");
	printer.println("");
	printer.println("--------------------------");
	printer.println("");

	// Read string from progmem char-by-char
	for c in LONG_TEXT.chars() {
		printer.print(c);
	}

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
}
