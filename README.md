# cds - Collection of Data Structures

`cds` implements handy data structures written for speed and ergonomic API.

[![crates.io][crates-badge]][crates-url]
[![docs.rs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/cds.svg
[crates-url]: https://crates.io/crates/cds
[docs-badge]: https://img.shields.io/docsrs/cds
[docs-url]: https://docs.rs/cds/latest/cds


## What's included?

- `ArrayVec` - an array with vector-like API


## Crate Features

Every data structure has a corresponding crate feature written in all lowercase.
For example, `arrayvec` enables `ArrayVec`. None of the data structures is enabled by default.

Additionally, the following crate features are available:

- `std`- enables usage of the Rust standard library. Without this feature the crate is `no_std`.


## Documentation

The documentation is @ [docs.rs/cds][docs-url]


## Roadmap

- `ArrayString` - a string with fixed capacity
- `SmallVec` - a vector with inline capacity to avoid heap allocation
- `SmallString` - a string with inline capacity to avoid heap allocation


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
