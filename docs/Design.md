# Design

This document describes the current renderer architecture and the trade-offs that are baked into the codebase.
It intentionally reflects the implementation as it exists today.

## Goals

- Render Dioxus components in terminals through a single, deterministic pipeline.
- Keep capability-dependent behavior explicit and testable (colors, inline images, input).
- Provide two “front doors”:
  - `launch*()` for an interactive TUI renderer.
  - `render()` for one-shot, stdout-friendly output.

## Current architecture

The core building block is `dioxus-native-dom::DioxusDocument`, which embeds the Blitz DOM/CSS/layout engine and is
driven by a Dioxus `VirtualDom`. We resolve layout for a pixel viewport, then paint into a terminal cell grid.

Key modules:

- `src/config.rs`: `Config` and `RenderingMode`.
- `src/capabilities.rs`: runtime capability probing (truecolor, inline image protocols, etc.).
- `src/layout.rs`: sets the Blitz viewport and calls `resolve()`; provides `node_rect()` helpers.
- `src/cell_render.rs`: paints the resolved document into a `Surface` (cell grid) and collects `PlacedImage` entries.
- `src/render.rs`: interactive renderer loop (termwiz input + alternate screen) and frame→change-stream conversion.
- `src/surface.rs`: the in-memory terminal grid representation used for tests and as the staging buffer.
- `src/image.rs`: image loading, fallbacks, and protocol emission helpers.

### Constraints (terminal-first)

- The output is a cell grid. Positions/sizes are mapped from Blitz pixel coordinates to terminal cells.
- Text width is based on Unicode display width; wide/combining glyphs are handled so the grid remains aligned.
- Terminals vary: color depth and image protocols are capability-driven and must have deterministic fallbacks.

## Rendering modes and feature gating

`RenderingMode` is the single switch for “configured vs not configured” behavior.

- `Visual`: interactive TUI renderer.
- `Debug`: prints a debug marker and exits (useful for diagnosing pipeline state).
- `Headless`: no terminal/no input thread (primarily for tests).

Blitz modes are explicit and compile-time gated:

- `BlitzTerminal` (feature: `blitz-terminal`)
  - Only affects `render()`.
  - If the terminal supports inline image protocols, render the *entire document* via Blitz offscreen rasterization and
    emit it as an inline terminal image.
  - If the terminal does not support image protocols, do not enter Blitz rendering and fall back to ANSI output.

- `BlitzGui` (feature: `blitz-gui`)
  - Only affects `launch*()`.
  - If the environment supports a GUI (`DISPLAY`, `WAYLAND_DISPLAY`, macOS, Windows), launch via `dioxus-native`.
  - Otherwise fall back to the TUI renderer.

Rationale: `render()` and `launch*()` have very different output contracts. `render()` is stdout-friendly (no alternate
screen and no cursor-addressing that overwrites existing output), while `launch*()` is an event loop that owns the
terminal.

## Images

There are two distinct image paths:

1) Inline `<img>` elements in the TUI pipeline
   - Painted as `PlacedImage` entries when `ImagePolicy::Inline` is selected.
   - If terminal image protocols are available, they are emitted; otherwise, behavior is controlled by
     `ImageDowngrade` (alt text, sampling, omit, error).

2) Full-document image (`RenderingMode::BlitzTerminal`)
   - When enabled and supported, the renderer paints the Blitz document offscreen using `blitz-paint` and
     rasterizes it with `anyrender_vello_cpu`.
   - The result is emitted as a single terminal image (OSC 1337 or sixel).

## Testing

Tests primarily assert on `Surface` contents (characters and attributes) because it is the stable, deterministic output
of the renderer. This keeps tests independent of terminal emulators and escape sequence quirks.

Recommended coverage areas:

- Layout mapping (`layout.rs`): cell rounding, clipping, and viewport sizing.
- Painting (`cell_render.rs`): text, backgrounds, attributes, and image downgrade paths.
- Capability behavior (`capabilities.rs`): truecolor detection and image protocol detection.
- Render-mode output (`render()`): frame-to-change-stream behavior and masking behavior around inline images.

## Render flow (today)

```mermaid
flowchart TD
  A[Config + capabilities] --> B[VirtualDom]
  B --> C[DioxusDocument (Blitz DOM/CSS/layout)]
  C --> D[layout.rs sets viewport + resolve()]
  D --> E[cell_render.rs paints Surface + PlacedImage]
  E --> F[render(): termwiz TerminfoRenderer to stdout]
  E --> G[launch(): alternate screen + input loop]
```
