//! Standard presentations exercise the ported curve algorithm through actual Graphic3d groups.
#[path = "../../../tkv3d/tests/support/stdprs_curve_geometry.rs"]
mod geometry;
use rustycad_tkernel::{ncollection::Sequence, standard::Handle};
use rustycad_tkservice::graphic3d::*;
use rustycad_tkv3d::{prs3d::Prs3dDrawer, std_prs::StdPrsCurve};
pub const MODES: [&str; 7] = [
    "stdprs-line",
    "stdprs-circle",
    "stdprs-ellipse",
    "stdprs-bezier",
    "stdprs-bspline",
    "stdprs-arrow",
    "stdprs-collect",
];

pub fn replace_curve(prs: &Handle<Graphic3dStructure>, mode: &str, copies: usize) {
    assert!(
        MODES.contains(&mode),
        "received {mode}; expected a standard curve fixture"
    );
    prs.clear(true);
    let kind = match mode {
        "stdprs-circle" => 1,
        "stdprs-ellipse" => 2,
        "stdprs-bezier" => 3,
        "stdprs-bspline" | "stdprs-collect" => 4,
        _ => 0,
    };
    let curve = geometry::curve_geometry(kind).unwrap();
    let drawer = Handle::new(Prs3dDrawer::new());
    drawer.setup_own_defaults().unwrap();
    drawer.set_discretisation(30);
    drawer.set_line_arrow_draw(mode == "stdprs-arrow");
    drawer.arrow_aspect().set_length(1.);
    drawer.arrow_aspect().set_angle(0.2).unwrap();
    for _ in 0..copies {
        prs.new_group().unwrap();
        if mode == "stdprs-collect" {
            StdPrsCurve::add_points(
                &prs.cast(),
                &curve,
                &drawer,
                &mut Sequence::default(),
                false,
            )
            .unwrap();
        } else {
            StdPrsCurve::add(&prs.cast(), &curve, &drawer, true).unwrap();
        }
    }
}
