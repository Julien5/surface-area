use core::fmt;

use crate::point::MercatorPoint;

#[derive(Clone)]
pub struct Triangle(pub MercatorPoint, pub MercatorPoint, pub MercatorPoint);

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A:{} B:{} C:{}", self.0, self.1, self.2)
    }
}

impl Triangle {
    pub fn flat(&self) -> Self {
        Triangle(self.0.flat(), self.1.flat(), self.2.flat())
    }
    pub fn as_vector(&self) -> Vec<MercatorPoint> {
        let mut ret = Vec::new();
        ret.push(self.0.clone());
        ret.push(self.1.clone());
        ret.push(self.2.clone());
        ret
    }
    pub fn area(&self) -> f64 {
        let p1 = &self.0;
        let p2 = &self.1;
        let p3 = &self.2;

        // Vector AB
        let ax = p2.x - p1.x;
        let ay = p2.y - p1.y;
        let az = p2.ele.unwrap() - p1.ele.unwrap();

        // Vector AC
        let bx = p3.x - p1.x;
        let by = p3.y - p1.y;
        let bz = p3.ele.unwrap() - p1.ele.unwrap();

        // Cross product components
        let cx = ay * bz - az * by;
        let cy = az * bx - ax * bz;
        let cz = ax * by - ay * bx;

        // 0.5 * magnitude of cross product
        0.5 * (cx * cx + cy * cy + cz * cz).sqrt()
    }
}

pub mod grid {
    use super::Triangle;
    use crate::point::MercatorPoint;
    use geo::Coord;
    use spade::{DelaunayTriangulation, Point2, Triangulation};

    pub fn triangulate(points: &[MercatorPoint]) -> Vec<Triangle> {
        if points.len() < 3 {
            return Vec::new();
        }

        // Create Delaunay triangulation
        let spade_points: Vec<Point2<f64>> = points.iter().map(|p| Point2::new(p.x, p.y)).collect();

        let mut triangulation = DelaunayTriangulation::<Point2<f64>>::new();

        for point in spade_points {
            triangulation.insert(point).ok();
        }

        // Extract triangles
        let mut triangles = Vec::new();
        for face in triangulation.inner_faces() {
            let [v1, v2, v3] = face.vertices();
            let p1 = find_matching_point(
                points,
                &Coord {
                    x: v1.position().x,
                    y: v1.position().y,
                },
            );
            let p2 = find_matching_point(
                points,
                &Coord {
                    x: v2.position().x,
                    y: v2.position().y,
                },
            );
            let p3 = find_matching_point(
                points,
                &Coord {
                    x: v3.position().x,
                    y: v3.position().y,
                },
            );

            triangles.push(Triangle(p1, p2, p3));
        }

        triangles
    }

    // Helper function to find the original MercatorPoint that matches a coordinate
    fn find_matching_point(points: &[MercatorPoint], coord: &Coord<f64>) -> MercatorPoint {
        let eps = 1e-10;

        for point in points {
            if (point.x - coord.x).abs() < eps && (point.y - coord.y).abs() < eps {
                return point.clone();
            }
        }
        assert!(false);

        // If no exact match found (shouldn't happen), create a new point without elevation
        MercatorPoint {
            x: coord.x,
            y: coord.y,
            ele: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::grid::triangulate;
    use super::*;
    use crate::point::MercatorPoint;
    use crate::svg;

    fn drawresult(triangles: &Vec<Triangle>, filename: &str) -> String {
        let mut svg = svg::SVG::init(&crate::point::MercatorBoundingBox {
            min: MercatorPoint {
                x: 0.0,
                y: 0.0,
                ele: None,
            },
            max: MercatorPoint {
                x: 500.0,
                y: 500.0,
                ele: None,
            },
        });
        svg.add_triangles(&triangles, true);
        let ret = svg.render();
        std::fs::write(filename, ret.clone()).unwrap();
        ret
    }

    // Helper function to generate random points
    fn generate_random_grid(count: usize) -> Vec<MercatorPoint> {
        // Simple linear congruential generator for reproducible randomness
        let mut rng_state = 0usize;
        let mut next_random = || {
            rng_state = (rng_state.wrapping_mul(1664525).wrapping_add(1013904223)) % (1 << 32);
            (rng_state as f64) / ((1u64 << 32) as f64)
        };
        let step = 350.0 / count as f64;
        let mut points = Vec::new();
        for nx in 0..count {
            for ny in 0..count {
                points.push(MercatorPoint {
                    x: nx as f64 * step + next_random() * step / 2.0,
                    y: ny as f64 * step + next_random() * step / 2.0,
                    ele: Some(next_random() * 100.0),
                });
            }
        }
        points
    }

    #[test]
    fn test_triangulate_simple_triangle() {
        let points = vec![
            MercatorPoint {
                x: 0.0,
                y: 0.0,
                ele: Some(0.0),
            },
            MercatorPoint {
                x: 100.0,
                y: 0.0,
                ele: Some(10.0),
            },
            MercatorPoint {
                x: 50.0,
                y: 100.0,
                ele: Some(20.0),
            },
        ];

        let triangles = triangulate(&points);
        drawresult(&triangles, "/tmp/simple.svg");
        assert_eq!(triangles.len(), 1);
    }

    #[test]
    fn test_triangulate_square() {
        let points = vec![
            MercatorPoint {
                x: 0.0,
                y: 0.0,
                ele: Some(0.0),
            },
            MercatorPoint {
                x: 100.0,
                y: 0.0,
                ele: Some(10.0),
            },
            MercatorPoint {
                x: 100.0,
                y: 100.0,
                ele: Some(20.0),
            },
            MercatorPoint {
                x: 0.0,
                y: 100.0,
                ele: Some(30.0),
            },
        ];

        let triangles = triangulate(&points);
        drawresult(&triangles, "/tmp/square.svg");
        assert_eq!(triangles.len(), 2);
    }

    #[test]
    fn test_triangulate_convex_polygon() {
        let points = vec![
            MercatorPoint {
                x: 0.0,
                y: 0.0,
                ele: Some(0.0),
            },
            MercatorPoint {
                x: 400.0,
                y: 0.0,
                ele: Some(10.0),
            },
            MercatorPoint {
                x: 400.0,
                y: 300.0,
                ele: Some(20.0),
            },
            MercatorPoint {
                x: 0.0,
                y: 300.0,
                ele: Some(30.0),
            },
        ];

        let triangles = triangulate(&points);
        drawresult(&triangles, "/tmp/convex.svg");
        assert_eq!(triangles.len(), 2);
    }

    #[test]
    fn test_triangulate_random_grid() {
        let points = generate_random_grid(20);
        let triangles = triangulate(&points);
        drawresult(&triangles, "/tmp/random-grid.svg");
    }
}
