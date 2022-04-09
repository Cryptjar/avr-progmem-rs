//
// This file test the functionality of the patters which should generate
// warnings (tho whether the warnings is actually generated is not tested)
//
// Apparently our warnings require a nightly feature
#![feature(extended_key_value_attributes)]
// Using unwrap in const context
#![feature(const_option)]

use avr_progmem::progmem;
use avr_progmem::string::LoadedString;
use avr_progmem::ProgMem;

progmem! {
	// Should notify that we should use the `progmem string` rule instead
	static progmem HAND_STRING: LoadedString<34> =
		LoadedString::new("hand crafted progmem loaded string").unwrap();
}

#[test]
fn hand_string() {
	// ensure that the static has the correct type.
	let hand_str: &ProgMem<LoadedString<34>> = &HAND_STRING;

	// and it should be loadable
	let loaded: LoadedString<34> = hand_str.load();

	// and have the expected content
	assert_eq!("hand crafted progmem loaded string", &loaded as &str);
}
