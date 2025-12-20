# Dioxus TUI

Render HTML via Blitz/Servo into a terminal grid with a single, deterministic pipeline powered by `TerminalScene` and
`FakeTerminal`.

## Why

- React-like authoring with Rust/Dioxus, rendered faithfully in terminals.
- Single render path: Servo display list → terminal primitives → cell layout → paint → surface.
- Deterministic, snapshot-testable output via `FakeTerminal` and capability-aware palette roles.

## Architecture (should-be)

- Servo boundary: Servo owns DOM, CSS cascade, and layout. This crate adapts the Servo display list to terminal cells.
- Pipeline: display list → `element.rs` primitives → `geometry.rs` cell mapping → `layout.rs` cell placement →
  `render.rs` frame buffer → `surface.rs` present (real terminal or `FakeTerminal`).
- Capabilities/config: `config.rs` (size, palette mode, policies), `capabilities.rs` (color depth, input, inline
  images), `geometry.rs` (single source for cell metrics and length mapping).
- Styling: default TUI CSS with palette roles; `styles.rs` maps Servo styles to terminal-friendly values and marks
  unsupported features.
- Images/media: `image.rs` policies (block, degrade, omit); animations use deterministic time sources.
- Events: terminal events normalized by `capabilities.rs`, dispatched by `scene.rs` back to Servo.
- Hooks: diagnostics, tracing, deterministic clock injection for tests/examples.

## TUI constraints (must-haves)

- Cell grid only: no subpixel positioning; all clipping/scrolling is cell-based.
- Grapheme-aware text: measure by grapheme width (wide/combining/emoji) before painting.
- Color caps: support 16/256/truecolor with palette-role fallbacks; no alpha blending.
- Limited effects: last paint per cell wins; avoid shadows/blur; prefer discrete state changes over animations.
- Input variance: keyboard-first; mouse/IME/clipboard are best-effort; focus cues must survive minimal terminals.

## Testing and examples

- Fixtures + snapshots: HTML/CSS fixtures mapped to snapshots (text grid + attributes) with normalization for
  portability.
- Determinism: fixed terminal size/palette, deterministic render/flush, deterministic time source.
- Interaction: tests cover tab order, scrolling, selection, forms, and event normalization.
- Capability variants: test canonical 16/256/truecolor profiles; `FakeTerminal` runs examples in CI for snapshot
  comparison.
- Diagnostics: grid diffs (including attributes) on mismatch.

If you want to render without entering the terminal alternate screen, see `examples/render_dashboard.rs`.

## Inline images

- Layout for `<img>` uses the same Blitz/Taffy pipeline as other nodes (`layout.rs` + `node_rect`).
- With `ImagePolicy::Inline`, if the terminal supports OSC 1337 inline images (iTerm2 protocol; also supported by WezTerm), the renderer emits an inline image.
- If inline images are unsupported, `ImageDowngrade` selects the fallback: `AltText`, `Sampling` (cell approximation), `Omit`, or `Error`.
- `ImagePolicy::Sampling` forces the cell approximation even when inline images are supported.
- `ImagePolicy::AltText` always renders `alt` (or `"<img unsupported>"`), `ImagePolicy::Omit` skips images, and `ImagePolicy::Error` returns a hard error.

Note: the OSC 1337 image emission uses `preserveAspectRatio=0` so the requested cell width/height is honored (some terminals will otherwise shrink one dimension).

## Default TUI CSS (roles)

Use palette roles and cell-friendly units; map roles per capability profile:

- 16-color: `--bg-primary=black`, `--bg-muted=black`, `--bg-focus=blue`, `--fg-primary=white`, `--accent=cyan`
- 256-color: `--bg-primary=16`, `--bg-muted=235`, `--bg-focus=24`, `--fg-primary=252`, `--accent=45`
- truecolor: `--bg-primary=#000000`, `--bg-muted=#111111`, `--bg-focus=#002b36`, `--fg-primary=#e0e0e0`,
  `--accent=#00bcd4`

```css
html, body {
    margin: 0;
    padding: 0;
    font-family: monospace;
    background: var(--bg-primary, black);
    color: var(--fg-primary, white);
}

a {
    color: var(--accent, cyan);
    text-decoration: underline;
}

strong, b {
    font-weight: bold;
}

em, i {
    font-style: italic;
}

code, pre {
    font-family: monospace;
    background: var(--bg-muted, #111);
    padding: 0;
}

ul, ol {
    margin: 0;
    padding-left: 2ch;
}

button, input, select, textarea {
    background: var(--bg-muted, #111);
    color: var(--fg-primary, white);
    border: none;
    padding: 0;
}

button:focus, input:focus, select:focus, textarea:focus {
    outline: none;
    background: var(--bg-focus, #002b36);
    color: var(--accent, cyan);
}
```

## TODOs (canonical design, migrate code to match)

- [x] Servo display list contract (`element.rs`): define coordinate space (logical vs px), rounding rules to cells, and
  invalidation/diff signals; adapt current extraction to that contract.
- [x] Deterministic clock (config/hooks): add a time provider hook for animations/time-based effects and thread it
  through render; use in tests/examples.
- [x] Single render path (scene/render/layout): remove parallel/manual rendering paths and route painting solely through
  Servo display list → primitives → cell layout → `render.rs` → `surface.rs`.
- [x] Capability profiles (`config.rs`/`capabilities.rs`): bake 16/256/truecolor palette role maps and expose them to
  styles/tests/examples; normalize color mapping accordingly.
- [x] Snapshot harness (`tests/fixtures`, `FakeTerminal`): finalize snapshot format + normalization; add fixtures for
  layout/styles/inheritance, links/lists/forms, overflow/wrapping, wide/combining/RTL text, malformed inputs, and event
  normalization.
- [x] Event model (`hooks.rs`/`scene.rs`): specify terminal event normalization (keyboard/mouse/focus) and ensure
  deterministic dispatch into Servo.
- [x] Image/media policy (`image.rs`): codify block/degrade/omit rules and fallbacks for terminals without image
  support; verify via tests/examples.
- [x] Common HTML rendering snapshots (`tests/render_html.rs`): add coverage for headings, paragraphs, and lists using the canonical pipeline.

For deeper details, see `docs/Design.md`.
