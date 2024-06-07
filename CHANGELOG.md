# Changelog

This file will document the most important changes for each released version.

## [v0.1.5]

## [v0.1.4]

### All
- Fixed incorrect calculation of the amount of banks in a ROM file

### ruboy_romdump
- Fixed incorrect output of lists (specifically, the hardware list of a ROM)
- Added licensee name resolving

## [v0.1.3]

### All
- Fixed incorrect decoding of `RLA`, `RRA`, `RCLA` and `RRCA` instruction. They were being confused with their prefixed counterparts (`RLA` and `RL A`, for example). Not anymore!

### ruboy_dasm

- Adds command line options to tweak instruction output format
- Fixes error in ROM file reader that could result in incorrect instruction decoding

## [v0.1.2]

### All
- Lots of under-the-hood progress of the main library, in order to support full cross-platform emulation.

### ruboy_dasm

- Corrected wrong instruction length for `RLA`, `RRA`, `RLCA` and `RRCA` instruction, which
  led to incorrect instruction decoding
- Improved output layout

### ruboy_romdump

- Improved output layout

## [v0.1.0]
Initial release
