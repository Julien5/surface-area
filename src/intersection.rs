use crate::{point::MercatorPoint, triangulation::Triangle};

// Helper function to compute barycentric coordinates
fn barycentric_coords(p: &MercatorPoint, t: &Triangle) -> (f64, f64, f64) {
    let v0x = t.1.x - t.0.x;
    let v0y = t.1.y - t.0.y;
    let v1x = t.2.x - t.0.x;
    let v1y = t.2.y - t.0.y;
    let v2x = p.x - t.0.x;
    let v2y = p.y - t.0.y;

    let den = v0x * v1y - v1x * v0y;
    let v = (v2x * v1y - v1x * v2y) / den;
    let w = (v0x * v2y - v2x * v0y) / den;
    let u = 1.0 - v - w;

    (u, v, w)
}

// Interpolate elevation using barycentric coordinates
fn interpolate_elevation(p: &MercatorPoint, t: &Triangle) -> Option<f64> {
    let (u, v, w) = barycentric_coords(p, t);

    match (t.0.ele, t.1.ele, t.2.ele) {
        (Some(e0), Some(e1), Some(e2)) => Some(u * e0 + v * e1 + w * e2),
        _ => None,
    }
}

use geo::{BooleanOps, Coord, LineString, MultiPolygon};

fn to_geo_polygon(points: &[MercatorPoint]) -> geo::Polygon {
    // 1. Convert MercatorPoints to geo::Coord
    let mut coords: Vec<Coord<f64>> = points.iter().map(|p| Coord { x: p.x, y: p.y }).collect();

    // 2. Ensure the ring is closed
    // geo-types LineStrings must have the same first and last point to be a valid ring
    if let (Some(first), Some(last)) = (coords.first(), coords.last()) {
        if first != last {
            coords.push(*first);
        }
    }

    // 3. Create the LineString (the exterior boundary)
    let exterior = LineString::new(coords);

    // 4. Create the Polygon (with no interior holes)
    let ret = geo::Polygon::new(exterior, vec![]);
    use geo::orient::{Direction, Orient};
    ret.orient(Direction::Default)
}

fn multipolygon_to_mercator(multi_poly: &MultiPolygon<f64>) -> Vec<MercatorPoint> {
    multi_poly
        .into_iter() // Iterates over each Polygon
        .flat_map(|poly| {
            let (exterior, _interiors) = poly.clone().into_inner();
            exterior.into_iter().map(|coord| MercatorPoint {
                x: coord.x,
                y: coord.y,
                ele: None, // Elevation is lost in geo-types
            })
        })
        .collect()
}

pub fn intersection(polygon: &Vec<MercatorPoint>, triangle: &Triangle) -> Vec<MercatorPoint> {
    let p1 = to_geo_polygon(&polygon);
    let p2 = to_geo_polygon(&triangle.as_vector());
    let p1_clean = p1.union(&p1);
    let p2_clean = p2.union(&p2);

    let m = p1_clean.intersection(&p2_clean);
    let mut ret = multipolygon_to_mercator(&m);

    for p in &mut ret {
        p.ele = interpolate_elevation(p, triangle);
    }
    ret
}
