# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added a simple client implementation.

## 0.2.2 - 2025-03-29

### Fixed

- Changed the `Message::decode` `data` parameter type from `&GenericArray<u8, Self::EncodedLen>` to `&[u8]`.

## 0.2.1 - 2025-03-29

### Fixed

- Reverted to Rust edition 2021.

## 0.2.0 - 2025-03-29

### Changed

- Renamed `CanId` to `Identifier`, added `ExtendedId` struct from `embedded-can`.
- Updated to Rust edition 2024.
