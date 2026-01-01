use clap::Parser;
use std::collections::BTreeSet;
use surface_area::dataset::Dataset;
use surface_area::point::MercatorPoint;
use surface_area::{intersection, polygon, read_kml, reference, svg, triangulation};

#[derive(Parser)]
struct Cli {
    path: String,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let input_polygon = read_kml::read_polyline(&args.path.as_str());
    input_polygon.info();
    let pbbox = input_polygon.wgsbbox();
    let mut gridpoints = BTreeSet::new();
    let mut projections = BTreeSet::new();
    for dataset in Dataset::select(&input_polygon.datasets()) {
        dataset.info();
        projections.insert(dataset.projection());
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
    projections.insert(input_polygon.projection());
    if projections.len() != 1 {
        log::error!("unsupported: polygon/datasets strech over multiple UTM zones");
        for p in projections {
            log::error!("{}", p);
        }
        return;
    }

    log::trace!("gridpoints: {}", gridpoints.len());

    let gridvec: Vec<MercatorPoint> = gridpoints.into_iter().collect();
    let gridtriangles = triangulation::grid::triangulate(&gridvec);
    log::trace!("grid triangles: {}", gridtriangles.len());

    let polygon = input_polygon.mercator();
    let mut svg = svg::SVG::init(&input_polygon.mercatorbbox());
    let colors = ["blue", "gray", "yellow", "green"];
    let mut planes = Vec::new();
    let mut ret = 0f64;
    let mut ret_flat = 0f64;
    for (i, gridtriangle) in gridtriangles.iter().enumerate() {
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
        ret += a3d;
        ret_flat += a2d;
        planes.push(plane.clone());
        svg.add_polygon(&plane, colors[i % colors.len()]);
    }
    log::trace!("planes: {}", planes.len());

    println!(
        "geodesic: {:.1} (geo crate)",
        reference::geodesic_area(&input_polygon.wgs)
    );
    println!(
        "  planar: {:.1} (geo crate)",
        reference::planar_area(&input_polygon.mercator())
    );
    println!();
    println!("    flat: {:.1}", ret_flat);
    println!(" surface: {:.1}", ret);
    let ratio = ret / ret_flat;
    println!("   ratio: +{:.1}%", (ratio - 1f64) * 100f64);
    println!();
    println!(
        "estimate: {:.1}",
        ratio * reference::geodesic_area(&input_polygon.wgs)
    );

    //svg.add_polygon(&polygon, "none");
    //svg.add_triangles(&atoms, true);
    //svg.add_triangles(&triangulation::polygon::triangulate(&polygon), true);
    svg.add_triangles(&gridtriangles, false);
    std::fs::write("/tmp/triangles.svg", svg.render()).unwrap();
}
