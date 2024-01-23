use bevy::{pbr::NotShadowCaster, prelude::*, render::mesh::*};
use geo_types::LineString;
use serde::{Deserialize, Serialize};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, FRAC_PI_6, PI};
use std::ops::Sub;
use strum_macros::EnumIter;

use crate::{KxyGeodesic, MapMaterialHandle};

#[derive(Serialize, Deserialize, Debug)]
pub struct Road {
    pub class: String,
}
#[derive(EnumIter, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RoadClass {
    // highway=motorway > trunk > primary > secondary > ... > living streets > ... > footway
    Motorway,     // - motorway
    Primary,      // - primary
    Secondary,    // - secondary
    Tertiary,     // - tertiary
    Residential,  // - residential
    LivingStreet, // - livingStreet # similar as residential but has implied legal restriction for motor vehicles (which can vary country by country)
    Trunk,        // - trunk
    Unclassified, // - unclassified # known roads, paved but of low importance which does not meet definition of being motorway, trunk, primary, secondary, tertiary
    ParkingAisle, // - parkingAisle # service road intended for parking
    Driveway,     // - driveway # service road intended for deliveries
    Pedestrian,   // - pedestrian
    Footway,      // - footway
    Steps,        // - steps
    Track,        // - track
    Cycleway,     // - cycleway
    Bridleway,    // - bridleway # similar as track but has implied access only for horses
    Unknown,      // - unknown
}
impl RoadClass {
    pub fn depth_bias(&self) -> f32 {
        match self {
            RoadClass::Motorway => 16.,
            RoadClass::Primary => 15.,
            RoadClass::Secondary => 14.,
            RoadClass::Tertiary => 13.,
            RoadClass::Residential => 12.,
            RoadClass::LivingStreet => 11.,
            RoadClass::Trunk => 10.,
            RoadClass::Unclassified => 9.,
            RoadClass::ParkingAisle => 8.,
            RoadClass::Driveway => 7.,
            RoadClass::Pedestrian => 6.,
            RoadClass::Footway => 5.,
            RoadClass::Steps => 4.,
            RoadClass::Track => 3.,
            RoadClass::Cycleway => 2.,
            RoadClass::Bridleway => 1.,
            RoadClass::Unknown => 0.1,
        }
    }
    pub fn from_string(s: &String) -> RoadClass {
        match s.as_str() {
            "motorway" => RoadClass::Motorway,
            "primary" => RoadClass::Primary,
            "secondary" => RoadClass::Secondary,
            "tertiary" => RoadClass::Tertiary,
            "residential" => RoadClass::Residential,
            "livingStreet" => RoadClass::LivingStreet,
            "trunk" => RoadClass::Trunk,
            "unclassified" => RoadClass::Unclassified,
            "parkingAisle" => RoadClass::ParkingAisle,
            "driveway" => RoadClass::Driveway,
            "pedestrian" => RoadClass::Pedestrian,
            "footway" => RoadClass::Footway,
            "steps" => RoadClass::Steps,
            "track" => RoadClass::Track,
            "cycleway" => RoadClass::Cycleway,
            "bridleway" => RoadClass::Bridleway,
            "unknown" => RoadClass::Unknown,
            _ => RoadClass::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Segment {
    pub translate: [f64; 2],
    pub line: Vec<[f64; 2]>,
    pub k: KxyGeodesic,
    pub road_class: RoadClass,
    pub width: Option<f32>,
}

#[derive(Resource, Debug)]
pub struct SegmentsRes {
    pub segments: Vec<Segment>,
}
pub fn line_string_road(
    line_string: LineString,
    k: KxyGeodesic,
    center: [f64; 2],
) -> ([f64; 2], Vec<[f64; 2]>) {
    let c1 = line_string
        .coords()
        .nth(0)
        .expect("To take exterior:0 coordinate");
    let first_point_xz: [f64; 2] = [c1.x * k[0] - center[0], -c1.y * k[1] - center[1]]; // Yto-Z

    let line: Vec<[f64; 2]> = line_string
        .coords()
        .map(|c| {
            [
                c.x * k[0] - center[0] - first_point_xz[0],
                -c.y * k[1] - center[1] - first_point_xz[1], // Yto-Z
            ]
        })
        .collect();
    (first_point_xz, line)
}
// pub fn line_string_base(line_string: &LineString) -> (f64, [f64; 2]) {
//     let c1 = line_string
//         .coords()
//         .nth(0)
//         .expect("To take line_string:0 coordinate");
//     let p1 = geo::Point(*c1);
//     let c2 = line_string
//         .coords()
//         .nth(1)
//         .expect("To take line_string:1 coordinate");
//     let p2 = geo::Point(*c2);
//     let geodesic_distance = p1.geodesic_distance(&p2);
//     let coord_distance = c1.add(c2.neg()).magnitude();
//     let k = geodesic_distance / coord_distance;
//     let first_point_position: [f64; 2] = [c1.x * k, c1.y * k];
//     (k, first_point_position)
// }

pub fn transportations_start(
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    transportations_res: Res<SegmentsRes>,
    map_materials: Res<MapMaterialHandle>,
) {
    for item in transportations_res.segments.iter() {
        spawn_transportation(&mut cmd, &mut meshes, &mut materials, item, &map_materials);
    }
}

impl From<&RoadClass> for Color {
    fn from(value: &RoadClass) -> Self {
        match value {
            RoadClass::Motorway => Color::DARK_GRAY,
            RoadClass::Primary => Color::GRAY,
            RoadClass::Secondary => Color::YELLOW,
            RoadClass::Tertiary => Color::ANTIQUE_WHITE,
            RoadClass::Residential => Color::BEIGE,
            RoadClass::LivingStreet => Color::SALMON,
            RoadClass::Trunk => Color::INDIGO,
            RoadClass::Unclassified => Color::WHITE,
            RoadClass::ParkingAisle => Color::AZURE,
            RoadClass::Driveway => Color::OLIVE,
            RoadClass::Pedestrian => Color::CRIMSON,
            RoadClass::Footway => Color::ORANGE_RED,
            RoadClass::Steps => Color::SILVER,
            RoadClass::Track => Color::LIME_GREEN,
            RoadClass::Cycleway => Color::GREEN,
            RoadClass::Bridleway => Color::DARK_GREEN,
            RoadClass::Unknown => Color::rgb(0.1, 0.1, 0.3),
        }
    }
}

type RoadWidth = f32;
impl From<&RoadClass> for RoadWidth {
    fn from(value: &RoadClass) -> RoadWidth {
        match value {
            RoadClass::Motorway => 12.,
            RoadClass::Primary => 10.,
            RoadClass::Secondary => 8.,
            RoadClass::Tertiary => 6.,
            RoadClass::Residential => 5.5,
            RoadClass::LivingStreet => 5.,
            RoadClass::Trunk => 4.5,
            RoadClass::Unclassified => 4.,
            RoadClass::ParkingAisle => 3.5,
            RoadClass::Driveway => 3.,
            RoadClass::Pedestrian => 2.5,
            RoadClass::Footway => 1.5,
            RoadClass::Steps => 1.4,
            RoadClass::Track => 1.3,
            RoadClass::Cycleway => 1.2,
            RoadClass::Bridleway => 1.1,
            RoadClass::Unknown => 1.,
        }
    }
}

pub fn spawn_transportation(
    cmd: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    transportation: &Segment,
    map_materials: &Res<MapMaterialHandle>,
) {
    let width = transportation
        .width
        .unwrap_or_else(|| RoadWidth::from(&transportation.road_class));

    let segment = RoadSegment::new(&transportation.line, width);
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::from(segment.vertices.clone()),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::from(segment.normals.clone()),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::from(segment.uvs.clone()),
    );
    mesh.set_indices(Some(Indices::U32(segment.indices.clone())));

    let translate: Vec3 = Vec3::new(
        transportation.translate[0] as f32,
        width * 0.01,
        transportation.translate[1] as f32,
    );
    let transform = Transform::from_translation(translate);
    cmd.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: map_materials
                .road
                .get(&transportation.road_class)
                .unwrap()
                .clone(),
            transform,
            ..Default::default()
        },
        segment,
        NotShadowCaster,
    ));
}

