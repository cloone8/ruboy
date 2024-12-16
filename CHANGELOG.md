# Changelog

This file will document the most important changes for each released version.

## [v0.1.9]

## [v0.1.8]

### Ruboy GUI
- ROMs can now be opened through the GUI menu

### ruboy_dasm

- Added option to not print instruction addresses

## [v0.1.7]

### All
- Changed emulator cycle-stepping method to being caller driven instead of emulator driven. This means that, instead of Ruboy internally running as fast as possible and throttling itself with busylooping to stay at the correct speed, it now runs exactly the amount of cycles needed to fill the delta-time given by the caller. This allows the emulator to run those cycles as fast as possible and to then return control back to the caller.
- Updated dependencies

## [v0.1.6]

### All
- Fixed CPU bugs
- Implemented CPU timer registers
- Fixed bug in boot ROM mapping
- Implemented (buggy for now) version of object drawing

## [v0.1.5]

### All
- Added README's and some basic documentation. Gotta start somewhere!
- Very basic PPU function. Renders background only. No window or objects
- Fixed some math bugs with rot instructions, and fixed decoding of specific LD instruction

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
