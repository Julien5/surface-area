use std::collections::BTreeSet;

use crate::{
    mercator::WebMercatorProjection,
    point::{MercatorBoundingBox, MercatorPoint, WGS84BoundingBox, WGS84Point},
};

pub struct Polygon {
    pub wgs: Vec<WGS84Point>,
}

impl Polygon {
    pub fn info(&self) {
        log::info!("polygon: len: {}", self.wgs.len());
        log::info!("polygon: wgs bbox: {}", self.wgsbbox());
        log::info!("polygon: mercator bbox: {}", self.mercatorbbox());
        log::info!("polygon: width: {:.1}", self.mercatorbbox().width());
        log::info!("polygon: height: {:.1}", self.mercatorbbox().height());
        log::info!("polygon: area: {:.1}", self.mercatorbbox().area());
        let d = self
            .datasets()
            .iter()
            .cloned()
            .collect::<Vec<String>>()
            .join(", ");
        log::info!("polygon: datasets: {}", d);
    }

    pub fn wgsbbox(&self) -> WGS84BoundingBox {
        let (min, max) = self.wgs.iter().fold(
            (self.wgs[0].clone(), self.wgs[0].clone()), // Initialize with the first coordinate
            |(min, max), curr| {
                (
                    WGS84Point {
                        lon: min.lon.min(curr.lon),
                        lat: min.lat.min(curr.lat),
                        ele: None,
                    }, // New min
                    WGS84Point {
                        lon: max.lon.max(curr.lon),
                        lat: max.lat.max(curr.lat),
                        ele: None,
                    }, // New
                )
            },
        );
        WGS84BoundingBox { min, max }
    }

    pub fn mercatorbbox(&self) -> MercatorBoundingBox {
        let mercatorpoints = self.mercator();
        let (min, max) = mercatorpoints.iter().fold(
            (mercatorpoints[0].clone(), mercatorpoints[0].clone()), // Initialize with the first coordinate
            |(min, max), curr| {
                (
                    MercatorPoint {
                        x: min.x.min(curr.x),
                        y: min.y.min(curr.y),
                        ele: None,
                    }, // New min
                    MercatorPoint {
                        x: max.x.max(curr.x),
                        y: max.y.max(curr.y),
                        ele: None,
                    }, // New
                )
            },
        );
        MercatorBoundingBox { min, max }
    }
    pub fn projection(&self) -> String {
        assert!(!self.wgs.is_empty());
        let wgs0 = self.wgs.first().unwrap().clone();
        wgs0.to_utm_proj4()
    }
    pub fn mercator(&self) -> Vec<MercatorPoint> {
        let proj = WebMercatorProjection::make(&self.projection());
        self.wgs.iter().map(|w| proj.project(&w)).collect()
    }
    pub fn datasets(&self) -> BTreeSet<String> {
        return dataset::datasets(&self);
    }
}

pub fn flat(polygon: &Vec<MercatorPoint>) -> Vec<MercatorPoint> {
    polygon.iter().map(|w| w.flat()).collect()
}

pub fn calculate_3d_surface_area(polygon: &Vec<MercatorPoint>) -> f64 {
    if polygon.len() < 3 {
        return 0.0;
    }

    let mut total_vec_x = 0.0;
    let mut total_vec_y = 0.0;
    let mut total_vec_z = 0.0;

    for i in 0..polygon.len() {
        let p1 = &polygon[i];
        let p2 = &polygon[(i + 1) % polygon.len()];

        // Elevation is treated as Z. We use 0.0 if ele is None.
        let z1 = p1.ele.unwrap();
        let z2 = p2.ele.unwrap();

        // Cross product components: (p1 x p2)
        total_vec_x += (p1.y * z2) - (z1 * p2.y);
        total_vec_y += (z1 * p2.x) - (p1.x * z2);
        total_vec_z += (p1.x * p2.y) - (p1.y * p2.x);
    }

    // The magnitude of the sum of cross products
    let magnitude = (total_vec_x.powi(2) + total_vec_y.powi(2) + total_vec_z.powi(2)).sqrt();

    magnitude / 2.0
}

mod dataset {
    use super::Polygon;
    use std::collections::BTreeSet;
    use std::env;

    pub fn datasetstring(s: &String) -> String {
        if s.contains(&"GL1") {
            "/home/julien/DEM/SRTM/GL1/S2/output_SRTMGL1.tif".to_string()
        } else if s.contains("HGT") {
            "/home/julien/DEM/SRTM/GL3/hgt/N18W070.hgt".to_string()
        } else {
            String::new()
        }
    }

    fn datasetsenv() -> Vec<String> {
        match env::var("DATASETS") {
            Ok(val) => return val.split(",").map(|s| s.to_string()).collect(),
            Err(_) => {}
        }
        Vec::new()
    }

    pub fn datasets(polygon: &Polygon) -> BTreeSet<String> {
        let ret1: BTreeSet<String> = datasetsenv().iter().map(|s| datasetstring(s)).collect();
        if !ret1.is_empty() {
            return ret1;
        }

        let mut ret = BTreeSet::new();
        let hgtdir = "/home/julien/DEM/SRTM/GL3/hgt";
        let htg: BTreeSet<String> = polygon
            .wgs
            .iter()
            .map(|w| format!("{}/{}", hgtdir, crate::hgt::hgt_basename(w)))
            .collect();
        for h in &htg {
            ret.insert(h.clone());
        }

        ret.insert("/home/julien/DEM/SRTM/GL1/S1/output_SRTMGL1.tif".to_string());
        ret.insert("/home/julien/DEM/SRTM/GL1/S2/output_SRTMGL1.tif".to_string());
        ret
    }
}
