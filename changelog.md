# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased
### Changed
- The current locale is no longer kept in an environmnet var and matched on; it is now a static mutex. This change speeds up reading by ~30 %.

## [0.1.0] 2025-12-20
### Added
- Reader, parser, and generator. The basics to make this crate work.
