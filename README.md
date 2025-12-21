# dioxus-tui

Render Dioxus components into a terminal cell grid using the Blitz DOM/CSS/layout stack (`dioxus-native-dom`), and
present the result either interactively (TUI) or as one-shot stdout output (`render()`).

## Why

- Author UI with Dioxus and render it in terminals with a deterministic, testable pipeline.
- Keep renderer behavior capability-aware (colors, inline images) without per-terminal special-casing.
- Provide a stable `Surface` abstraction that can be snapshot-tested.

## Design

At a high level, the renderer does:

- Build a Dioxus `VirtualDom`.
- Drive it with `dioxus-native-dom::DioxusDocument` (Blitz DOM + CSS + layout).
- Resolve layout for a specific pixel viewport (`layout.rs`).
- Paint into a terminal cell grid (`Surface`) and (optionally) collect placed images (`cell_render.rs`).
- Present either through an interactive terminal loop (`launch*()`) or as a one-shot ANSI stream (`render()`).

Terminal constraints shape the output:

- Cell grid only: everything is mapped to cells (no subcell positioning).
- Grapheme-aware text width (wide/combining/emoji).
- Capability-aware colors (16/256/truecolor).

## Usage

There are two primary APIs:

- `launch*()` starts an interactive TUI session (alternate screen + input loop).
- `render()` renders a single frame to stdout (no alternate screen; scrollback-friendly).

Configuration is provided via `Config`. The most important knob is `RenderingMode`:

- `Visual`: interactive TUI renderer.
- `Debug`: debug snapshot mode.
- `Headless`: no terminal/no input thread (useful for tests).
- `BlitzTerminal`: `render()` only. If the terminal supports image protocols, render the full document as a terminal
  image; otherwise fall back to ANSI text output.
- `BlitzGui`: `launch*()` only. If the environment supports a GUI, launch via `dioxus-native`; otherwise fall back to
  the TUI renderer.

This crate has no default Cargo features:

- `hot-reload`: enables Dioxus hot reloading in debug builds.
- `blitz-terminal`: enables `RenderingMode::BlitzTerminal` (full-document offscreen rasterization via `blitz-paint` and
  emission via terminal image protocols).
- `blitz-gui`: enables `RenderingMode::BlitzGui` (GUI launch via `dioxus-native`).

Example (one-shot render that prefers terminal image protocols when available):

```rust
use dioxus::prelude::*;
use dioxus_tui::{Config, RenderingMode};

fn app() -> Element {
    rsx!(div { "Hello" })
}

fn main() -> anyhow::Result<()> {
    dioxus_tui::render_cfg(app, Config::new().with_rendering_mode(RenderingMode::BlitzTerminal))
}
```

## Images
Run the dashboard example:

```sh
TERM=wezterm cargo run --example render -- --rendering-mode=visual dashboard
```

![Screenshot](https://i.imgur.com/UigQy8M.png)


### Inline `<img>` elements

- With `ImagePolicy::Inline`, when the terminal supports inline image protocols, the renderer emits images using
  OSC 1337 (iTerm2; also supported by WezTerm) or sixel, depending on capabilities.
- If inline images are unsupported, `ImageDowngrade` selects the fallback (`AltText`, `Sampling`, `Omit`, `Error`).

Note: OSC 1337 emission uses `preserveAspectRatio=0` so the requested cell width/height is honored (some terminals
otherwise shrink one dimension).

### Full-document image (`RenderingMode::BlitzTerminal`)

When `RenderingMode::BlitzTerminal` is selected in `render()` and the terminal supports image protocols, the renderer
rasterizes the full document offscreen (Blitz paint pipeline) and emits it as a single terminal image.

## Development

- Examples live under `examples/`.
- Tests render into `Surface` and compare snapshots under `tests/`.

For deeper design notes, see `docs/Design.md`.
