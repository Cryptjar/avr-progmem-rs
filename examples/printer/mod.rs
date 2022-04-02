//!
//! Universal printer type.
//!
//! This small type will print given text to the Arduino Uno standard serial
//! output, and for non-AVR targets, it will print to stdout.
//!
//! Quite useful for examples.
//!


// Import the Arduino libraries, interestingly they don't cause problems perse
// on other architectures. Through, we will not use there.
use arduino_uno::prelude::*;


#[cfg(target_arch = "avr")]
pub struct Printer(pub arduino_uno::Serial<arduino_uno::hal::port::mode::Floating>);
#[cfg(not(target_arch = "avr"))]
pub struct Printer;

impl Printer {
	pub fn println(&mut self, s: &str) {
		#[cfg(target_arch = "avr")]
		ufmt::uwriteln!(&mut self.0, "{}\r", s).void_unwrap();

		#[cfg(not(target_arch = "avr"))]
		println!("{}", s);
	}

	pub fn print(&mut self, c: char) {
		#[cfg(target_arch = "avr")]
		ufmt::uwrite!(&mut self.0, "{}", c).void_unwrap();

		#[cfg(not(target_arch = "avr"))]
		print!("{}", c);
	}
}
