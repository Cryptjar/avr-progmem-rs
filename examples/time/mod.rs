// This file is an adaption on Rahix blog post on Arduino millis function
// https://blog.rahix.de/005-avr-hal-millis/
//
// Original source:
// https://github.com/Rahix/avr-hal/blob/e897783816437a677aa577ddfdaa34e9a1e86d96/examples/arduino-uno/src/bin/uno-millis.rs#L15-L71

use core::cell::Cell;
use core::marker::PhantomData;

use arduino_hal::clock::Clock;
use arduino_hal::pac::TC0;
use avr_device::interrupt::Mutex;


pub const MAX_INTERVAL: u32 = 16;

/// Counts the number of "millis" interrupts.
///
/// This simply counts unit-less interrupt events. However, this counter
/// is the basis for `ClockTimer::millis`, hence the name.
/// The interpretation of this static depends on the `Resolution` that is used
/// with the current `ClockTimer`.
///
// There is only a single static for any TimerClock instance, because,
// we can only have the one interrupt handler.
// The fact that `TimerClock` takes `TC0` by value, means should ever be only a
// single instance of `TimerClock`, anyway.
static MILLIS_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));


// Compatibility type
cfg_if::cfg_if! {
	if #[cfg(target_arch = "avr")] {
		pub type TClock = TimerClock::<arduino_hal::DefaultClock>;
	} else {
		// Just use standard
		pub struct StdClock(pub std::time::Instant);
		impl StdClock {
			pub fn millis(&self) -> u32 {
				self.0.elapsed().as_millis() as u32
			}
			pub fn micros(&self) -> u32 {
				self.0.elapsed().as_micros() as u32
			}
		}

		pub type TClock = StdClock;
	}
}


/// Represents one of the few valid prescaler values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Prescaler {
	P1,
	P8,
	P64,
	P256,
	P1024,
}
impl Prescaler {
	/// Returns the next best prescaler for the given prescaler exponent.
	///
	/// The next best prescaler means here, the next bigger value, unless,
	/// the value goes beyond 10, which is the highest supported prescaler
	/// exponent.
	const fn from_exp(exp: u32) -> Option<Self> {
		let prescaler = match exp {
			0 => Self::P1,
			1..=3 => Self::P8,
			4..=6 => Self::P64,
			7..=8 => Self::P256,
			9..=10 => Self::P1024,
			_ => return None,
		};
		Some(prescaler)
	}

	/// Gives the exponent of this prescaler.
	const fn to_exp(self) -> u8 {
		match self {
			Self::P1 => 0,
			Self::P8 => 3,
			Self::P64 => 6,
			Self::P256 => 8,
			Self::P1024 => 10,
		}
	}

	/// Returns the numeric value of this prescaler.
	const fn to_val(self) -> u16 {
		1 << self.to_exp()
	}
}


/// Represents the smallest resolvable interval of `millis` function.
///
/// Also effects the smallest resolvable interval of the `micros` function.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Resolution {
	exp: u8,
}

impl Resolution {
	pub const _16_MS: Self = Self::from_ms(16).unwrap();
	pub const _1_MS: Self = Self::from_ms(1).unwrap();
	pub const _2_MS: Self = Self::from_ms(2).unwrap();
	pub const _4_MS: Self = Self::from_ms(4).unwrap();
	pub const _8_MS: Self = Self::from_ms(8).unwrap();

	pub const fn from_ms(ms: u32) -> Option<Self> {
		// Check whether `ms` is a power of two
		if ms.count_ones() != 1 {
			return None;
		}

		// Calculate the log2 of `ms`, exactly
		let exp = u32::BITS - ms.leading_zeros() - 1;
		let value = (1_u16 << exp) as u32;

		if value > MAX_INTERVAL {
			return None;
		}

		if value == ms {
			Some(Self {
				exp: exp as u8,
			})
		} else {
			None
		}
	}

	/// The minimal resolvable interval of `millis` in milliseconds
	pub const fn as_ms(self) -> u32 {
		// Notice: u32 shifts seem not to be supported as of Rust 1.51,
		// yields: "undefined reference to `__ashlsi3'" link errors
		(1_u16 << self.exp) as u32
	}

