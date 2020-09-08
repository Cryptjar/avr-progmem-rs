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
#![no_main]

// Import a halting panic implementation for AVR
#[cfg(target_arch = "avr")]
use panic_halt as _;

// Our library to be actually test here!
use avr_progmem::progmem;


// The length of the below data block.
const TEXT_LEN: usize = 2177;
progmem! {
	/// The static data to be stored in program code section
	/// Notice this is just about 2 kiB, and the Arduino Uno only has 2 kiB of
	/// SRAM, so if where not stored in progmem, would not work at all.
	static progmem TEXT: [u8;TEXT_LEN] = *include_bytes!("./test_text.txt");
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

	// Loop through the entire `TEXT` and print it char-by-char
	let mut idx = 0;
	loop {

		printer.print(TEXT.load_at(idx) as char);

		idx += 1;

		if idx >= TEXT_LEN {
			break
		}
	}

	// Print some final lines
	printer.println("");
	printer.println("--------------------------");
	printer.println("");
	printer.println("DONE");

	// It is very convinient to just exit on non-AVR platforms, otherwise users
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
