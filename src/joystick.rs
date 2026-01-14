#[derive(Debug, Clone, Copy, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn normalize(&self) -> Vec2 {
        let len = self.length();
        if len > 0.0 {
            Vec2 {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            Vec2::ZERO
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f32> for Vec2 {
    type Output = Vec2;

    fn div(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl std::ops::AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;
    }
}

/// 🕹️ Joystick structure for touch input
#[derive(Debug, Clone)]
pub struct Joystick {
    /// Where the finger started (center of joystick)
    pub center: Vec2,
    /// Current finger position
    pub current: Vec2,
    /// Is the joystick currently active?
    pub active: bool,
    /// Max joystick reach (radius)
    pub radius: f32,
}

impl Joystick {
    pub fn new(radius: f32) -> Self {
        Self {
            center: Vec2::ZERO,
            current: Vec2::ZERO,
            active: false,
            radius,
        }
    }

    /// 🖱️ Touch start - Initialize joystick at touch position
    pub fn on_touch_start(&mut self, pos: Vec2) {
        self.center = pos;
        self.current = pos;
        self.active = true;
    }

    /// 🖱️ Touch move - Update joystick position (clamped to radius)
    pub fn on_touch_move(&mut self, pos: Vec2) {
        if !self.active {
            return;
        }

        let mut delta = pos - self.center;

        // Keep joystick inside circle
        if delta.length() > self.radius {
            delta = delta.normalize() * self.radius;
        }

        self.current = self.center + delta;
    }

    /// 🖱️ Touch end - Deactivate joystick
    pub fn on_touch_end(&mut self) {
        self.active = false;
        self.current = self.center;
    }

    /// 🎮 Get movement vector from -1.0 to 1.0 on both axes
    pub fn get_input(&self) -> Vec2 {
        if !self.active {
            return Vec2::ZERO;
        }

        (self.current - self.center) / self.radius
    }
}
