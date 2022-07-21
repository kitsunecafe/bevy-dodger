use std::{ops::Range, time::Duration};

use bevy::{prelude::*, sprite::collide_aabb::collide};
use rand::Rng;

const SPRITE_SIZE: f32 = 16.0;
const SCREEN_X_RANGE: Range<f32> = -320.0..320.0;
const SCREEN_Y_RANGE: Range<f32> = -220.0..220.0;
const OBJECT_SIZE: Range<f32> = 0.5..5.0;
const OBJECT_SPEED: Range<f32> = 50.0..125.0;
const PLAYER_SPEED: f32 = 100.0;

const SCOREBOARD_FONT_SIZE: f32 = 32.0;
const SUMMARY_FONT_SIZE: f32 = 64.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(16.0);

const TEXT_COLOR: Color = Color::ANTIQUE_WHITE;
const SCORE_COLOR: Color = Color::YELLOW;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    Title,
    Playing,
    GameOver,
}

#[derive(Component)]
struct SpawnTimer {
    timer: Timer,
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

struct CollisionEvent(Entity, Entity);

struct TextFont(Handle<Font>);
struct SpriteSheet(Handle<TextureAtlas>);

struct Scoreboard {
    score: f32,
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Dodger".to_string(),
            width: 640.0,
            height: 480.0,
            ..default()
        })
        .add_state(GameState::Title)
        .add_event::<CollisionEvent>()
        .insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0.0 })
        .add_system_set(SystemSet::on_enter(GameState::Title).with_system(setup_title))
        .add_system_set(SystemSet::on_update(GameState::Title).with_system(start_game))
        .add_system_set(SystemSet::on_exit(GameState::Title).with_system(cleanup))
        .add_system_set(SystemSet::on_enter(GameState::Playing).with_system(setup))
        .add_system_set(
            SystemSet::on_update(GameState::Playing)
                .with_system(apply_velocity)
                .with_system(enemy_spawner)
                .with_system(player_movement)
                .with_system(check_collisions)
                .with_system(end_on_collision)
                .with_system(update_score),
        )
        .add_system_set(SystemSet::on_exit(GameState::Playing).with_system(cleanup))
        .add_system_set(SystemSet::on_enter(GameState::GameOver).with_system(show_summary))
        .add_system_set(SystemSet::on_update(GameState::GameOver).with_system(start_game))
        .add_system_set(SystemSet::on_exit(GameState::GameOver).with_system(cleanup))
        .run();
}

fn setup_title(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("pixeled.ttf");

    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Dodger".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: SUMMARY_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                },
            ],
            alignment: TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            },
            ..default()
        },
        style: Style {
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                left: Val::Px(320.0 - SUMMARY_FONT_SIZE),
                ..default()
            },
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Press Space".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                },
            ],
            alignment: TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            },
            ..default()
        },
        style: Style {
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                left: Val::Px(320.0 - SCOREBOARD_FONT_SIZE),
                top: Val::Px(220.0 + SCOREBOARD_FONT_SIZE),
                ..default()
            },
            ..default()
        },
        ..default()
    });

    commands.insert_resource(TextFont(font));
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut scoreboard: ResMut<Scoreboard>,
    font: Res<TextFont>
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    scoreboard.score = 0.0;

    let handle: Handle<Image> = asset_server.load("colored-transparent.png");
    let texture_atlas =
        TextureAtlas::from_grid_with_padding(handle, Vec2::splat(16.0), 49, 22, Vec2::splat(1.0));

    let texture_atlas_handle = atlases.add(texture_atlas);

    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, SCREEN_Y_RANGE.start, 0.0),
                scale: Vec3::splat(1.0),
                ..default()
            },
            sprite: TextureAtlasSprite::new(1042),
            ..default()
        })
        .insert(Player);

    commands.insert_resource(SpriteSheet(texture_atlas_handle));

    commands.insert_resource(SpawnTimer {
        timer: Timer::new(Duration::from_secs(1), true),
    });

    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                    },
                },
            ],
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

fn cleanup(mut commands: Commands, query: Query<Entity>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn start_game(keyboard_input: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    if keyboard_input.pressed(KeyCode::Space) {
        state.set(GameState::Playing).unwrap();
    }
}

fn show_summary(mut commands: Commands, font: Res<TextFont>, scoreboard: Res<Scoreboard>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: SUMMARY_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                },
                TextSection {
                    value: format!("{}", scoreboard.score as i16),
                    style: TextStyle {
                        font: font.0.clone(),
                        font_size: SUMMARY_FONT_SIZE,
                        color: SCORE_COLOR,
                    },
                },
            ],
            alignment: TextAlignment {
                horizontal: HorizontalAlign::Center,
                vertical: VerticalAlign::Center,
            },
            ..default()
        },
        style: Style {
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            position_type: PositionType::Absolute,
            position: Rect {
                left: Val::Px(320.0 - SUMMARY_FONT_SIZE),
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

fn apply_velocity(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    let delta_time = time.delta_seconds();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * delta_time;
    }
}

fn enemy_spawner(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    sprite_sheet: Res<SpriteSheet>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.finished() {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(SCREEN_X_RANGE);
        let velocity = rng.gen_range(OBJECT_SPEED);
        let scale = rng.gen_range(OBJECT_SIZE);

        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite::new(1069),
                texture_atlas: sprite_sheet.0.clone(),
                transform: Transform {
                    translation: Vec3::new(x, 220.0, 0.0),
                    scale: Vec3::new(scale, scale, 1.0),
                    ..default()
                },
                ..default()
            })
            .insert(Velocity(Vec3::new(0.0, -velocity, 0.0)))
            .insert(Collider);
    }
}

fn player_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let delta_time = time.delta_seconds();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) {
        direction += 1.0;
    }

    for mut transform in query.iter_mut() {
        let new_position = transform.translation.x + direction * PLAYER_SPEED * delta_time;
        transform.translation.x = new_position;
    }
}

fn update_score(time: Res<Time>, mut scoreboard: ResMut<Scoreboard>, mut query: Query<&mut Text>) {
    scoreboard.score += time.delta_seconds();
    let mut text = query.single_mut();
    text.sections[1].value = format!("{}", scoreboard.score as i16);
}

fn check_collisions(
    mut ev_collision: EventWriter<CollisionEvent>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    projectile_query: Query<(Entity, &Transform), With<Collider>>,
) {
    for (player, player_transform) in player_query.iter() {
        let player_size = player_transform.scale.truncate() * SPRITE_SIZE;

        for (projectile, projectile_transform) in projectile_query.iter() {
            let collision = collide(
                player_transform.translation,
                player_size,
                projectile_transform.translation,
                projectile_transform.scale.truncate() * SPRITE_SIZE,
            );

            if collision.is_some() {
                ev_collision.send(CollisionEvent(player, projectile));
            }
        }
    }
}

fn end_on_collision(
    mut ev_collision: EventReader<CollisionEvent>,
    mut state: ResMut<State<GameState>>,
) {
    for _collision in ev_collision.iter() {
        if *state.current() != GameState::Playing {
            return;
        }

        state.set(GameState::GameOver).unwrap();
    }
}
