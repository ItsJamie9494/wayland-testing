# Electrum

A Wayland-only Window Compositing and Window Management solution.

###

[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](http://www.gnu.org/licenses/gpl-3.0)

## Dependencies

This list is non-exhaustive, if `cargo build` fails at any point, please check and see if it is related to a 
missing dependency.

- `wayland`
- `wayland-protocols`

## Building

Simply running `cargo build` should be enough.

<!-- TODO: Instructions for Systemd setup -->

## Testing

Electrum contains two test applications, `output` and `image`. More details may be found in the [/src/bin] directory.

## Running

Electrum may be ran with `cargo run`. When running, please make sure to get the Wayland socket name, usually in the
form of `wayland-#`. This socket name is how you can connect a Wayland client to Electrum.

To run Electrum alongside another Wayland server, you may use the `WAYLAND_DISPLAY` environment variable per-app.
For example, to run the `input` test,

```bash
# Assuming Electrum is bound to socket wayland-1
WAYLAND_DISPLAY=wayland-1 cargo run --bin image
```

## Backends

Electrum will pick a backend based off the `ELECTRUM_BACKEND` variable. If this is missing or invalid, Electrum will fallback to the `winit` backend in development.

## Installing

Electrum cannot be installed at this time.

## Documentation

Documentation is scattered throughout this repository, it is recommended to find a folder of interest (for example,
`/src/bin` for test programs, and `/src/backend` for rendering backends)
All docs are located in README.md files in their respective folders.
