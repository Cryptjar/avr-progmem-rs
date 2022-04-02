//
// This file provides a example on how to use strings on an Arduino Uno.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![no_main]
//
// To unwrap the Option in const context
#![feature(const_option)]


use avr_progmem::progmem; // The macro
use avr_progmem::string::ByteString; // Helper for storing strings
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR


progmem! {
	/// The static data to be stored in program code section
	/// Notice the usage of `ByteString`, to store a string as `[u8;N]`,
	/// because you can't store a `str` and storing a `&str` wouldn't have
	/// much of an effect.
	static progmem SOME_TEXT: ByteString<189> = ByteString::new("
A long test string literal, that is stored in progmem instead of DRAM.
However, to use it, it needs to be temporarily load into DRAM,
so an individual `ByteString` shouldn't be too long.
	").unwrap();

	/// More data to be stored in program code section
	static progmem MORE_TEXT: ByteString<102> = ByteString::new("
However, you easily store your strings individual, limiting the amount of
temporary DRAM necessary.
	").unwrap();

	/// Unicode works of course as expected
	///
	static progmem UNICODE_TEXT: ByteString<137> = ByteString::new(
		"dai 大賢者 kenja, Völlerei lässt grüßen, le garçon de théâtre, Ελληνική Δημοκρατία, Слава Україні"
	).unwrap();
}

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

			let dp = arduino_uno::Peripherals::take().unwrap();

			let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

			let serial = arduino_uno::Serial::new(
				dp.USART0,
				pins.d0,
				pins.d1.into_output(&mut pins.ddr),
				9600,
			);
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
	printer.println(avr_progmem::progmem_str!("And another one"));

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
