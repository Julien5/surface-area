use crate::point::{MercatorPoint, WGS84Point};

pub struct WebMercatorProjection {
    wgs84_spec: proj4rs::proj::Proj,
    dst_spec: proj4rs::proj::Proj,
}

impl WebMercatorProjection {
    pub fn make() -> WebMercatorProjection {
        // The PROJ.4 parameters for EPSG:3857 (also known as Web Mercator or Pseudo-Mercator) are:
        // +proj=merc +lon_0=0 +k=1 +x_0=0 +y_0=0 +datum=WGS84 +units=m +no_defs
        // https://gis.stackexchange.com/questions/159572/proj4-for-epsg3857
        use proj4rs::proj::Proj;
        /*let spec = format!(
                    "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0 +k=1.0 +units=m +nadgrids=@null +wktext  +no_defs"
        );*/
        // EPSG:32619 (domrep)
        let spec = "+proj=utm +zone=19 +datum=WGS84 +units=m +no_defs +type=crs".to_string();
        let dst_spec = Proj::from_proj_string(spec.as_str()).unwrap();

        let spec = "+proj=longlat +ellps=WGS84 +datum=WGS84 +no_defs";
        let wgs84_spec = Proj::from_proj_string(spec).unwrap();
        WebMercatorProjection {
            wgs84_spec,
            dst_spec,
        }
    }
    pub fn project(&self, wgs: &WGS84Point) -> MercatorPoint {
        if !wgs.in_epsg32619() {
            log::warn!("not in epsg: {}", wgs);
        }
        let mut p = (wgs.lon.to_radians(), wgs.lat.to_radians());
        proj4rs::transform::transform(&self.wgs84_spec, &self.dst_spec, &mut p).unwrap();
        MercatorPoint {
            x: p.0,
            y: p.1,
            ele: wgs.ele,
        }
    }
}
