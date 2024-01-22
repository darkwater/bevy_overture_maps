use duckdb::Connection;

pub fn cache_location(lon: f64, lat: f64, name: &str) {
    let shift = 0.01;
    let lon_max = lon + shift;
    let lon_min = lon - shift;
    let lat_max = lat + shift;
    let lat_min = lat - shift;

    // let polygon_str = format!("{lon_min} {lat_min}, {lon_min} {lat_max}, {lon_max} {lat_max}, {lon_max} {lat_min}, {lon_min} {lat_min}", );
    // let where_str = format!("ST_Within(ST_GeomFromWkb(geometry), ST_Envelope(ST_GeomFromText('POLYGON(({polygon_str}))')))");
    let where_str = format!("bbox.minX > {lon_min} AND bbox.minY > {lat_min} AND bbox.maxX < {lon_max} AND bbox.maxY < {lat_max}");

    let path = "./data.duckdb";
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch("INSTALL httpfs; LOAD httpfs;").unwrap();
    conn.execute_batch("INSTALL spatial; LOAD spatial;")
        .unwrap();

    let lonlatname = format!("{lon}_{lat}_{name}");

    let from_segment =
        "read_parquet('/mnt/overture/theme=transportation/type=segment/*')".to_string();
    let mut stmt = conn
        .prepare(&format!(
            "COPY (SELECT * FROM {from_segment} WHERE {where_str})
            TO 'parquet/{lonlatname}_transportation.parquet' (FORMAT 'parquet')"
        ))
        .unwrap();
    let _ = stmt.query([]).unwrap();

    let from_building = "read_parquet('/mnt/overture/theme=buildings/type=building/*')".to_string();
    let mut stmt = conn
        .prepare(&format!(
            "COPY (SELECT * FROM {from_building} WHERE {where_str})
            TO 'parquet/{lonlatname}_building.parquet' (FORMAT 'parquet')"
        ))
        .unwrap();
    let _ = stmt.query([]).unwrap();
}
