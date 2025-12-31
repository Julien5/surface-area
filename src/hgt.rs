use crate::point::WGS84Point;

pub fn hgt_basename_lonlat(lon: f64, lat: f64) -> String {
    // SRTM tiles are named by the coordinates of their southwest corner
    // Floor the coordinates to get the tile's southwest corner
    let lat_floor = lat.floor() as i32;
    let lon_floor = lon.floor() as i32;

    // Determine latitude prefix (N or S)
    let lat_prefix = if lat_floor >= 0 { 'N' } else { 'S' };
    let lat_abs = lat_floor.abs();

    // Determine longitude prefix (E or W)
    let lon_prefix = if lon_floor >= 0 { 'E' } else { 'W' };
    let lon_abs = lon_floor.abs();

    // Format: N/S + 2-digit latitude + E/W + 3-digit longitude + .hgt
    format!(
        "{}{:02}{}{:03}.hgt",
        lat_prefix, lat_abs, lon_prefix, lon_abs
    )
}

pub fn hgt_basename(point: &WGS84Point) -> String {
    hgt_basename_lonlat(point.lon, point.lat)
}

#[cfg(test)]
mod tests {
    use super::hgt_basename;
    use super::*;
    use crate::point::WGS84Point;

    #[test]
    fn test_positive_coordinates() {
        assert_eq!(
            hgt_basename(&WGS84Point {
                lon: 98.5,
                lat: 3.7,
                ele: None
            }),
            "N03E098.hgt"
        );
        assert_eq!(hgt_basename_lonlat(72.3, 4.2), "N04E072.hgt");
    }

    #[test]
    fn test_negative_latitude() {
        assert_eq!(hgt_basename_lonlat(46.8, -23.5), "S24E046.hgt");
    }

    #[test]
    fn test_negative_longitude() {
        assert_eq!(hgt_basename_lonlat(-46.8, 23.5), "N23W047.hgt");
    }

    #[test]
    fn test_both_negative() {
        assert_eq!(hgt_basename_lonlat(-46.8, -23.5), "S24W047.hgt");
    }

    #[test]
    fn test_zero_coordinates() {
        assert_eq!(hgt_basename_lonlat(0.5, 0.5), "N00E000.hgt");
    }

    #[test]
    fn test_edge_cases() {
        assert_eq!(hgt_basename_lonlat(180.0, 90.0), "N90E180.hgt");
        assert_eq!(hgt_basename_lonlat(-180.0, -90.0), "S90W180.hgt");
    }

    #[test]
    fn test_domrep() {
        assert_eq!(
            hgt_basename_lonlat(-69.14255483688711, 18.882967637045482),
            "N18W070.hgt"
        );
    }
}