	/// Calculates the optimal prescaler and counter value for the given clock
	/// frequency in Hz.
	///
	/// Returns `None`, if there there is no valid configuration for this
	/// resolution at the given frequency.
	const fn params_for_frq(self, freq_hz: u32) -> Option<(Prescaler, u8)> {
		// The maximum valid counter value
		const MAX: u32 = u8::MAX as u32; // 255

		let cycles_per_second = freq_hz;
		// Combine for better precision:
		//     let cycles_per_ms = (cycles_per_second + 499) / 1_000;
		//     let cycles_per_interrupt = cycles_per_ms * self.as_ms();
		let cycles_per_interrupt = (cycles_per_second * self.as_ms() + 499) / 1_000; // rounded

		// Calculate a perfect prescaler.
		// It is also the minimum prescaler, because it yield the highest
		// yet valid counter value.
		// So, if need to tweak the prescaler, we need to make it bigger.
		// Thus, we already calculate this rounded up
		let perfect_prescaler: u32 = (cycles_per_interrupt + MAX - 1) / MAX;

		// Calculate the log2 of `perfect_prescaler`, rounded up
		// To get the correct result for powers of two, we will subtract 1
		// if we have a power of two. Power of two have exactly one `1` in
		// binary.
		let sub_for_pot = if perfect_prescaler.count_ones() == 1 {
			1
		} else {
			0
		};
		let perfect_prescaler_exp = u32::BITS - perfect_prescaler.leading_zeros() - sub_for_pot;

		// Get the next best (i.e. exact or bigger) available prescaler, if any
		let prescaler = match Prescaler::from_exp(perfect_prescaler_exp) {
			Some(p) => p,
			None => return None,
		};

		// The scalar value of the available perscaler
		let prescaler_val: u16 = prescaler.to_val();

		// Calculate the number of prescaled cycles per interrupt
		let cnt = (cycles_per_interrupt + (prescaler_val / 2) as u32) / (prescaler_val as u32); // rounded

		// If we calculated correctly, it holds: `cnt <= MAX`
		let cnt: u8 = cnt as u8; //cnt.try_into().unwrap();

		Some((prescaler, cnt))
	}
}

/// A Timer-based Clock, tells an approximated wall time.
///
#[derive(Debug)]
pub struct TimerClock<ClockFreq> {
	/// The timer register, gives this instance unique control over it.
	tc0: TC0,
	/// Represents how fine the resolution of the millis of this clock should be.
	///
	/// A high resolution, incurs more timer interrupts.
	res: Resolution,
	/// Caches the number of micro seconds per counter value
	um_p_cnt: u32,
	/// Caches the maximum counter value.
	max_cnt: u8,
	/// Dummy for the generic
	_clock_frq: PhantomData<ClockFreq>,
}

impl<ClockFreq: Clock> TimerClock<ClockFreq> {
	/// Initialize the clock.
	///
	/// The clock start running immediately after this call.
	/// However, in order to work properly interrupts must be enabled too, e.g.:
	///
	/// ```rust
	/// // Enable interrupts globally
	/// unsafe { avr_device::interrupt::enable() };
	/// ```
	pub fn new(tc0: TC0, res: Resolution) -> Result<Self, TC0> {
		let (prescaler, timer_cnt) = {
			match res.params_for_frq(ClockFreq::FREQ) {
				Some(p) => p,
				None => return Err(tc0),
			}
		};

		// Reset the global millisecond counter
		avr_device::interrupt::free(|cs| {
			MILLIS_COUNTER.borrow(cs).set(0);
		});

		// Configure the timer for the above interval (in CTC mode)
		// and enable its interrupt.
		tc0.tccr0a.write(|w| w.wgm0().ctc());
		tc0.ocr0a.write(|w| unsafe { w.bits(timer_cnt) });
		tc0.tccr0b.write(|w| {
			match prescaler {
				Prescaler::P1 => w.cs0().direct(),
				Prescaler::P8 => w.cs0().prescale_8(),
				Prescaler::P64 => w.cs0().prescale_64(),
				Prescaler::P256 => w.cs0().prescale_256(),
				Prescaler::P1024 => w.cs0().prescale_1024(),
			}
		});
		tc0.timsk0.write(|w| w.ocie0a().set_bit());

		// Calculate how many microseconds a single counter value represents
		let um_p_cnt = 1_000_000 * u32::from(prescaler.to_val()) / ClockFreq::FREQ;

		Ok(Self {
			_clock_frq: PhantomData,
			um_p_cnt,
			max_cnt: timer_cnt,
			tc0,
			res,
		})
	}

