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

// Bullet projectile
struct Bullet {
    pos: Vec2,
    velocity: Vec2,
    life: f32,
}

// Enemy rocket
struct Enemy {
    pos: Vec2,
    velocity: Vec2,
    rotation: f32,
    health: i32,
    size: f32,
}

// Explosion effect
struct Explosion {
    pos: Vec2,
    life: f32,
    size: f32,
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
    
    // Combat
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    explosions: Vec<Explosion>,
    shoot_cooldown: f32,
    enemy_spawn_timer: f32,
    
    // Game state
    health: i32,
    score: i32,
    time: f32,
    intro_alpha: f32,
    game_started: bool,
    safe_time: f32,
    game_over: bool,
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
            bullets: Vec::new(),
            enemies: Vec::new(),
            explosions: Vec::new(),
            shoot_cooldown: 0.0,
            enemy_spawn_timer: 0.0,
            health: 3,
            score: 0,
            time: 0.0,
            intro_alpha: 1.0,
            game_started: false,
            safe_time: 3.0,
            game_over: false,
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

        // Shooting mechanic - auto-fire when aiming
        self.shoot_cooldown -= dt;
        if self.right_joystick.active && self.shoot_cooldown <= 0.0 {
            self.shoot();
            self.shoot_cooldown = 0.15; // Fire rate
        }

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
        
        // Update bullets
        self.bullets.retain_mut(|bullet| {
            bullet.pos += bullet.velocity * dt;
            bullet.life -= dt;
            
            // Remove bullets off screen or expired
            bullet.life > 0.0 
                && bullet.pos.x > 0.0 && bullet.pos.x < screen_width()
                && bullet.pos.y > 0.0 && bullet.pos.y < screen_height()
        });
        
        // Spawn enemies
        if self.safe_time <= 0.0 {
            self.enemy_spawn_timer -= dt;
            if self.enemy_spawn_timer <= 0.0 {
                self.spawn_enemy();
                self.enemy_spawn_timer = rand::gen_range(1.0, 2.5); // Spawn every 1-2.5 seconds
            }
        }
        
        // Update enemies - they chase the player!
        for enemy in &mut self.enemies {
            let player_pos = to_mac_vec2(self.player.position);
            let to_player = player_pos - enemy.pos;
            let distance = to_player.length();
            
            if distance > 0.0 {
                // Chase player
                let direction = to_player / distance;
                enemy.velocity = direction * 150.0; // Enemy speed
                enemy.rotation = direction.y.atan2(direction.x);
            }
            
            enemy.pos += enemy.velocity * dt;
            
            // Wrap enemies around screen
            if enemy.pos.x < -50.0 { enemy.pos.x = screen_width() + 50.0; }
            if enemy.pos.x > screen_width() + 50.0 { enemy.pos.x = -50.0; }
            if enemy.pos.y < -50.0 { enemy.pos.y = screen_height() + 50.0; }
            if enemy.pos.y > screen_height() + 50.0 { enemy.pos.y = -50.0; }
        }
        
        // Check bullet vs enemy collisions
        let mut enemies_to_remove = Vec::new();
        for (i, enemy) in self.enemies.iter_mut().enumerate() {
            for bullet in &mut self.bullets {
                let dist = (bullet.pos - enemy.pos).length();
                if dist < enemy.size + 10.0 {
                    enemy.health -= 1;
                    bullet.life = 0.0; // Remove bullet
                    
                    if enemy.health <= 0 {
                        enemies_to_remove.push(i);
                        self.score += 100;
                        self.explosions.push(Explosion {
                            pos: enemy.pos,
                            life: 0.5,
                            size: enemy.size * 2.0,
                        });
                    }
                }
            }
        }
        
        // Remove dead enemies
        for &i in enemies_to_remove.iter().rev() {
            self.enemies.remove(i);
        }
        
        // Check player vs enemy collisions
        if self.safe_time <= 0.0 {
            let player_pos = to_mac_vec2(self.player.position);
            let mut collision_index = None;
            
            for (i, enemy) in self.enemies.iter().enumerate() {
                let dist = (player_pos - enemy.pos).length();
                if dist < 40.0 + enemy.size {
                    collision_index = Some((i, enemy.pos, enemy.size));
                    break;
                }
            }
            
            if let Some((idx, pos, size)) = collision_index {
                self.enemies.remove(idx);
                self.explosions.push(Explosion {
                    pos,
                    life: 0.5,
                    size: size * 2.0,
                });
                self.health -= 1;
                // Flash effect
                self.safe_time = 0.3; // Brief invulnerability
            }
        }
        
