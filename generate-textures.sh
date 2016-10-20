#!/bin/sh
aseprite -b ase/7x12.ase --sheet assets/7x12.png --sheet-type rows --sheet-width 112 > /dev/null
aseprite -b ase/5x7.ase --sheet assets/5x7.png --sheet-type rows --sheet-width 80 > /dev/null
aseprite -b ase/3x5.ase --sheet assets/3x5.png --sheet-type rows --sheet-width 48 > /dev/null
aseprite -b ase/playback-state.ase --sheet assets/playback-state.png > /dev/null
aseprite -b ase/spinner.ase --sheet assets/spinner.png > /dev/null
