use macroquad::prelude::*;
use crate::joystick::{Joystick, Vec2 as JoyVec2};
use crate::player::Player;

// Helper to convert between our Vec2 and Macroquad's Vec2
fn to_mac_vec2(v: JoyVec2) -> Vec2 {
    Vec2::new(v.x, v.y)
}

fn from_mac_vec2(v: Vec2) -> JoyVec2 {
    JoyVec2::new(v.x, v.y)
}

// Particle for background atmosphere
struct Particle {
    pos: Vec2,
    velocity: Vec2,
    size: f32,
    alpha: f32,
}

// Trail segment for player movement
struct TrailSegment {
    pos: Vec2,
    life: f32,
    size: f32,
}

// Obstacle in the world
struct Obstacle {
    pos: Vec2,
    size: f32,
    glow_phase: f32,
}

pub struct GameState {
    left_joystick: Joystick,
    right_joystick: Joystick,
    player: Player,
    left_touch_id: Option<u64>,
    right_touch_id: Option<u64>,
    
    // Visual enhancements
    particles: Vec<Particle>,
    trail: Vec<TrailSegment>,
    obstacles: Vec<Obstacle>,
    
    // Game state
    health: i32,
    time: f32,
    intro_alpha: f32,
    game_started: bool,
    safe_time: f32,
}

impl GameState {
    pub fn new() -> Self {
        let screen_width = screen_width();
        let screen_height = screen_height();
        
        // Create atmospheric particles
        let mut particles = Vec::new();
        for _ in 0..150 {
            particles.push(Particle {
                pos: Vec2::new(
                    rand::gen_range(0.0, screen_width),
                    rand::gen_range(0.0, screen_height),
                ),
                velocity: Vec2::new(
                    rand::gen_range(-15.0, 15.0),
                    rand::gen_range(-15.0, 15.0),
                ),
                size: rand::gen_range(1.0, 3.0),
                alpha: rand::gen_range(0.1, 0.4),
            });
        }
        
        // Create some obstacles
        let mut obstacles = Vec::new();
        for i in 0..5 {
            obstacles.push(Obstacle {
                pos: Vec2::new(
                    screen_width * 0.3 + (i as f32 * 150.0),
                    screen_height * 0.5 + (i as f32 * 50.0).sin() * 100.0,
                ),
                size: 40.0,
                glow_phase: rand::gen_range(0.0, 6.28),
            });
        }
        
        Self {
            left_joystick: Joystick::new(80.0),
            right_joystick: Joystick::new(80.0),
            player: Player::new(JoyVec2::new(screen_width / 2.0, screen_height / 2.0)),
            left_touch_id: None,
            right_touch_id: None,
            particles,
            trail: Vec::new(),
            obstacles,
            health: 3,
            time: 0.0,
            intro_alpha: 1.0,
            game_started: false,
            safe_time: 3.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        
        // Fade in intro
        if self.intro_alpha > 0.0 {
            self.intro_alpha -= dt * 0.5;
            if self.intro_alpha < 0.0 {
                self.intro_alpha = 0.0;
                self.game_started = true;
            }
        }
        
        // Safe period countdown
        if self.safe_time > 0.0 {
            self.safe_time -= dt;
        }
        
        self.handle_input();

        // Get input from joysticks
        let movement = self.left_joystick.get_input();
        let aim = self.right_joystick.get_input();

        // Update player
        self.player.update(movement, aim, dt);

        // Add trail segment
        let player_pos = to_mac_vec2(self.player.position);
        if movement.x.abs() > 0.1 || movement.y.abs() > 0.1 {
            self.trail.push(TrailSegment {
                pos: player_pos,
                life: 1.0,
                size: 30.0,
            });
        }
        
        // Update trail
        self.trail.retain_mut(|seg| {
            seg.life -= dt * 2.0;
            seg.size -= dt * 40.0;
            seg.life > 0.0
        });
        
        // Keep only last 20 trail segments
        if self.trail.len() > 20 {
            self.trail.drain(0..self.trail.len() - 20);
        }

        // Wrap player around screen edges
        let mut pos = self.player.position;
        if pos.x < 0.0 { pos.x = screen_width(); }
        if pos.x > screen_width() { pos.x = 0.0; }
        if pos.y < 0.0 { pos.y = screen_height(); }
        if pos.y > screen_height() { pos.y = 0.0; }
        self.player.position = pos;
        
        // Update particles (breathing world)
        for particle in &mut self.particles {
            particle.pos += particle.velocity * dt;
            
            // Wrap particles
            if particle.pos.x < 0.0 { particle.pos.x = screen_width(); }
            if particle.pos.x > screen_width() { particle.pos.x = 0.0; }
            if particle.pos.y < 0.0 { particle.pos.y = screen_height(); }
            if particle.pos.y > screen_height() { particle.pos.y = 0.0; }
            
            // Gentle pulse
            particle.alpha = 0.2 + (self.time * 2.0 + particle.pos.x * 0.01).sin() * 0.1;
        }
        
        // Update obstacles
        for obstacle in &mut self.obstacles {
            obstacle.glow_phase += dt * 2.0;
        }
        
        // Check collision with obstacles (only after safe time)
        if self.safe_time <= 0.0 {
            let player_pos = to_mac_vec2(self.player.position);
            for obstacle in &self.obstacles {
                let dist = (player_pos - obstacle.pos).length();
                if dist < obstacle.size + 20.0 {
                    // TODO: Handle collision (flash, lose health, etc.)
                }
            }
        }
    }

    fn handle_input(&mut self) {
        let touches = touches();
        let screen_width = screen_width();
        let left_side_x = screen_width / 2.0;

        // Handle each touch
        for touch in &touches {
            let pos = from_mac_vec2(Vec2::new(touch.position.x, touch.position.y));

            match touch.phase {
                TouchPhase::Started => {
                    // Left side = movement joystick
                    if touch.position.x < left_side_x && self.left_touch_id.is_none() {
                        self.left_joystick.on_touch_start(pos);
                        self.left_touch_id = Some(touch.id);
                    }
                    // Right side = aim joystick
                    else if touch.position.x >= left_side_x && self.right_touch_id.is_none() {
                        self.right_joystick.on_touch_start(pos);
                        self.right_touch_id = Some(touch.id);
                    }
                }
                TouchPhase::Moved => {
                    if Some(touch.id) == self.left_touch_id {
                        self.left_joystick.on_touch_move(pos);
                    } else if Some(touch.id) == self.right_touch_id {
                        self.right_joystick.on_touch_move(pos);
                    }
                }
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if Some(touch.id) == self.left_touch_id {
                        self.left_joystick.on_touch_end();
                        self.left_touch_id = None;
                    } else if Some(touch.id) == self.right_touch_id {
                        self.right_joystick.on_touch_end();
                        self.right_touch_id = None;
                    }
                }
                _ => {}
            }
        }

        // Fallback to mouse for desktop
        if touches.is_empty() {
            let mouse_pos = mouse_position();
            let pos = from_mac_vec2(Vec2::new(mouse_pos.0, mouse_pos.1));

            if is_mouse_button_pressed(MouseButton::Left) {
                if mouse_pos.0 < left_side_x {
                    self.left_joystick.on_touch_start(pos);
                } else {
                    self.right_joystick.on_touch_start(pos);
                }
            } else if is_mouse_button_down(MouseButton::Left) {
                if mouse_pos.0 < left_side_x && self.left_joystick.active {
                    self.left_joystick.on_touch_move(pos);
                } else if mouse_pos.0 >= left_side_x && self.right_joystick.active {
                    self.right_joystick.on_touch_move(pos);
                }
            } else if is_mouse_button_released(MouseButton::Left) {
                self.left_joystick.on_touch_end();
                self.right_joystick.on_touch_end();
            }
        }
    }

