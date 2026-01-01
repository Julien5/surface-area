use clap::Parser;
use std::collections::BTreeSet;
use std::path::Path;
use surface_area::dataset::Dataset;
use surface_area::point::MercatorPoint;
use surface_area::polygon::Polygon;
use surface_area::{intersection, polygon, read_polygon, reference, svg, triangulation, typst};

fn process(name: String, input_polygon: &Polygon) -> typst::Data {
    input_polygon.info();
    let pbbox = input_polygon.wgsbbox();
    let mut gridpoints = BTreeSet::new();
    for dataset in Dataset::select(&input_polygon) {
        dataset.info();
        let dbbox = dataset.wgsbbox();
        if let Some(mut bbox) = pbbox.intersection(&dbbox) {
            log::trace!("bbox: {}", bbox);
            dataset.snap(&mut bbox);
            log::trace!("snap: {}", bbox);
            let mercpoints = dataset.points_inside(&bbox);
            for m in &mercpoints {
                gridpoints.insert(m.clone());
            }
        }
    }

    log::trace!("gridpoints: {}", gridpoints.len());

    let gridvec: Vec<MercatorPoint> = gridpoints.into_iter().collect();
    let gridtriangles = triangulation::grid::triangulate(&gridvec);
    log::trace!("grid triangles: {}", gridtriangles.len());

    let polygon = input_polygon.mercator();

    let mut svg = svg::SVG::init(&input_polygon.mercatorbbox());
    //svg.add_polygon(&input_polygon.mercatorbbox().as_vector(), "gray");
    svg.add_polygon(&polygon, "gray");
    let mut planes = Vec::new();
    let mut projected3d = 0f64;
    let mut projected2d = 0f64;
    for (_i, gridtriangle) in gridtriangles.iter().enumerate() {
        let plane = intersection::intersection(&polygon, &gridtriangle);
        if plane.is_empty() {
            continue;
        }
        let a2d = polygon::calculate_3d_surface_area(&polygon::flat(&plane));
        let a3d = polygon::calculate_3d_surface_area(&plane);
        if a2d < 0.001 {
            log::info!("remove artifact with area {:.4}", a2d);
            continue;
        }
        // let rat = 100.0 * (a3d / a2d - 1.0);
        // log::trace!("plane area: {:6.2} {:6.2} +{:3.1}%", a3d, a2d, rat);
        projected3d += a3d;
        projected2d += a2d;
        planes.push(plane.clone());
        svg.add_polygon(&plane, &svg::color_for_slope(polygon::slope(&plane)));
    }

    log::trace!("planes: {}", planes.len());
    let geodesic2d = reference::geodesic_area(&input_polygon.wgs);
    let planar2d = reference::planar_area(&input_polygon.mercator());
    println!("geodesic: {:.1} (geo crate)", geodesic2d);
    println!("  planar: {:.1} (geo crate)", planar2d);
    println!();
    println!("    flat: {:.1}", projected2d);
    println!(" surface: {:.1}", projected3d);
    let ratio = projected3d / projected2d;
    println!("   ratio: +{:.1}%", (ratio - 1f64) * 100f64);
    println!();
    let geodesic3d = ratio * reference::geodesic_area(&input_polygon.wgs);
    println!("estimate: {:.1}", geodesic3d);

    //svg.add_polygon(&polygon, "none");
    //svg.add_triangles(&atoms, true);
    //svg.add_triangles(&triangulation::polygon::triangulate(&polygon), true);
    //svg.add_triangles(&gridtriangles, false);
    std::fs::write("/tmp/triangles.svg", svg.render()).unwrap();
    typst::Data {
        name: Path::new(&name)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        geodesic2d,
        planar2d,
        projected2d,
        projected3d,
        geodesic3d,
        svg: svg.render(),
        nplanes: planes.len(),
    }
}

#[derive(Parser)]
struct Cli {
    paths: Vec<String>,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let mut data: Vec<typst::Data> = Vec::new();
    for path in &args.paths {
        let input_polygons = read_polygon::read_polyline(&path);
        let ldata: Vec<_> = input_polygons
            .iter()
            .map(|p| process(path.clone(), &p))
            .collect();
        data.extend_from_slice(&ldata);
    }
    let doc = typst::make_typst_document(&data);
    std::fs::write("/tmp/document.typst", &doc).unwrap();
}
