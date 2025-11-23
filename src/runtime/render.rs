use dioxus_native_core::{prelude::*, tree::TreeRef};
use ratatui::{layout::Rect, style::Color};
use taffy::{
    geometry::Point,
    prelude::{Dimension, Layout, Size},
    Taffy,
};

use crate::runtime::{
    focus::Focused,
    layout::TaffyLayout,
    layout_to_screen_space,
    style::{RinkColor, RinkStyle},
    style_attributes::{BorderEdge, BorderStyle, Borders, StyleModifier},
    widget::{RinkBuffer, RinkCell, RinkWidget, WidgetWithContext},
    config::Config,
};

const RADIUS_MULTIPLIER: [f32; 2] = [1.0, 0.5];

pub(crate) fn render_vnode(
    frame: &mut ratatui::Frame,
    layout: &Taffy,
    node: NodeRef,
    cfg: Config,
    parent_location: Point<f32>,
) {
    if let NodeType::Placeholder = &*node.node_type() {
        return;
    }

    let Layout { mut location, size, .. } =
        layout.layout(node.get::<TaffyLayout>().unwrap().node.unwrap()).unwrap();
    location.x += parent_location.x;
    location.y += parent_location.y;

    let Point { x: fx, y: fy } = location;
    let x = layout_to_screen_space(fx).round() as u16;
    let y = layout_to_screen_space(fy).round() as u16;
    let Size { width, height } = *size;
    let width = layout_to_screen_space(fx + width).round() as u16 - x;
    let height = layout_to_screen_space(fy + height).round() as u16 - y;

    match &*node.node_type() {
        NodeType::Text(text) => {
            #[derive(Default)]
            struct Label<'a> {
                text: &'a str,
                style: RinkStyle,
            }

            impl<'a> RinkWidget for Label<'a> {
                fn render(self, area: Rect, mut buf: RinkBuffer) {
                    for (i, c) in self.text.char_indices() {
                        let mut new_cell = RinkCell::default();
                        new_cell.set_style(self.style);
                        new_cell.symbol = c.to_string();
                        buf.set(area.left() + i as u16, area.top(), new_cell);
                    }
                }
            }

            let label = Label { text: &text.text, style: node.get::<StyleModifier>().unwrap().core };
            let area = Rect::new(x, y, width, height);

            if area.width > 0 && area.height > 0 {
                frame.render_widget(WidgetWithContext::new(label, cfg), area);
            }
        }
        NodeType::Element { .. } => {
            let area = Rect::new(x, y, width, height);

            if area.width > 0 && area.height > 0 {
                frame.render_widget(WidgetWithContext::new(node, cfg), area);
            }

            let node_id = node.id();
            let rdom = node.real_dom();
            for child_id in rdom.tree_ref().children_ids_advanced(node_id, true) {
                let c = rdom.get(child_id).unwrap();
                render_vnode(frame, layout, c, cfg, location);
            }
        }
        NodeType::Placeholder => unreachable!(),
    }
}

