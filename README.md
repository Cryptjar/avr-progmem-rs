# avr-progmem

<!-- cargo-sync-readme start -->


Progmem utilities for the AVR architectures.

This crate provides unsafe utilities for working with data stored in
the program memory of an AVR micro-controller. Additionally, it defines a
'best-effort' safe wrapper struct [`ProgMem`] to simplify working with it.

This crate is implemented only in Rust and some short assembly, it does NOT
depend on the [`avr-libc`] or any other C-library. However, due to the use
of inline assembly, this crate may only be compiled using a **nightly Rust**
compiler.


# AVR Memory

This crate is specifically for [AVR-base micro-controllers][avr] such as
the Arduino Uno (and some other Arduino boards, but not all), which have a
modified Harvard architecture which implies the strict separation of program
code and data while having special instructions to read and write to
program memory.

While, of course, all ordinary data is stored in the data domain where it is
perfectly usable, the harsh constraints of most AVR processors make it very
appealing to use the program memory (also referred to as _progmem_) for
storing constant values. However, due to the Harvard design, those values
are not usable with normal instructions (i.e. those emitted from normal
Rust code). Instead, special instructions are required to load data from
the program code domain, i.e. the `lpm` (load _from_ program memory)
instruction. And because there is no way to emit it from Rust code, this
crate uses inline assembly to emit that instruction.

However, since a pointer into program code cannot be differentiated from a
normal data pointer, it is entirely up to the programmer to ensure that
these different 'pointer-types' are not accidentally mixed. In other words,
this is `unsafe` in the context of Rust.


# Loading Data from Program Memory

The first part of this crate simply provides a few functions (e.g.
[`read_byte`]) to load constant data (i.e. a Rust `static` that is
immutable) from the program memory into the data domain, so that
sub-sequentially it is normal usable data, i.e. as owned data on the stack.

Because, as aforementioned, a simple `*const u8` in Rust does not specify
whether is lives in the program code domain or the data domain, all
functions which simply load a given pointer from the program memory are
inherently `unsafe`.

Notice that using a `&u8` reference might make things rather worse than
safe. Because keeping a pointer/reference/address into the program memory
as Rust reference might easily cause it to be dereferenced, even in safe
code. But since that address is only valid in the program code domain (and
Rust doesn't know about it) it would illegally load the address from the
data memory, causing **undefined behavior**!

## Example

```rust
use avr_progmem::read_byte;

// This `static` must never be directly dereferenced/accessed!
// So a `let data: u8 = P_BYTE;` is **undefined behavior**!!!
/// Static byte stored in progmem!
#[link_section = ".progmem"]
static P_BYTE: u8 = b'A';

// Load the byte from progmem
// Here, it is sound, because due to the link_section it is indeed in the
// program code memory.
let data: u8 = unsafe { read_byte(&P_BYTE) };
assert_eq!(b'A', data);
```


# The best-effort Wrapper

Since working with progmem data is inherently unsafe and rather
difficult to do correctly, this crate introduces the best-effort 'safe'
wrapper [`ProgMem`], that is supposed to only wrap data in progmem, thus
offering only functions to load its content using the progmem loading
function.
The latter are fine and safe, given that the wrapper type really contains
data in the program memory. Therefore, to keep that invariant up, the
constructor is `unsafe`.

Yet to make that also easier, this crate provides the [`progmem!`] macro
(it has to be a macro), which will create a static variable in program
memory for you and wrap it in the `ProgMem` struct. It will ensure that the
`static` will be stored in the program memory by defining the
`#[link_section = ".progmem"]` attribute on it. This makes the load
functions on that struct sound and additionally prevents users to
accidentally access that `static` directly, which, since it is in progmem,
would be fundamentally unsound.

## Example

```rust
use avr_progmem::progmem;

// It will be wrapped in the ProgMem struct and expand to:
// ```
// #[link_section = ".progmem"]
// static P_BYTE: ProgMem<u8> = unsafe { ProgMem::new(b'A') };
// ```
// Thus it is impossible for safe Rust to directly dereference/access it!
progmem! {
    /// Static byte stored in progmem!
    static progmem P_BYTE: u8 = b'A';
}

// Load the byte from progmem
// It is still sound, because the `ProgMem` guarantees us that it comes
// from the program code memory.
let data: u8 = P_BYTE.load();
assert_eq!(b'A', data);
```


# Other Architectures

As mentioned before, this crate is specifically designed to be use with
AVR-base micro-controllers. But since most of us don't write their programs
on an AVR system but e.g. on x86 systems, and might want to test them
there (well as far as it is possible), this crate also has a fallback
implementation for all other architectures that are not AVR, falling back
to a simple Rust `static` in the default data segment. And all the data
loading functions will just dereference the pointed-to data, assuming that
they just live in the default location.

This fallback is perfectly safe on x86 and friend, and should also be fine
on all further architectures, otherwise normal Rust `static`s would be
broken. However, it is an important point to know when for instance writing
a library that is not limited to AVR.


# Implementation Limitations

Aside from what has been already been covered, the current implementation
has two further limitations.

First, since this crate uses an inline assembly loop on a 8-bit
architecture, the loop counter only allows values up to 255. This means
that not more that 255 bytes can be loaded at once with any of the methods
of this crate. However, this only applies to a single continuous load
operation, so for instance `ProgMem<[u8;1024]>::load()` will panic, but
accessing such a big type in smaller chunks e.g.
`ProgMem<[u8;1024]>::load_sub_array::<[u8;128]>(512)` is perfectly fine
because the to be loaded type `[u8;128]` is only 128 bytes in size.

Second, since this crate only uses the `lpm` instruction, which is limited
by a 16-bit pointer, this crate may only be used with data stored in the
lower 64 kiB of program memory. Since this property has not be tested it is
unclear whether it will cause a panic or right-up undefined behavior, so be
very wary when working with AVR chips having more then 64 kiB of program
memory.
This second restriction, of course, dose not apply to non-AVR architectures.


[`ProgMem`]: https://docs.rs/avr-progmem/latest/avr_progmem/struct.ProgMem.html
[`read_byte`]: https://docs.rs/avr-progmem/latest/avr_progmem/fn.read_byte.html
[`progmem!`]: https://docs.rs/avr-progmem/latest/avr_progmem/macro.progmem.html
[`avr-libc`]: https://crates.io/crates/avr-libc
[avr]: https://en.wikipedia.org/wiki/AVR_microcontrollers


<!-- cargo-sync-readme end -->

# License

Licensed under Apache License, Version 2.0 ([LICENSE](LICENSE) or https://www.apache.org/licenses/LICENSE-2.0).

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.


