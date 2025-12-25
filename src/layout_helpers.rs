//! Layout helpers for TUI components.

#[derive(Clone, Copy, Debug)]
pub struct ColumnSpec {
    pub min: u16,
    pub weight: f32,
}

pub fn distribute_columns(total: u16, specs: &[ColumnSpec]) -> Vec<u16> {
    if specs.is_empty() {
        return Vec::new();
    }

    let mut widths: Vec<u16> = specs.iter().map(|s| s.min.max(1)).collect();
    let mut sum: u16 = widths.iter().sum();

    if sum < total {
        let extra = total - sum;
        let weight_sum: f32 = specs.iter().map(|s| s.weight.max(0.0)).sum();
        let mut parts: Vec<(usize, f32)> = specs
            .iter()
            .enumerate()
            .map(|(idx, s)| (idx, if weight_sum > 0.0 { s.weight.max(0.0) / weight_sum } else { 1.0 / specs.len() as f32 }))
            .collect();

        let mut allocated = vec![0u16; specs.len()];
        let mut remainder = extra;
        for (idx, frac) in parts.iter().copied() {
            let add = (extra as f32 * frac).floor() as u16;
            allocated[idx] = add;
            remainder = remainder.saturating_sub(add);
        }

        parts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        for (idx, _) in parts {
            if remainder == 0 {
                break;
            }
            allocated[idx] += 1;
            remainder -= 1;
        }

        for (idx, add) in allocated.iter().enumerate() {
            widths[idx] = widths[idx].saturating_add(*add);
        }
    } else if sum > total {
        while sum > total {
            if let Some((idx, _)) = widths
                .iter()
                .enumerate()
                .filter(|(_, w)| **w > 1)
                .max_by_key(|(_, w)| *w)
            {
                widths[idx] -= 1;
                sum -= 1;
            } else {
                break;
            }
        }
    }

    widths
}

pub fn taffy_columns(total: u16, specs: &[ColumnSpec]) -> Vec<u16> {
    if specs.is_empty() {
        return Vec::new();
    }

    use taffy::prelude::TaffyMaxContent;

    let mut tree: taffy::TaffyTree<()> = taffy::TaffyTree::new();
    let mut children = Vec::with_capacity(specs.len());
    for spec in specs {
        let style = taffy::style::Style {
            flex_grow: spec.weight.max(0.0),
            flex_shrink: 1.0,
            min_size: taffy::geometry::Size {
                width: taffy::style::Dimension::length(spec.min.max(1) as f32),
                height: taffy::style::Dimension::length(1.0),
            },
            ..Default::default()
        };
        let node = tree.new_leaf(style).expect("leaf");
        children.push(node);
    }

    let root_style = taffy::style::Style {
        size: taffy::geometry::Size {
            width: taffy::style::Dimension::length(total as f32),
            height: taffy::style::Dimension::length(1.0),
        },
        display: taffy::style::Display::Flex,
        flex_direction: taffy::style::FlexDirection::Row,
        ..Default::default()
    };
    let root = tree.new_with_children(root_style, &children).expect("root");
    tree.compute_layout(root, taffy::geometry::Size::MAX_CONTENT)
        .expect("layout");

    let mut widths: Vec<u16> = children
        .iter()
        .map(|node| {
            let layout = tree.layout(*node).expect("layout");
            layout.size.width.round().max(1.0) as u16
        })
        .collect();

    let sum: i32 = widths.iter().map(|w| *w as i32).sum();
    let total_i32 = total as i32;
    if sum != total_i32 {
        let diff = total_i32 - sum;
        if let Some(last) = widths.last_mut() {
            let new = (*last as i32 + diff).max(1) as u16;
            *last = new;
        }
    }

    widths
}
