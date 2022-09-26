[![crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/cds.svg
[crates-url]: https://crates.io/crates/cds
[docs-badge]: https://img.shields.io/docsrs/cds
[docs-url]: https://docs.rs/cds/latest/cds


# cds - Collection of Optimized Data Structures

`cds` implements handy data structures written for speed, small memory footprint and security.


## What's included?

- `SpareMemoryPolicy` - a customizable policy for handling spare memory in collections
  (allows wiping unused memory to delete potentially sensitive data)
- `LengthType` - a customizable type to track collection length
  (allows creation of very compact collection types)
- `ArrayVec` - an array with vector-like API
- `ArrayString` - an array with string-like API
- `lformat!` - a macro to format a string on stack, without memory allocation
  (yields an `ArrayString`)
- `SmallVec` - a growable array with optimization for small capacities


## Crate Features

- `arrayvec` - enables `ArrayVec`
- `arraystring` - enables `ArrayString`
- `smallvec` - enables `SmallVec`, implies `alloc`.
- `alloc` - enables usage of the standard [alloc] crate
- `std`- enables usage of the Rust standard library. Implies `alloc`, and enables implementation
  of standard traits which are not available in `core`. Without this feature the crate is `no_std`.

By default, all optional features are enabled. To build in `no_std` environment, or to avoid
compilation of unneeded functionality, use `default-features = false` and choose the required
features explicitly.

[alloc]: https://doc.rust-lang.org/alloc/


## Documentation

The documentation is at [docs.rs/cds][docs-url]


## Roadmap

- `SmallString` - a string with local capacity to avoid heap allocation
- `aformat!` - a fallible macro to format a string on stack


## Changelog

The changelog is maintained in [CHANGELOG.md](CHANGELOG.md)


## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.


## Tags

`small` `secure` `vector` `string` `array` `smallvec` `smallstr`