    pub fn draw(&self) {
        // Deep space background
        clear_background(Color::from_rgba(5, 5, 15, 255));

        // Draw breathing particles
        for particle in &self.particles {
            draw_circle(
                particle.pos.x,
                particle.pos.y,
                particle.size,
                Color::from_rgba(100, 120, 200, (particle.alpha * 255.0) as u8),
            );
        }
        
        // Draw obstacles with glow
        for obstacle in &self.obstacles {
            let glow = (obstacle.glow_phase.sin() * 0.3 + 0.7).max(0.3);
            
            // Outer glow
            draw_circle(
                obstacle.pos.x,
                obstacle.pos.y,
                obstacle.size + 15.0,
                Color::from_rgba(80, 40, 120, (glow * 40.0) as u8),
            );
            
            // Main circle
            draw_circle(
                obstacle.pos.x,
                obstacle.pos.y,
                obstacle.size,
                Color::from_rgba(140, 80, 200, (glow * 180.0) as u8),
            );
            
            // Core
            draw_circle(
                obstacle.pos.x,
                obstacle.pos.y,
                obstacle.size * 0.4,
                Color::from_rgba(200, 150, 255, (glow * 255.0) as u8),
            );
        }

        // Draw player trail
        for (i, seg) in self.trail.iter().enumerate() {
            let alpha = (seg.life * 100.0) as u8;
            let size = seg.size * seg.life;
            
            // Outer glow
            draw_circle(
                seg.pos.x,
                seg.pos.y,
                size,
                Color::from_rgba(100, 200, 255, alpha / 3),
            );
            
            // Inner glow
            draw_circle(
                seg.pos.x,
                seg.pos.y,
                size * 0.6,
                Color::from_rgba(150, 220, 255, alpha / 2),
            );
        }

        // Draw enhanced player
        self.draw_player();

        // Draw minimal joysticks (only when active, very transparent)
        if self.left_joystick.active {
            self.draw_minimal_joystick(&self.left_joystick, Color::from_rgba(100, 200, 255, 60));
        }
        if self.right_joystick.active {
            self.draw_minimal_joystick(&self.right_joystick, Color::from_rgba(255, 100, 100, 60));
        }

        // Minimal UI - top corners only
        self.draw_ui();
        
        // Intro fade
        if self.intro_alpha > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                screen_width(),
                screen_height(),
                Color::from_rgba(5, 5, 15, (self.intro_alpha * 255.0) as u8),
            );
        }
    }

    fn draw_minimal_joystick(&self, joystick: &Joystick, color: Color) {
        let center = to_mac_vec2(joystick.center);
        let current = to_mac_vec2(joystick.current);

        // Very subtle outer ring
        draw_circle_lines(center.x, center.y, joystick.radius, 1.0, color);

        // Thumb indicator
        draw_circle(current.x, current.y, 20.0, color);
        draw_circle(
            current.x,
            current.y,
            15.0,
            Color::from_rgba(255, 255, 255, 100),
        );
    }

    fn draw_player(&self) {
        let pos = to_mac_vec2(self.player.position);
        let rotation = self.player.rotation;

        // Player size - BIGGER
        let size = 35.0;
        
        // Calculate ship points
        let front = Vec2::new(
            pos.x + rotation.cos() * size,
            pos.y + rotation.sin() * size,
        );
        let back_left = Vec2::new(
            pos.x + (rotation + 2.5).cos() * size * 0.6,
            pos.y + (rotation + 2.5).sin() * size * 0.6,
        );
        let back_right = Vec2::new(
            pos.x + (rotation - 2.5).cos() * size * 0.6,
            pos.y + (rotation - 2.5).sin() * size * 0.6,
        );

        // Outer glow
        draw_circle(pos.x, pos.y, size + 20.0, Color::from_rgba(100, 200, 255, 30));
        draw_circle(pos.x, pos.y, size + 10.0, Color::from_rgba(120, 220, 255, 60));
        
        // Ship body with glow
        draw_triangle(front, back_left, back_right, Color::from_rgba(150, 220, 255, 255));
        draw_triangle_lines(front, back_left, back_right, 3.0, Color::from_rgba(200, 240, 255, 255));

        // Engine glow
        let movement = self.left_joystick.get_input();
        if movement.x.abs() > 0.1 || movement.y.abs() > 0.1 {
            let thrust_power = (movement.x * movement.x + movement.y * movement.y).sqrt();
            let flame_pos = Vec2::new(
                pos.x - rotation.cos() * size * 0.5,
                pos.y - rotation.sin() * size * 0.5,
            );
            let flame_size = thrust_power * 25.0;
            
            // Outer flame
            draw_circle(flame_pos.x, flame_pos.y, flame_size, Color::from_rgba(255, 150, 50, 150));
            // Inner flame
            draw_circle(flame_pos.x, flame_pos.y, flame_size * 0.6, Color::from_rgba(255, 200, 100, 200));
            // Core
            draw_circle(flame_pos.x, flame_pos.y, flame_size * 0.3, Color::from_rgba(255, 255, 200, 255));
        }
        
        // Core dot
        draw_circle(pos.x, pos.y, 4.0, Color::from_rgba(255, 255, 255, 255));
    }
    
    fn draw_ui(&self) {
        // Top-left: Hearts
        for i in 0..self.health {
            let x = 30.0 + (i as f32 * 40.0);
            let y = 30.0;
            
            // Heart emoji effect (simple circles for now)
            draw_circle(x - 5.0, y, 8.0, Color::from_rgba(255, 100, 120, 255));
            draw_circle(x + 5.0, y, 8.0, Color::from_rgba(255, 100, 120, 255));
            draw_circle(x, y + 8.0, 8.0, Color::from_rgba(255, 100, 120, 255));
        }
        
        // Top-right: Timer
        let minutes = (self.time / 60.0) as i32;
        let seconds = (self.time % 60.0) as i32;
        let time_text = format!("{:02}:{:02}", minutes, seconds);
        
        let font_size = 30.0;
        let text_width = measure_text(&time_text, None, font_size as u16, 1.0).width;
        
        draw_text(
            &time_text,
            screen_width() - text_width - 30.0,
            40.0,
            font_size,
            Color::from_rgba(200, 220, 255, 200),
        );
        
        // Safe period indicator
        if self.safe_time > 0.0 {
            let safe_text = "Safe Zone";
            let safe_width = measure_text(safe_text, None, 25 as u16, 1.0).width;
            let alpha = ((self.safe_time * 3.0).sin() * 127.0 + 128.0) as u8;
            
            draw_text(
                safe_text,
                (screen_width() - safe_width) / 2.0,
                screen_height() / 2.0 - 100.0,
                25.0,
                Color::from_rgba(100, 255, 150, alpha),
            );
        }
    }
}