        // Update explosions
        self.explosions.retain_mut(|exp| {
            exp.life -= dt * 2.0;
            exp.size += dt * 100.0;
            exp.life > 0.0
        });
        
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
        
        // Game over
        if self.health <= 0 {
            // TODO: Game over screen
        }
    }
    
    fn shoot(&mut self) {
        let player_pos = to_mac_vec2(self.player.position);
        let rotation = self.player.rotation;
        
        // Bullet starts from front of ship
        let bullet_start = Vec2::new(
            player_pos.x + rotation.cos() * 45.0,
            player_pos.y + rotation.sin() * 45.0,
        );
        
        // Bullet velocity
        let bullet_velocity = Vec2::new(
            rotation.cos() * 600.0,
            rotation.sin() * 600.0,
        );
        
        self.bullets.push(Bullet {
            pos: bullet_start,
            velocity: bullet_velocity,
            life: 2.0,
        });
    }
    
    fn spawn_enemy(&mut self) {
        let screen_width = screen_width();
        let screen_height = screen_height();
        
        // Spawn from edges
        let side = rand::gen_range(0, 4);
        let pos = match side {
            0 => Vec2::new(rand::gen_range(0.0, screen_width), -50.0), // Top
            1 => Vec2::new(rand::gen_range(0.0, screen_width), screen_height + 50.0), // Bottom
            2 => Vec2::new(-50.0, rand::gen_range(0.0, screen_height)), // Left
            _ => Vec2::new(screen_width + 50.0, rand::gen_range(0.0, screen_height)), // Right
        };
        
        self.enemies.push(Enemy {
            pos,
            velocity: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            health: 2,
            size: 25.0,
        });
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
        for seg in self.trail.iter() {
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
        
        // Draw bullets
        for bullet in &self.bullets {
            // Bullet glow
            draw_circle(bullet.pos.x, bullet.pos.y, 8.0, Color::from_rgba(100, 255, 200, 150));
            draw_circle(bullet.pos.x, bullet.pos.y, 5.0, Color::from_rgba(150, 255, 220, 255));
            draw_circle(bullet.pos.x, bullet.pos.y, 2.0, Color::from_rgba(255, 255, 255, 255));
        }
        
        // Draw enemies
        for enemy in &self.enemies {
            self.draw_enemy(enemy);
        }
        
        // Draw explosions
        for explosion in &self.explosions {
            let alpha = (explosion.life * 255.0) as u8;
            draw_circle(
                explosion.pos.x,
                explosion.pos.y,
                explosion.size,
                Color::from_rgba(255, 150, 50, alpha / 2),
            );
            draw_circle(
                explosion.pos.x,
                explosion.pos.y,
                explosion.size * 0.7,
                Color::from_rgba(255, 200, 100, alpha),
            );
            draw_circle(
                explosion.pos.x,
                explosion.pos.y,
                explosion.size * 0.4,
                Color::from_rgba(255, 255, 200, alpha),
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
    
    fn draw_enemy(&self, enemy: &Enemy) {
        let pos = enemy.pos;
        let rotation = enemy.rotation;
        let size = enemy.size;
        
        // Enemy rocket - menacing red design
        let front = Vec2::new(
            pos.x + rotation.cos() * size,
            pos.y + rotation.sin() * size,
        );
        
        let left = Vec2::new(
            pos.x + (rotation + 2.5).cos() * size * 0.6,
            pos.y + (rotation + 2.5).sin() * size * 0.6,
        );
        
        let right = Vec2::new(
            pos.x + (rotation - 2.5).cos() * size * 0.6,
            pos.y + (rotation - 2.5).sin() * size * 0.6,
        );
        
        // Red glow
        draw_circle(pos.x, pos.y, size + 15.0, Color::from_rgba(255, 50, 50, 40));
        draw_circle(pos.x, pos.y, size + 8.0, Color::from_rgba(255, 80, 80, 80));
        
        // Body
        draw_triangle(front, left, right, Color::from_rgba(255, 80, 80, 255));
        draw_triangle_lines(front, left, right, 2.0, Color::from_rgba(255, 150, 150, 255));
        
        // Core
        draw_circle(pos.x, pos.y, 4.0, Color::from_rgba(255, 200, 200, 255));
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

        // Flash if recently hit
        let flash = if self.safe_time > 0.0 && self.safe_time < 0.3 {
            ((self.safe_time * 30.0).sin() * 127.0 + 128.0) as u8
        } else {
            255
        };

        // Player size - BIGGER and more imposing
        let size = 40.0;
        
        // Calculate ship points - sleeker design
        let front = Vec2::new(
            pos.x + rotation.cos() * size,
            pos.y + rotation.sin() * size,
        );
        
        // Main body wings
        let left_wing = Vec2::new(
            pos.x + (rotation + 2.3).cos() * size * 0.7,
            pos.y + (rotation + 2.3).sin() * size * 0.7,
        );
        let right_wing = Vec2::new(
            pos.x + (rotation - 2.3).cos() * size * 0.7,
            pos.y + (rotation - 2.3).sin() * size * 0.7,
        );
        
        // Back engine points
        let back_left = Vec2::new(
            pos.x + (rotation + 2.8).cos() * size * 0.5,
            pos.y + (rotation + 2.8).sin() * size * 0.5,
        );
        let back_right = Vec2::new(
            pos.x + (rotation - 2.8).cos() * size * 0.5,
            pos.y + (rotation - 2.8).sin() * size * 0.5,
        );
        let back_center = Vec2::new(
            pos.x - rotation.cos() * size * 0.4,
            pos.y - rotation.sin() * size * 0.4,
        );

        // Massive outer glow - makes it feel powerful
        draw_circle(pos.x, pos.y, size + 30.0, Color::from_rgba(80, 180, 255, 20));
        draw_circle(pos.x, pos.y, size + 20.0, Color::from_rgba(100, 200, 255, 40));
        draw_circle(pos.x, pos.y, size + 10.0, Color::from_rgba(120, 220, 255, 70));
        
        // Engine flames FIRST (so they're behind ship)
        let movement = self.left_joystick.get_input();
        if movement.x.abs() > 0.1 || movement.y.abs() > 0.1 {
            let thrust_power = (movement.x * movement.x + movement.y * movement.y).sqrt();
            let flame_length = thrust_power * 35.0;
            let pulse = (self.time * 15.0).sin() * 0.2 + 0.8;
            
            // Left engine flame
            let flame_left = Vec2::new(
                back_left.x - rotation.cos() * 10.0,
                back_left.y - rotation.sin() * 10.0,
            );
            self.draw_engine_flame(flame_left, rotation, flame_length * pulse, thrust_power);
            
            // Right engine flame
            let flame_right = Vec2::new(
                back_right.x - rotation.cos() * 10.0,
                back_right.y - rotation.sin() * 10.0,
            );
            self.draw_engine_flame(flame_right, rotation, flame_length * pulse * 0.95, thrust_power);
            
            // Center engine flame (main thrust)
            let flame_center = Vec2::new(
                back_center.x - rotation.cos() * 5.0,
                back_center.y - rotation.sin() * 5.0,
            );
            self.draw_engine_flame(flame_center, rotation, flame_length * pulse * 1.2, thrust_power);
        }
        
        // Ship shadow/depth layer (darker)
        draw_triangle(front, left_wing, back_left, Color::from_rgba(60, 120, 180, flash));
        draw_triangle(front, right_wing, back_right, Color::from_rgba(60, 120, 180, flash));
        
        // Main ship body (brighter)
        draw_triangle(front, left_wing, right_wing, Color::from_rgba(140, 210, 255, flash));
        draw_triangle(left_wing, right_wing, back_center, Color::from_rgba(120, 190, 240, flash));
        
        // Cockpit window (glowing)
        let cockpit = Vec2::new(
            pos.x + rotation.cos() * size * 0.5,
            pos.y + rotation.sin() * size * 0.5,
        );
        draw_circle(cockpit.x, cockpit.y, 6.0, Color::from_rgba(100, 220, 255, 180));
        draw_circle(cockpit.x, cockpit.y, 4.0, Color::from_rgba(180, 240, 255, flash));
        
        // Wing edges (sharp glowing lines)
        draw_line(front.x, front.y, left_wing.x, left_wing.y, 3.0, Color::from_rgba(200, 240, 255, flash));
        draw_line(front.x, front.y, right_wing.x, right_wing.y, 3.0, Color::from_rgba(200, 240, 255, flash));
        draw_line(left_wing.x, left_wing.y, back_left.x, back_left.y, 2.0, Color::from_rgba(180, 230, 255, flash));
        draw_line(right_wing.x, right_wing.y, back_right.x, back_right.y, 2.0, Color::from_rgba(180, 230, 255, flash));
        
        // Energy lines on wings (detail)
        let wing_line_left = Vec2::new(
            pos.x + (rotation + 2.0).cos() * size * 0.5,
            pos.y + (rotation + 2.0).sin() * size * 0.5,
        );
        let wing_line_right = Vec2::new(
            pos.x + (rotation - 2.0).cos() * size * 0.5,
            pos.y + (rotation - 2.0).sin() * size * 0.5,
        );
        draw_line(pos.x, pos.y, wing_line_left.x, wing_line_left.y, 2.0, Color::from_rgba(100, 200, 255, 150));
        draw_line(pos.x, pos.y, wing_line_right.x, wing_line_right.y, 2.0, Color::from_rgba(100, 200, 255, 150));
        
        // Nose tip (bright point)
        draw_circle(front.x, front.y, 4.0, Color::from_rgba(255, 255, 255, flash));
        draw_circle(front.x, front.y, 2.0, Color::from_rgba(180, 240, 255, flash));
        
        // Core center glow
        draw_circle(pos.x, pos.y, 5.0, Color::from_rgba(200, 240, 255, 200));
    }
    
    fn draw_engine_flame(&self, pos: Vec2, rotation: f32, length: f32, power: f32) {
        let flame_back = Vec2::new(
            pos.x - rotation.cos() * length,
            pos.y - rotation.sin() * length,
        );
        
        // Outer flame (orange glow)
        let points_outer = 8;
        for i in 0..points_outer {
            let angle = rotation + (i as f32 / points_outer as f32) * std::f32::consts::PI * 0.5 - std::f32::consts::PI * 0.25;
            let flame_point = Vec2::new(
                flame_back.x + angle.cos() * length * 0.4,
                flame_back.y + angle.sin() * length * 0.4,
            );
            draw_triangle(
                pos,
                flame_back,
                flame_point,
                Color::from_rgba(255, 100, 30, (power * 100.0) as u8),
            );
        }
        
        // Middle flame (yellow)
        draw_line(pos.x, pos.y, flame_back.x, flame_back.y, length * 0.3, Color::from_rgba(255, 180, 50, (power * 200.0) as u8));
        
        // Inner flame (bright white core)
        draw_line(pos.x, pos.y, flame_back.x, flame_back.y, length * 0.15, Color::from_rgba(255, 255, 200, (power * 255.0) as u8));
        
        // Flame tip glow
        draw_circle(flame_back.x, flame_back.y, length * 0.2, Color::from_rgba(255, 150, 50, (power * 80.0) as u8));
        draw_circle(flame_back.x, flame_back.y, length * 0.1, Color::from_rgba(255, 220, 100, (power * 150.0) as u8));
    }
    
    fn draw_ui(&self) {
        // Top-left: Hearts
        for i in 0..self.health.max(0) {
            let x = 30.0 + (i as f32 * 40.0);
            let y = 30.0;
            
            // Heart emoji effect (simple circles for now)
            draw_circle(x - 5.0, y, 8.0, Color::from_rgba(255, 100, 120, 255));
            draw_circle(x + 5.0, y, 8.0, Color::from_rgba(255, 100, 120, 255));
            draw_circle(x, y + 8.0, 8.0, Color::from_rgba(255, 100, 120, 255));
        }
        
        // Top-center: Score
        let score_text = format!("SCORE: {}", self.score);
        let font_size = 30.0;
        let score_width = measure_text(&score_text, None, font_size as u16, 1.0).width;
        
        draw_text(
            &score_text,
            (screen_width() - score_width) / 2.0,
            40.0,
            font_size,
            Color::from_rgba(100, 255, 150, 255),
        );
        
        // Top-right: Timer
        let minutes = (self.time / 60.0) as i32;
        let seconds = (self.time % 60.0) as i32;
        let time_text = format!("{:02}:{:02}", minutes, seconds);
        
        let text_width = measure_text(&time_text, None, font_size as u16, 1.0).width;
        
        draw_text(
            &time_text,
            screen_width() - text_width - 30.0,
            40.0,
            font_size,
            Color::from_rgba(200, 220, 255, 200),
        );
        
        // Safe period indicator
        if self.safe_time > 0.0 && self.game_started {
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
        
        // Game instructions hint
        if self.time < 5.0 {
            let hint = "Right joystick to AIM & SHOOT!";
            let hint_width = measure_text(hint, None, 20 as u16, 1.0).width;
            let alpha = ((self.time * 2.0).sin() * 127.0 + 128.0) as u8;
            
            draw_text(
                hint,
                (screen_width() - hint_width) / 2.0,
                screen_height() - 80.0,
                20.0,
                Color::from_rgba(255, 200, 100, alpha),
            );
        }
    }
}
