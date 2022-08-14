# Test Programs

All of these programs require a WAYLAND_DISPLAY variable. Normally this is `wayland-1`, check Electrum's logs to verify.

- `image` - Displays an image on a Wayland output. Execute with `cargo run --bin image [IMAGE]`. A sample image can be found in `resources/testing`
- `output` - Lists all outputs on a Wayland display. Execute with `cargo run --bin output`.

These programs can be used to test Electum's window management and compositing functionality, however are just demonstrations and should not be treated as full-featured programs, nor examples for learning Wayland.