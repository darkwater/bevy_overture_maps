#![feature(array_chunks)]
#![feature(array_windows)]
#![feature(iter_array_chunks)]

mod building;
mod camera;
mod config;
mod geo_util;
mod ground;
mod light;
mod material;
mod parquet_import;
mod query_buildings;
mod query_transportation;
mod transportation;

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin,
    SystemInformationDiagnosticsPlugin,
};
use bevy::pbr::DefaultOpaqueRendererMethod;
use bevy::{pbr::DirectionalLightShadowMap, prelude::*, window::WindowResolution};
use bevy_egui::egui::{self, Area, FontData, FontDefinitions, FontFamily};

use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::pointer::{PointerId, PointerLocation};
use bevy_mod_picking::{focus::HoverMap, DefaultPickingPlugins};

use building::*;
use geo_util::*;
use material::*;
use query_buildings::*;
use query_transportation::*;
use transportation::*;

pub use geo_types::Coord;

use crate::{
    camera::PlayerCameraPlugin,
    config::SceneConfig,
    ground::plane_start,
    light::{animate_light_direction, light_start_system},
};

#[cfg(feature = "fps")]
mod dash;

fn main() {
    dotenv::dotenv().ok();

    let lat = std::env::var("MAP_LAT").expect("MAP_LAT env");
    let lat = lat.parse::<f64>().expect("lat to be f64");
    let lon = std::env::var("MAP_LON").expect("MAP_LON env");
    let lon = lon.parse::<f64>().expect("lon to be f64");
    let name = std::env::var("MAP_NAME").expect("MAP_NAME env");
    let lonlatname = format!("{lon}_{lat}_{name}");
    println!("{lonlatname}");

    let k = geodesic_to_coord(Coord { x: lon, y: lat });
    let center_xz: [f64; 2] = [lon * k[0], -lat * k[1]]; // Yto-Z

    let from_transportation =
        format!("read_parquet('parquet/{lonlatname}_transportation.parquet')");
    let from_building = format!("read_parquet('parquet/{lonlatname}_building.parquet')");
    println!("from_transportation:{}", &from_transportation);
    println!("from_building:{}", &from_building);

    let bevy_transportation = query_transportation(TransportationQueryParams {
        from_string: from_transportation,
        limit: None,
        k,
        center: center_xz,
    });

    let bevy_buildings = query_buildings(BuildingsQueryParams {
        from_string: from_building,
        limit: None,
        k,
        center: center_xz,
    });

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Darkmap".to_string(),
                    #[cfg(not(target_arch = "wasm32"))]
                    resolution: WindowResolution::new(1920., 1080.),
                    #[cfg(target_arch = "wasm32")]
                    resolution: WindowResolution::new(720., 360.),
                    canvas: Some("#darkmap".to_string()),
                    ..default()
                }),
                ..default()
            }),
            EntityCountDiagnosticsPlugin,
            FrameTimeDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin,
            LogDiagnosticsPlugin { ..default() },
            PlayerCameraPlugin,
            DefaultPickingPlugins,
            EguiPlugin,
            WorldInspectorPlugin::new(),
            #[cfg(feature = "fps")]
            crate::dash::DashPlugin,
        ))
        .init_resource::<MapMaterialHandle>()
        .insert_resource(Msaa::Off)
        .insert_resource(DefaultOpaqueRendererMethod::deferred())
        // .insert_resource(DirectionalLightShadowMap { size: 2048 * 2 })
        .insert_resource(SceneConfig::default())
        .insert_resource(Buildings {
            buildings: bevy_buildings,
        })
        .insert_resource(SegmentsRes {
            segments: bevy_transportation,
        })
        .insert_resource(GizmoConfig {
            depth_bias: -0.5,
            ..default()
        })
        .add_systems(
            Startup,
            (
                plane_start,
                light_start_system,
                buildings_start,
                transportations_start,
                install_font,
            ),
        )
        .add_systems(Update, (animate_light_direction, draw_hover_text))
        .run();
}

fn install_font(mut egui: EguiContexts) {
    let mut fonts = FontDefinitions::default();

    fonts.font_data.insert(
        "noto-sans-jp".to_owned(),
        FontData::from_static(include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf")),
    );

    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .push("noto-sans-jp".to_owned());

    egui.ctx_mut().set_fonts(fonts);
}

fn draw_hover_text(
    mut egui: EguiContexts,
    hovers: Res<HoverMap>,
    pointers: Query<(&PointerId, &PointerLocation)>,
    buildings: Query<(&Building, &Transform, &Handle<Mesh>)>,
    roads: Query<(&RoadSegment, &Transform, &Handle<Mesh>)>,
    meshes: Res<Assets<Mesh>>,
    mut gizmos: Gizmos,
) {
    let ctx = egui.ctx_mut();

    let building = hovers
        .get(&PointerId::Mouse)
        .and_then(|hits| hits.iter().find_map(|(ent, _)| buildings.get(*ent).ok()));

    let road = hovers
        .get(&PointerId::Mouse)
        .and_then(|hits| hits.iter().find_map(|(ent, _)| roads.get(*ent).ok()));

    let pointer = pointers
        .iter()
        .find(|(id, _)| id.is_mouse())
        .and_then(|(_, loc)| loc.location.as_ref());

    if let Some((road, transform, mesh)) = road {
        let mesh = meshes.get(mesh).unwrap();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let indices = mesh.indices().unwrap();

        for (idx, pos) in positions
            .iter()
            .map(|&p| transform.transform_point(p.into()))
            .enumerate()
        {
            let color = if idx % 2 == 0 {
                Color::GREEN
            } else {
                Color::BLUE
            };

            gizmos.line(pos, Vec3::new(pos.x, 1., pos.z), color);
        }

        for [a, b, c] in indices
            .iter()
            .map(|i| transform.transform_point(positions[i].into()))
            .array_chunks()
        {
            gizmos.linestrip([a, b, c, a], Color::RED);
        }

        gizmos.linestrip(
            road.points
                .iter()
                .copied()
                .map(|p| transform.transform_point(p)),
            Color::BLUE,
        );
    }

    if let (Some((building, transform, mesh)), Some(pointer)) = (building, pointer) {
        let mesh = meshes.get(mesh).unwrap();
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        let indices = mesh.indices().unwrap();

        for pos in positions
            .iter()
            .map(|&p| transform.transform_point(p.into()))
        {
            gizmos.line(pos, Vec3::new(pos.x, 0., pos.z), Color::GREEN);
        }

        for [a, b, c] in indices
            .iter()
            .map(|i| transform.transform_point(positions[i].into()))
            .array_chunks()
        {
            gizmos.linestrip([a, b, c, a], Color::RED);
        }

        egui::show_tooltip_at(
            ctx,
            "hover text".into(),
            Some(egui::Pos2::from(pointer.position.to_array()) + egui::vec2(4., 24.)),
            |ui| {
                if let Some(text) = building
                    .names
                    .as_ref()
                    .and_then(|n| n.common_local().map(|c| c.to_string()))
                {
                    ui.label(text);
                }

                if let Some(text) = building.class.as_ref().map(|c| format!("{c:?}")) {
                    ui.label(text);
                }
            },
        );
    }
}
