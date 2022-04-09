//
// This file provides a example on how to use strings on an Arduino Uno.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![cfg_attr(target_arch = "avr", no_main)]
//
// To unwrap the Option in const context
#![feature(const_option)]


use avr_progmem::progmem; // The macro
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR


//
// Defining strings as statics in progmem
//

progmem! {
	/// A static string to be stored in program code section.
	/// Notice the usage of special `string` modifier, that accepts a `&str`
	/// as value and will wrap it in a `PmString` instead of a
	/// standard `ProgMem`.
	static progmem string SOME_TEXT = concat!(
		"A long test string literal, that is stored in progmem instead of RAM. ",
		"If used via `load`, it is load as `LoadedString` entirely into RAM, ",
		"so for those use-case an individual string shouldn't be too long."
	);

	/// You can also load a file as string via the std `include_str` macro.
	/// And because this is stored in progmem, it can be big,
	/// e.g. this one is over 2 KiB is size.
	static progmem string MUCH_LONGER_TEXT = include_str!("./test_text.txt");

	/// Of course, Unicode works as expected.
	static progmem string UNICODE_TEXT =
		"dai 大賢者 kenja, Völlerei lässt grüßen, le garçon de théâtre, Ελληνική Δημοκρατία, Слава Україні";
}

// Include a fancy printer supporting Arduino Uno's USB-Serial output as well
// as stdout on non-AVR targets.
mod printer;
use printer::Printer;

#[cfg_attr(target_arch = "avr", arduino_hal::entry)]
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


	//
	// Using string from progmem
	//

	// Option 1)
	// We can load the entire string at once and use references to the resulting
	// `LoadedString` everywhere where a `&str` is expected.
	// However, the string must of limited in size to not exceed RAM.
	{
		// Scope to limit the lifetime of `text_buffer`, since it is big.

		// The temporary DRAM buffer for the string of type `LoadedString`
		let text_buffer = SOME_TEXT.load();
		// Just derefs to `str`
		let _text: &str = &text_buffer;
		// This function only accepts `&str`, deref makes this possible:
		printer.println(&text_buffer);
	}


	// Option 2)
	// We can use the `char`-iterator to access the text iteratively,
	// this has the advantage of limiting the stack usage.
	for c in MUCH_LONGER_TEXT.chars() {
		// Here, `c` is just a `char`
		let _c: char = c;
		printer.print(c);
	}
	printer.println("");


	// Option 3)
	// We directly use the Display/uDisplay impl, which uses the char-iterator.
	#[cfg(feature = "ufmt")] // this however requires the `ufmt` a crate feature
	ufmt::uwriteln!(&mut printer, "{}\r", UNICODE_TEXT).unwrap();


	// Option 4)
	// We can use a in-place immediate string, similar to the popular C macro:
	use avr_progmem::progmem_str as F;
	ufmt::uwriteln!(&mut printer, "{}\r", F!("Some immediate string")).unwrap();



	// Print some final lines
	printer.println("");
	printer.println("--------------------------");
	printer.println("");
	printer.println("DONE");

	// It is convenient to just exit on non-AVR platforms.
	#[cfg(not(target_arch = "avr"))]
	std::process::exit(0);

	// Otherwise, that is on AVR, just go into an infinite loop.
	loop {
		// Done, just do nothing
	}
}
