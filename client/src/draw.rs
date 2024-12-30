use eframe::egui::{vec2, Color32, Label, Rect, RichText, Rounding, Stroke, Ui, Vec2};
use neurojam24_core::{
    ActionType, DisplayData, PlayerAction, PlayerStatus, SpatialDirection, TemporalDirection, SIZE,
};

use crate::Input;

pub fn draw_board(ui: &mut Ui, rect: Rect, display: &DisplayData, t: usize, input: Option<Input>) {
    let (min, size) = (rect.left_top(), rect.size());
    let unit = size.min_elem() / SIZE as f32;
    let arrow_length = unit / 5.;
    for (j, row) in display[t].iter().enumerate() {
        for (i, tile) in row.iter().enumerate() {
            let rect = Rect::from_min_size(
                min + vec2(i as f32 * unit, j as f32 * unit),
                vec2(unit, unit),
            );
            let col = if let Some((player_id, active, status)) = tile.player() {
                player_col(player_id, active)
            } else {
                Color32::GRAY
            };
            ui.painter()
                .rect(rect, Rounding::same(0.5), col, (5.0, Color32::DARK_GRAY));
            // if tile.is_attacked() {
            //     ui.painter()
            //         .circle(rect.center(), unit / 4., Color32::ORANGE, Stroke::NONE);
            // }
            if let Some(action) = tile.outgoing() {
                let col = action_col(action);
                let dir = action_dir(action) * arrow_length;
                ui.painter().arrow(rect.center() + dir, dir, (3., col));
            }
            for (player_id, action, attack) in tile.incoming() {
                match attack {
                    true => {
                        let col = Color32::ORANGE;
                        let dir = action_dir(*action) * arrow_length;
                        ui.painter().arrow(rect.center() - dir, dir, (3., col));
                    }
                    false => {
                        let col = move_col(action.direction.temporal);
                        match action.action_type {
                            ActionType::Move => {
                                let dir = action_dir(*action) * arrow_length;
                                ui.painter().arrow(rect.center() - dir, dir, (3., col));
                            }
                            ActionType::Attack => {
                                ui.painter()
                                    .circle(rect.center(), arrow_length, col, Stroke::NONE);
                            }
                        }
                    }
                }
            }
            if let Some((player_id, active, status)) = tile.player() {
                draw_status(ui, rect, status);
            }
            if let Some(input) = input {
                if let Some((player_id, active, status)) = tile.player() {
                    if active && player_id == input.player_id {
                        draw_input(ui, rect, &input);
                        // draw_status(ui, rect, status);
                    }
                }
            }
        }
    }
}

fn player_col(player_id: u8, active: bool) -> Color32 {
    match player_id {
        0 => match active {
            true => Color32::LIGHT_GREEN,
            false => Color32::DARK_GREEN,
        },
        1 => match active {
            true => Color32::LIGHT_BLUE,
            false => Color32::DARK_BLUE,
        },
        _ => Color32::DEBUG_COLOR,
    }
}

fn move_col(temporal: TemporalDirection) -> Color32 {
    match temporal {
        TemporalDirection::Forward => Color32::from_rgb(200, 0, 200),
        TemporalDirection::Backward => Color32::from_rgb(120, 0, 120),
    }
}

fn action_col(action: PlayerAction) -> Color32 {
    match action.action_type {
        ActionType::Move => move_col(action.direction.temporal),
        ActionType::Attack => Color32::ORANGE,
    }
}

fn action_dir(action: PlayerAction) -> Vec2 {
    spatial_dir(action.direction.spatial)
}

fn spatial_dir(spatial: SpatialDirection) -> Vec2 {
    match spatial {
        SpatialDirection::Left => vec2(-1., 0.),
        SpatialDirection::Right => vec2(1., 0.),
        SpatialDirection::Up => vec2(0., -1.),
        SpatialDirection::Down => vec2(0., 1.),
    }
}

pub fn draw_input(ui: &mut Ui, rect: Rect, input: &Input) {
    let (cen, size) = (rect.center(), rect.size());
    let diag = size.length();
    let k = 2. / 3.;
    let arrow_col = match input.temporal {
        Some(temporal) => move_col(temporal),
        None => Color32::BLACK,
    };
    match input.spatial {
        Some(spatial) => {
            let arrow_dir = spatial_dir(spatial);
            ui.painter()
                .arrow(cen, arrow_dir * size.min_elem() / 3., (3., arrow_col));
        }
        None => {
            ui.painter()
                .circle_filled(cen, size.min_elem() / 12., arrow_col);
        }
    }
    if let Some(action) = input.action {
        let char = match action {
            ActionType::Move => '👣',
            ActionType::Attack => '⚔',
        };
        ui.put(
            Rect::from_center_size(
                cen + vec2(1., 1.).normalized() * diag * k / 2.,
                size * k / 2.,
            ),
            Label::new(
                RichText::new(char)
                    .color(Color32::BLACK)
                    .size(size.y * k / 4.),
            ),
        );
    }
    // ui.painter().circle_filled(
    //     cen,
    //     size.min_elem() / 12.,
    //     player_col(input.player_id, true),
    // );
}
pub fn draw_status(ui: &mut Ui, rect: Rect, status: PlayerStatus) {
    let (cen, size) = (rect.center(), rect.size());
    let diag = size.length();
    let k = 2. / 3.;
    let mut draw = |vec: Vec2, text: String| {
        ui.put(
            Rect::from_center_size(cen + vec.normalized() * diag * k / 2., size * k / 2.),
            Label::new(
                RichText::new(text)
                    .color(Color32::BLACK)
                    .size(size.y * k / 5.),
            ),
        );
    };
    draw(
        vec2(1., -1.),
        "❤\n".to_string() + &status.health.to_string(),
    );
    draw(
        vec2(-1., -1.),
        "🛡\n".to_string() + &status.iframes.to_string(),
    );
    draw(vec2(-1., 1.), "⏰\n".to_string() + &status.time.to_string());
}
