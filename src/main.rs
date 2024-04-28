use bevy::{asset::AssetMetaCheck, math::{vec2, vec3}, prelude::*, window::WindowResolution};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::Rng;
#[derive(Component, Default, Debug)]
struct Position(Vec2);
#[derive(Component, Default, Debug)]
struct Velocity(Vec2);
#[derive(Component, Default, Debug)]
struct Area(f32);
#[derive(Component, Default, Debug)]
struct Mass(f32);
#[derive(Component, Default, Debug)]
struct Force(Vec2);
#[derive(Component, Default, Debug)]
struct Player;
#[derive(Default, Bundle)]
struct PlayerBundle {
    player: Player,
    position: Position,
    velocity: Velocity,
    force: Force,
    mass: Mass,
    #[bundle()]
    sprite: SpriteSheetBundle
}
#[derive(Component, Default)]
struct Phase(u32);
#[derive(Component, Default)]
struct Enemy;
#[derive(Default, Bundle)]
struct EnemyBundle {
    enemy: Enemy,
    position: Position,
    velocity: Velocity,
    force: Force,
    mass: Mass,
    phase: Phase,
    #[bundle()]
    sprite: SpriteBundle
}
#[derive(Default, Component)]
struct Bullet;
#[derive(Default, Bundle)]
struct BulletBundle {
    position: Position,
    velocity: Velocity,
    bullet: Bullet,
    #[bundle()]
    sprite: SpriteBundle
}
#[derive(Resource, Default, Debug)]
struct TickCounter {
    inner: u32
}
#[derive(Default, Resource)]
struct Score(u32);
impl TickCounter {
    fn tick(&mut self) {
        self.inner = self.inner.wrapping_add(1);
    }
    fn is_n(&self, n: u32) -> bool {
        self.inner % n == 0
    }
}
fn bullet_collision(mut commands: Commands, bullets: Query<(&Position, Entity), With<Bullet>>, enemies: Query<(&Position, Entity), With<Enemy>>,
    mut score: ResMut<Score>
) {
    for (Position(bullet_pos), bullet) in bullets.iter() {
        for (Position(enemy_pos), enemy) in enemies.iter() {
            if (*enemy_pos - *bullet_pos).length() < 15.0 {
                commands.entity(enemy).despawn();
                commands.entity(bullet).despawn();
                score.0 += 100;
                break;
            }
        }
    }
}
fn player_collision(mut commands: Commands, mut player: Query<&mut Position, With<Player>>, enemies: Query<(&Position, Entity), (With<Enemy>,Without<Player>)>,
    mut score: ResMut<Score>
) {
    let mut player_pos = player.single_mut();
    for (Position(enemy_pos), _) in enemies.iter() {
        if (*enemy_pos - player_pos.0).length() < 10.0 {
            for (_, enemy) in enemies.iter() {
                commands.entity(enemy).despawn();
            }
            score.0 = 0;
            player_pos.0 = vec2(128.0, 20.0);
        }
    }
}
fn main() {
    App::new()
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(TickCounter::default())
        .insert_resource(Score(0))
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((EmbeddedAssetPlugin {mode: bevy_embedded_assets::PluginMode::ReplaceAndFallback { path: "assets".to_owned() }}, DefaultPlugins.set(ImagePlugin::default_nearest())))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            tick_counter, 
            bullet_collision, 
            player_collision,
            kinematics, 
            movement, 
            spawn_enemy_schedule, 
            player_bound,
        ))
        .add_systems(Update, (handle_keyboard, render, render_player,update_score , enemy_movement, despawn_oob))
        .run()
}
fn tick_counter(mut counter: ResMut<TickCounter>) {
    counter.tick()
}
fn spawn_enemy(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let mut r = rand::thread_rng();
    commands.spawn(EnemyBundle {
        position: Position(vec2(r.gen_range(12.0..244.0), 380.0)),
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/enemy1.png"),
            ..Default::default()
        },
        mass: Mass(1.0),
        phase: Phase(r.gen_range(0..60)),
        ..Default::default()
    });
}
fn spawn_enemy_schedule(mut commands: Commands, asset_server: Res<AssetServer>, tick: Res<TickCounter>, score: Res<Score>) {
    if tick.is_n(240) {
        spawn_enemy(&mut commands, &asset_server);
    }
    if tick.is_n(120) && rand::thread_rng().gen_bool(0.7) {
        spawn_enemy(&mut commands, &asset_server);
    }
    if tick.is_n(40) && rand::thread_rng().gen_bool((0.0003 * (score.0 as f64)).clamp(0.001, 0.999)) {
        spawn_enemy(&mut commands, &asset_server);
    }
    if tick.is_n(13) && rand::thread_rng().gen_bool((0.0001 * (score.0 as f64)).clamp(0.001, 0.999)) {
        spawn_enemy(&mut commands, &asset_server);
    }
}
fn setup(
    mut commands: Commands, 
    mut window: Query<&mut Window>, 
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform { translation: Vec3::new(128.0, 192.0, 0.0), scale: vec3(0.5, 0.5, 1.0),..Default::default() },

        ..Default::default()
    });
    let _handle: Handle<Image> = asset_server.load("sprites/enemy1.png");
    let __handle: Handle<Image> = asset_server.load("sprites/bullet.png");
    let texture = asset_server.load("sprites/player.png");
    let layout = TextureAtlasLayout::from_grid(Vec2::new(16.0, 32.0), 3, 2, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn(PlayerBundle {
        sprite: SpriteSheetBundle {
            texture,
            atlas: TextureAtlas { layout: texture_atlas_layout, index: 1 },
            ..Default::default()
        },
        mass: Mass(1.0),
        position: Position(vec2(128.0, 20.0)),
        ..Default::default()
    });
    commands.spawn(Text2dBundle {
        text: Text::from_section("00000", TextStyle::default()),
        transform: Transform {translation: vec3(224.0, 20.0, 0.0), ..Default::default()},
        ..Default::default()
    });
    window.single_mut().resolution = WindowResolution::new(512.0, 768.0);

}
fn kinematics(mut q: Query<(&mut Position, &mut Velocity, &Mass, &Force)>) {
    for (mut position, mut velocity, mass, force) in q.iter_mut() {
        velocity.0 += force.0 / mass.0 * 0.017;
        velocity.0 *= 0.95;
        if velocity.0.length() > 15.0 {
            velocity.0 *= 0.95
        }
        position.0 += velocity.0;
        
    }
}
fn movement(mut q: Query<(&mut Position, &Velocity), Without<Force>>) {
    for (mut pos, vel) in q.iter_mut() {
        pos.0 += vel.0;
    }
}
fn player_bound(mut q: Query<(&mut Position, &mut Velocity), With<Player>>) {
    let (mut position, mut velocity) = q.single_mut();
    if position.0.x > 256.0 || position.0.x < 0.0 {
        velocity.0.x = 0.0;
        position.0.x = position.0.x.clamp(0.0, 256.0);
    }
    if position.0.y > 384.0 || position.0.y < 0.0 {
        velocity.0.y = 0.0;
        position.0.y = position.0.y.clamp(0.0, 384.0);
    }
}
fn render(mut q: Query<(&Position, &mut Transform)>) {
    for (pos, mut trans) in q.iter_mut() {
        *trans = trans.with_translation([pos.0.x, pos.0.y, 0.0].into());
    }
}
fn render_player(mut q: Query<(&mut TextureAtlas, &Velocity), With<Player>>, tick: Res<TickCounter>) {
    let (mut atlas, force) = q.single_mut();
    if force.0.x >= 3.0 {
        atlas.index = 2;
    }
    else if force.0.x <= -3.0 {
        atlas.index = 0;
    }
    else {atlas.index = 1}
    if tick.is_n(15) {
        atlas.index += 3;
    };
}
fn handle_keyboard(
    mut q: Query<(&mut Force, &Position), With<Player>>, mut commands: Commands,
    ev: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    tick: Res<TickCounter>
) {
    let acc = Vec2::new(
          if ev.pressed(KeyCode::KeyD) {10.0} else {0.0} 
        - if ev.pressed(KeyCode::KeyA) {10.0} else {0.0}
        ,
          if ev.pressed(KeyCode::KeyW) {10.0} else {0.0} 
        - if ev.pressed(KeyCode::KeyS) {10.0} else {0.0}
    );
    let (mut force, pos) = q.single_mut();
    force.0 = acc;
    if ev.pressed(KeyCode::Space) && tick.is_n(12) {
        spawn_bullet(&mut commands, pos.0, &asset_server);
    }
}
fn spawn_bullet(commands: &mut Commands, pos: Vec2, asset_server: &Res<AssetServer>) {
    commands.spawn(BulletBundle {
        position: Position(pos),
        velocity: Velocity(vec2(0.0, 10.0)),
        sprite: SpriteBundle {
            texture: asset_server.load("sprites/bullet.png"),
            ..Default::default()
        },
        ..Default::default()
    });
}
fn enemy_movement(mut q: Query<(&mut Force, &Phase), With<Enemy>>, tick: Res<TickCounter>) {
    for (mut force, phase) in q.iter_mut() {
        force.0.y = -2.0;
        let t = (tick.inner + phase.0) % 60;
        let t = t as f32;
        let x = t.sin();
        force.0.x = 6.0 * x;
    }
}
fn despawn_oob(q: Query<(Entity, &Position), (With<Enemy>, Without<Bullet>)>,
    q2: Query<(Entity, &Position), (Without<Enemy>, With<Bullet>)>,
    mut commands: Commands) {
    for (entity, pos) in q.iter() {
        if pos.0.y < -10.0 {
            commands.entity(entity).despawn()
        }
    }
    for (entity, pos) in q2.iter() {
        if pos.0.y > 500.0 {
            commands.entity(entity).despawn()
        }
    }
}
fn update_score(mut q: Query<&mut Text>, score: Res<Score>) {
    *q.single_mut() = Text::from_section(format!("{:0>5}", score.0 % 10000), TextStyle::default());
}
