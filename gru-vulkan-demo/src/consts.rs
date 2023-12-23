/* bg.vert
	- RADIUS = 2 * CAVE_RADIUS
	- BACK_DISTANCE = BLOCK_LENGTH * (BLOCK_DESPAWN_BACK_DISTANCE + 1)
	- FRONT_DISTANCE = BLOCK_LENGTH * (BLOCK_SPAWN_FRONT_DISTANCE + 1)
*/

// from here
pub const CAVE_RADIUS: f32 = 15.0;
pub const BLOCK_LENGTH: f32 = 30.0;
pub const BLOCK_SPAWN_FRONT_DISTANCE: i32 = 10;
pub const BLOCK_DESPAWN_BACK_DISTANCE: i32 = 3;
// to here relevant for bg.vert

pub const CAVE_RESOLUTION: u32 = 80;
pub const CAVE_GEN_OCTAVES: usize = 2;
pub const CAVE_GEN_FREQUENCY: f64 = 0.07;
pub const CAVE_GEN_LUCUNARITY: f64 = 1.7;
pub const CAVE_GEN_PERSISTANCE: f64 = 0.8;
pub const CAVE_GEN_BIAS: f32 = -0.1;
pub const CAVE_GEN_BORDER_POWER: i32 = 4;
pub const CAVE_GEN_BORDER_STRENGTH: f32 = 4.0;
pub const CAVE_GEN_SPAWN_STRENGTH: f32 = 4.0;
pub const CAVE_GEN_SPAWN_DECAY_RATE: f32 = -3.0;
pub const CAVE_GEN_GRADIENT_EPSILON: f32 = 0.05;
pub const CAVE_GEN_GRADIENT_EPSILON_2: f32 = 2.0 * CAVE_GEN_GRADIENT_EPSILON;

pub const CAM_NEAR: f32 = 0.01;
pub const CAM_FAR: f32 = BLOCK_LENGTH * BLOCK_SPAWN_FRONT_DISTANCE as f32 * 1.5; //needs to be larger than bg.vert::FRONT_DISTANCE
pub const CAM_ANGLE: f32 = std::f32::consts::FRAC_PI_4;
pub const MOUSE_SENSITIVITY: f32 = 0.0001;

pub const C: f32 = 5.0;
pub const WAIT_TIME: f32 = 3.0;
pub const Z_BIAS_OFFSET: f32 = 10.0;
pub const MAX_BIAS: f32 = 100.0;

pub const LIGHT_COLOR: (f32, f32, f32) = (24.0, 14.0, 6.0); //(12.0, 7.0, 3.0); TODO revert to this when MSAA x SRGB bug fixed!!!
pub const LIGHT_ANGLE: f32 = -0.2;
pub const LIGHT_ANGLE_INNER: f32 = std::f32::consts::TAU / 360.0 * 10.0;
pub const LIGHT_ANGLE_OUTER: f32 = std::f32::consts::TAU / 360.0 * 15.0;
pub const LIGHT_FREQUENCY: f64 = 0.6;
pub const LIGHT_BIAS: f64 = 0.5;
pub const AMBIENT_LIGHT_COLOR: (f32, f32, f32) = (0.16, 0.16, 0.16); //(0.08, 0.08, 0.08); TODO revert to this when MSAA x SRGB bug fixed!!!

pub const GRAV: f32 = 10.0;
pub const COLLISION_FORCE: f32 = 400.0;
pub const COLLISION_DRAG: f32 = 0.005;
pub const SURFACE_DRAG: f32 = 0.08;
pub const AIR_DRAG: f32 = 0.95;
pub const ACCELERATION: f32 = 20.0;
pub const MIN_ACCELERATION: f32 = 6.0;
pub const JUMP: f32 = 5.5;
pub const JUMP_COOLDOWN: f32 = 1.0;
pub const CONTROL_TIME: f32 = 0.2;

pub const FIGUR_HEIGHT: f32 = 0.9;
pub const FIGUR_WIDTH: f32 = 0.25 * FIGUR_HEIGHT;
pub const EYE_HEIGHT: f32 = 7.0 / 8.0 * FIGUR_HEIGHT;

pub const FLASH_POWER: f32 = 5.0;
pub const FLASH_AMBIENT_POWER: f32 = 1.5;
pub const FLASH_AMBIENT_DECAY: f32 = 0.05;
pub const FLASH_RADIUS: f32 = 0.1;
pub const FLASH_RADIUS_SQ: f32 = FLASH_RADIUS * FLASH_RADIUS;
pub const FLASH_HEIGHT: f32 = 0.3;
pub const FLASH_BLOCK_PROB: f32 = 0.2;
pub const FLASH_EPS: f32 = 0.05;
pub const FLASH_RESOLUTION: u32 = 100;
pub const PICKUP_RANGE: f32 = 3.0;

pub const SCORE_DIGITS: usize = 3; //needs to be changed in main.rs under "compute score"
