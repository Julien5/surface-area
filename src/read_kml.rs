use crate::point::WGS84Point;
use crate::polygon::Polygon;
use kml::types::Geometry;
use kml::Kml;
use std::fs::File;
use std::io::Read;

pub fn read_polyline(filename: &str) -> Polygon {
    // 1. Read file content
    let mut file = File::open(filename).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

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
    Polygon { wgs }
}

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
