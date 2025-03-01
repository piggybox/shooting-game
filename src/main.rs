use bevy::prelude::*;
use rand::prelude::*;
use std::time::Duration;

// Components
#[derive(Component)]
struct Player {
    speed: f32,
    shoot_timer: Timer,
}

#[derive(Component)]
struct Bullet {
    speed: f32,
}

#[derive(Component)]
struct Enemy {
    speed: f32,
}

#[derive(Component)]
struct ScoreText;

#[derive(Resource)]
struct Score(u32);

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Shooting Game".into(),
                resolution: (800., 600.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_state::<GameState>()
        .insert_resource(Score(0))
        .insert_resource(EnemySpawnTimer {
            timer: Timer::new(Duration::from_secs_f32(1.0), TimerMode::Repeating),
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                confine_player_movement,
                player_shooting,
                bullet_movement,
                spawn_enemies,
                enemy_movement,
                bullet_enemy_collision,
                update_score_text,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnEnter(GameState::GameOver), game_over)
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Player
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                color: Color::BLUE,
                ..default()
            },
            transform: Transform::from_xyz(0.0, -200.0, 0.0),
            ..default()
        },
        Player {
            speed: 300.0,
            shoot_timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating),
        },
    ));

    // Score text
    commands.spawn((
        TextBundle::from_section(
            "Score: 0",
            TextStyle {
                font_size: 30.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        ScoreText,
    ));
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&Player, &mut Transform)>,
    time: Res<Time>,
) {
    if let Ok((player, mut transform)) = player_query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            direction.x += 1.0;
        }

        if direction.length() > 0.0 {
            direction = direction.normalize();
        }

        transform.translation += direction * player.speed * time.delta_seconds();
    }
}

fn confine_player_movement(mut player_query: Query<&mut Transform, With<Player>>) {
    if let Ok(mut player_transform) = player_query.get_single_mut() {
        let x = player_transform.translation.x;
        player_transform.translation.x = x.clamp(-350.0, 350.0);
    }
}

fn player_shooting(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Player, &Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if let Ok((mut player, transform)) = query.get_single_mut() {
        player.shoot_timer.tick(time.delta());

        if keyboard_input.pressed(KeyCode::Space) && player.shoot_timer.finished() {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(5.0, 15.0)),
                        color: Color::YELLOW,
                        ..default()
                    },
                    transform: Transform::from_xyz(
                        transform.translation.x,
                        transform.translation.y + 30.0,
                        0.0,
                    ),
                    ..default()
                },
                Bullet { speed: 500.0 },
            ));
            player.shoot_timer.reset();
        }
    }
}

fn bullet_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &Bullet, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, bullet, mut transform) in query.iter_mut() {
        transform.translation.y += bullet.speed * time.delta_seconds();

        if transform.translation.y > 400.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_enemies(mut commands: Commands, mut timer: ResMut<EnemySpawnTimer>, time: Res<Time>) {
    timer.timer.tick(time.delta());

    if timer.timer.finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(-350.0..350.0);

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(Vec2::new(40.0, 40.0)),
                    color: Color::RED,
                    ..default()
                },
                transform: Transform::from_xyz(x, 300.0, 0.0),
                ..default()
            },
            Enemy { speed: 100.0 },
        ));
    }
}

fn enemy_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &Enemy, &mut Transform)>,
    time: Res<Time>,
) {
    for (entity, enemy, mut transform) in query.iter_mut() {
        transform.translation.y -= enemy.speed * time.delta_seconds();

        if transform.translation.y < -300.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_enemy_collision(
    mut commands: Commands,
    mut score: ResMut<Score>,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (enemy_entity, enemy_transform) in enemy_query.iter() {
            let distance = bullet_transform
                .translation
                .distance(enemy_transform.translation);

            if distance < 20.0 {
                commands.entity(bullet_entity).despawn();
                commands.entity(enemy_entity).despawn();
                score.0 += 10;
            }
        }
    }
}

fn update_score_text(score: Res<Score>, mut query: Query<&mut Text, With<ScoreText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!("Score: {}", score.0);
    }
}

fn game_over(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "Game Over!",
            TextStyle {
                font_size: 50.0,
                color: Color::RED,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            left: Val::Px(300.0),
            top: Val::Px(250.0),
            ..default()
        }),
    );
}
