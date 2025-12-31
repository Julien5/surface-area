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
    let kml_polygon = read_kml::read_polyline(&args.path.as_str());
    kml_polygon.info();
    let pbbox = kml_polygon.wgsbbox();
    let mut gridpoints = BTreeSet::new();
    for b in kml_polygon.datasets() {
        let dataset = Dataset::open(&b);
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

    let polygon = kml_polygon.mercator();
    let mut svg = svg::SVG::init(&kml_polygon.mercatorbbox());
    let colors = ["blue", "gray", "yellow", "green"];
    let mut planes = Vec::new();
    for (i, gridtriangle) in gridtriangles.iter().enumerate() {
        let plane = intersection::intersection(&polygon, &gridtriangle);
        if plane.is_empty() {
            continue;
        }
        planes.push(plane.clone());
        svg.add_polygon(&plane, colors[i % colors.len()]);
    }
    log::trace!("planes: {}", planes.len());

    let mut ret = 0f64;
    let mut ret_flat = 0f64;
    for p in planes {
        ret += polygon::calculate_3d_surface_area(&p);
        ret_flat += polygon::calculate_3d_surface_area(&polygon::flat(&p));
    }
    println!(
        "geodesic: {:.1} (geo crate)",
        reference::geodesic_area(&kml_polygon.wgs)
    );
    println!(
        "  planar: {:.1} (geo crate)",
        reference::planar_area(&kml_polygon.mercator())
    );
    println!();
    println!("    flat: {:.1}", ret_flat);
    println!(" surface: {:.1}", ret);
    let ratio = ret / ret_flat;
    println!("   ratio: +{:.1}%", (ratio - 1f64) * 100f64);
    println!();
    println!(
        "estimate: {:.1}",
        ratio * reference::geodesic_area(&kml_polygon.wgs)
    );

    //svg.add_polygon(&polygon, "none");
    //svg.add_triangles(&atoms, true);
    //svg.add_triangles(&triangulation::polygon::triangulate(&polygon), true);
    svg.add_triangles(&gridtriangles, false);
    std::fs::write("/tmp/triangles.svg", svg.render()).unwrap();
}
