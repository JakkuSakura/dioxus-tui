#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    pub(crate) rendering_mode: RenderingMode,
    #[allow(dead_code)]
    pub(crate) color_mode: ColorMode,
    /// Controls if the terminal quit when the user presses `ctrl+c`?
    /// To handle quitting on your own, use the `TuiContext` root context.
    pub(crate) ctrl_c_quit: bool,
    /// Tick interval used when polling the terminal for input. Can be set to zero for deterministic tests.
    pub(crate) tick_rate: std::time::Duration,
    /// Palette role mapping by capability (16/256/truecolor) for deterministic color selection.
    pub(crate) palette_roles: PaletteRoles,
    /// Policy for handling images/media in terminals without inline image support.
    pub(crate) image_policy: ImagePolicy,
    /// Whether `ImagePolicy::Inline` should downgrade to cell-based rendering when inline images
    /// are unsupported.
    pub(crate) image_downgrade: ImageDowngrade,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rendering_mode(self, rendering_mode: RenderingMode) -> Self {
        Self {
            rendering_mode,
            ..self
        }
    }

    pub fn with_color_mode(self, color_mode: ColorMode) -> Self {
        Self { color_mode, ..self }
    }

    pub fn without_ctrl_c_quit(self) -> Self {
        Self {
            ctrl_c_quit: false,
            ..self
        }
    }

    pub fn with_tick_rate(self, tick_rate: std::time::Duration) -> Self {
        Self { tick_rate, ..self }
    }

    pub fn with_palette_roles(self, palette_roles: PaletteRoles) -> Self {
        Self {
            palette_roles,
            ..self
        }
    }

    pub fn with_image_policy(self, image_policy: ImagePolicy) -> Self {
        Self {
            image_policy,
            ..self
        }
    }

    pub fn with_image_downgrade(self, image_downgrade: ImageDowngrade) -> Self {
        Self {
            image_downgrade,
            ..self
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rendering_mode: RenderingMode::Visual,
            color_mode: ColorMode::Rgb,
            ctrl_c_quit: true,
            tick_rate: std::time::Duration::from_millis(10),
            palette_roles: PaletteRoles::default(),
            image_policy: ImagePolicy::Inline,
            image_downgrade: ImageDowngrade::Sampling,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderingMode {
    /// Render normally
    Visual,
    /// Render debug info instead of UI
    Debug,
    /// In `render()` mode: if the terminal supports image protocols, render the full document
    /// via Blitz offscreen rasterization and emit it as a terminal image; otherwise, fall back
    /// to ANSI text rendering.
    BlitzTerminal,
    /// In `launch()` mode: if the environment supports a GUI, launch via `dioxus-native`;
    /// otherwise, fall back to the TUI renderer.
    BlitzGui,
    /// Do not create a terminal or input thread (useful for tests)
    Headless,
}

impl Default for RenderingMode {
    fn default() -> Self {
        RenderingMode::Visual
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorMode {
    /// only 16 colors accessed by name, no alpha support
    BaseColors,
    /// 8 bit colors, downsampled from rgb colors
    Ansi,
    /// 24 bit colors, most terminals support this
    Rgb,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::Rgb
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PaletteRoles {
    pub bg_primary: PaletteEntry,
    pub bg_muted: PaletteEntry,
    pub bg_focus: PaletteEntry,
    pub fg_primary: PaletteEntry,
    pub accent: PaletteEntry,
}

impl Default for PaletteRoles {
    fn default() -> Self {
        Self {
            bg_primary: PaletteEntry::Rgb(0, 0, 0),
            bg_muted: PaletteEntry::Rgb(17, 17, 17),
            bg_focus: PaletteEntry::Rgb(0, 43, 54),
            fg_primary: PaletteEntry::Rgb(224, 224, 224),
            accent: PaletteEntry::Rgb(0, 188, 212),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PaletteEntry {
    /// Use a 0-15 palette index
    Ansi(u8),
    /// Use a 0-255 palette index
    Palette256(u8),
    /// Truecolor RGB
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug)]
pub enum ImagePolicy {
    /// Prefer native terminal image protocols when available (OSC 1337 on supported terminals).
    /// If inline images are unsupported, behavior is controlled by `image_downgrade`.
    Inline,
    /// Always render a cell-based approximation (▀ sampling).
    Sampling,
    /// Always render the `alt` text (or `"<img unsupported>"`).
    AltText,
    /// Omit images entirely.
    Omit,
    /// Return a hard error if an image cannot be rendered according to the policy.
    Error,
}

#[derive(Clone, Copy, Debug)]
pub enum ImageDowngrade {
    /// Render the `alt` text (or `"<img unsupported>"`).
    AltText,
    /// Render a cell-based approximation (▀ sampling).
    Sampling,
    /// Omit images.
    Omit,
    /// Return a hard error.
    Error,
}
