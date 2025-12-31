use geo::algorithm::geodesic_area::GeodesicArea;
use geo::orient::Direction;
use geo::{Area, Coord, Orient, Polygon};

use crate::point::{MercatorPoint, WGS84Point};

pub fn geodesic_area(polygon: &Vec<WGS84Point>) -> f64 {
    if polygon.len() < 3 {
        return 0.0;
    }
    // Convert WGS84Point to geo::Coord
    let coords: Vec<Coord<f64>> = polygon
        .iter()
        .map(|p| Coord { x: p.lon, y: p.lat })
        .collect();

    // Create a geo::Polygon (exterior ring, no holes)
    let mut geo_polygon = Polygon::new(coords.into(), vec![]);
    geo_polygon = geo_polygon.orient(Direction::Default);

    // Compute geodesic area in square meters
    geo_polygon.geodesic_area_unsigned()
}

pub fn planar_area(polygon: &Vec<MercatorPoint>) -> f64 {
    if polygon.len() < 3 {
        return 0.0;
    }
    // Convert WGS84Point to geo::Coord
    let coords: Vec<Coord<f64>> = polygon.iter().map(|p| Coord { x: p.x, y: p.y }).collect();

    // Create a geo::Polygon (exterior ring, no holes)
    let mut geo_polygon = Polygon::new(coords.into(), vec![]);
    geo_polygon = geo_polygon.orient(Direction::Default);

    // Compute geodesic area in square meters
    geo_polygon.unsigned_area()
}
