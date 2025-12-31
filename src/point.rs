use core::fmt;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct WGS84Point {
    pub lon: f64,
    pub lat: f64,
    pub ele: Option<f64>,
}

impl WGS84Point {
    pub fn in_epsg32619(&self) -> bool {
        -72.0 <= self.lon && self.lon <= -66.0 && 0.0 <= self.lat && self.lat <= 84.0
    }
}

impl fmt::Display for WGS84Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let e = match self.ele {
            Some(z) => format!("{:.1}", z),
            None => "None".to_string(),
        };
        write!(
            f,
            "wgs(lat: {:.5}, lon: {:.5}, ele: {})",
            self.lat, self.lon, e
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MercatorPoint {
    pub x: f64,
    pub y: f64,
    pub ele: Option<f64>,
}

impl MercatorPoint {
    pub fn flat(&self) -> Self {
        let mut ret = self.clone();
        ret.ele = Some(0f64);
        ret
    }
    pub fn x_y(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl fmt::Display for MercatorPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let e = match self.ele {
            Some(z) => format!("{:.1}", z),
            None => "None".to_string(),
        };
        write!(f, "mercator(x: {:.5}, y: {:.5}, z: {})", self.x, self.y, e)
    }
}

impl Eq for MercatorPoint {}

impl PartialOrd for MercatorPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MercatorPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare lon first, then lat (Lexicographical order)
        self.x
            .total_cmp(&other.x)
            .then_with(|| self.y.total_cmp(&other.y))
    }
}

#[derive(Clone, Debug)]
pub struct WGS84BoundingBox {
    pub min: WGS84Point,
    pub max: WGS84Point,
}

impl WGS84BoundingBox {
    pub fn from(p1: &WGS84Point, p2: &WGS84Point) -> Self {
        let min = WGS84Point {
            lon: p1.lon.min(p2.lon),
            lat: p1.lat.min(p2.lat),
            ele: None,
        };
        let max = WGS84Point {
            lon: p1.lon.max(p2.lon),
            lat: p1.lat.max(p2.lat),
            ele: None,
        };
        Self { min, max }
    }
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        // Calculate the intersection bounds
        let min_lon = self.min.lon.max(other.min.lon);
        let min_lat = self.min.lat.max(other.min.lat);
        let max_lon = self.max.lon.min(other.max.lon);
        let max_lat = self.max.lat.min(other.max.lat);

        // Check if the intersection is valid (boxes actually overlap)
        if min_lon <= max_lon && min_lat <= max_lat {
            Some(WGS84BoundingBox {
                min: WGS84Point {
                    lon: min_lon,
                    lat: min_lat,
                    ele: None,
                },
                max: WGS84Point {
                    lon: max_lon,
                    lat: max_lat,
                    ele: None,
                },
            })
        } else {
            // No intersection exists
            None
        }
    }
    pub fn contains(&self, w: &WGS84Point) -> bool {
        w.lon >= self.min.lon
            && w.lon <= self.max.lon
            && w.lat >= self.min.lat
            && w.lat <= self.max.lat
    }
}

impl fmt::Display for WGS84BoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wgsbbox(min: {}, max: {})", self.min, self.max)
    }
}

#[derive(Clone, Debug)]
pub struct MercatorBoundingBox {
    pub min: MercatorPoint,
    pub max: MercatorPoint,
}

impl MercatorBoundingBox {
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
    pub fn area(&self) -> f64 {
        self.width() * self.height()
    }
}
impl fmt::Display for MercatorBoundingBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Customize your output format here
        write!(f, "wgsbbox(min: {}, max: {})", self.min, self.max)
    }
}
