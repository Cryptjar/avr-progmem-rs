//
// This file provides a example on how to use array statics on an
// Arduino Uno.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![cfg_attr(target_arch = "avr", no_main)]


use avr_progmem::progmem; // The macro
use avr_progmem::wrapper::ProgMem;
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR


progmem! {
	// Just one array of size 3
	static progmem<const ARRAY_A_LEN: usize> ARRAY_A: [u8; ARRAY_A_LEN] = [1,2,3];

	// Another array of size 5
	static progmem<const ARRAY_B_LEN: usize> ARRAY_B: [u8; ARRAY_B_LEN] = [1,2,3,4,5];
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
	// Coercing arrays into slices
	//

	// Assume that we want to put those to arrays into a vec (or just another
	// array), but the problem is that since the have difference size, they have
	// different types. One way to get rid of the static size, is to convert
	// them dynamically sized slices, like this:

	let a: ProgMem<[u8]> = ARRAY_A;
	let b: ProgMem<[u8]> = ARRAY_B;

	// Now they have the same type, and we can put them into a collection.

	let collection = [a, b];

	// Iterate through the collection
	for (i, arr) in collection.iter().enumerate() {
		ufmt::uwriteln!(&mut printer, "Element #{}, size: {}\r", i, arr.len()).unwrap();

		// Iterate through the slice
		for i in 0..arr.len() {
			// Only load a single byte at a time
			let e = arr.load_at(i);
			ufmt::uwrite!(&mut printer, "{}, ", e).unwrap();
		}
		printer.println("");
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
