//
// This file provides a example on how to use array statics on an
// Arduino Uno.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![cfg_attr(target_arch = "avr", no_main)]
//
// To unwrap the Option in const context
#![feature(const_option)]
//
#![feature(extended_key_value_attributes)]


use avr_progmem::progmem; // The macro
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR


progmem! {
	// Simple Auto-sizing
	// This is equivalent to:
	// `static progmem ARR: [u8; 3] = [1,2,3];`
	// But you don't have to count the elements yourself, and you get an nice
	// constant `ARR_LEN` to refer to the length of the array (e.g. to name the
	// type).
	static progmem<const ARR_LEN: usize> ARR: [u8; ARR_LEN] = [1,2,3];

	// The size in the array type is automatically derived from the value.
	// As note: don't confuse this with text, see `uno-string.rs` for those!
	// Notice this is just about 2 kiB, and the Arduino Uno only has 2 kiB of
	// SRAM, so if where not stored in progmem, would not work at all.
	static progmem<const LONG_LEN: usize> LONG: [u8; LONG_LEN] =
		*include_bytes!("./test_text.txt");
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
	// Using auto-sized arrays
	//

	// We get the const to refer to the size of the array
	ufmt::uwriteln!(&mut printer, "The array is {} bytes long\r", ARR_LEN).unwrap();
	printer.println("");

	// So we can easily name the type of the array, and load it as a whole:
	let arr: [u8; ARR_LEN] = ARR.load();
	ufmt::uwriteln!(&mut printer, "{:?}\r", arr).unwrap();
	printer.println("");

	// We can of course still use the array element accessors:
	assert_eq!(2, ARR.load_at(1));

	// Particularly useful is this auto-sizing with data from an external file.
	ufmt::uwriteln!(&mut printer, "The long data is {} bytes long\r", LONG_LEN).unwrap();

	// For longer data, it can easily become problematic to load it all at once,
	// instead you can use a byte-by-byte iterator to get the only load them
	// one at a time.
	for b in LONG.iter() {
		// assuming its all ASCII, so we actually print it,
		// but if you happen to have text, better use strings
		// see: `uno-string.rs`
		printer.print(b as char);
	}


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
