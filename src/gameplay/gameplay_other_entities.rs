use super::{gameplay::*, MatchState};
use bones_framework::prelude::*;

/// Represents the ball in the game
#[derive(HasSchema, Default, Clone)]
#[repr(C)]
pub struct Ball {
    pub velocity: Vec2,
}

/// Represents the floor in the game
#[derive(HasSchema, Default, Clone)]
#[repr(C)]
pub struct Floor;

/// Represents the net in the game
#[derive(HasSchema, Default, Clone)]
#[repr(C)]
pub struct Net;

/// Creates a Path2d that will visualize the ball
pub fn create_circle_path(radius: f32, color: Color) -> Path2d {
    let num_segments = 32;
    let mut points = Vec::with_capacity(num_segments + 1);

    for i in 0..=num_segments {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (num_segments as f32);
        let x = radius * angle.cos();
        let y = radius * angle.sin();
        points.push(Vec2::new(x, y));
    }

    Path2d {
        color,
        points,
        thickness: 2.0,
        line_breaks: vec![],
    }
}

/// Handles ball movement and collision with boundaries
pub fn ball_movement(
    entities: Res<Entities>,
    mut balls: CompMut<Ball>,
    mut transforms: CompMut<Transform>,
    mut match_state: ResMut<MatchState>,
) {
    if match_state.is_finished() {
        return;
    }

    for (_ent, (ball, transform)) in entities.iter_with((&mut balls, &mut transforms)) {
        // Apply gravity
        ball.velocity.y -= GRAVITY;

        // Update position
        transform.translation.x += ball.velocity.x;
        transform.translation.y += ball.velocity.y;

        // Handle horizontal boundary collisions
        if transform.translation.x - BALL_RADIUS <= LEFT_BOUNDARY
            || transform.translation.x + BALL_RADIUS >= RIGHT_BOUNDARY
        {
            ball.velocity.x = -ball.velocity.x * BALL_BOUNCE_FACTOR;
            transform.translation.x = transform
                .translation
                .x
                .clamp(LEFT_BOUNDARY + BALL_RADIUS, RIGHT_BOUNDARY - BALL_RADIUS);
        }

        // Handle ceiling collision
        if transform.translation.y + BALL_RADIUS >= 290.0 {
            ball.velocity.y = -ball.velocity.y * BALL_BOUNCE_FACTOR;
            transform.translation.y = 290.0 - BALL_RADIUS;
        }

        // Handle floor collision and scoring
        if transform.translation.y + BALL_RADIUS <= GROUND_LEVEL {
            let reset_to_right = transform.translation.x > CENTER_BOUNDARY;
            let scoring_player = if reset_to_right { 0 } else { 1 };
            match_state.increment_player_score(scoring_player);
            ball.reset(reset_to_right, transform);
        }

        // Clamp ball speed
        let speed = ball.velocity.length();
        if speed > MAX_BALL_SPEED {
            ball.velocity = ball.velocity.normalize() * MAX_BALL_SPEED;
        }
    }
}

impl Ball {
    /// Resets the ball's position and velocity
    pub fn reset(&mut self, reset_to_right: bool, transform: &mut Transform) {
        transform.translation.x = if reset_to_right { 290.0 } else { -290.0 };
        transform.translation.y = 0.0;
        self.velocity = Vec2::new(0.0, GRAVITY * 30.0);
    }
}

/// Handles ball collision with the net
pub fn ball_net_collision(
    entities: Res<Entities>,
    mut balls: CompMut<Ball>,
    mut transforms: CompMut<Transform>,
    nets: Comp<Net>,
    match_state: Res<MatchState>,
) {
    if match_state.is_finished() {
        return;
    }

    let mut ball_updates = Vec::new();
    // Find the net position (assuming there's only one net)
    let mut net_position = Vec3::ZERO;
    for (_net_ent, (_net, net_transform)) in entities.iter_with((&nets, &transforms)) {
        net_position = net_transform.translation;
        break; // We only need one net
    }

    for (ball_ent, (ball, ball_transform)) in entities.iter_with((&balls, &transforms)) {
        let ball_center = Vec2::new(ball_transform.translation.x, ball_transform.translation.y);

        // Check for collision with the net
        if (ball_center.x - net_position.x).abs() < NET_WIDTH / 2.0 + BALL_RADIUS
            && ball_center.y > net_position.y
            && ball_center.y < net_position.y + NET_HEIGHT + BALL_RADIUS
        {
            let mut new_velocity = ball.velocity;
            let mut new_position = ball_center;

            // Check if it's a top collision
            if ball_center.y >= net_position.y + NET_HEIGHT - BALL_RADIUS {
                // Top collision: Reverse y velocity and maintain x velocity
                new_velocity.y = -new_velocity.y * BALL_BOUNCE_FACTOR;
                new_position.y = net_position.y + NET_HEIGHT + BALL_RADIUS;
            } else {
                // Side collision: Reverse x velocity
                new_velocity.x = -new_velocity.x * BALL_BOUNCE_FACTOR;

                // Adjust x position to prevent sticking
                new_position.x = if ball_center.x < net_position.x {
                    net_position.x - NET_WIDTH / 2.0 - BALL_RADIUS - 1.0
                } else {
                    net_position.x + NET_WIDTH / 2.0 + BALL_RADIUS + 1.0
                };
            }

            ball_updates.push((ball_ent, new_velocity, new_position));
        }
    }

    // Apply updates
    for (ball_ent, new_velocity, new_position) in ball_updates {
        if let (Some(ball), Some(ball_transform)) =
            (balls.get_mut(ball_ent), transforms.get_mut(ball_ent))
        {
            ball.velocity = new_velocity;
            ball_transform.translation.x = new_position.x;
            ball_transform.translation.y = new_position.y;
        }
    }
}

/// Updates the visibility of the ball based on the match state, used for hiding the ball when match finished.
pub fn update_ball_visibility(
    entities: Res<Entities>,
    mut paths: CompMut<Path2d>,
    balls: Comp<Ball>,
    match_state: Res<MatchState>,
) {
    for (_ent, (_ball, path)) in entities.iter_with((&balls, &mut paths)) {
        let current_color = path.color;
        path.color = if match_state.is_finished() {
            Color::rgba(current_color.r(), current_color.g(), current_color.b(), 0.0)
        } else {
            Color::rgba(current_color.r(), current_color.g(), current_color.b(), 1.0)
        };
    }
}
