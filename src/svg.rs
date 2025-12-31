use crate::{
    point::{MercatorBoundingBox, MercatorPoint},
    triangulation::Triangle,
};

pub struct SVG {
    mercator_bbox: MercatorBoundingBox,
    padding: f64,
    polygons: Vec<String>,
}

impl SVG {
    pub fn init(b: &MercatorBoundingBox) -> Self {
        Self {
            mercator_bbox: b.clone(),
            padding: 0.1,
            polygons: Vec::new(),
        }
    }
    pub fn add_triangles(&mut self, triangles: &Vec<Triangle>, altfill: bool) {
        // Add each triangle as a polygon
        for (i, triangle) in triangles.iter().enumerate() {
            let (x1, y1) = self.transform(triangle.0.x, triangle.0.y);
            let (x2, y2) = self.transform(triangle.1.x, triangle.1.y);
            let (x3, y3) = self.transform(triangle.2.x, triangle.2.y);
            let mut fill = "none";
            if altfill {
                if i % 2 == 0 {
                    fill = "blue";
                }
            }
            let p = format!(
                r#"  <polygon points="{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}" fill="{}" stroke="black" stroke-width="1"/>"#,
                x1, y1, x2, y2, x3, y3, fill
            );

            self.polygons.push(p);
        }
    }
    pub fn add_polygon(&mut self, points: &Vec<MercatorPoint>, fill: &str) {
        // Add each triangle as a polygon
        let s = points
            .iter()
            .map(|p| {
                let (x, y) = self.transform(p.x, p.y);
                format!("{:.2},{:.2}", x, y)
            })
            .collect::<Vec<String>>()
            .join(" ");
        let p = format!(
            r#"<polygon points="{}" fill="{}" stroke="red" stroke-width="3"/>"#,
            s, fill
        );
        self.polygons.push(p);
    }
    pub fn render(&self) -> String {
        // Build SVG string
        let mut svg =
            String::from(r#"<svg width="500" height="500" xmlns="http://www.w3.org/2000/svg">"#);
        svg.push('\n');
        for p in &self.polygons {
            svg.push_str(&p.clone());
            svg.push('\n');
        }
        svg.push_str("</svg>");
        svg
    }
    pub fn width(&self) -> f64 {
        self.mercator_bbox.max.x - self.mercator_bbox.min.x
    }
    pub fn height(&self) -> f64 {
        self.mercator_bbox.max.y - self.mercator_bbox.min.y
    }
    pub fn scale(&self) -> f64 {
        let padded_width = self.width() * (1.0 + 2.0 * self.padding);
        let padded_height = self.height() * (1.0 + 2.0 * self.padding);
        500.0 / padded_width.max(padded_height)
    }
    fn transform(&self, x: f64, y: f64) -> (f64, f64) {
        let svg_x = (x - self.mercator_bbox.min.x + self.width() * self.padding) * self.scale();
        let svg_y =
            500.0 - (y - self.mercator_bbox.min.y + self.height() * self.padding) * self.scale();
        (svg_x, svg_y)
    }
}
