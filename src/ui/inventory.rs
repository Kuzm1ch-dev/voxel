use glam::Vec2;
use crate::engine::{Engine, ui::{Anchor, BROWN, Button, Container, DARK_GRAY, GRAY, GREEN, LIGHT_GRAY, LayoutType, RED, SizeMode, Text, UI, WHITE}};

use crate::{game_state::GameState, systems::ui_system::{Component, UIState}};

pub struct InventoryComponent {}

impl Component for InventoryComponent {
    fn render(
        &mut self,
        mut ui: UI,
        engine: &mut Engine,
        game_state: &GameState,
        ui_state: &mut UIState,
    ) -> UI {
        let container = Container::new(LayoutType::Vertical { spacing: 10.0 })
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
                    .on_click(|| println!("Close")),
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
            );
        ui.add_widget(container)
    }
}