#[derive(Component, Debug)]
pub struct RoadSegment {
    pub points: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub norm: Vec<Vec3>,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

impl RoadSegment {
    pub fn empty() -> Self {
        Self {
            points: vec![],
            indices: vec![],
            norm: vec![],
            vertices: vec![],
            normals: vec![],
            uvs: vec![],
        }
    }

    pub fn new(line: &[[f64; 2]], width: f32) -> Self {
        let half_width: f32 = width / 2.;
        let mut segm = Self::empty();
        segm.points = line
            .iter()
            .map(|pos| Vec3::new(pos[0] as f32, 0., pos[1] as f32))
            .collect::<Vec<Vec3>>();

        let first_angle = segm.points[1].sub(segm.points[0]).normalize();
        let first_left = Quat::from_rotation_y(FRAC_PI_2).mul_vec3(first_angle);
        let first_right = Quat::from_rotation_y(-FRAC_PI_2).mul_vec3(first_angle);
        let first_left_behind = Quat::from_rotation_y(FRAC_PI_2 + FRAC_PI_3).mul_vec3(first_angle);
        let first_right_behind =
            Quat::from_rotation_y(-FRAC_PI_2 - FRAC_PI_3).mul_vec3(first_angle);

        segm.vertices
            .push((segm.points[0] + first_left_behind * half_width).into());
        segm.vertices
            .push((segm.points[0] + first_right_behind * half_width).into());
        segm.vertices
            .push((segm.points[0] + first_left * half_width).into());
        segm.vertices
            .push((segm.points[0] + first_right * half_width).into());

        segm.normals.push(Vec3::Y.into());
        segm.normals.push(Vec3::Y.into());
        segm.normals.push(Vec3::Y.into());
        segm.normals.push(Vec3::Y.into());

        segm.uvs.push([0., 0.]);
        segm.uvs.push([0., 1.]);
        segm.uvs.push([0.1, 0.]);
        segm.uvs.push([0.1, 1.]);

        for [prev, this, next] in segm.points.array_windows().copied() {
            let prev_angle = this.sub(prev).normalize();
            let next_angle = next.sub(this).normalize();
            let angle = (prev_angle + next_angle).normalize();

            let left = Quat::from_rotation_y(FRAC_PI_2).mul_vec3(angle);
            let right = Quat::from_rotation_y(-FRAC_PI_2).mul_vec3(angle);

            segm.vertices.push((this + left * half_width).into());
            segm.vertices.push((this + right * half_width).into());

            segm.normals.push(Vec3::Y.into());
            segm.normals.push(Vec3::Y.into());

            segm.uvs.push([0.5, 0.]);
            segm.uvs.push([0.5, 1.]);
        }

        let last_angle = segm.points[segm.points.len() - 1]
            .sub(segm.points[segm.points.len() - 2])
            .normalize();
        let last_left = Quat::from_rotation_y(FRAC_PI_2).mul_vec3(last_angle);
        let last_right = Quat::from_rotation_y(-FRAC_PI_2).mul_vec3(last_angle);
        let last_left_after = Quat::from_rotation_y(FRAC_PI_2 - FRAC_PI_3).mul_vec3(last_angle);
        let last_right_after = Quat::from_rotation_y(-FRAC_PI_2 + FRAC_PI_3).mul_vec3(last_angle);

        segm.vertices
            .push((segm.points[segm.points.len() - 1] + last_left * half_width).into());
        segm.vertices
            .push((segm.points[segm.points.len() - 1] + last_right * half_width).into());
        segm.vertices
            .push((segm.points[segm.points.len() - 1] + last_left_after * half_width).into());
        segm.vertices
            .push((segm.points[segm.points.len() - 1] + last_right_after * half_width).into());

        segm.normals.push(Vec3::Y.into());
        segm.normals.push(Vec3::Y.into());
        segm.normals.push(Vec3::Y.into());
        segm.normals.push(Vec3::Y.into());

        segm.uvs.push([0.9, 0.]);
        segm.uvs.push([0.9, 1.]);
        segm.uvs.push([1., 0.]);
        segm.uvs.push([1., 1.]);

        // segm.indices.extend([0, 1, 2]);
        // segm.indices.extend([2, 1, 3]);

        for n in (0..segm.vertices.len() - 2).step_by(2) {
            segm.indices.extend([n as u32, n as u32 + 1, n as u32 + 2]);
            segm.indices
                .extend([n as u32 + 2, n as u32 + 1, n as u32 + 3]);
        }

        // let material_length = 1.;
        // let mut len: f32 = 0.;

        // for (i, p) in segm.points.iter().enumerate() {
        //     let last: bool = i + 1 == segm.points.len();
        //     if last {
        //     } else {
        //         let ix2: u32 = i as u32 * 4;
        //         let (tri_1, tri_2) = ([ix2, ix2 + 1, ix2 + 2], [ix2 + 2, ix2 + 1, ix2 + 3]);
        //         segm.indices.extend(tri_1);
        //         segm.indices.extend(tri_2);
        //         segm.norm.push(Vec3::Y);

        //         let i_next: usize = i + 1;
        //         let point: Vec3 = *p;
        //         let point_next: Vec3 = segm.points[i_next];

        //         let dir: Vec3 = (point_next - point).normalize();
        //         let left_norm = Quat::from_rotation_y(FRAC_PI_2).mul_vec3(dir);
        //         let side = left_norm * half_width;
        //         let (l1, r1) = (point + side, point - side);
        //         let (l2, r2) = (point_next + side, point_next - side);
        //         segm.vertices.push((l1).into());
        //         segm.vertices.push((r1).into());
        //         segm.vertices.push((l2).into());
        //         segm.vertices.push((r2).into());

        //         let l_uv = len / material_length;
        //         segm.uvs.push([l_uv, 0.]);
        //         segm.uvs.push([l_uv, 0.]);
        //         segm.uvs.push([l_uv, 1.]);
        //         segm.uvs.push([l_uv, 1.]);

        //         let normal = segm.norm[i].to_array();
        //         segm.normals.push(normal);
        //         segm.normals.push(normal);
        //         segm.normals.push(normal);
        //         segm.normals.push(normal);

        //         let diff = point_next.sub(point).length();
        //         len += diff;
        //     }
        // }

        segm
    }
}
