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
use avr_progmem::string::LoadedString; // Helper for storing strings
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR


progmem! {
	/// The static data to be stored in program code section
	/// Notice the usage of `LoadedString`, to store a string as `[u8;N]`,
	/// because you can't store a `str` and storing a `&str` wouldn't have
	/// much of an effect.
	static progmem SOME_TEXT: LoadedString<191> = LoadedString::new("
A long test string literal, that is stored in progmem instead of DRAM.
However, to use it, it needs to be temporarily load into DRAM,
so an individual `LoadedString` shouldn't be too long.
	").unwrap();

	/// More data to be stored in program code section
	static progmem MORE_TEXT: LoadedString<102> = LoadedString::new("
However, you easily store your strings individual, limiting the amount of
temporary DRAM necessary.
	").unwrap();

	/// Unicode works of course as expected
	///
	static progmem UNICODE_TEXT: LoadedString<137> = LoadedString::new(
		"dai 大賢者 kenja, Völlerei lässt grüßen, le garçon de théâtre, Ελληνική Δημοκρατία, Слава Україні"
	).unwrap();
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

	// Scope to limit the lifetime of `text_buffer`
	{
		// The temporary DRAM buffer for the string
		let text_buffer = SOME_TEXT.load();
		let text: &str = &text_buffer; // Just derefs to `str`
		printer.println(text);
	}

	// Or just using temporaries
	printer.println(&MORE_TEXT.load());
	printer.println(&UNICODE_TEXT.load());

	// Even more convenient: use a one-off in-place progmem static via `progmem_str`
	printer.println(avr_progmem::progmem_str!("Just a lone literal progmem str"));
	use avr_progmem::progmem_str as F;
	printer.println(F!("And another one"));

	// Using the ufmt impl
	#[cfg(feature = "ufmt")]
	ufmt::uwriteln!(&mut printer, "{}\r", UNICODE_TEXT.load()).unwrap();

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
