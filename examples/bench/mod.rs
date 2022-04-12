//
// This file provides a few crude utilities to perform some benchmarking
// on Arduino Uno.
//


use ufmt;
use ufmt::uDisplay;
use ufmt::uWrite;
use ufmt::uwrite;
use ufmt::Formatter;

use super::time;


pub struct Fraction {
	nom: u32,
	den: u32,
}
impl uDisplay for Fraction {
	fn fmt<W: ?Sized>(&self, fmt: &mut Formatter<W>) -> Result<(), W::Error>
	where
		W: uWrite,
	{
		let mut precision = 0;
		let mut d = self.den;
		while d >= 20 {
			d /= 10;
			precision += 1;
		}

		let q = self.nom / self.den;
		let mut rest = self.nom % self.den;

		uwrite!(fmt, "{}", q)?;

		if precision > 0 {
			uwrite!(fmt, ".",)?;

			for _ in 0..(precision - 1) {
				let v = rest * 10;
				let q = v / self.den;
				rest = v % self.den;

				uwrite!(fmt, "{}", q)?;
			}

			// Last digit with rounding, saturate at 9
			let v = rest * 10 + self.den / 2 - 1;
			let q = (v / self.den).min(9);
			uwrite!(fmt, "{}", q)?;
		}

		Ok(())
	}
}

pub struct Stats {
	duration_um: u32,
	counts: u32,
}
impl uDisplay for Stats {
	fn fmt<W: ?Sized>(&self, fmt: &mut Formatter<W>) -> Result<(), W::Error>
	where
		W: uWrite,
	{
		uwrite!(
			fmt,
			"{} um/i ({} ms / {} it)",
			Fraction {
				nom: self.duration_um,
				den: self.counts
			},
			self.duration_um / 1_000,
			self.counts
		)
	}
}

pub struct Bencher<W> {
	pub test_writer: W,
	pub clock: time::TClock,
}

impl<W> Bencher<W>
where
	W: uWrite,
{
	pub fn iter(&mut self, mut f: impl FnMut(&mut W)) -> Stats {
		// Warm up
		//uwrite!(&mut self.test_writer, "Benchmarking, warmup");

		let mut counts = 1;
		let mut last_duration;
		while {
			let start = self.clock.millis();
			for _ in 0..counts {
				f(&mut self.test_writer)
			}
			let end = self.clock.millis();

			last_duration = end - start;

			//uwrite!(&mut self.test_writer, "Warmup: {} c @ {} ms\r\n", counts, last_duration);

			last_duration < 100
		} {
			counts *= 2;
		}

		let counts = ((counts * 1_000) + last_duration / 2) / last_duration;

		//uwrite!(&mut self.test_writer, "Benchmarking count: {}", counts);

		let start = self.clock.micros();
		for _ in 0..counts {
			f(&mut self.test_writer)
		}
		let end = self.clock.micros();

		let diff = end - start;

		//uwrite!(&mut self.test_writer, "Benchmarking duration: {} ms", diff / 1_000);

		Stats {
			duration_um: diff,
			counts,
		}
	}
}
