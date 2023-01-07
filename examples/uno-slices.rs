//
// This file provides a example on how to work with slice in progmem (sort of)
// on an Arduino Uno.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![cfg_attr(target_arch = "avr", no_main)]


use avr_progmem::progmem; // The macro
use avr_progmem::wrapper::ProgMem;
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR

// First, there are no slices in progmem. All data must be statically size,
// such as an array:
progmem! {
	// Just one array of size 3
	static progmem<const ARRAY_A_LEN: usize> ARRAY_A: [u8; ARRAY_A_LEN] = [1,2,3];

	// Another array of size 5
	static progmem<const ARRAY_B_LEN: usize> ARRAY_B: [u8; ARRAY_B_LEN] = [1,2,3,4,5];

	// Yet another array
	static progmem<const ARRAY_C_LEN: usize> ARRAY_C: [u8; ARRAY_C_LEN] = [42];
}

// Include a fancy printer supporting Arduino Uno's USB-Serial output as well
// as stdout on non-AVR targets.
mod printer;
use printer::Printer;

#[cfg_attr(target_arch = "avr", arduino_hal::entry)]
fn main() -> ! {
	// Setup the output
	let mut printer = setup();

	//
	// Working with slices.
	//
	// So, we actually only have statically sized arrays in progmem. However,
	// sometimes slices are nicer to work with, for instance if you want to put
	// them into a list or something, e.g. to iterate over them.
	//
	// There are basically to way to accomplish this, first just load those
	// arrays into RAM and coerce the standard Rust arrays into standard Rust
	// slices as usual. The drawback is the potential high RAM usage.
	//
	// In order to have the benefits of slices while the data is still in
	// progmem, we can also just coerce the ProgMems of arrays into ProgMems of
	// slices, just like that:

	let a_slice: ProgMem<[u8]> = ARRAY_A;
	let b_slice: ProgMem<[u8]> = ARRAY_B;
	let c_slice: ProgMem<[u8]> = ARRAY_C;

	// Now they have the same type, and we can put them into a list.
	let list_of_slices = [a_slice, b_slice, c_slice];

	// And for instance iterate through that list.
	for (i, slice) in list_of_slices.iter().enumerate() {
		// Here `slice` is a `ProgMem<[u8]>`, which has (among others) a `len` and a
		// `load_at` method.

		ufmt::uwriteln!(
			&mut printer,
			"Element #{}, size: {}, first element: {}\r",
			i,
			slice.len(),
			slice.load_at(0)
		)
		.unwrap();

		// We can also use a progmem-iterator that gives a ProgMem wrapper for
		// each element of that slice (without yet loading any of them).
		for elem_wrapper in slice.wrapper_iter() {
			// Fun fact, if that element happened to be another array, we would
			// also iterate its elements without loading it yet.
			// That allows to iterate multi-dimensional arrays and only ever
			// loading a single element into RAM.

			// Only load a single element at a time
			let e = elem_wrapper.load();
			ufmt::uwrite!(&mut printer, "{}, ", e).unwrap();
		}
		printer.println("");
	}

	// "end" the program
	finish(printer)
}

//
// Following are just some auxiliary functions to setup and finish up the Arduino
//


// Setup the serial UART as output at 9600 baud and print a "Hello from Arduino"
fn setup() -> Printer {
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

	// return the printer
	printer
}

// Print a "DONE" and exit (or go into an infinite loop).
fn finish(mut printer: Printer) -> ! {
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
