# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2022-03-12
### Added
- add `ArrayString::add_str`. The method copies characters from a string slice,
  as much as the spare capacity allows, and returns the number of bytes copied.
- add `arraystring::format` function that formats and returns an `ArrayString`
- add `cds::format!` macro for convenient `ArrayString` formatting

## [0.3.0] - 2022-03-06
This release is dedicated to `ArrayString`.

### Added
- add crate feature `alloc` which enables linkage with the standard [alloc] crate
  and usage of its types. `std` feature which enables the standard library implicitly
  enables `alloc`.
- add struct `IndexError` which denotes an out-of-bounds or misaligned index.
- add initial implementation of `ArrayString`
- add initial implementation of array-string fuzz test and benchmark

[alloc]: https://doc.rust-lang.org/alloc/

## [0.2.0] - 2022-03-03
This is a small refactoring-only release done in preparation for `ArrayString`.

### Added
- add new top-level module `cds::mem` for code that deals with memory
- add new top-level module `cds::len` for length types

### Changed
- rename enum `InsertError` and `InsertErrorVal` variants:
  - `IndexOutOfBounds` to `InvalidIndex`
  - `CapacityError` to `InsufficientCapacity`
- move trait `SpareMemoryPolicy` and its implementors from `cds::defs` to `cds::mem`
- move trait `LengthType` and its implementors from `cds::defs` to `cds::len`

### Removed
- remove the top-level module `cds::defs` as its contents were moved out to other modules

## [0.1.0] - 2022-02-18
### Added
- add performance tests for `ArrayVec`

## [0.0.6] - 2022-02-05
### Added
- add `ArrayVec::copy_from_slice`
- add `ArrayVec::copy_from_slice_unchecked`
- add `ArrayVec::try_copy_from_slice`
- implement `std::io::Write` on `ArrayVec`

## [0.0.5] - 2022-02-04
### Added
- add `ArrayVec::resize` implementation
- add `ArrayVec::resize_mut` implementation

## [0.0.4] - 2022-01-28
### Added
- add `ArrayVec::drain` implementation
- add `ArrayVec::retain` implementation
- add `ArrayVec::retain_mut` implementation
- implement the `Extend` trait on `ArrayVec`
- add a fuzz test

## [0.0.3] - 2021-12-18
### Added
- add `ArrayVec::try_push_val` and `ArrayVec::try_insert_val` methods, to return an element to the caller
  in case of an error.
- add `InsertErrorVal` and `CapacityErrorVal` types to be used in `try_insert_val` and `try_push_val` respectively.
- add full test coverage for existing methods.

## [0.0.2] - 2021-12-10
### Added
- `LengthType` - allows customization of the type used to track a fixed-capacity collection's
  length.
- `SpareMemoryPolicy` - allows custom behavior with spare memory in a collection.
- `ArrayVec` initial implementation. This is mostly untested and many features missing,
  but a good skeleton to start with.

## [0.0.1] - 2021-11-20
### Added
`crates.io` placeholder.


[0.0.1]: https://github.com/r-bk/cds/releases/tag/v0.0.1
[0.0.2]: https://github.com/r-bk/cds/compare/v0.0.1...v0.0.2
[0.0.3]: https://github.com/r-bk/cds/compare/v0.0.2...v0.0.3
[0.0.4]: https://github.com/r-bk/cds/compare/v0.0.3...v0.0.4
[0.0.5]: https://github.com/r-bk/cds/compare/v0.0.4...v0.0.5
[0.0.6]: https://github.com/r-bk/cds/compare/v0.0.5...v0.0.6
[0.1.0]: https://github.com/r-bk/cds/compare/v0.0.6...v0.1.0
[0.2.0]: https://github.com/r-bk/cds/compare/v0.1.0...v0.2.0
[0.3.0]: https://github.com/r-bk/cds/compare/v0.2.0...v0.3.0
[0.4.0]: https://github.com/r-bk/cds/compare/v0.3.0...v0.4.0
