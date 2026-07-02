// =============================================================
//  DULAHI 3D – Application Entry Point
//  File: src/main.rs
//
//  Initialises Bevy, locks the window to landscape orientation,
//  and registers all plugins including the master UiPlugin.
// =============================================================

mod ui;

use bevy::prelude::*;
use bevy::window::{WindowMode, WindowResolution};
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title:        "Dulahi 3D".into(),
                        // Landscape 20:9 logical resolution.
                        // On Android the OS overrides this with the real
                        // screen size, so this value matters mainly on desktop.
                        resolution:   WindowResolution::new(2340, 1080)
                                          .with_scale_factor_override(1.0),
                        mode:         WindowMode::BorderlessFullscreen(
                                          MonitorSelection::Current,
                                      ),
                        resizable:    false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        // ── UI state machine + all screens ──────────────────
        .add_plugins(UiPlugin)
        // ── Persistent 3-D scene (floor + camera) ───────────
        // These stay alive across all UI states; the in-game
        // plugin will take over once GameState::InGame is set.
        .add_systems(Startup, (spawn_floor, spawn_camera))
        .run();
}

// ── Persistent 3-D floor ─────────────────────────────────────

fn spawn_floor(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(
            -std::f32::consts::FRAC_PI_2,
        )),
    ));
}

// ── Persistent camera ─────────────────────────────────────────

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}