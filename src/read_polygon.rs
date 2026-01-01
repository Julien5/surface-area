use crate::point::WGS84Point;
use crate::polygon::Polygon;
use kml::types::Geometry;
use kml::Kml;
use std::fs::File;
use std::io::Read;

mod lockml {
    use super::*;
    fn find_first_line_string(kml: &Kml) -> Option<geo::Polygon> {
        match kml {
            Kml::KmlDocument(doc) => doc.elements.iter().find_map(find_first_line_string),
            Kml::Document { elements, .. } => elements.iter().find_map(find_first_line_string),
            Kml::Folder(z) => z.elements.iter().find_map(find_first_line_string),
            Kml::Placemark(p) => {
                if let Some(Geometry::Polygon(ls)) = &p.geometry {
                    // Convert kml::types::LineString to geo::LineString
                    // This requires the 'geo-types' feature (enabled by default in kml crate)
                    Some(geo::Polygon::from(ls.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn read(content: &str) -> Vec<Polygon> {
        // 2. Parse KML string
        let kml: Kml = content.parse().unwrap();

        // 3. Extract LineString from the KML structure
        // KML can be complex (folders, multiple placemarks),
        // so we need a recursive helper or a find-first logic.
        let geo_geometry = find_first_line_string(&kml)
            .ok_or("No LineString found in the KML file")
            .unwrap();
        let wgs: Vec<_> = geo_geometry
            .exterior()
            .0
            .iter()
            .map(|p| WGS84Point {
                lon: p.x,
                lat: p.y,
                ele: None,
            })
            .collect();
        vec![Polygon { wgs }]
    }
}

mod locgpx {
    use super::*;
    use gpx::Gpx;
    use std::io::Cursor;

    pub fn read(content: &str) -> Vec<Polygon> {
        // Parse the GPX content
        let gpx: Gpx = gpx::read(Cursor::new(content)).expect("Failed to parse GPX content");

        // Extract track segments and convert them into polygons
        gpx.tracks
            .iter()
            .flat_map(|track| {
                track.segments.iter().map(|segment| {
                    let wgs: Vec<WGS84Point> = segment
                        .points
                        .iter()
                        .map(|point| WGS84Point {
                            lon: point.point().x(),
                            lat: point.point().y(),
                            ele: None,
                        })
                        .collect();
                    Polygon { wgs }
                })
            })
            .collect()
    }
}

mod locjson {
    use super::*;
    use geojson::{GeoJson, Geometry, Value};

    pub fn read(content: &str) -> Vec<Polygon> {
        // Parse the GeoJSON content
        let geojson: GeoJson = content.parse().expect("Failed to parse GeoJSON content");

        // Extract polygons from the GeoJSON
        match geojson {
            GeoJson::FeatureCollection(collection) => collection
                .features
                .iter()
                .filter_map(|feature| {
                    if let Some(geometry) = &feature.geometry {
                        geometry_to_polygon(geometry)
                    } else {
                        None
                    }
                })
                .collect(),
            GeoJson::Feature(feature) => {
                if let Some(geometry) = feature.geometry {
                    geometry_to_polygon(&geometry).into_iter().collect()
                } else {
                    vec![]
                }
            }
            GeoJson::Geometry(geometry) => geometry_to_polygon(&geometry).into_iter().collect(),
        }
    }

    fn geometry_to_polygon(geometry: &Geometry) -> Option<Polygon> {
        match &geometry.value {
            Value::Polygon(coords) => {
                let wgs: Vec<WGS84Point> = coords[0]
                    .iter()
                    .map(|p| WGS84Point {
                        lon: p[0],
                        lat: p[1],
                        ele: None,
                    })
                    .collect();
                Some(Polygon { wgs })
            }
            Value::MultiPolygon(multi_coords) => {
                // Flatten the first polygon in the MultiPolygon
                if let Some(coords) = multi_coords.first() {
                    let wgs: Vec<WGS84Point> = coords[0]
                        .iter()
                        .map(|p| WGS84Point {
                            lon: p[0],
                            lat: p[1],
                            ele: None,
                        })
                        .collect();
                    Some(Polygon { wgs })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub fn read_polyline(filename: &str) -> Vec<Polygon> {
    // 1. Read file content
    let mut file = File::open(filename).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    if filename.ends_with("kml") {
        return lockml::read(&content);
    } else if filename.ends_with("gpx") {
        return locgpx::read(&content);
    } else if filename.ends_with("geojson") {
        return locjson::read(&content);
    }
    Vec::new()
}
