//!
//! Universal printer type.
//!
//! This small type will print given text to the Arduino Uno standard serial
//! output, and for non-AVR targets, it will print to stdout.
//!
//! Quite useful for examples.
//!

// This a shared model, so there are function only used in specific cases
#![allow(dead_code)]

// Import the Arduino libraries, interestingly they don't cause problems perse
// on other architectures. Through, we will not use there.
cfg_if::cfg_if! {
	if #[cfg(target_arch = "avr")] {
		use arduino_hal::port::mode::AnyInput;
		use arduino_hal::port::mode::Input;
		use arduino_hal::port::mode::Output;
		use arduino_hal::port::Pin;
		use arduino_hal::prelude::*;
	}
}


#[cfg(target_arch = "avr")]
pub struct Printer(
	pub  arduino_hal::usart::Usart<
		avr_device::atmega328p::USART0,
		Pin<Input<AnyInput>, atmega_hal::port::PD0>,
		Pin<Output, atmega_hal::port::PD1>,
	>,
);
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

impl ufmt::uWrite for Printer {
	type Error = void::Void;

	fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
		#[cfg(target_arch = "avr")]
		ufmt::uwrite!(&mut self.0, "{}", s).void_unwrap();

		#[cfg(not(target_arch = "avr"))]
		println!("{}", s);

		Ok(())
	}

	fn write_char(&mut self, c: char) -> Result<(), Self::Error> {
		#[cfg(target_arch = "avr")]
		ufmt::uwrite!(&mut self.0, "{}", c).void_unwrap();

		#[cfg(not(target_arch = "avr"))]
		print!("{}", c);

		Ok(())
	}
}
