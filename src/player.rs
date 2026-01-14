use crate::joystick::Vec2;

#[derive(Debug)]
pub struct Player {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: f32, // in radians
}

impl Player {
    pub fn new(start_pos: Vec2) -> Self {
        Self {
            position: start_pos,
            velocity: Vec2::ZERO,
            rotation: 0.0,
        }
    }

    /// 🚀 Update player physics based on joystick input
    pub fn update(&mut self, movement: Vec2, aim: Vec2, dt: f32) {
        // Apply movement (thrust)
        self.velocity += movement * 400.0 * dt;

        // Apply some friction/damping
        self.velocity = self.velocity * 0.98;

        // Update position
        self.position += self.velocity * dt;

        // Update rotation based on aim joystick
        if aim.x != 0.0 || aim.y != 0.0 {
            self.rotation = aim.y.atan2(aim.x);
        }
    }

    pub fn reset_velocity(&mut self) {
        self.velocity = Vec2::ZERO;
    }
}
