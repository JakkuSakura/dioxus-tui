use crate::surface::Cell;
use crate::Surface;
use std::collections::HashMap;
use std::io::{self, Write};
use termwiz::color::{ColorAttribute, SrgbaTuple};

/// Render a [`Surface`] to an ANSI-colored text stream.
///
/// This is designed for non-interactive output (stdout/stderr, pipes, snapshots) and
/// intentionally does not enter the terminal alternate screen.
///
/// If `NO_COLOR` is set in the environment, this falls back to plain text output.
pub fn write_surface_ansi_cropped(out: &mut dyn Write, surface: &Surface) -> io::Result<()> {
    if std::env::var_os("NO_COLOR").is_some() {
        return write_surface_plain_cropped(out, surface);
    }

    let width = surface.width() as usize;
    let height = surface.height() as usize;
    if width == 0 || height == 0 {
        return Ok(());
    }

    let canvas_bg = dominant_background(surface);

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_visible_content(&surface.content[y * width..(y + 1) * width], canvas_bg))
    else {
        return Ok(());
    };

    for y in 0..=bottom_row {
        let row = &surface.content[y * width..(y + 1) * width];
        let right_col = (0..width)
            .rev()
            .find(|&x| cell_has_visible_content(&row[x], canvas_bg));

        match right_col {
            None => {
                write!(out, "\x1b[0m\n")?;
            }
            Some(right_col) => {
                // Establish a stable background color for the line; we'll override per-cell as needed.
                write!(out, "\x1b[0m")?;
                let mut current_fg = ColorAttribute::Default;
                let mut current_bg = ColorAttribute::Default;
                if canvas_bg != ColorAttribute::Default {
                    write_color_sgr(out, canvas_bg, false)?;
                    current_bg = canvas_bg;
                }

                for x in 0..=right_col {
                    let cell = &row[x];
                    let fg = cell.fg.unwrap_or(ColorAttribute::Default);
                    let bg = cell.bg.unwrap_or(ColorAttribute::Default);

                    if fg != current_fg {
                        write_color_sgr(out, fg, true)?;
                        current_fg = fg;
                    }
                    if bg != current_bg {
                        write_color_sgr(out, bg, false)?;
                        current_bg = bg;
                    }

                    write!(out, "{}", cell.ch)?;
                }

                // Reset styles before the newline so we don't leak color into the shell prompt.
                write!(out, "\x1b[0m\n")?;
            }
        }
    }

    out.flush()
}

pub fn write_surface_plain_cropped(out: &mut dyn Write, surface: &Surface) -> io::Result<()> {
    let width = surface.width() as usize;
    let height = surface.height() as usize;
    if width == 0 || height == 0 {
        return Ok(());
    }

    let canvas_bg = dominant_background(surface);

    let Some(bottom_row) = (0..height)
        .rev()
        .find(|&y| row_has_visible_content(&surface.content[y * width..(y + 1) * width], canvas_bg))
    else {
        return Ok(());
    };

    for y in 0..=bottom_row {
        let row = &surface.content[y * width..(y + 1) * width];
        let right_col = (0..width)
            .rev()
            .find(|&x| cell_has_visible_content(&row[x], canvas_bg));

        if let Some(right_col) = right_col {
            for x in 0..=right_col {
                write!(out, "{}", row[x].ch)?;
            }
        }
        writeln!(out)?;
    }

    out.flush()
}

fn row_has_visible_content(row: &[Cell], canvas_bg: ColorAttribute) -> bool {
    row.iter().any(|cell| cell_has_visible_content(cell, canvas_bg))
}

fn cell_has_visible_content(cell: &Cell, canvas_bg: ColorAttribute) -> bool {
    if cell.ch != ' ' {
        return true;
    }

    let fg = cell.fg.unwrap_or(ColorAttribute::Default);
    let bg = cell.bg.unwrap_or(ColorAttribute::Default);

    fg != ColorAttribute::Default || bg != canvas_bg
}

fn dominant_background(surface: &Surface) -> ColorAttribute {
    let mut counts: HashMap<ColorAttribute, usize> = HashMap::new();
    for cell in &surface.content {
        let bg = cell.bg.unwrap_or(ColorAttribute::Default);
        *counts.entry(bg).or_default() += 1;
    }

    counts
        .into_iter()
        .max_by_key(|(_bg, count)| *count)
        .map(|(bg, _count)| bg)
        .unwrap_or(ColorAttribute::Default)
}

fn write_color_sgr(out: &mut dyn Write, color: ColorAttribute, is_foreground: bool) -> io::Result<()> {
    let (default_code, palette_base, rgb_prefix) = if is_foreground {
        (39, 38, "38")
    } else {
        (49, 48, "48")
    };

    match color {
        ColorAttribute::Default => write!(out, "\x1b[{default_code}m"),
        ColorAttribute::PaletteIndex(idx) => match idx {
            0..=7 => write!(out, "\x1b[{}m", palette_base + idx as u16),
            8..=15 => write!(out, "\x1b[{}m", (palette_base + 60) + (idx as u16 - 8)),
            _ => write!(out, "\x1b[{rgb_prefix};5;{idx}m"),
        },
        ColorAttribute::TrueColorWithPaletteFallback(srgb, _) | ColorAttribute::TrueColorWithDefaultFallback(srgb) => {
            let (r, g, b) = srgb_to_u8(srgb);
            write!(out, "\x1b[{rgb_prefix};2;{r};{g};{b}m")
        }
    }
}

fn srgb_to_u8(srgb: SrgbaTuple) -> (u8, u8, u8) {
    let SrgbaTuple(r, g, b, _) = srgb;

    fn to_u8(v: f32) -> u8 {
        let v = v.clamp(0.0, 1.0) * 255.0;
        v.round() as u8
    }

    (to_u8(r), to_u8(g), to_u8(b))
}

