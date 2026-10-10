//! Counterpart of tools/oracle/tkbool_probe_interference_tool.cxx.
use super::common::{Reader, end, flag, int, pnt2d, real, text};
use rustycad_tkbool::top_ope_brep_ds::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction2, Point2};
type Tool = TopOpeBRepDSInterferenceTool;
type Kind = TopOpeBRepDSKind;

fn interference_fields(interference: &Handle<TopOpeBRepDSInterference>) {
    let value = interference.get().unwrap();
    text(interference.dynamic_type().unwrap().name().as_bytes());
    int(value.support_type() as i32);
    int(value.support());
    int(value.geometry_type() as i32);
    int(value.geometry());
    let transition = value.transition();
    int(transition.before() as i32);
    int(transition.after() as i32);
    int(transition.shape_before() as i32);
    int(transition.shape_after() as i32);
    int(transition.index_before());
    int(transition.index_after());
}

fn interference_curves(
    transition: &TopOpeBRepDSTransition,
    support: i32,
    geometry: i32,
    parameter: f64,
) {
    let edge = Tool::make_edge_interference(
        transition,
        Kind::Edge,
        support,
        Kind::Vertex,
        geometry,
        parameter,
    );
    let curve = Tool::make_curve_interference(
        transition,
        Kind::Curve,
        support,
        Kind::Point,
        geometry,
        parameter,
    );
    interference_fields(&edge);
    real(Tool::parameter(&edge).unwrap());
    interference_fields(&curve);
    real(Tool::parameter(&curve).unwrap());
    let copy = Tool::duplicate_curve_point_interference(&curve).unwrap();
    interference_fields(&copy);
    flag(copy.object_address() != curve.object_address());
    let alias = curve.clone();
    Tool::set_parameter(&curve, parameter + 1.0).unwrap();
    real(Tool::parameter(&alias).unwrap());
    real(Tool::parameter(&copy).unwrap());
}

fn interference_surfaces(
    transition: &TopOpeBRepDSTransition,
    support: i32,
    geometry: i32,
    has_curve: bool,
) {
    let pcurve: Handle<dyn Geom2dCurve> = if has_curve {
        Handle::new(Geom2dLine::from_point(Point2::new(1., 2.), Direction2::X)).cast()
    } else {
        Handle::default()
    };
    let face = Tool::make_face_curve_interference(transition, support, geometry, &pcurve);
    interference_fields(&face);
    let typed: Handle<TopOpeBRepDSSurfaceCurveInterference> = face.cast();
    let retained = typed.get().unwrap().pcurve();
    flag(retained.object_address() == pcurve.object_address());
    flag(retained.is_null());
    if let Some(curve) = retained.get() {
        pnt2d(curve.value(0.5).unwrap());
    }
    interference_fields(&Tool::make_solid_surface_interference(
        transition, support, geometry,
    ));
}

fn interference_shapes(
    transition: &TopOpeBRepDSTransition,
    support: i32,
    geometry: i32,
    parameter: f64,
    config: TopOpeBRepDSConfig,
    bound: bool,
) {
    let edge = Tool::make_edge_vertex_interference(
        transition, support, geometry, bound, config, parameter,
    );
    interference_fields(&edge);
    let typed: Handle<TopOpeBRepDSEdgeVertexInterference> = edge.cast();
    flag(typed.get().unwrap().gbound());
    int(typed.get().unwrap().config() as i32);
    real(typed.get().unwrap().parameter());
    let face = Tool::make_face_edge_interference(transition, support, geometry, bound, config);
    interference_fields(&face);
    let typed: Handle<TopOpeBRepDSFaceEdgeInterference> = face.cast();
    flag(typed.get().unwrap().gbound());
    int(typed.get().unwrap().config() as i32);
}

/// Compares every factory and shared parameter edit; `interference_tool_case(&[0., 1., 2., 0.5, 0., 1.])`.
pub fn interference_tool_case(values: &[f64]) {
    let mut reader = Reader::new(values);
    let mut transition =
        TopOpeBRepDSTransition::from_orientation(Orientation::ALL[reader.integer() as usize]);
    transition.set_index_before(11);
    transition.set_index_after(13);
    let support = reader.integer();
    let geometry = reader.integer();
    let parameter = reader.real();
    let configs = [
        TopOpeBRepDSConfig::UnshGeometry,
        TopOpeBRepDSConfig::SameOriented,
        TopOpeBRepDSConfig::DiffOriented,
    ];
    let config = configs[reader.integer() as usize];
    let bound = reader.integer() != 0;
    interference_curves(&transition, support, geometry, parameter);
    interference_surfaces(&transition, support, geometry, bound);
    interference_shapes(&transition, support, geometry, parameter, config, bound);
    end();
}