impl RinkWidget for NodeRef<'_> {
    fn render(self, area: Rect, mut buf: RinkBuffer<'_>) {
        use ratatui::symbols::line::*;

        enum Direction {
            Left,
            Right,
            Up,
            Down,
        }

        fn draw(
            buf: &mut RinkBuffer,
            points_history: [[i32; 2]; 3],
            symbols: &Set,
            pos: [u16; 2],
            color: &Option<RinkColor>,
        ) {
            let [before, current, after] = points_history;
            let start_dir = match [before[0] - current[0], before[1] - current[1]] {
                [1, 0] => Direction::Right,
                [-1, 0] => Direction::Left,
                [0, 1] => Direction::Down,
                [0, -1] => Direction::Up,
                [a, b] => panic!(
                    "draw({before:?} {current:?} {after:?}) {a}, {b} no cell adjacent"
                ),
            };
            let end_dir = match [after[0] - current[0], after[1] - current[1]] {
                [1, 0] => Direction::Right,
                [-1, 0] => Direction::Left,
                [0, 1] => Direction::Down,
                [0, -1] => Direction::Up,
                _ => panic!(
                    "draw({before:?} {current:?} {after:?}) no cell adjacent to after"
                ),
            };

            let mut new_cell = RinkCell::default();
            if let Some(c) = color { new_cell.fg = *c; }
            new_cell.symbol = match [start_dir, end_dir] {
                [Direction::Down, Direction::Up] => symbols.vertical,
                [Direction::Down, Direction::Right] => symbols.top_left,
                [Direction::Down, Direction::Left] => symbols.top_right,
                [Direction::Up, Direction::Down] => symbols.vertical,
                [Direction::Up, Direction::Right] => symbols.bottom_left,
                [Direction::Up, Direction::Left] => symbols.bottom_right,
                [Direction::Right, Direction::Left] => symbols.horizontal,
                [Direction::Right, Direction::Up] => symbols.bottom_left,
                [Direction::Right, Direction::Down] => symbols.top_left,
                [Direction::Left, Direction::Up] => symbols.bottom_right,
                [Direction::Left, Direction::Right] => symbols.horizontal,
                [Direction::Left, Direction::Down] => symbols.top_right,
            }
            .to_string();
            buf.set(
                (current[0] + pos[0] as i32) as u16,
                (current[1] + pos[1] as i32) as u16,
                new_cell,
            );
        }

        fn draw_arc(
            pos: [u16; 2],
            starting_angle: f32,
            arc_angle: f32,
            radius: f32,
            symbols: &Set,
            buf: &mut RinkBuffer,
            color: &Option<RinkColor>,
        ) {
            if radius < 0.0 { return; }

            let num_points = (radius * arc_angle) as i32;
            let starting_point = [
                (starting_angle.cos() * (radius * RADIUS_MULTIPLIER[0])) as i32,
                (starting_angle.sin() * (radius * RADIUS_MULTIPLIER[1])) as i32,
            ];
            let mut points_history = [
                [0, 0],
                {
                    let ddx = -starting_angle.sin();
                    let ddy = starting_angle.cos();
                    if ddx.abs() > ddy.abs() {
                        [starting_point[0] - ddx.signum() as i32, starting_point[1]]
                    } else {
                        [starting_point[0], starting_point[1] - ddy.signum() as i32]
                    }
                },
                starting_point,
            ];

            for i in 1..=num_points {
                let angle = (i as f32 / num_points as f32) * arc_angle + starting_angle;
                let x = angle.cos() * radius * RADIUS_MULTIPLIER[0];
                let y = angle.sin() * radius * RADIUS_MULTIPLIER[1];
                let new = [x as i32, y as i32];

                if new != points_history[2] {
                    points_history = [points_history[1], points_history[2], new];

                    let dx = points_history[2][0] - points_history[1][0];
                    let dy = points_history[2][1] - points_history[1][1];
                    if dx != 0 && dy != 0 {
                        let connecting_point = match [dx, dy] {
                            [1, 1] => [points_history[1][0] + 1, points_history[1][1]],
                            [1, -1] => [points_history[1][0], points_history[1][1] - 1],
                            [-1, 1] => [points_history[1][0], points_history[1][1] + 1],
                            [-1, -1] => [points_history[1][0] - 1, points_history[1][1]],
                            _ => unreachable!(),
                        };
                        draw(
                            buf,
                            [points_history[0], points_history[1], connecting_point],
                            symbols,
                            pos,
                            color,
                        );
                        points_history = [points_history[1], connecting_point, points_history[2]];
                    }

                    draw(buf, points_history, symbols, pos, color);
                }
            }

            points_history = [points_history[1], points_history[2], {
                let ddx = -(starting_angle + arc_angle).sin();
                let ddy = (starting_angle + arc_angle).cos();
                if ddx.abs() > ddy.abs() {
                    [points_history[2][0] + ddx.signum() as i32, points_history[2][1]]
                } else {
                    [points_history[2][0], points_history[2][1] + ddy.signum() as i32]
                }
            }];

            draw(buf, points_history, symbols, pos, color);
        }

        fn get_radius(border: &BorderEdge, area: Rect) -> f32 {
            match border.style {
                BorderStyle::Hidden | BorderStyle::None => 0.0,
                _ => match border.radius {
                    Dimension::Percent(p) => p * area.width as f32 / 100.0,
                    Dimension::Points(p) => p,
                    _ => unreachable!(),
                }
                .abs()
                .min((area.width as f32 / RADIUS_MULTIPLIER[0]) / 2.0)
                .min((area.height as f32 / RADIUS_MULTIPLIER[1]) / 2.0),
            }
        }

        fn draw_rounded_rect(
            buf: &mut RinkBuffer,
            area: Rect,
            borders: &Borders,
            color: Option<RinkColor>,
        ) {
            let rect = area;

            let BorderEdge { radius: tl, .. } = borders.top_left;
            let BorderEdge { radius: tr, .. } = borders.top_right;
            let BorderEdge { radius: bl, .. } = borders.bottom_left;
            let BorderEdge { radius: br, .. } = borders.bottom_right;

            let tl_radius = get_radius(&borders.top_left, rect);
            let tr_radius = get_radius(&borders.top_right, rect);
            let bl_radius = get_radius(&borders.bottom_left, rect);
            let br_radius = get_radius(&borders.bottom_right, rect);

            let symbols = borders.top.style.symbol_set().unwrap_or(NORMAL);
            let pos = [rect.x, rect.y];

            if tl_radius > 0.0 {
                draw_arc(pos, std::f32::consts::PI, std::f32::consts::PI / 2.0, tl_radius, &symbols, buf, &color);
            }
            if tr_radius > 0.0 {
                draw_arc(pos, 3.0 * std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0, tr_radius, &symbols, buf, &color);
            }
            if bl_radius > 0.0 {
                draw_arc(pos, std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0, bl_radius, &symbols, buf, &color);
            }
            if br_radius > 0.0 {
                draw_arc(pos, 0.0, std::f32::consts::PI / 2.0, br_radius, &symbols, buf, &color);
            }
        }

        let style = self.get::<StyleModifier>().unwrap();
        let mut borders = style.modifier.borders.clone();

        if let Some(border_color) = borders.top.color.or(borders.right.color).or(borders.bottom.color).or(borders.left.color) {
            let area = area;
            draw_rounded_rect(&mut buf, area, &borders, Some(border_color));
        }

        if self.get::<Focused>().is_some() {
            let mut focused_style = RinkStyle::default();
            focused_style.bg = Some(RinkColor { color: Color::Blue, alpha: 50 });
            let mut cell = RinkCell::default();
            cell.set_style(focused_style);
            for x in area.left()..area.right() {
                for y in area.top()..area.bottom() {
                    buf.set(x, y, cell.clone());
                }
            }
        }
    }
}
