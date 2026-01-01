use std::path::Path;

use crate::{
    mercator::WebMercatorProjection,
    point::{MercatorBoundingBox, MercatorPoint, WGS84BoundingBox, WGS84Point},
    polygon::Polygon,
};

pub struct Raster {
    upper_left: WGS84Point,
    xsize: usize,
    ysize: usize,
    xstep: f64,
    ystep: f64,
}

struct RasterBox {
    min: (isize, isize),
    max: (isize, isize),
}

impl Raster {
    pub fn make(dataset: &gdal::Dataset) -> Raster {
        let geo = dataset.geo_transform().unwrap();
        let raster_size = dataset.raster_size();
        // [0] Upper Left Easting (Longitude)
        // [1] Pixel Width
        // [2] Row Rotation (usually 0)
        // [3] Upper Left Northing (Latitude)
        // [4] Column Rotation (usually 0)
        // [5] Pixel Height (usually negative)
        Raster {
            upper_left: WGS84Point {
                lon: geo[0],
                lat: geo[3],
                ele: None,
            },
            xsize: raster_size.0,
            ysize: raster_size.1,
            xstep: geo[1],
            ystep: geo[5],
        }
    }
    pub fn coordinates(&self, world: &WGS84Point) -> (f64, f64) {
        let x = (world.lon - self.upper_left.lon) / self.xstep;
        let y = (world.lat - self.upper_left.lat) / self.ystep;
        (x, y)
    }

    pub fn icoordinates(&self, world: &WGS84Point) -> (isize, isize) {
        let x = (world.lon - self.upper_left.lon) / self.xstep;
        let y = (world.lat - self.upper_left.lat) / self.ystep;
        (x.round() as isize, y.round() as isize)
    }

    pub fn wgs84(&self, col: isize, row: isize) -> WGS84Point {
        let lon = self.upper_left.lon + (col as f64) * self.xstep;
        let lat = self.upper_left.lat + (row as f64) * self.ystep;
        WGS84Point {
            lon,
            lat,
            ele: None,
        }
    }
}

pub struct Dataset {
    pub filename: String,
    g: gdal::Dataset,
    raster: Raster,
    projection: String,
}

impl Dataset {
    pub fn open(filename: &String, projection: &String) -> Self {
        let path = Path::new(filename.as_str());
        let g = gdal::Dataset::open(path).unwrap();
        let transform = Raster::make(&g);
        Dataset {
            filename: filename.clone(),
            g,
            raster: transform,
            projection: projection.clone(),
        }
    }
    pub fn info(&self) {
        log::info!("dataset: {}", self.filename);
        log::info!("dataset: {}", self.projection);
        log::info!("dataset: xsize {}", self.raster.xsize);
        log::info!("dataset: ysize {}", self.raster.ysize);
        log::info!("dataset: xstep {:.5}", self.raster.xstep);
        log::info!("dataset: ystep {:.5}", self.raster.ystep);
        log::info!("dataset: wgs bbox: {}", self.wgsbbox());
        log::info!("dataset: mercator bbox: {}", self.mercatorbbox());
        log::info!("dataset: width: {:.1}", self.mercatorbbox().width());
        log::info!("dataset: height: {:.1}", self.mercatorbbox().height());
        log::info!("dataset: area: {:.1}", self.mercatorbbox().area());
    }
    pub fn wgsbbox(&self) -> WGS84BoundingBox {
        let (width, height) = self.g.raster_size();
        let geo = self.g.geo_transform().unwrap();
        let ul_lon = geo[0];
        let ul_lat = geo[3];
        let pixel_width = geo[1];
        let pixel_height = geo[5];
        let p1 = WGS84Point {
            lon: ul_lon,
            lat: ul_lat,
            ele: None,
        };
        let p2 = WGS84Point {
            lon: p1.lon + (width as f64 * pixel_width),
            lat: p1.lat + (height as f64 * pixel_height),
            ele: None,
        };

        WGS84BoundingBox {
            min: WGS84Point {
                lon: p1.lon.min(p2.lon),
                lat: p1.lat.min(p2.lat),
                ele: None,
            },
            max: WGS84Point {
                lon: p1.lon.max(p2.lon),
                lat: p1.lat.max(p2.lat),
                ele: None,
            },
        }
    }
    fn raster_box(&self, b: &WGS84BoundingBox) -> RasterBox {
        // Convert the intersection bbox to raster coordinates
        // For min point (top-left in intersection)
        let p1 = self.raster.coordinates(&b.min);
        let p2 = self.raster.coordinates(&b.max);
        let minpix = (p1.0.min(p2.0), p1.1.min(p2.1));
        let maxpix = (p1.0.max(p2.0), p1.1.max(p2.1));
        RasterBox {
            min: (minpix.0.floor() as isize, minpix.1.floor() as isize),
            max: (maxpix.0.ceil() as isize, maxpix.1.ceil() as isize),
        }
    }
    pub fn snap(&self, b: &mut WGS84BoundingBox) {
        let rb = self.raster_box(b);
        let p1 = self.raster.wgs84(rb.min.0, rb.min.1);
        let p2 = self.raster.wgs84(rb.max.0, rb.max.1);
        let ret = WGS84BoundingBox::from(&p1, &p2);
        b.min = ret.min;
        b.max = ret.max;
    }
    pub fn mercatorbbox(&self) -> MercatorBoundingBox {
        let projection = WebMercatorProjection::make(&self.projection);
        let wgs = self.wgsbbox();
        let min = projection.project(&wgs.min);
        let max = projection.project(&wgs.max);
        MercatorBoundingBox { min, max }
    }
}

