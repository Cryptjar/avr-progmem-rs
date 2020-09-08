//
// This file provides a example on how to use this library on an Arduino Uno.
//
// It can be compiled with the current nightly Rust compiler (1.48.0-nightly):
// ```sh
// cargo +nightly build --examples -Z build-std=core --target ./avr-atmega328p.json --all-features
// ```
//
// This file is derived from the serial example of the Arduino Uno crate:
// https://github.com/Rahix/avr-hal/blob/master/boards/arduino-uno/examples/uno-serial.rs
//

#![cfg_attr(target_arch = "avr", no_std)]
#![no_main]

#[cfg(target_arch = "avr")]
use panic_halt as _;

use arduino_uno::prelude::*;
use avr_progmem::progmem;

const TEXT_LEN: usize = 1243;
progmem! {
	static progmem TEXT: [u8;TEXT_LEN] = *include_bytes!("./test_text.txt");
}

// This example opens a serial connection to the host computer.  On most POSIX operating systems (like GNU/Linux or
// OSX), you can interface with the program by running (assuming the device appears as ttyACM0)
//
// $ sudo screen /dev/ttyACM0 57600

#[cfg(target_arch = "avr")]
struct Printer(arduino_uno::Serial<arduino_uno::hal::port::mode::Floating>);
#[cfg(not(target_arch = "avr"))]
struct Printer;

impl Printer {
	fn println(&mut self, s: &str) {
		#[cfg(target_arch = "avr")]
		ufmt::uwriteln!(&mut self.0, "{}\r", s);

		#[cfg(not(target_arch = "avr"))]
		println!("{}", s);
	}
	fn print(&mut self, c: char) {
		#[cfg(target_arch = "avr")]
		ufmt::uwrite!(&mut self.0, "{}", c);

		#[cfg(not(target_arch = "avr"))]
		print!("{}", c);
	}
}

#[no_mangle]
fn main() -> ! {
	let mut printer = {
		#[cfg(target_arch = "avr")]
		{
			let dp = arduino_uno::Peripherals::take().unwrap();

			let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

			let mut serial = arduino_uno::Serial::new(
				dp.USART0,
				pins.d0,
				pins.d1.into_output(&mut pins.ddr),
				9600,
			);
			Printer(serial)
		}
		#[cfg(not(target_arch = "avr"))]
		{
			Printer
		}
	};

	printer.println("Hello from Arduino!");
	printer.println("");
	printer.println("--------------------------");
	printer.println("");

	let mut idx = 0;
	loop {

		printer.print(TEXT.load_at(idx) as char);

		idx += 1;

		if idx >= TEXT_LEN {
			idx = 0;

			printer.println("");
			printer.println("--------------------------");
			printer.println("");
			printer.println("DONE");
			break
		}
	}

	#[cfg(not(target_arch = "avr"))]
	std::process::exit(0);

	loop {
	}
}
