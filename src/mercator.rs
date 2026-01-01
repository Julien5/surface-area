use crate::point::{MercatorPoint, WGS84Point};

pub struct WebMercatorProjection {
    wgs84: proj4rs::proj::Proj,
    dst: proj4rs::proj::Proj,
    dst_spec: String,
}

impl WebMercatorProjection {
    pub fn make(dst_spec: &str) -> WebMercatorProjection {
        assert!(!dst_spec.is_empty());
        use proj4rs::proj::Proj;
        let dst = Proj::from_proj_string(dst_spec).unwrap();
        let wgs84_spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let wgs84 = Proj::from_proj_string(wgs84_spec).unwrap();
        WebMercatorProjection {
            wgs84: wgs84,
            dst: dst,
            dst_spec: dst_spec.to_string(),
        }
    }
    pub fn project(&self, wgs: &WGS84Point) -> MercatorPoint {
        if wgs.to_utm_proj4() != self.dst_spec {
            log::warn!(
                "UTM zone mismatch: [{}] vs [{}]",
                wgs.to_utm_proj4(),
                self.dst_spec
            );
        }
        let mut p = (wgs.lon.to_radians(), wgs.lat.to_radians());
        proj4rs::transform::transform(&self.wgs84, &self.dst, &mut p).unwrap();
        MercatorPoint {
            x: p.0,
            y: p.1,
            ele: wgs.ele,
        }
    }
}
