oxidamp - A Digital Amplifier in Rust
=====================================

Introduction
------------

oxidamp is a digital amp. modeller written in Rust. Currently it is
purely a partial reincarnation of an older, and incomplete, DSP project
called [tintamp](https://github.com/daniel-thompson/tintamp).

tintamp's only particular noteworthy feature was that is could be
configured to support low power microcontrollers without an FPU. oxidamp
is still being written with an eye on low resource machines although
it isn't yet tested with no_std. It also requires an FPU. In the ten years
since tintamp was created then FPUs now routinely appear in the mid-level
microcontrollers needed to handle audio.

In short, at present oxidamp is just a personal playground... but maybe
one day it will be more!

License
-------

This program is free software: you can redistribute it and/or modify it
under the terms of the [GNU General Public License](LICENSE.md) as
published by the Free Software Foundation, either version 3 of the
License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
General Public License for more details.
