use glam::{IVec3, Vec3};
use winit::{event::ElementState, keyboard::KeyCode};

pub struct BoundingBox {
    min: Vec3,
    max: Vec3,
}

impl BoundingBox {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_points(points: &[Vec3]) -> Self {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for point in points {
            min = min.min(*point);
            max = max.max(*point);
        }

        Self { min, max }
    }

    pub fn from_chunk_position(chunk_position: IVec3) -> Self {
        let min = Vec3::new(
            chunk_position.x as f32 * 16.0,
            chunk_position.y as f32 * 16.0,
            chunk_position.z as f32 * 16.0,
        );
        let max = min + Vec3::new(16.0, 16.0, 16.0);

        Self { min, max }
    }
}

#[derive(Debug)]
pub struct Camera {
    pub eye: glam::Vec3,
    pub target: glam::Vec3,
    pub up: glam::Vec3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            eye: glam::Vec3::new(3.0, 30.0, 3.0),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            aspect: width as f32 / height as f32,
            fovy: 45.0 * std::f32::consts::PI / 180.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }

    pub fn build_view_projection_matrix(&self) -> glam::Mat4 {
        let view = glam::Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = glam::Mat4::perspective_rh(self.fovy, self.aspect, self.znear, self.zfar);
        proj * view
    }

    fn calculate_frustum_planes(&self) -> [glam::Vec4; 6] {
        let view_proj = self.build_view_projection_matrix();
        let mut planes = [glam::Vec4::ZERO; 6];

        // Left plane
        planes[0] = view_proj.row(3) + view_proj.row(0);

        // Right plane
        planes[1] = view_proj.row(3) - view_proj.row(0);

        // Bottom plane
        planes[2] = view_proj.row(3) + view_proj.row(1);

        // Top plane
        planes[3] = view_proj.row(3) - view_proj.row(1);

        // Near plane
        planes[4] = view_proj.row(3) + view_proj.row(2);

        // Far plane
        planes[5] = view_proj.row(3) - view_proj.row(2);

        planes
    }

    pub fn is_in_frustum(&self, bbox: &BoundingBox) -> bool {
        let frustum_planes = self.calculate_frustum_planes(); 
        for plane in frustum_planes {
            let p = Vec3::new(
                if plane.x > 0.0 { bbox.max.x } else { bbox.min.x },
                if plane.y > 0.0 { bbox.max.y } else { bbox.min.y },
                if plane.z > 0.0 { bbox.max.z } else { bbox.min.z },
            );
            if plane.dot(p.extend(1.0)) < 0.0 {
                return false;
            }
        }
        true
    }
}

pub struct CameraController {
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            rotation_x: 0.,
            rotation_y: 0.,
            rotation_z: 0.,
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn process_keyboard(&mut self, key: KeyCode, state: ElementState) -> bool {
        let is_pressed = state == ElementState::Pressed;
        match key {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::Space => {
                self.is_up_pressed = is_pressed;
                true
            }
            KeyCode::ShiftLeft => {
                self.is_down_pressed = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: f32) {
        let forward = camera.target - camera.eye;
        let forward_norm = forward.normalize();
        let right = forward_norm.cross(camera.up);

        if self.is_forward_pressed {
            camera.eye += forward_norm * self.speed * dt;
            camera.target += forward_norm * self.speed * dt;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed * dt;
            camera.target -= forward_norm * self.speed * dt;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed * dt;
            camera.target += right * self.speed * dt;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed * dt;
            camera.target -= right * self.speed * dt;
        }
        if self.is_up_pressed {
            camera.eye += camera.up * self.speed * dt;
            camera.target += camera.up * self.speed * dt;
        }
        if self.is_down_pressed {
            camera.eye -= camera.up * self.speed * dt;
            camera.target -= camera.up * self.speed * dt;
        }
    }
}

// In your Rust code
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn update_view_proj(
        &mut self,
        camera: &mut Camera,
        camera_controller: &CameraController,
        dt: f32,
    ) {
        camera_controller.update_camera(camera, dt);
        let model = glam::Mat4::from_rotation_x(camera_controller.rotation_x)
            * glam::Mat4::from_rotation_y(camera_controller.rotation_y)
            * glam::Mat4::from_rotation_z(camera_controller.rotation_z);
        let view_proj = camera.build_view_projection_matrix();
        self.view_proj = view_proj.to_cols_array_2d();
        self.model = model.to_cols_array_2d();
    }
}