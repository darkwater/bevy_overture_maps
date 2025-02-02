use bevy::{pbr::NotShadowCaster, prelude::*};

use crate::config::SceneConfig;

pub fn light_start_system(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    scene_config: Res<SceneConfig>,
) {
    cmd.insert_resource(AmbientLight {
        color: Color::rgb_u8(210, 220, 240),
        brightness: 0.9,
    });

    cmd.spawn(DirectionalLightBundle {
        // directional_light: DirectionalLight {
        //     illuminance: 40_000.,
        //     shadows_enabled: true,
        //     ..default()
        // },
        transform: Transform {
            translation: Vec3::new(0., 0., 0.),
            rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_8),
            ..default()
        },
        // cascade_shadow_config: CascadeShadowConfigBuilder {
        //     maximum_distance: 2500.,
        //     minimum_distance: 0.2,
        //     num_cascades: 3,
        //     first_cascade_far_bound: 200.,
        //     ..default()
        // }
        // .into(),
        ..default()
    });

    cmd.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::default())),
            material: materials.add(StandardMaterial {
                base_color: Color::hex("888888").unwrap(),
                unlit: true,
                cull_mode: None,
                ..default()
            }),
            transform: Transform::from_scale(Vec3::splat(scene_config.size)),
            ..default()
        },
        NotShadowCaster,
    ));
}

const K: f32 = 2.;

pub fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::H) {
        for mut transform in &mut query {
            transform.rotate_y(time.delta_seconds() * K);
        }
    }
    if input.pressed(KeyCode::L) {
        for mut transform in &mut query {
            transform.rotate_y(-time.delta_seconds() * K);
        }
    }
    if input.pressed(KeyCode::J) {
        for mut transform in &mut query {
            transform.rotate_x(time.delta_seconds() * K);
        }
    }
    if input.pressed(KeyCode::K) {
        for mut transform in &mut query {
            transform.rotate_x(-time.delta_seconds() * K);
        }
    }
}
