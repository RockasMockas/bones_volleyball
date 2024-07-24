use super::{gameplay::*, Ball, MatchState};
use crate::input::MatchInputs;
use bones_framework::prelude::*;

/// Represents the local player in the game
#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct LocalPlayer {
    /// The index of the local player (0 or 1)
    pub idx: u32,
}

/// Represents a player in the game
#[derive(HasSchema, Default, Clone)]
#[repr(C)]
pub struct Player {
    /// The current velocity of the player
    pub velocity: Vec2,
    /// Whether the player is touching the ground
    pub is_grounded: bool,
    /// The index of the player (0 or 1)
    pub idx: usize,
}

/// Handles player movement based on input and game state
pub fn player_movement(
    entities: Res<Entities>,
    mut players: CompMut<Player>,
    mut transforms: CompMut<Transform>,
    match_inputs: Res<MatchInputs>,
    match_state: Res<MatchState>,
) {
    if match_state.is_finished() {
        return;
    }

    for (_ent, (player, transform)) in entities.iter_with((&mut players, &mut transforms)) {
        let player_control = match_inputs.get_control(player.idx);

        // Calculate horizontal movement
        let movement = (player_control.right - player_control.left).clamp(-1.0, 1.0);
        let jump = player_control.jump_pressed;

        // Apply gravity
        player.velocity.y -= GRAVITY;

        // Set horizontal velocity
        player.velocity.x = movement * MOVE_SPEED;

        // Handle jumping
        if jump && player.is_grounded {
            player.velocity.y = JUMP_VELOCITY;
            player.is_grounded = false;
        }

        // Update position
        transform.translation.x += player.velocity.x;
        transform.translation.y += player.velocity.y;

        // Determine player boundaries
        let (left_bound, right_bound) = if player.idx == 0 {
            (
                LEFT_BOUNDARY + PLAYER_WIDTH / 2.0,
                CENTER_BOUNDARY - PLAYER_WIDTH / 2.0 - NET_WIDTH,
            )
        } else {
            (
                CENTER_BOUNDARY + NET_WIDTH + PLAYER_WIDTH / 2.0,
                RIGHT_BOUNDARY - PLAYER_WIDTH / 2.0,
            )
        };

        // Clamp player position within boundaries
        transform.translation.x = transform.translation.x.clamp(left_bound, right_bound);

        // Handle ground collision
        if transform.translation.y <= GROUND_LEVEL {
            transform.translation.y = GROUND_LEVEL;
            player.velocity.y = 0.0;
            player.is_grounded = true;
        } else {
            player.is_grounded = false;
        }
    }
}

/// Handles collisions between the ball and players
pub fn ball_player_collision(
    entities: Res<Entities>,
    mut balls: CompMut<Ball>,
    mut transforms: CompMut<Transform>,
    players: Comp<Player>,
    match_state: Res<MatchState>,
) {
    if match_state.is_finished() {
        return;
    }

    let mut ball_updates = Vec::new();
    for (ball_ent, (_ball, ball_transform)) in entities.iter_with((&balls, &transforms)) {
        let ball_center = Vec2::new(ball_transform.translation.x, ball_transform.translation.y);
        for (_player_ent, (player, player_transform)) in entities.iter_with((&players, &transforms))
        {
            let player_center = Vec2::new(
                player_transform.translation.x,
                player_transform.translation.y + PLAYER_HEIGHT / 2.0,
            );
            let rel_x = ball_center.x - player_center.x;
            let rel_y = ball_center.y - player_center.y;

            // Check for collision
            if rel_x.abs() < PLAYER_WIDTH / 2.0 + BALL_RADIUS
                && rel_y.abs() < PLAYER_HEIGHT / 2.0 + BALL_RADIUS
            {
                // Calculate relative position on the player
                let mut relative_x_pos = rel_x / (PLAYER_WIDTH / 2.0);
                if player.idx == 1 {
                    relative_x_pos = -relative_x_pos;
                }

                // Calculate bounce angle
                let max_angle = std::f32::consts::FRAC_PI_4;
                let bounce_angle = relative_x_pos * max_angle;

                // Calculate new velocity
                let speed = MAX_BALL_SPEED * PLAYER_BOUNCE_FACTOR;
                let mut new_velocity =
                    Vec2::new(bounce_angle.sin() * speed, bounce_angle.cos() * speed);

                if player.idx == 1 {
                    new_velocity.x = -new_velocity.x;
                }

                // Add player's velocity to the ball
                let final_velocity = new_velocity + player.velocity * 0.5;

                // Calculate new position
                let new_position = Vec2::new(
                    player_center.x + rel_x.signum() * (PLAYER_WIDTH / 2.0 + BALL_RADIUS + 1.0),
                    player_center.y + PLAYER_HEIGHT / 2.0 + BALL_RADIUS + 1.0,
                );

                ball_updates.push((ball_ent, final_velocity, new_position));
                break;
            }
        }
    }

    // Apply updates to balls
    for (ball_ent, new_velocity, new_position) in ball_updates {
        if let (Some(ball), Some(ball_transform)) =
            (balls.get_mut(ball_ent), transforms.get_mut(ball_ent))
        {
            ball.velocity = new_velocity;
            ball_transform.translation.y = new_position.y;

            // Clamp ball speed
            let speed = ball.velocity.length();
            if speed > MAX_BALL_SPEED {
                ball.velocity = ball.velocity.normalize() * MAX_BALL_SPEED;
            }
        }
    }
}
