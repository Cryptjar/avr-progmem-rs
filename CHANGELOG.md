Changelog
=========

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).




## [0.3.2] - 2023-01-22
[0.3.2]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.3.1...v0.3.2

Changes since `v0.3.1`.

### Added

- Add an `at` method on array wrappers (`ProgMem<[T; N]>`), which returns a wrapper to an array element, without loading it, by [@gergoerdi](https://github.com/gergoerdi) (https://github.com/Cryptjar/avr-progmem-rs/pull/9)
- Add a `wrapper_iter` method on array wrappers (`ProgMem<[T; N]>`), which returns a `PmWrapperIter`, an iterator of wrappers of each element, by [@gergoerdi](https://github.com/gergoerdi) (https://github.com/Cryptjar/avr-progmem-rs/pull/9)
- Add similar methods on slice wrappers (`ProgMem<[T]>`) as there are already on array wrappers and an `as_slice` method on array wrappers (https://github.com/Cryptjar/avr-progmem-rs/pull/11)
- Add the "unsize" crate feature that allows to directly coerce `ProgMem`s.
- Implement `Copy`, `Clone`, and `Debug` on `ProgMem` and `PmString`

### Changed

- Lift `Sized` constraint on `ProgMem`, which allows to wrap types such as slices (tho a slice can not be stored directly in progmem, instead a stored array can be coerce to a slice at runtime) (https://github.com/Cryptjar/avr-progmem-rs/pull/11)



## [0.3.1] - 2022-06-11
[0.3.1]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.3.0...v0.3.1

Changes since `v0.3.0`.

### Added

- Implement `IntoIterator` on array wrappers (i.e. `ProgMem<[T;N]>`) by [@mutantbob](https://github.com/mutantbob) (https://github.com/Cryptjar/avr-progmem-rs/pull/7)
- A `len` method on array wrappers (i.e. `ProgMem<[T;N]>`).


## [0.2.1] - 2022-06-11
[0.2.1]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.2.0...v0.2.1

Changes since `v0.2.0`.

### Added

- Implement `IntoIterator` on array wrappers (i.e. `ProgMem<[T;N]>`) by [@mutantbob](https://github.com/mutantbob) (https://github.com/Cryptjar/avr-progmem-rs/pull/7)
- A `len` method on array wrappers (i.e. `ProgMem<[T;N]>`).



## [0.3.0] - 2022-06-04
[0.3.0]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.2.0...v0.3.0

Changes since `v0.2.0`.

### Breaking Changes

- Bump MSRV to `nightly-2022-05-10`.
- Deny storing a reference in progmem (i.e. a direct `&T`) via the `progmem` macro, this should catch some common mistakes.
- Deny storing a `LoadedString` directly in progmem via the `progmem` macro, use the special `string` rule instead.

### Internal changes

- Migrate from `llvm_asm` to `asm`.
- Use the `addr_of` macro to never ever have a reference to progmem, just raw pointers.



## [0.2.0] - 2022-04-14
[0.2.0]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.1.5...v0.2.0

Changes since `v0.1.5`.

### Breaking Changes

- Remove the `read_slice` function, because it is based on references to program memory.
- Move the `read_value` and `read_byte` into the new [`raw`] sub-module.
- Move the `ProgMem` struct into the new [`wrapper`] sub-module.
- [`ProgMem::new`] now must be given a pointer to a value in progmem, instead of a value and being stored in progmem itself.

### Added

- Add the [`string`] sub-module, which adds utilities for storing Unicode strings in progmem, especial useful for non-ASCII strings (https://github.com/Cryptjar/avr-progmem-rs/issues/3)
- Add an rule to the `progmem` macro to easily store Unicode strings in progmem (see the [string section](https://docs.rs/avr-progmem/0.2.0/avr_progmem/macro.progmem.html#strings))
- Add an rule to the `progmem` macro to create "auto-sized" arrays (see the [auto-sizing section](https://docs.rs/avr-progmem/0.2.0/avr_progmem/macro.progmem.html#auto-sizing))
- Add an Arduino-like progmem macro (aka `F`) for inline progmem strings: [`progmem_str`] and [`progmem_display`] by [@mutantbob](https://github.com/mutantbob) (https://github.com/Cryptjar/avr-progmem-rs/pull/4)
- Introudce `nightly-2021-01-07` as MSRV for `v0.2.x` (which due to the upcoming changes from `llvm_asm` to `asm`, is also the highest supported version for `v0.2.x`).
- Emit a warning when just storing a reference in progmem instead of the actual data via the `progmem` macro.
- Add an [`PmIter::iter`] method to lazily iterate through an array (loading only one element at a time).

### Internal changes

- Change `ProgMem` from a direct value wrapper into a pointer wrapper, thus no more references into program memory are kept only raw pointers (the accessible `static`s containing the `ProgMem` struct are in RAM, the data is stored in hidden `static`s in progmem).
- Patch `ufmt` to fix u32 formatting (https://github.com/Cryptjar/avr-progmem-rs/commit/9d351038fc31d769206b29cd7228b35aa457b518)
- Add the `uno-timing` example as a poor-man's benchmarking suit.

[`raw`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/raw/index.html
[`wrapper`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/wrapper/index.html
[`ProgMem::new`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/wrapper/struct.ProgMem.html#method.new
[`string`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/string/index.html
[`PmIter::iter`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/wrapper/struct.ProgMem.html#method.iter
[`progmem_str`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/macro.progmem_str.html
[`progmem_display`]: https://docs.rs/avr-progmem/0.2.0/avr_progmem/macro.progmem_display.html



## [0.1.5] - 2022-04-02
[0.1.5]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.1.4...v0.1.5

Changes since `v0.1.4`.

### Fixed

- Fix the clobber list of `lpm-asm-loop` inline assembly (which is used by default).



## [0.1.4] - 2022-04-02
[0.1.4]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.1.3...v0.1.4

Changes since `v0.1.3`.

### Documentation

- Fix `docs.rs` build by excluding all the inline assemblies and disabling the now defunct `llvm_asm` Rust feature in documentation builds (we can't upgrade to the new `asm` yet, because the current Rust compiler doesn't work for AVR at all).



## [0.1.3] - 2022-04-02
[0.1.3]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.1.2...v0.1.3

Changes since `v0.1.2`.

### Deprecated

- Deprecate [`read_slice`] function, because it is based on passing around plain slices (aka a reference) to program memory, which is extremely hazardous, if not **UB**, and thus will not be supported in the future.

### Internal changes

- Pin the Rust toolchain to `nightly-2021-01-07`, because at the time of writing it is the latest Rust version that supports AVR (future version are broken, also see <https://github.com/rust-lang/compiler-builtins/issues/400>).
- Add Github CI.

[`read_slice`]: https://docs.rs/avr-progmem/0.1.4/avr_progmem/fn.read_slice.html



## [0.1.2] - 2021-01-04
[0.1.2]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.1.1...v0.1.2

Changes since `v0.1.1`.

### Documentation

- Fix `docs.rs` build by excluding the cargo config



## [0.1.1] - 2021-01-04
[0.1.1]: https://github.com/Cryptjar/avr-progmem-rs/compare/v0.1.0...v0.1.1

Changes since `v0.1.0`.

### Changes

- Use the `.progmem.data` section instead of just `.progmem` to keep compatibility with `avr-binutils >= 2.30` by [@Rahix](https://github.com/Rahix) (https://github.com/Cryptjar/avr-progmem-rs/pull/2)

### Internal changes

- Setup a cargo config to target the Arduino Uno by default (instead of the host), and allow to run the examples directly via `cargo run` by [@Rahix](https://github.com/Rahix) (https://github.com/Cryptjar/avr-progmem-rs/pull/2)



## 0.1.0 - 2020-09-08

Initial release.


