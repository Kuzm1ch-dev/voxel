use glam::Vec2;
use voxel_engine::{Engine, ui::{Anchor, Text, UI, WHITE}};

use crate::systems::ui_system::{Component, UIState};

pub struct HUDComponent {}

impl Component for HUDComponent {
    fn render(
        &mut self,
        mut ui: UI,
        engine: &mut Engine,
        game_state: &crate::game_state::GameState,
        ui_state: &mut UIState,
    ) -> UI {
        let player_pos = game_state.player.get_camera_position();
        let text = Text::new(&format!(
            "x: {:.1} y: {:.1} z: {:.1}",
            player_pos.x, player_pos.y, player_pos.z
        ))
        .with_style(|s| {
            s.position = Vec2::new(16.0, 16.0);
            s.color = WHITE;
        });

        // Меняем ui напрямую, не клонируем
        ui = ui.add_widget(text);

        let ray_pos = game_state.player.get_camera_position();
        let ray_dir = (game_state.player.get_camera_target() - ray_pos).normalize();

        if let Some(hit) =
            crate::systems::raycast::Raycast::cast_ray(ray_pos, ray_dir, 10.0, &game_state.world)
        {
            let block_id = game_state.world.get_block_at(hit.block_pos);
            let look_at_text =
                Text::new(&format!("Looking at: {} At {:?}", block_id, hit.block_pos)).with_style(
                    |s| {
                        s.position = Vec2::new(16.0, 40.0);
                        s.color = WHITE;
                    },
                );
            let look_at_face_text = Text::new(&format!(
                "Face: {:?} Distance: {:.1}",
                hit.face, hit.distance
            ))
            .with_style(|s| {
                s.position = Vec2::new(16.0, 64.0);
                s.color = WHITE;
            });

            // Добавляем к существующему ui
            ui = ui.add_widget(look_at_text);
            ui = ui.add_widget(look_at_face_text);
        } else {
            let look_at_text = Text::new("Looking at: air").with_style(|s| {
                s.position = Vec2::new(16.0, 40.0);
                s.color = WHITE;
            });
            ui = ui.add_widget(look_at_text);
        }

        let crosshair = Text::new("+").with_style(|s| {
            s.anchor = Anchor::Center;
            s.color = WHITE;
            s.size = Vec2::new(0.0, 0.0)
        });

        ui = ui.add_widget(crosshair);
        ui
    }
}