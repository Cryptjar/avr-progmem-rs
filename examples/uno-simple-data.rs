//
// This file provides a example on how to use this library on an Arduino Uno.
//
// It can be compiled with the current nightly Rust compiler (1.48.0-nightly):
// ```sh
// cargo +nightly build --examples -Z build-std=core --target ./avr-atmega328p.json
// ```
//
// It will compile an binary contains the 1 kB `TEXT` within this program code
// and not the data segment.
//
// This file is derived from the serial example of the Arduino Uno crate:
// https://github.com/Rahix/avr-hal/blob/master/boards/arduino-uno/examples/uno-serial.rs
//
// However, this example contains several `cfg`s just to make it work under
// both AVR and x86. So you can also run just on your host with:
// ```sh
// cargo +nightly run --example uno-serial
// ```
//
// Notice on an AVR target,
// this example opens a serial connection to the host computer.  On most POSIX
// operating systems (like GNU/Linux or OSX), you can interface with the
// program by running (assuming the device appears as ttyACM0):
//
// ```sh
// sudo screen /dev/ttyACM0 9600
// ```
//
// On other target, stdout will be used instead.
//


// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![cfg_attr(target_arch = "avr", no_main)]


use avr_progmem::progmem;
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR
use ufmt::derive::uDebug;


// We can store any `Copy + Sized` data in progmem
#[derive(Debug, uDebug, Copy, Clone)]
struct MyStuff {
	int: u32,
	opt: Option<bool>,
}


progmem! {
	/// The static data to be stored in program code section
	static progmem DATA: (u8, u16, u32) = (1, 2, 3);

	/// Custom data in progmem
	static progmem STUFF: MyStuff = MyStuff {int: 42, opt: Some(true)};
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


	// We can load the data via `load`
	let data: (u8, u16, u32) = DATA.load();
	ufmt::uwriteln!(&mut printer, "Data: {:?}\r", data).unwrap();

	// We can also load it where we need it
	ufmt::uwriteln!(&mut printer, "Stuff: {:?}\r", STUFF.load()).unwrap();


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