impl Dataset {
    pub fn remove_redundant_datasets(datasets: &mut Vec<Dataset>) {
        let mut indices_to_remove = Vec::new();

        for (i1, dataset1) in datasets.iter().enumerate() {
            let bbox1 = dataset1.wgsbbox();
            for (i2, dataset2) in datasets.iter().enumerate() {
                if i1 == i2 {
                    continue;
                }
                let bbox2 = dataset2.wgsbbox();
                // Check if dataset1 should be removed:
                // - dataset1 has lower resolution
                // - bbox1 is contained in bbox2
                if dataset1.raster.xstep > dataset2.raster.xstep && bbox2.contains_other(&bbox1) {
                    log::trace!(
                        "discard {} (prefer {} instead)",
                        dataset1.filename,
                        dataset2.filename
                    );
                    indices_to_remove.push(i1);
                    break; // No need to check other datasets for this one
                }
            }
        }

        // Remove duplicates and sort in descending order to remove from the end
        indices_to_remove.sort_unstable();
        indices_to_remove.dedup();

        // Remove in reverse order to maintain correct indices
        for &idx in indices_to_remove.iter().rev() {
            datasets.remove(idx);
        }
    }

    pub fn select(polygon: &Polygon) -> Vec<Dataset> {
        let candidates = polygon.candidates();
        for filename in &candidates {
            log::trace!("found candidate: {}", filename);
        }
        let mut datasets: Vec<_> = candidates
            .iter()
            .map(|file| Dataset::open(file, &polygon.projection()))
            .collect();
        Self::remove_redundant_datasets(&mut datasets);
        let polybox = polygon.wgsbbox();
        datasets.retain(|dataset| {
            let databox = dataset.wgsbbox();
            let ret = databox.intersection(&polybox);
            if ret.is_none() {
                log::trace!("discard {} (bbox)", dataset.filename);
            }
            ret.is_some()
        });
        datasets
    }

    pub fn points_inside(&self, snapped_box: &WGS84BoundingBox) -> Vec<MercatorPoint> {
        let mut ret = Vec::new();

        let dataset_bbox = self.wgsbbox();
        let intersection = dataset_bbox.intersection(snapped_box);
        if intersection.is_none() {
            assert!(false);
            return ret;
        }
        let inter = intersection.unwrap();

        // the input box is snapped => we want integer coordinates.
        let p1 = self.raster.icoordinates(&inter.min);
        let p2 = self.raster.icoordinates(&inter.max);

        let minpix = (p1.0.min(p2.0), p1.1.min(p2.1));
        let maxpix = (p1.0.max(p2.0), p1.1.max(p2.1));

        log::trace!("minpix:{:?}", minpix);
        let col_start = minpix.0;
        let row_start = minpix.1;

        log::trace!("maxpix:{:?}", maxpix);
        let col_end = maxpix.0;
        let row_end = maxpix.1;

        // Clamp to valid raster bounds
        assert!(col_start >= 0);
        assert!(row_start >= 0);
        //assert!(col_end < self.raster.xsize as isize);
        //assert!(row_end < self.raster.ysize as isize);
        let col_end = col_end.min((self.raster.xsize - 1) as isize);
        let row_end = row_end.min((self.raster.ysize - 1) as isize);
        let projection = WebMercatorProjection::make(&self.projection);
        //log::info!("row: {row_start}..{row_end}");
        //log::info!("col: {col_start}..{col_end}");

        // Read the elevation data for the region of interest
        let rasterband = self.g.rasterband(1).expect("Failed to get rasterband");
        let window_xsize = (col_end - col_start + 1) as usize;
        let window_ysize = (row_end - row_start + 1) as usize;

        let window = (col_start as isize, row_start as isize);
        let window_size = (window_xsize, window_ysize);

        let buffer = rasterband
            .read_as::<f64>(window, window_size, window_size, None)
            .expect("Failed to read raster data");

        for row in row_start..=row_end {
            for col in col_start..=col_end {
                let mut wgs = self.raster.wgs84(col, row);
                if !snapped_box.contains_point(&wgs) {
                    log::trace!("bbox:{}", snapped_box);
                    log::trace!("point:{}", wgs);
                }
                assert!(snapped_box.contains_point(&wgs));

                // Calculate buffer index
                let buffer_col = (col - col_start) as usize;
                let buffer_row = (row - row_start) as usize;
                let buffer_index = buffer_row * window_xsize + buffer_col;

                wgs.ele = Some(buffer.data()[buffer_index]);

                let mercator = projection.project(&wgs);
                ret.push(mercator);
            }
        }

        ret
    }
}
