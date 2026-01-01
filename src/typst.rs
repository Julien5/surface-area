#[derive(Clone)]
pub struct Data {
    pub name: String,
    pub geodesic2d: f64,
    pub planar2d: f64,
    pub projected2d: f64,
    pub projected3d: f64,
    pub geodesic3d: f64,
    pub svg: String,
    pub nplanes: usize,
}

pub fn make_typst_document(data: &Vec<Data>) -> String {
    let mut doc = String::from("#set page(paper: \"a4\")\n#set text(size: 11pt)\n\n");

    for item in data {
        let ratio = item.geodesic3d / item.geodesic2d - 1f64;
        // 1. Main line with name in bold
        doc.push_str(&format!("== * {} *\n\n", item.name));

        doc.push_str(&format!("== * ratio: +{:.2}% * \n", 100f64 * ratio));

        doc.push_str("#table(\n");
        doc.push_str("  columns: (1fr, 1fr, 1fr),\n");
        doc.push_str("  inset: 10pt,\n");
        doc.push_str("  align: horizon,\n");
        doc.push_str("  [*Field*], [*2D Value*], [*3D Value*],\n");
        doc.push_str(&format!(
            "  [Geodesic], [{:.0} $m^2$], [{:.0} $m^2$],\n",
            item.geodesic2d, item.geodesic3d,
        ));
        doc.push_str(&format!(
            "  [Mercator (UTM)], [{:.0} $m^2$], [{:.0} $m^2$],\n",
            item.planar2d, item.projected3d
        ));
        doc.push_str(")\n\n");

        // 3. SVG Image
        // We use triple backticks ``` to wrap the SVG content safely
        doc.push_str("#align(center, image(\n");
        doc.push_str(&format!("  bytes(```{}```.text),\n", item.svg));
        doc.push_str("  width: 80%\n");
        doc.push_str("))\n\n");

        doc.push_str("#pagebreak()\n\n");
    }

    doc
}
