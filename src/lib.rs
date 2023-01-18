// We don't need anything from std, and on AVR there is no std anyway.
#![no_std]
//
// We need inline assembly for the `lpm` instruction.
// As of now (mid 2022), inline assembly for AVR is still unstable.
#![feature(asm_experimental_arch)]
//
// We to access the length of a slice pointer (for unsized `ProgMem`s)
#![feature(slice_ptr_len)]
//
// Allow to implement `CoerceUnsized` on `ProgMem`
#![feature(coerce_unsized)]
//
// Needed for implementing `CoerceUnsized` on `ProgMem`
#![feature(unsize)]
//
// Allows to document required crate features on items
#![cfg_attr(doc, feature(doc_auto_cfg))]
//
// Allow `unsafe` in `unsafe fn`s, and make `unsafe` blocks everywhere a
// necessity.
#![forbid(unsafe_op_in_unsafe_fn)]

//!
//! Progmem utilities for the AVR architectures.
//!
//! This crate provides unsafe utilities for working with data stored in
//! the program memory of an AVR micro-controller. Additionally, it defines a
//! 'best-effort' safe wrapper struct [`ProgMem`](crate::wrapper::ProgMem)
//! to simplify working with it,
//! as well as a [`PmString`](crate::string::PmString) wrapper for string
//! handling.
//!
//! This crate is implemented only in Rust and some short assembly, it does NOT
//! depend on the [`avr-libc`] or any other C-library. However, due to the use
//! of inline assembly, this crate may only be compiled using a **nightly Rust**
//! compiler (as of mid 2022, inline assembly for AVR is still 'experimental').
//!
//! ## MSRV
//!
//! This crate works with a Rust `nightly-2022-05-10` compiler.
//! All versions `0.3.x` will adhere to work with `nightly-2022-05-10`.
//! Other Rust compiler version (particularly newer ones) might also work,
//! but due to the use of experimental compiler features it is possible that
//! some future Rust compiler version will fail to work.
//!
//! Future versions such as `0.4.x` might required a newer Rust compiler
//! version.
//!
//!
//! # AVR Memory
//!
//! This crate is specifically for [AVR-base micro-controllers][avr] such as
//! the Arduino Uno (and some other Arduino boards, but not all), which have a
//! modified Harvard architecture which implies the strict separation of program
//! code and data while having special instructions to read and write to
//! program memory.
//!
//! While, of course, all ordinary data is stored in the data domain where it is
//! perfectly usable, the harsh constraints of most AVR processors make it very
//! appealing to use the program memory (also referred to as _progmem_) for
//! storing constant values. However, due to the Harvard design, those values
//! are not usable with normal instructions (i.e. those emitted from normal
//! Rust code). Instead, special instructions are required to load data from
//! the program code domain, such as the `lpm` (load \[from\] program memory)
//! instruction. And because there is no way to emit it from Rust code, this
//! crate uses inline assembly to emit that instruction.
//!
//! However, since a pointer into program code cannot be differentiated from a
//! normal data pointer, it is entirely up to the programmer to ensure that
//! these different 'pointer-types' are not accidentally mixed. In other words,
//! this is `unsafe` in the context of Rust.
//!
//!
//! # Loading Data from Program Memory
//!
//! The first part of this crate simply provides a few functions (e.g.
//! [`read_byte`](crate::raw::read_byte)) to load constant data (i.e. a Rust `static` that is
//! immutable) from the program memory into the data domain, so that
//! sub-sequentially it is normal usable data, i.e. as owned data on the stack.
//!
//! Because, as aforementioned, a simple `*const u8` in Rust does not specify
//! whether is lives in the program code domain or the data domain, all
//! functions which simply load a given pointer from the program memory are
//! inherently `unsafe`.
//!
//! Notice that using references (e.g. `&u8`) to the program code domain should
//! generally be avoided because references in Rust should be dereferencable,
//! which the program code domain is not.
//!
//! Additionally references can be easily dereferenced by safe code,
//! which would be **undefined behavior** if that reference points into the
//! program memory.
//! Therefore, a Rust reference to a `static` that is stored in program memory
//! must be considered hazardous (if not **UB**),
//! and it is recommended to only use raw pointers to those `static`s,
//! e.g. obtained via the [`addr_of!`](core::ptr::addr_of) macro,
//! which directly creates raw pointers without needing a reference.
//!
//! ## Example
//!
//! ```
//! use avr_progmem::raw::read_byte;
//! use core::ptr::addr_of;
//!
//! // This `static` must never be directly dereferenced/accessed!
//! // So a `let data: u8 = P_BYTE;` ⚠️ is **undefined behavior**!!!
//! /// Static byte stored in progmem!
//! #[link_section = ".progmem.data"]
//! static P_BYTE: u8 = b'A';
//!
//! // Load the byte from progmem
//! // Here, it is sound, because due to the link_section it is indeed in the
//! // program code memory.
//! let data: u8 = unsafe { read_byte(addr_of!(P_BYTE)) };
//! assert_eq!(b'A', data);
//! ```
//!
//!
//! # The best-effort Wrapper
//!
//! Since working with progmem data is inherently unsafe and rather
//! difficult to do correctly, this crate introduces the best-effort 'safe'
//! wrapper [`ProgMem`](crate::wrapper::ProgMem),
//! that is supposed to only wrap data in progmem, thus
//! offering only functions to load its content using the progmem loading
//! function introduced above.
//! Using these functions is sound, if that the wrapper data is really stored
//! in the program memory. Therefore, to enforce this invariant,
//! the constructor of `ProgMem` is `unsafe`.
//!
//! Additionally, since proper Rust references (unlike pointers) come with a lot
//! special requirements, it should be considered hazardous to have a reference
//! to data stored in program memory.
//! Instead, only raw pointers to this kind of data should be kept,
//! created e.g. via the [`addr_of!`](core::ptr::addr_of) macro.
//! Consequently, the `ProgMem` just wrap a pointer to data in progmem,
//! which in turn must be stored in a `static` marked with
//! `#[link_section = ".progmem.data"]`.
//! However, since, safe Rust can always create a "normal" Rust reference to any
//! (accessible) `static`, it must be considered hazardous if not just unsound,
//! to expose such a `static` to safe Rust code.
//!
//! To also make this easier (and less hazardous), this crate provides the
//! [`progmem!`] macro, which will create a hidden `static` in program memory
//! initialized with the data you give it,
//! wrap it's pointer in the `ProgMem` struct,
//! and put that wrapper into yet another (normal RAM) static, so you can
//! access it.
//! This will ensure that the `static` that is stored in program memory can not
//! be referenced by safe Rust code (because it is not accessible),
//! while the accessible `ProgMem` wrapper allows access to the underling data
//! by loading it correctly from program memory.
//!
//! ## Example
//!
//! ```
//! use avr_progmem::progmem;
//!
//! // It will be wrapped in the ProgMem struct and expand to:
//! // ```
//! // static P_BYTE: ProgMem<u8> = {
//! //     #[link_section = ".progmem.data"]
//! //     static INNER_HIDDEN: u8 = 42;
//! //     unsafe { ProgMem::new(addr_of!(INNER_HIDDEN)) }
//! // };
//! // ```
//! // Thus it is impossible for safe Rust to directly access the progmem data!
//! progmem! {
//!     /// Static byte stored in progmem!
//!     static progmem P_BYTE: u8 = 42;
//! }
//!
//! // Load the byte from progmem
//! // This is sound, because the `ProgMem` always uses the special operation to
//! // load the data from program memory.
//! let data: u8 = P_BYTE.load();
//! assert_eq!(42, data);
//! ```
//!
//!
//! # Strings
//!
//! Using strings such as `&str` with [`ProgMem`](crate::wrapper::ProgMem)
//! is rather difficult, and
//! surprisingly hard if Unicode support is needed
//! (see [issue #3](https://github.com/Cryptjar/avr-progmem-rs/issues/3)).
//! Thus, to make working with string convenient the
//! [`PmString`](string::PmString) struct is provided on top of
//! [`ProgMem`](crate::wrapper::ProgMem).
//!
//! [`PmString`](string::PmString) stores any given `&str` as statically sized
//! UTF-8 byte array (with full Unicode support).
//! To make its content usable, it provides a `Display` & `uDisplay`
//! implementation, a lazy [`chars`](string::PmString::chars) iterator,
//! and [`load`](string::PmString::load) function similar to
//! [`ProgMem`](crate::wrapper::ProgMem)'s
//! `load`,
//! that yields a [`LoadedString`](string::LoadedString),
//! which in turn defers to `&str`.
//!
//! For more details see the [string](crate::string) module.
//!
//! ## Example
//!
//! ```rust
//! use avr_progmem::progmem;
//!
//! progmem! {
//!     // A simple Unicode string in progmem.
//!     static progmem string TEXT = "Hello 大賢者";
//! }
//!
//! // You can load it and use that as `&str`
//! let buffer = TEXT.load();
//! assert_eq!("Hello 大賢者", &*buffer);
//!
//! // Or you use directly the `Display` impl
//! assert_eq!("Hello 大賢者", format!("{}", TEXT));
//! ```
//!
//! Additionally, two special macros are provided similar to the `F` macro
//! of the Arduino IDE, that allows to "mark" a string as to be stored in
//! progmem while being returned at this place as a loaded `&str`.
//!
//! ```rust
//! // Or you skip the static and use in-line progmem strings:
//! use avr_progmem::progmem_str as F;
//! use avr_progmem::progmem_display as D;
//!
//! // Either as `&str`
//! assert_eq!("Foo 大賢者", F!("Foo 大賢者"));
//!
//! // Or as some `impl Display + uDisplay`
//! assert_eq!("Bar 大賢者", format!("{}", D!("Bar 大賢者")));
//! ```
//!
//! If you enabled the `ufmt` crate feature (its a default feature),
//! you can also use `uDisplay` in addition to `Display`.
//!
//! ```rust
//! # #[cfg(feature = "ufmt")] // requires the `ufmt` crate feature
//! # {
//! #
//! use avr_progmem::progmem;
//! use avr_progmem::progmem_str as F;
//! use avr_progmem::progmem_display as D;
//!
//! fn foo<W: ufmt::uWrite>(writer: &mut W) {
//!     progmem! {
//!         // A simple Unicode string in progmem.
//!         static progmem string TEXT = "Hello 大賢者";
//!     }
//!
//!     // You can use the `uDisplay` impl
//!     ufmt::uwriteln!(writer, "{}", TEXT);
//!
//!     // Or use the in-line `&str`
//!     writer.write_str(F!("Foo 大賢者\n"));
//!
//!     // Or the in-line `impl uDisplay`
//!     ufmt::uwriteln!(writer, "{}", D!("Bar 大賢者"));
//! }
//! # struct MyWriter(String);
//! # impl ufmt::uWrite for MyWriter {
//! #     type Error = ();
//! #     fn write_str(&mut self, s: &str) -> Result<(),()> {
//! #         self.0.push_str(s);
//! #         Ok(())
//! #     }
//! # }
//! //
//! # let mut writer = MyWriter(String::new());
//! # foo(&mut writer);
//! # assert_eq!("Hello 大賢者\nFoo 大賢者\nBar 大賢者\n", writer.0);
//! # }
//! ```
//!
//!
//! # Other Architectures
//!
//! As mentioned before, this crate is specifically designed to be use with
//! AVR-base micro-controllers. But since most of us don't write their programs
//! on an AVR system but e.g. on x86 systems, and might want to test them
//! there (well as far as it is possible), this crate also has a fallback
//! implementation for all other architectures that are not AVR, falling back
//! to a simple Rust `static` in the default data segment. And all the data
//! loading functions will just dereference the pointed-to data, assuming that
//! they just live in the default location.
//!
//! This fallback is perfectly safe on x86 and friend, and should also be fine
//! on all further architectures, otherwise normal Rust `static`s would be
//! broken. However, it is an important point to know when for instance writing
//! a library that is not limited to AVR.
//!
//!
//! # Implementation Limitations
//!
//! Aside from what has been already been covered, the current implementation
//! has two further limitations.
//!
//! First, since this crate uses an inline assembly loop on a 8-bit
//! architecture, the loop counter only allows values up to 255. This means
//! that not more that 255 bytes can be loaded at once with any of the methods
//! of this crate. However, this only applies to a single continuous load
//! operation, so for instance `ProgMem<[u8;1024]>::load()` will panic, but
//! accessing such a big type in smaller chunks e.g.
//! `ProgMem<[u8;1024]>::load_sub_array::<[u8;128]>(512)` is perfectly fine
//! because the to be loaded type `[u8;128]` is only 128 bytes in size.
//! Notice that the same limitation holds for `PmString<N>::load()`
//! (i.e. you can only use it if `N <= 255` holds.
//! On the other hand, there is no such limitation on `PmString<N>::chars()`
//! and `PmString`'s `Display`/`uDisplay` implementation,
//! because those, just load each `char` individually
//! (i.e. no more that 4 bytes at a time).
//!
//! Second, since this crate only uses the `lpm` instruction, which is limited
//! by a 16-bit pointer, this crate may only be used with data stored in the
//! lower 64 kiB of program memory. Since this property has not be tested it is
//! unclear whether it will cause a panic or right-up undefined behavior, so be
//! very wary when working with AVR chips that have more then 64 kiB of program
//! memory.
//!
//! [`progmem!`]: https://docs.rs/avr-progmem/latest/avr_progmem/macro.progmem.html
//! [`avr-libc`]: https://crates.io/crates/avr-libc
//! [avr]: https://en.wikipedia.org/wiki/AVR_microcontrollers
//!



pub mod raw;
pub mod string;
pub mod wrapper;