	/// Stops the clock and returns back the used timer
	pub fn dismantle(self) -> arduino_hal::pac::TC0 {
		self.tc0.tccr0b.write(|w| w.cs0().no_clock());
		self.tc0.timsk0.write(|w| w.ocie0a().clear_bit());

		self.tc0
	}

	/// Returns the number of milliseconds since this clock was started
	pub fn millis(&self) -> u32 {
		// Get the current number of "millis" interrupts
		let m = avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get());

		// Calculate the proper millisecond value
		m * self.res.as_ms()
	}

	/// Returns the number of microseconds since this clock was started
	pub fn micros(&self) -> u32 {
		/*
		// Source: https://garretlab.web.fc2.com/en/arduino/inside/hardware/arduino/avr/cores/arduino/wiring.c/micros.html

		uint8_t oldSREG = SREG, t;

		cli();
		m = timer0_overflow_count;
		t = TCNT0;

		if ((TIFR0 & _BV(TOV0)) && (t < 255))
			m++;

		SREG = oldSREG;

		return ((m<<8) + t) * (64 / clockCyclesPerMicrosecond());
		*/

		let (mut m, t, tifr) = avr_device::interrupt::free(|cs| {
			let m: u32 = MILLIS_COUNTER.borrow(cs).get().into();

			let (t, tifr) = {
				let t: u8 = self.tc0.tcnt0.read().bits();
				let tifr: bool = self.tc0.tifr0.read().ocf0a().bit();

				(t, tifr)
			};

			(m, t, tifr)
		});

		let counter_micros = u32::from(t) * self.um_p_cnt;

		// Check whether a interrupt was pending when we read the counter value,
		// which typically means it wrapped around, without the millis getting
		// incremented, so we do it here manually:
		if tifr && t < self.max_cnt {
			m += 1;
		}

		let millis = m * self.res.as_ms();

		millis * 1000 + counter_micros
	}
}

// The timer interrupt service routine
#[cfg(target_arch = "avr")]
#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
	// We just increment the "millis" interrupt counter, because an interrupt
	// happened.
	avr_device::interrupt::free(|cs| {
		let counter_cell = MILLIS_COUNTER.borrow(cs);
		let counter = counter_cell.get();
		counter_cell.set(counter + 1);
	})
}


#[cfg(test)]
mod test {

	// Possible Values @ 16 MHz:
	//
	// ╔═══════════╦══════════════╦═══════════════════╗
	// ║ PRESCALER ║ TIMER_COUNTS ║ Overflow Interval ║
	// ╠═══════════╬══════════════╬═══════════════════╣
	// ║        64 ║          250 ║              1 ms ║
	// ║       256 ║          125 ║              2 ms ║
	// ║       256 ║          250 ║              4 ms ║
	// ║      1024 ║          125 ║              8 ms ║
	// ║      1024 ║          250 ║             16 ms ║
	// ╚═══════════╩══════════════╩═══════════════════╝
	#[test]
	fn test_16mhz() {
		let frq = 16_000_000;

		let (pre, cnt) = Resolution::_1_MS.params_for_frq(frq);
		assert_eq!((64, 250), (pre.to_val(), cnt));

		let (pre, cnt) = Resolution::_2_MS.params_for_frq(frq);
		assert_eq!((256, 125), (pre.to_val(), cnt));

		let (pre, cnt) = Resolution::_4_MS.params_for_frq(frq);
		assert_eq!((256, 250), (pre.to_val(), cnt));

		let (pre, cnt) = Resolution::_8_MS.params_for_frq(frq);
		assert_eq!((1024, 125), (pre.to_val(), cnt));

		let (pre, cnt) = Resolution::_16_MS.params_for_frq(frq);
		assert_eq!((1024, 250), (pre.to_val(), cnt));
	}
}
