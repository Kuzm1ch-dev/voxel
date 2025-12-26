use glam::Vec3;

pub struct GamePlayer {
    position: Vec3,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
    sensitivity: f32,
}

impl GamePlayer {
    pub fn new(position: Vec3) -> Self {
        println!("[DEBUG] Creating player at position: {:?}", position);
        Self {
            position,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            speed: 10.0,
            sensitivity: 0.005,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.velocity *= 0.9;
    }

    pub fn get_camera_position(&self) -> Vec3 {
        self.position + Vec3::new(0.0, 1.8, 0.0)
    }

    pub fn get_camera_target(&self) -> Vec3 {
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        self.get_camera_position() + forward
    }

    pub fn get_camera_up(&self) -> Vec3 {
        Vec3::Y
    }

    pub fn move_forward(&mut self, amount: f32) {
        let forward = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize();
        self.velocity += forward * amount * self.speed;
    }

    pub fn move_backward(&mut self, amount: f32) {
        self.move_forward(-amount);
    }

    pub fn move_left(&mut self, amount: f32) {
        let right = Vec3::new(self.yaw.sin(), 0.0, -self.yaw.cos()).normalize();
        self.velocity += right * amount * self.speed;
    }

    pub fn move_right(&mut self, amount: f32) {
        self.move_left(-amount);
    }

    pub fn move_up(&mut self, amount: f32) {
        self.velocity.y += amount * self.speed;
    }

    pub fn move_down(&mut self, amount: f32) {
        self.velocity.y -= amount * self.speed;
    }

    pub fn look(&mut self, yaw: f32, pitch: f32) {
        self.yaw += yaw * self.sensitivity;
        self.pitch += pitch * self.sensitivity;
        self.pitch = self.pitch.clamp(-1.5, 1.5);
    }
}