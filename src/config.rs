#[derive(Clone, Copy)]
#[non_exhaustive]
pub struct Config {
    pub(crate) rendering_mode: RenderingMode,
    #[allow(dead_code)]
    pub(crate) color_mode: ColorMode,
    /// Controls if the terminal quit when the user presses `ctrl+c`?
    /// To handle quitting on your own, use the `TuiContext` root context.
    pub(crate) ctrl_c_quit: bool,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rendering_mode(self, rendering_mode: RenderingMode) -> Self {
        Self { rendering_mode, ..self }
    }

    pub fn with_color_mode(self, color_mode: ColorMode) -> Self {
        Self { color_mode, ..self }
    }

    pub fn without_ctrl_c_quit(self) -> Self {
        Self { ctrl_c_quit: false, ..self }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            rendering_mode: RenderingMode::Visual,
            color_mode: ColorMode::Rgb,
            ctrl_c_quit: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RenderingMode {
    /// Render normally
    Visual,
    /// Render debug info instead of UI
    Debug,
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
