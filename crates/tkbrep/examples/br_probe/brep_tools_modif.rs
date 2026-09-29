//! BRepTools_Modifier with each modification, and the wire explorer, over a scene; the
//! Rust twin of tools/oracle/tkbrep_probe_breptools_modif.cxx.
use super::brep::scene;
use super::brep_tools::written;
use super::common::{Reader, attempt, end, flag, int, text};
use super::graph::{Graph, desc};
use rustycad_tkbrep::brep_tools::{
    self, BRepToolsModification, CopyModification, GTrsfModification, Modifier,
    NurbsConvertModification, TrsfModification, WireExplorer,
};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{GeneralTransform3, Matrix3, Xyz};

/// `bt_modification`: the modification of a request.
fn modification(g: &Graph, p: &[f64], kind: i32) -> Handle<dyn BRepToolsModification> {
    match kind {
        0 => {
            let m = TrsfModification::new(g.loc(p[0] as i32).transformation());
            m.set_copy_mesh(p[1] != 0.0);
            Handle::new(m).cast()
        }
        1 => {
            let matrix = Matrix3::new([[p[0], p[1], p[2]], [p[3], p[4], p[5]], [p[6], p[7], p[8]]]);
            let gtrsf = GeneralTransform3::new(matrix, Xyz::new(p[9], p[10], p[11]));
            Handle::new(GTrsfModification::new(gtrsf)).cast()
        }
        2 => Handle::new(CopyModification::new(p[0] != 0.0, p[1] != 0.0)).cast(),
        _ => Handle::new(NurbsConvertModification::new()).cast(),
    }
}

/// `bt_modified_shapes`.
fn modified_shapes(g: &Graph, m: &Modifier) {
    for x in g.shapes.iter().filter(|x| !x.is_null()) {
        attempt(|| m.modified_shape(x).map(|y| flag(y.is_same(x))));
    }
}

/// `bt_modify`.
fn modify(
    g: &Graph,
    sh: &Shape,
    modification: &Handle<dyn BRepToolsModification>,
    mutable_input: bool,
) -> Result<(), Failure> {
    let mut m = Modifier::new(mutable_input);
    flag(m.is_mutable_input());
    m.init(sh)?;
    m.perform(modification, &ProgressRange::default())?;
    flag(m.is_done());
    text(&written(&m.modified_shape(sh)?, true, true, 3)?);
    modified_shapes(g, &m);
    if let Some(n) = modification.cast::<NurbsConvertModification>().get() {
        int(n.updated_edges().len() as i32);
    }
    Ok(())
}

/// `br_tools_modif script kind params... mutable`.
pub fn modif_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let kind = r.integer();
    let count = match kind {
        0 | 2 => 2,
        1 => 12,
        _ => 0,
    };
    let params: Vec<f64> = (0..count).map(|_| r.real()).collect();
    let mutable_input = r.flag();
    for sh in s.graph.shapes.iter().filter(|sh| !sh.is_null()) {
        attempt(|| {
            modify(
                &s.graph,
                sh,
                &modification(&s.graph, &params, kind),
                mutable_input,
            )
        });
    }
    end();
}

/// `bt_explore`.
fn explore(g: &Graph, x: &mut WireExplorer) -> Result<(), Failure> {
    let mut n = 0;
    while x.more() && n < 64 {
        desc(g, x.current());
        attempt(|| x.orientation().map(|o| int(o as i32)));
        desc(g, x.current_vertex());
        x.next()?;
        n += 1;
    }
    int(n);
    flag(x.more());
    x.clear();
    flag(x.more());
    Ok(())
}

/// `br_tools_wire script`.
pub fn wire_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let s = scene(&mut r);
    let g = &s.graph;
    let of_type = |kind: ShapeEnum| {
        g.shapes
            .iter()
            .filter(move |sh| !sh.is_null() && sh.shape_type().unwrap() == kind)
    };
    for w in of_type(ShapeEnum::Wire) {
        let wire = topods::wire(w).unwrap();
        attempt(|| explore(g, &mut WireExplorer::with_wire(&wire)?));
        for f in of_type(ShapeEnum::Face) {
            let face = topods::face(f).unwrap();
            attempt(|| explore(g, &mut WireExplorer::with_wire_on_face(&wire, &face)?));
            attempt(|| {
                let bounds = brep_tools::uv_bounds(&face)?;
                let mut x = WireExplorer::new();
                x.init_with_bounds(&wire, &face, bounds)?;
                explore(g, &mut x)
            });
        }
    }
    flag(WireExplorer::new().more());
    end();
}
