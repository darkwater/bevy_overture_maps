use duckdb::Connection;
use geo_types::Geometry;
use geozero::wkb::FromWkb;
use geozero::wkb::WkbDialect;

use crate::transportation::line_string_road;
use crate::transportation::BevyTransportation;

pub struct TransportationQueryParams {
    pub limit: usize,
    pub where_string: String,
    pub from_string: String,
    pub k: f64,
    pub translate: [f64; 2],
}

use serde::{Deserialize, Serialize};

// https://docs.overturemaps.org/reference/transportation/segment
// https://github.com/alexichepura/overture_maps_rs/issues/1

#[derive(Serialize, Deserialize, Debug)]
struct Road {
    class: String,
}
#[derive(Serialize, Deserialize, Debug)]
enum RoadClass {
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

pub fn query_transportation(params: TransportationQueryParams) -> Vec<BevyTransportation> {
    let path = "./data.duckdb";
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("INSTALL httpfs; LOAD httpfs;").unwrap();
    conn.execute_batch("INSTALL spatial; LOAD spatial;")
        .unwrap();
    let limit = params.limit;
    let where_string = params.where_string;
    let from = params.from_string;
    let mut stmt = conn
        // .prepare(&format!(
        //     "SELECT
        //         id,
        //         subtype,
        //         ST_GeomFromWkb(geometry) AS geometry
        //         FROM {from}
        //         WHERE id='segment.87269954dffffff-13F6AD9C53A876A2'
        //         LIMIT {limit}"
        // ))
        .prepare(&format!(
            "SELECT
                id,
                ST_GeomFromWkb(geometry) AS geometry,
                road,
                level
                FROM {from}
                WHERE {where_string}
                LIMIT {limit}"
        ))
        .unwrap();
    #[derive(Debug)]
    struct Transportation {
        id: String,
        geom: Vec<u8>,
        road: Option<String>,
        level: Option<String>,
        // connectors: Option<String>,
    }
    let query_iter = stmt
        .query_map([], |row| {
            Ok(Transportation {
                id: row.get(0)?,
                geom: row.get(1)?,
                road: row.get(2)?,
                level: row.get(3)?,
                // connectors: row.get(2)?,
            })
        })
        .unwrap();

    let mut bevy_transportations: Vec<BevyTransportation> = vec![];
    for item in query_iter {
        let item = item.unwrap();

        if let Some(road) = &item.road {
            let p: Road = serde_json::from_str(road).expect("road");
            println!("- item.road: {:?}", road);
            println!("- item.road json: {:?}", p);
        }
        if let Some(level) = &item.level {
            println!("- item.level: {:?}", level);
        }

        let raw = item.geom;
        // println!(
        //     "statement for transportation - item geom: {raw:?}:l={}",
        //     raw.len()
        // );
        // MAGIC TO GET ARRAY THAT WORKS, COMPARED TO BINARY FROM PARQUET DIRECTLY
        // 0, 1, 104, 0, 0, 0, 0, 0, 1
        let raw = &raw[9..];
        let prefix: [u8; 2] = [1, 2];
        let raw = [prefix.as_slice(), &raw].concat();
        let mut rdr = std::io::Cursor::new(raw);
        let g = Geometry::from_wkb(&mut rdr, WkbDialect::Wkb);
        match g {
            Ok(g) => match g {
                Geometry::LineString(line_string) => {
                    let bevy_transportation =
                        line_string_road(line_string, params.k, params.translate);
                    bevy_transportations.push(bevy_transportation);
                    // dbg!(line_string);
                }
                Geometry::Polygon(polygon) => {
                    dbg!(polygon);
                    // let bevy_building =
                    //     polygon_building(polygon, base_k, base_pos, query_item.height);
                    // bevy_buildings.push(bevy_building);
                }
                not_polygon => {
                    dbg!(&not_polygon);
                }
            },
            Err(e) => {
                dbg!(e);
            }
        }
    }

    bevy_transportations
}
