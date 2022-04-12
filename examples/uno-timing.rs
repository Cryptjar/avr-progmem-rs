// Define no_std only for AVR
#![cfg_attr(target_arch = "avr", no_std)]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(const_option)]
#![feature(llvm_asm)]
#![feature(extended_key_value_attributes)]
#![feature(test)]
#![feature(int_bits_const)]



use avr_progmem::progmem;
use avr_progmem::string::LoadedString;
#[cfg(target_arch = "avr")]
use panic_halt as _; // halting panic implementation for AVR
use void::ResultVoidExt;


mod bench;
mod time;


use bench::Bencher;



// Base line, a direct RAM string, no overhead
static LONG_REAL: &str = "A long test string literal, that is stored in progmem instead of DRAM.
Of course, it needs to be temporarily load into DRAM.
However, unlike a `ByteString`, it will be only read a char at a time,
thus a `ProgMemByteString` can never be too long.";

// A simple byte array in progmem
progmem! {
	static progmem LONG_BYTES: [u8;245] = *b"A long test string literal, that is stored in progmem instead of DRAM.
Of course, it needs to be temporarily load into DRAM.
However, unlike a `ByteString`, it will be only read a char at a time,
thus a `ProgMemByteString` can never be too long.";
}

// A LoadedString in progmem
progmem! {
	static progmem LONG_STRING: LoadedString<245> = LoadedString::new("A long test string literal, that is stored in progmem instead of DRAM.
Of course, it needs to be temporarily load into DRAM.
However, unlike a `ByteString`, it will be only read a char at a time,
thus a `ProgMemByteString` can never be too long.").unwrap();
}

// A proper PmString in progmem
progmem! {
static progmem string LONG_PM =
		"A long test string literal, that is stored in progmem instead of DRAM.
Of course, it needs to be temporarily load into DRAM.
However, unlike a `ByteString`, it will be only read a char at a time,
thus a `ProgMemByteString` can never be too long.";
}


// Include a fancy printer supporting Arduino Uno's USB-Serial output as well
// as stdout on non-AVR targets.
mod printer;
use printer::Printer;
use ufmt::uWrite;



//#[avr_device::entry]
#[no_mangle]
fn main() -> ! {
	let (mut printer, clock) = {
		#[cfg(target_arch = "avr")]
		{
			// Initialize the USB-Serial output on the Arduino Uno

			let dp = arduino_hal::Peripherals::take().unwrap();

			let pins = arduino_hal::pins!(dp);
			// max (kinda broken): 2000000
			// highest working:     500000
			// normal speed:        115200
			let serial = arduino_hal::default_serial!(dp, pins, 115200);

			let clock =
				time::TimerClock::<arduino_hal::DefaultClock>::new(dp.TC0, time::Resolution::_1_MS)
					.unwrap();

			// Enable interrupts globally
			unsafe { avr_device::interrupt::enable() };

			(Printer(serial), clock)
		}
		#[cfg(not(target_arch = "avr"))]
		{
			// Just use stdout for non-AVR targets, and Instant
			(Printer, time::StdClock(std::time::Instant::now()))
		}
	};

	let early = clock.micros();


	struct MyDummyWriter;
	impl uWrite for MyDummyWriter {
		type Error = void::Void;

		fn write_str(&mut self, s: &str) -> Result<(), void::Void> {
			core::hint::black_box(s);
			Ok(())
		}
	}

	#[allow(unused_variables)] // it is an alternative to the real printer
	#[allow(unused_mut)]
	let mut dummy = MyDummyWriter;

	// Print some introduction text
	printer.println("Hello from Arduino!");
	printer.println("");

	ufmt::uwriteln!(&mut printer, "Early at {} us!\r", early).void_unwrap();
	ufmt::uwriteln!(
		&mut printer,
		"Continue at {} ms = {} um!\r",
		clock.millis(),
		clock.micros()
	)
	.void_unwrap();
	printer.println("");
	printer.println("--------------------------");
	printer.println("");
	printer.println("Benchmarking ..");
	printer.println("");

	let start = clock.micros();

	let mut bencher = Bencher {
		test_writer: printer,
		//test_writer: dummy, // test without the printing overhead
		clock,
	};

	// Nothing
	// 1.8 us @ 2M release
	// 1.8 us @ 115k release
	let res_nothing = bencher.iter(|s| {
		core::hint::black_box(s);
	});

	// Direct RAM access
	// 3.5 us @ dummy release
	// 1230 us @ 2M release
	// 4920 us @ 500k release
	// 20910 us @ 115k release
	let res_direct_ram = bencher.iter(|s| {
		ufmt::uwriteln!(s, "{}\r", LONG_REAL).unwrap();
	});

	// Only load Byte Array
	// 248 us always with asm-loop
	// 495 us without asm-loop
	let res_just_load_array = bencher.iter(|s| {
		core::hint::black_box(s);
		let stuff: [u8; 245] = LONG_BYTES.load();
		core::hint::black_box(stuff);
	});

	// Byte Array
	// 126 us @ dummy release
	// 372 us @ dummy release without asm-loop
	// 1345 us @ 2M release
	// 5005 us @ 500k release
	// 20910 us @ 115k release
	let res_byte_array = bencher.iter(|s| {
		let stuff: [u8; 245] = LONG_BYTES.load();
		ufmt::uwriteln!(s, "{}\r", unsafe { core::str::from_utf8_unchecked(&stuff) }).unwrap();
	});

	// Old-style array loop
	// 1119 us @ dummy release
	// 889 us @ dummy release without asm-loop
	// 1809 us @ 2M release
	// 4880 us @ 500k release
	// 20740 us @ 115k release
	let res_byte_array_old_loop = bencher.iter(|s| {
		let mut idx = 0;
		loop {
			s.write_char(LONG_BYTES.load_at(idx) as char).unwrap();

			idx += 1;

			if idx >= 245 {
				break;
			}
		}
	});

	// New-style array iter
	// 1057 us @ dummy release
	// 858 us @ dummy release without asm-loop
	// 1777 us @ 2M release
	// 4880 us @ 500k release
	// 20740 us @ 115k release
	let res_byte_array_new_iter = bencher.iter(|s| {
		for b in LONG_BYTES.iter() {
			s.write_char(b as char).unwrap();
		}
	});

	// LoadedString in PM
	// 126 us @ dummy release
	// 372 us @ dummy release without asm-loop
	// 1348 us @ 2M release
	// 5008 us @ 500k release
	// 20910 us @ 115k release
	let res_loaded_string = bencher.iter(|s| {
		ufmt::uwriteln!(s, "{}\r", *LONG_STRING.load()).void_unwrap();
	});

	// PmString at once
	// 250 us @ dummy release
	// 496 us @ dummy release without asm-loop
	// 1472 us @ 2M release
	// 5133 us @ 500k release
	// 20997 us @ 115k release
	let res_pm_string_once = bencher.iter(|s| {
		ufmt::uwriteln!(s, "{}\r", *LONG_PM.load()).void_unwrap();
	});

	// PmString display
	// 1450 us @ dummy release
	// 1297 us @ dummy release without asm-loop
	// 2218 us @ 2M release
	// 4920 us @ 500k release
	// 20910 us @ 115k release
	#[cfg(feature = "ufmt")]
	let res_pm_display = bencher.iter(|s| {
		ufmt::uwriteln!(s, "{}\r", LONG_PM).void_unwrap();
	});


	let clock = bencher.clock;
	// Re-extract the printer serial wrapper
	let mut printer: Printer = bencher.test_writer;

	let end = clock.micros();

	// Print some final lines
	printer.println("");
	printer.println("--------------------------");
	printer.println("");

	ufmt::uwriteln!(
		&mut printer,
		"Done at {} ms = {} um!\r",
		clock.millis(),
		clock.micros()
	)
	.void_unwrap();


	// print results
	ufmt::uwrite!(&mut printer, "Nothing: {}\r\n", res_nothing).void_unwrap();
	ufmt::uwrite!(&mut printer, "Direct RAM access: {}\r\n", res_direct_ram).void_unwrap();
	ufmt::uwrite!(&mut printer, "Load array only: {}\r\n", res_just_load_array).void_unwrap();
	ufmt::uwrite!(&mut printer, "Byte array load once: {}\r\n", res_byte_array).void_unwrap();
	ufmt::uwrite!(
		&mut printer,
		"Byte array old loop: {}\r\n",
		res_byte_array_old_loop
	)
	.void_unwrap();
	ufmt::uwrite!(
		&mut printer,
		"Byte array new iter: {}\r\n",
		res_byte_array_new_iter
	)
	.void_unwrap();
	ufmt::uwrite!(
		&mut printer,
		"LoadedString in PM: {}\r\n",
		res_loaded_string
	)
	.void_unwrap();
	ufmt::uwrite!(&mut printer, "PmString at once: {}\r\n", res_pm_string_once).void_unwrap();
	#[cfg(feature = "ufmt")]
	ufmt::uwrite!(&mut printer, "PmString uDisplay: {}\r\n", res_pm_display).void_unwrap();


	printer.println("");
	printer.println("--------------------------");
	printer.println("");

	ufmt::uwriteln!(&mut printer, "Start: {} us!\r", start).void_unwrap();
	ufmt::uwriteln!(&mut printer, "End: {} us!\r", end).void_unwrap();
	ufmt::uwriteln!(&mut printer, "Duration: {} us!\r", end - start).void_unwrap();

	printer.println("");
	printer.println("DONE");

	/*
	// Test run to check whether the clock returns monotonic values, which
	// happens if the impl is racy, which it should be no more, i.e. this code
	// should never output anything, expect for for overflows, such as
	// after 71 min (I never tested that, tho)
	let mut older = clock.micros();
	let mut old = clock.micros();
	let mut fails = 0;
	loop {
		let new = clock.micros();

		if new < old {
			let newer = clock.micros();
			fails += 1;
			let ms = clock.millis();
			let m_p_f = (ms as u16) / fails;
			ufmt::uwriteln!(
				&mut printer,
				"[{}] Fail: {} -> {} => {} -> {}\r",
				ms,
				older,
				old,
				new,
				newer
			)
			.void_unwrap();

			ufmt::uwriteln!(
				&mut printer,
				"  {:?} -> {:?} => {:?} -> {:?}\r",
				older,
				old,
				new,
				newer
			)
			.void_unwrap();
			ufmt::uwriteln!(&mut printer, "  rate: {}\r", m_p_f).void_unwrap();
		}

		unsafe {
			llvm_asm!("nop" :::: "volatile");
		}

		older = old;
		old = new;
	}
	*/

	/*
	// Just keep printing the current time, good for checking that the clock
	// scale is not too far off.
	loop {
		ufmt::uwriteln!(
			&mut printer,
			"Done at {} ms = {} um!\r",
			clock.millis(),
			clock.micros()
		)
		.void_unwrap();
	}
	*/

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
