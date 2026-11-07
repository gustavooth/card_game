use royale_defs::ArenaDef;

pub const DT: f32 = 1.0 / royale_shared::TICK_HZ as f32;
pub const MS_PER_TICK: f32 = 1000.0 / royale_shared::TICK_HZ as f32;

pub fn distance(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let dx = bx - ax;
    let dy = by - ay;
    (dx * dx + dy * dy).sqrt()
}

pub fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

pub fn in_river(arena: &ArenaDef, y: f32) -> bool {
    y > arena.river_top && y < arena.river_bottom
}

pub fn bridge_bounds(arena: &ArenaDef, bridge: usize) -> (f32, f32) {
    let pair = arena.bridges.get(bridge).copied().unwrap_or([0.0, 0.0]);
    (pair[0], pair[1])
}

pub fn bridge_center(arena: &ArenaDef, bridge: usize) -> f32 {
    let (a, b) = bridge_bounds(arena, bridge);
    (a + b) * 0.5
}

pub fn on_bridge(arena: &ArenaDef, x: f32) -> bool {
    arena
        .bridges
        .iter()
        .any(|pair| x >= pair[0] && x <= pair[1])
}

pub fn is_crossing(arena: &ArenaDef, from_y: f32, to_y: f32) -> bool {
    (from_y < arena.river_top && to_y > arena.river_bottom)
        || (from_y > arena.river_bottom && to_y < arena.river_top)
}

pub fn nearest_bridge(arena: &ArenaDef, x: f32, target_x: f32) -> usize {
    let mut best = 0;
    let mut best_score = f32::MAX;
    for (index, pair) in arena.bridges.iter().enumerate() {
        let center = (pair[0] + pair[1]) * 0.5;
        let score = (x - center).abs() + (target_x - center).abs() * 0.35;
        if score < best_score {
            best_score = score;
            best = index;
        }
    }
    best
}

/// Ponto de rota para atravessar o rio pela ponte correta.
pub fn crossing_waypoint(
    arena: &ArenaDef,
    x: f32,
    y: f32,
    target_x: f32,
    target_y: f32,
    radius: f32,
) -> Option<(f32, f32)> {
    if !is_crossing(arena, y, target_y) {
        return None;
    }
    let bridge = nearest_bridge(arena, x, target_x);
    let center = bridge_center(arena, bridge);
    let (left, right) = bridge_bounds(arena, bridge);
    let usable_left = left + radius + 0.08;
    let usable_right = right - radius - 0.08;
    let lane_x = clamp(x, usable_left, usable_right)
        .min(usable_right)
        .max(usable_left);

    if in_river(arena, y) {
        let far = if y < (arena.river_top + arena.river_bottom) * 0.5 {
            arena.river_bottom + radius + 0.15
        } else {
            arena.river_top - radius - 0.15
        };
        return Some((lane_x, far));
    }

    let near_side_bottom = y > arena.river_bottom;
    let near_y = if near_side_bottom {
        arena.river_bottom + radius + 0.2
    } else {
        arena.river_top - radius - 0.2
    };
    let far_y = if near_side_bottom {
        arena.river_top - radius - 0.2
    } else {
        arena.river_bottom + radius + 0.2
    };

    let at_bridge_entrance = (x - center).abs() < (right - left) * 0.5 - radius;
    if at_bridge_entrance {
        Some((center.clamp(usable_left, usable_right), far_y))
    } else {
        Some((center.clamp(usable_left, usable_right), near_y))
    }
}
