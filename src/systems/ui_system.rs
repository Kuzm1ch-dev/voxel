use glam::Vec2;
use crate::engine::{ui::*, Engine};

pub struct UISystem {
    pub is_open: bool,
}

pub struct UIState {
    pub lock_screen: bool,
}

pub trait Component {
    fn render(
        &mut self, ui: UI, engine: &mut Engine, game_state: &crate::game_state::GameState, ui_state: &mut UIState) -> UI;
}

impl UISystem {
    pub fn new() -> Self {
        Self { is_open: false }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn render(
        &mut self,
        engine: &mut Engine,
        player_pos: glam::Vec3,
        game_state: &crate::game_state::GameState,
    ) {
        let mut ui = UI::new();

        // Всегда показываем координаты
        ui = ui.add_widget(
            Text::new(&format!(
                "x: {:.1} y: {:.1} z: {:.1}",
                player_pos.x, player_pos.y, player_pos.z
            ))
            .with_style(|s| {
                s.position = Vec2::new(16.0, 16.0);
                s.color = WHITE;
            }),
        );

        // Показываем информацию о блоке
        let ray_pos = game_state.player.get_camera_position();
        let ray_dir = (game_state.player.get_camera_target() - ray_pos).normalize();
        if let Some(hit) =
            crate::systems::raycast::Raycast::cast_ray(ray_pos, ray_dir, 10.0, &game_state.world)
        {
            let block_id = game_state.world.get_block_at(hit.block_pos);
            ui = ui.add_widget(
                Text::new(&format!("Looking at: {} At {:?}", block_id, hit.block_pos)).with_style(
                    |s| {
                        s.position = Vec2::new(16.0, 40.0);
                        s.color = WHITE;
                    },
                ),
            );

            ui = ui.add_widget(
                Text::new(&format!(
                    "Face: {:?} Distance: {:.1}",
                    hit.face, hit.distance
                ))
                .with_style(|s| {
                    s.position = Vec2::new(16.0, 64.0);
                    s.color = WHITE;
                }),
            );
        } else {
            ui = ui.add_widget(Text::new("Looking at: air").with_style(|s| {
                s.position = Vec2::new(16.0, 40.0);
                s.color = WHITE;
            }));
        }

        // Прицел
        ui = ui.add_widget(Text::new("+").with_style(|s| {
            s.anchor = Anchor::Center;
            s.color = WHITE;
            s.size = Vec2::new(0.0, 0.0)
        }));

        if self.is_open {
            ui = ui.add_widget(self.create_inventory_ui());
        }

        engine.renderer.ui.set_ui(ui);
    }

    fn create_inventory_ui(&mut self) -> Container {
        Container::new(LayoutType::Vertical { spacing: 10.0 })
            .with_style(|s| {
                s.anchor = Anchor::Center;
                s.size = Vec2::new(400.0, 600.0);
                s.color = DARK_GRAY;
                s.padding = Vec2::new(20.0, 20.0);
            })
            .add_text(
                Text::new("Inventory")
                    .with_style(|s| s.color = WHITE)
                    .with_scale(2.0),
            )
            .add_button(
                Button::new("Close")
                    .with_style(|s| {
                        s.size = Vec2::new(100.0, 30.0);
                        s.color = RED;
                    })
                    .with_text_color(WHITE)
                    .on_click(|| println!("Close button clicked!")),
            )
            .add_container(
                Container::new(LayoutType::Grid {
                    columns: 4,
                    spacing: 5.0,
                })
                .with_style(|s| {
                    s.size = Vec2::new(360.0, 100.0);
                    s.color = GRAY;
                    s.padding = Vec2::new(10.0, 10.0);
                })
                .add_button(Button::new("Stone").with_style(|s| {
                    s.color = LIGHT_GRAY;
                    s.size_mode = SizeMode::FillParent;
                }))
                .add_button(Button::new("Dirt").with_style(|s| {
                    s.color = BROWN;
                    s.size_mode = SizeMode::FillParent;
                }))
                .add_button(Button::new("Grass").with_style(|s| {
                    s.color = GREEN;
                    s.size_mode = SizeMode::FillParent;
                }))
                .add_button(Button::new("Wood").with_style(|s| {
                    s.color = BROWN;
                    s.size_mode = SizeMode::FillParent;
                })),
            )
    }

    pub fn handle_click(&mut self, engine: &mut Engine, pos: Vec2) {
        engine.renderer.ui.handle_click(pos);
    }
}
