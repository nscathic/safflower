# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.0] 2026-01-01
### Added
- `!include` config line to inlcude more files for parsing. Paths are relative to parent file.

## [0.3.0] 2025-12-30
### Added 
- `Name` struct, for assuring key and locale validity. 
- `Generator` now produces comments to the public members, unless in testing mode.
- `Reader` no longer panics, but throws proper errors.
- Keys entries may be split over separate lines, i.e. the same key may be repeated if none of its entries use the same locale.

### Changed
- `Parser` structure. It is now slightly faster and more convenient.

### Removed
- Benchmark binary. It was included by mistake.

## [0.2.1] 2025-12-21
### Changed
- The routine for parsing args has been corrected, so that arguments may now only start with an alphabetic character or be all digits. This gets rid of the "redundant arguments" error.

## [0.2.0] 2025-12-21
### Changed
- The current locale is no longer kept in an environmnet var and matched on; it is now a static mutex. This change speeds up reading by ~30 %.

## [0.1.0] 2025-12-20
### Added
- Reader, parser, and generator. The basics to make this crate work.
