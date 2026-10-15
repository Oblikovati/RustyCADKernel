//! The BRepFilletAPI fillets and chamfers (`fl_make_fillet`); the twin of
//! tools/oracle/tkfillet_probe_api.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkbo::brep_algo_api::{BRepAlgoAPICut, BRepAlgoAPIFuse};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkfillet::brep_fillet_api::{BRepFilletAPIMakeChamfer, BRepFilletAPIMakeFillet};
use rustycad_tkfillet::chfi3d::FilletShape;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Axis2, Point3};
use rustycad_tkprim::brep_prim_api::{
    BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder, BRepPrimAPIMakeWedge,
};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// `fl_api_box`: the box of the corner (x, y, z) and the sizes.
fn api_box(x: f64, y: f64, z: f64, dx: f64, dy: f64, dz: f64) -> Result<Shape, Failure> {
    Ok(
        BRepPrimAPIMakeBox::at_point(Point3::new(x, y, z), dx, dy, dz)?
            .shape()?
            .clone(),
    )
}

/// `fl_api_solid`: the solid of the request kind (see the native probe).
fn api_solid(kind: i32) -> Result<Shape, Failure> {
    if kind == 1 {
        let mut w = BRepPrimAPIMakeWedge::with_limits(10., 10., 10., [5., 5., 5., 5.])?;
        return Ok(w.shape()?.clone());
    }
    let base = api_box(0., 0., 0., 10., 10., 5.)?;
    Ok(match kind {
        2 => {
            let axes = Axis2::new(Point3::new(5., 5., 5.), gp::dz())?;
            let cyl = BRepPrimAPIMakeCylinder::with_axes(axes, 2., 3.)
                .shape()?
                .clone();
            BRepAlgoAPIFuse::from_shapes(&base, &cyl)?.shape().clone()
        }
        3 => BRepAlgoAPICut::from_shapes(&base, &api_box(3., 3., 3., 4., 4., 3.)?)?
            .shape()
            .clone(),
        4 => BRepAlgoAPICut::from_shapes(&base, &api_box(5., 5., -1., 10., 10., 7.)?)?
            .shape()
            .clone(),
        5 => BRepAlgoAPIFuse::from_shapes(&base, &api_box(0., 0., 5., 5., 10., 3.)?)?
            .shape()
            .clone(),
        _ => api_box(0., 0., 0., 20., 20., 20.)?,
    })
}

/// The number of sub-shapes of a kind met by an explorer.
fn count(s: &Shape, kind: ShapeEnum) -> Result<i32, Failure> {
    let mut ex = Explorer::new(s, kind, ShapeEnum::Shape)?;
    let mut n = 0;
    while ex.more() {
        n += 1;
        ex.next()?;
    }
    Ok(n)
}

/// `fl_api_result`: done, then the face and edge counts, volume, area and validity.
fn api_result(api: &impl BRepBuilderAPIMakeShape) -> Result<(), Failure> {
    flag(api.is_done());
    if !api.is_done() {
        return Ok(());
    }
    let s = api.make_shape_base().my_shape.clone();
    int(count(&s, ShapeEnum::Face)?);
    int(count(&s, ShapeEnum::Edge)?);
    let mut v = GProps::new();
    BRepGProp::volume_properties(&s, &mut v, false, false, false)?;
    real(v.mass());
    let mut a = GProps::new();
    BRepGProp::surface_properties(&s, &mut a, false, false)?;
    real(a.mass());
    flag(BRepCheckAnalyzer::new(&s, true, false, false)?.is_valid()?);
    Ok(())
}

/// The fillet methods 0, 1, 5 and 6 of `fl_make_fillet`.
fn fillet(s: &Shape, edges: &[Edge], method: i32, d1: f64, d2: f64) -> Result<(), Failure> {
    let fshape = match method {
        5 => FilletShape::QuasiAngular,
        6 => FilletShape::Polynomial,
        _ => FilletShape::Rational,
    };
    let mut api = BRepFilletAPIMakeFillet::new(s, fshape)?;
    for e in edges {
        if method == 1 {
            api.add_with_radii(d1, d2, e)?;
        } else {
            api.add_with_radius(d1, e)?;
        }
    }
    api.build(&ProgressRange::default())?;
    int(api.nb_faulty_contours());
    int(api.nb_faulty_vertices());
    api_result(&api)
}

/// The chamfer methods 2, 3 and 4 of `fl_make_fillet`, on the first face of each edge.
fn chamfer(
    s: &Shape,
    ef: &IndexedDataMapOfShapeListOfShape,
    edges: &[Edge],
    method: i32,
    d1: f64,
    d2: f64,
) -> Result<(), Failure> {
    let mut api = BRepFilletAPIMakeChamfer::new(s)?;
    for e in edges {
        let first = ef.find_from_key(&e.clone().into_shape())?.first().cloned();
        let f =
            topods::face(&first.ok_or_else(|| {
                Failure::with_message("Standard_NoSuchObject", "edge without face")
            })?)?;
        match method {
            2 => api.add_with_dist(d1, e)?,
            3 => api.add_with_dists(d1, d2, e, &f)?,
            _ => api.add_dist_angle(d1, d2, e, &f)?,
        }
    }
    api.build(&ProgressRange::default())?;
    api_result(&api)
}

/// `fl_make_fillet solid edge method d1 d2`: see tools/oracle/tkfillet_probe_api.cxx.
pub fn make_fillet_case(values: &[f64]) {
    let r = &mut Reader::new(values);
    let (kind, iedge, method) = (r.integer(), r.integer(), r.integer());
    let (d1, d2) = (r.real(), r.real());
    attempt(|| {
        let s = api_solid(kind)?;
        let mut ef = IndexedDataMapOfShapeListOfShape::default();
        topexp::map_shapes_and_ancestors(&s, ShapeEnum::Edge, ShapeEnum::Face, &mut ef)?;
        let mut edges = Vec::new();
        for k in 1..=ef.extent() {
            if iedge == 0 || k as i32 == iedge {
                edges.push(topods::edge(ef.find_key(k)?)?);
            }
        }
        if matches!(method, 0 | 1 | 5 | 6) {
            fillet(&s, &edges, method, d1, d2)
        } else {
            chamfer(&s, &ef, &edges, method, d1, d2)
        }
    });
    end();
}
