use glam::Vec2;
use voxel_engine::{Engine, ui::{colors, elements::{Rect, Text, containers::Container}, layout::Layout, traits::Element}};
// use crate::ui::colors::*;

pub struct UISystem {
    pub is_open: bool,
    inventory_ui: Option<Box<dyn Element>>,
}

impl UISystem {
    pub fn new() -> Self {
        Self {
            is_open: false,
            inventory_ui: None,
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn render(&mut self, engine: &mut Engine, player_pos: glam::Vec3, game_state: &crate::game_state::VoxelGameState) {
        // Всегда показываем координаты
        let pos_text = Text::builder("pos_text")
            .text(format!("x: {:.1} y: {:.1} z: {:.1}", player_pos.x, player_pos.y, player_pos.z))
            .with_base(|base| base
                .color(colors::WHITE)
                .layout(Layout::default())
                .position(Vec2::new(16.0, 22.0))
            )
            .build();
        engine.renderer.ui.add_element(Box::new(pos_text));
        // // Показываем информацию о блоке
        let ray_pos = game_state.player.get_camera_position();
        let ray_dir = (game_state.player.get_camera_target() - ray_pos).normalize();
        if let Some(hit) = crate::systems::raycast::Raycast::cast_ray(ray_pos, ray_dir, 10.0, &game_state.world) {
            let block_id = game_state.world.get_block_at(hit.block_pos);
            let looking_text = Text::builder("looking")
                .text(format!("Looking at: {} At {:?}", block_id, hit.block_pos))
                .with_base(|base| base
                    .color(colors::WHITE)
                    .layout(Layout::default())
                    .position(Vec2::new(16.0, 36.0))
                )
                .build();
            engine.renderer.ui.add_element(Box::new(looking_text));
            
            let face_text = Text::builder("face")
                .text(format!("Face: {:?} Distance: {:.1}", hit.face, hit.distance))
                .with_base(|base| base
                    .color(colors::WHITE)
                    .layout(Layout::default())
                    .position(Vec2::new(16.0, 60.0))
                )
                .build(); 
            engine.renderer.ui.add_element(Box::new(face_text));
        } else {
            let air_text = Text::builder("air")
                .text("Lookint at: air".to_string())
                .with_base(|base| base
                    .color(colors::WHITE)
                    .layout(Layout::default())
                    .position(Vec2::new(16.0, 36.0))
                )
                .build();
            engine.renderer.ui.add_element(Box::new(air_text));
        }

        // Прицел
        let crosshair = Text::builder("air")
            .text("+".to_string())
            .with_base(|base| base
                .color(colors::WHITE)
                .layout(Layout::default())
                .position(Vec2::new(400.0, 300.0))
            )
            .build();
        engine.renderer.ui.add_element(Box::new(crosshair));
        if self.is_open {
            if self.inventory_ui.is_none() {
                engine.renderer.ui.add_element(self.create_inventory_ui(&game_state.world.world.registry));
            }
        }
    }
    
    fn create_inventory_ui(&mut self, registry: &crate::common::block_registry::BlockRegistry) -> Box<dyn Element> {
        // Основной контейнер инвентаря
        // let mut inventory_container = Container::new("inventory_container", Vec2::new(400.0, 300.0), Vec2::ONE, Layout::default());
        // inventory_container.add_child(Box::new(Rect::new("inventory_bg", Vec2::ZERO, Vec2::ONE, colors::DARK_GRAY, Layout::default())));

        let inventory = Text::builder("inventory")
            .text("+".to_string())
            .with_base(|base| base
                .color(colors::WHITE)
                .layout(Layout::default())
                .position(Vec2::new(400.0, 300.0))
            )
            .build();
        return Box::new(inventory);
    }
    
    pub fn handle_click(&mut self, pos: Vec2, screen_size: Vec2) {
        // if !self.is_open {
        //     return;
        // }
        
        // let screen_pos = pos * screen_size;
        
        // if let Some(ref ui) = self.inventory_ui {
        //     if let Some(clicked_id) = ui.hit_test(screen_pos, Vec2::ZERO) {
        //         if clicked_id == "close_btn" || clicked_id == "close_text" {
        //             self.is_open = false;
        //         }
        //     }
        // }
    }
}