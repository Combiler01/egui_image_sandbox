# egui_image_sandbox

An interactive image manipulation sandbox and showcase built with Rust, [egui](https://github.com/emilk/egui), and [eframe](https://github.com/emilk/egui/tree/master/crates/eframe).

`egui_image_sandbox` demonstrates geometrically stable, anchor-based interactive image editing in an immediate-mode GUI environment.

---

## Features

- **8-Point Anchor-Based Resizing**:
  - Interactive handles positioned on all 4 corners and 4 edge midpoints.
  - Mathematically locks the diagonally opposite point or edge as an invariant anchor.
  - Clamped with minimum size limits to avoid inversions or zero-division glitches.
- **Non-Destructive Invariant-Scale Cropping**:
  - Dragging crop handles adjusts normalized UV texture coordinates (`[0.0, 1.0]`) and screen bounding boxes in lockstep.
  - Image content scale remains strictly invariant—preventing jumping, stretching, or pixel distortion while cropping.
- **Image File Loading**:
  - Open images locally (`PNG`, `JPEG`, `WEBP`, `BMP`) via system native dialogs using `rfd`.
- **Comprehensive Unit Tests**:
  - Verification of mathematical invariants across corner and midpoint anchor resizes and lockstep crop ratios.

---

## Getting Started

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (edition 2024 / latest stable Rust toolchain).

### Build and Run

To run the application locally:

```bash
cargo run --release
```

### Run Tests

Execute unit tests verifying anchor locks and crop math:

```bash
cargo test
```

---

## Dependencies

- **[egui](https://crates.io/crates/egui)** & **[eframe](https://crates.io/crates/eframe)**: Immediate-mode GUI framework and runner.
- **[image](https://crates.io/crates/image)**: Image decoding and processing.
- **[rfd](https://crates.io/crates/rfd)**: Native file dialogs for cross-platform file picking.

---

## License

This project is licensed under the Apache License, Version 2.0. See the [LICENSE](LICENSE) file for details.
