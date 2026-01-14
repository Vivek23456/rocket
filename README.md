# 🕹️ Rust Joystick Game

A simple joystick/cursor control system for touch-based games in Rust.

## Features

- ✅ Dual joystick support (movement + aim)
- ✅ Touch start/move/end event handlers
- ✅ Automatic joystick clamping (stays within radius)
- ✅ Normalized input (-1.0 to 1.0 range)
- ✅ Player physics with velocity and rotation
- ✅ No external dependencies (pure Rust)

## Structure

```
src/
├── joystick.rs  - Vec2 math and Joystick implementation
├── player.rs    - Player physics
└── main.rs      - Demo with simulated touch input
```

## How It Works

### Joystick

Each joystick tracks:
- **center**: Where the finger started
- **current**: Current finger position
- **active**: Whether currently being touched
- **radius**: Maximum reach distance

### Touch Handlers

- **on_touch_start(pos)**: Initialize joystick at touch position
- **on_touch_move(pos)**: Update position (clamped to radius)
- **on_touch_end()**: Deactivate joystick

### Get Input

```rust
let movement = left_joystick.get_input();  // Returns Vec2 from -1.0 to 1.0
let aim = right_joystick.get_input();

player.velocity += movement * 400.0 * dt;
player.rotation = aim.y.atan2(aim.x);
```

## Run the Demo

```bash
cargo run
```

This will simulate touch input and show player movement and rotation based on dual joystick control.

## Usage in Your Game

1. Create joysticks:
```rust
let mut left_joystick = Joystick::new(100.0);
let mut right_joystick = Joystick::new(100.0);
```

2. Hook up to your touch events:
```rust
// On touch start
left_joystick.on_touch_start(touch_position);

// On touch move
left_joystick.on_touch_move(touch_position);

// On touch end
left_joystick.on_touch_end();
```

3. Use in your game loop:
```rust
let movement = left_joystick.get_input();
let aim = right_joystick.get_input();

player.update(movement, aim, delta_time);
```

## License

Free to use!
