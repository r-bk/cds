# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.6] - Unreleased
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
