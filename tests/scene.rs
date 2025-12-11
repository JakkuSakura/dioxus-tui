use std::collections::VecDeque;

use anyrender::types::{Glyph, PaintRef};
use anyrender::PaintScene;
use dioxus_tui::scene::{CellMetrics, InlineImage, TerminalScene};
use dioxus_tui::{ColorMode, Surface};
use kurbo::{Affine, Rect, Stroke};
use peniko::{BlendMode, Color, Fill, FontData, StyleRef};
use peniko::Blob;

fn metrics() -> CellMetrics {
    CellMetrics {
        cell_w_px: 1.0,
        cell_h_px: 1.0,
    }
}

fn assert_cell(surface: &Surface, x: u16, y: u16, ch: char) {
    let idx = y as usize * surface.width() as usize + x as usize;
    let cell = &surface.content[idx];
    assert_eq!(cell.ch, ch);
}

#[test]
fn fill_sets_background_color() {
    let mut surface = Surface::new(4, 2);
    let mut images = VecDeque::new();
    {
        let mut scene = TerminalScene::new(&mut surface, &mut images, metrics(), ColorMode::Rgb, true);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Solid(Color::from_rgb8(200, 10, 10)),
            None,
            &Rect::new(0.0, 0.0, 2.0, 1.0),
        );
    }
    let cell = &surface.content[0];
    assert!(cell.bg.is_some());
}

#[test]
fn stroke_draws_border_with_color() {
    let mut surface = Surface::new(4, 2);
    let mut images = VecDeque::new();
    {
        let mut scene = TerminalScene::new(&mut surface, &mut images, metrics(), ColorMode::Rgb, true);
        scene.stroke(
            &Stroke::default(),
            Affine::IDENTITY,
            PaintRef::Solid(Color::from_rgb8(0, 0, 255)),
            None,
            &Rect::new(0.0, 0.0, 3.0, 1.0),
        );
    }
    // Top edge: spaces with background color
    for x in 0..3 {
        let idx = x as usize;
        assert_eq!(surface.content[idx].ch, ' ');
        assert!(surface.content[idx].bg.is_some());
    }
    // Vertical edges have background painted
    assert!(surface.content[0].bg.is_some());
    assert!(surface.content[3].bg.is_some());
}

#[test]
fn clipping_limits_paint() {
    let mut surface = Surface::new(2, 2);
    let mut images = VecDeque::new();
    {
        let mut scene = TerminalScene::new(&mut surface, &mut images, metrics(), ColorMode::Rgb, true);
        scene.push_layer(BlendMode::default(), 1.0, Affine::IDENTITY, &Rect::new(0.0, 0.0, 1.0, 1.0));
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Solid(Color::from_rgb8(50, 200, 50)),
            None,
            &Rect::new(0.0, 0.0, 2.0, 2.0),
        );
        scene.pop_layer();
    }
    assert!(surface.content[0].bg.is_some());
    assert!(surface.content[1].bg.is_none());
}

#[test]
fn draw_glyphs_sets_char_and_color() {
    let mut surface = Surface::new(2, 1);
    let mut images = VecDeque::new();
    let font = FontData::new(Blob::from(Vec::<u8>::new()), 0);
    {
        let mut scene = TerminalScene::new(&mut surface, &mut images, metrics(), ColorMode::Rgb, true);
        scene.draw_glyphs(
            &font,
            12.0,
            false,
            &[],
            StyleRef::Fill(Fill::NonZero),
            PaintRef::Solid(Color::from_rgb8(10, 200, 10)),
            1.0,
            Affine::IDENTITY,
            None,
            std::iter::once(Glyph { id: 1, x: 0.0, y: 0.0 }),
        );
    }
    let cell = &surface.content[0];
    assert_ne!(cell.ch, ' ');
    assert!(cell.fg.is_some());
}

#[test]
fn reset_clears_surface_and_images() {
    let mut surface = Surface::new(2, 1);
    let mut images = VecDeque::new();
    images.push_back(InlineImage {
        data: vec![255, 0, 0, 255],
        width_px: 1,
        height_px: 1,
        x_px: 0.0,
        y_px: 0.0,
    });
    {
        let mut scene = TerminalScene::new(&mut surface, &mut images, metrics(), ColorMode::Rgb, true);
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Solid(Color::from_rgb8(0, 0, 0)),
            None,
            &Rect::new(0.0, 0.0, 1.0, 1.0),
        );
        scene.reset();
    }
    assert!(surface.content.iter().all(|c| c.ch == ' ' && c.fg.is_none() && c.bg.is_none()));
    assert!(images.is_empty());
}
