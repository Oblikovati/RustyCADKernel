//! StdSelect owner filters, the twin of tools/oracle/tkv3d_probe_std_select_filters.cxx.
use super::print::{Reader, end, flag, int, show};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomSurface, GeomSurfaceOfRevolution};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Axis3, Circle3, Cone3, Cylinder3, Direction3, Ellipse3, Line3, Plane3, Point3,
    Sphere3, Torus3,
};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeVertex,
};
use rustycad_tkv3d::select_mgr::{
    SelectMgrEntityOwner, SelectMgrEntityOwnerSource, SelectMgrFilter,
};
use rustycad_tkv3d::std_select::{
    StdSelectBRepOwner, StdSelectEdgeFilter, StdSelectFaceFilter, StdSelectShapeTypeFilter,
    StdSelectTypeOfEdge, StdSelectTypeOfFace,
};

const EDGES: [StdSelectTypeOfEdge; 3] = [
    StdSelectTypeOfEdge::AnyEdge,
    StdSelectTypeOfEdge::Line,
    StdSelectTypeOfEdge::Circle,
];
const FACES: [StdSelectTypeOfFace; 7] = [
    StdSelectTypeOfFace::AnyFace,
    StdSelectTypeOfFace::Plane,
    StdSelectTypeOfFace::Cylinder,
    StdSelectTypeOfFace::Sphere,
    StdSelectTypeOfFace::Torus,
    StdSelectTypeOfFace::Revol,
    StdSelectTypeOfFace::Cone,
];
const MODES: [ShapeEnum; 9] = [
    ShapeEnum::Compound,
    ShapeEnum::CompSolid,
    ShapeEnum::Solid,
    ShapeEnum::Shell,
    ShapeEnum::Face,
    ShapeEnum::Wire,
    ShapeEnum::Edge,
    ShapeEnum::Vertex,
    ShapeEnum::Shape,
];

/// The shape of a kind (`v3_stdsel_shape`).
fn shape(kind: i32) -> Result<Shape, Failure> {
    let z = Direction3::new(0., 0., 1.)?;
    let x = Direction3::new(1., 0., 0.)?;
    let origin = Point3::new(1., 2., 3.);
    let ax2 = Axis2::with_x_direction(origin, z, x)?;
    let ax3 = Axis3::new(origin, z, x)?;
    let bounds = (0., 1., 0., 1.);
    let face = |maker: BRepBuilderAPIMakeFace| -> Result<Shape, Failure> {
        Ok(Shape::from(maker.face()?))
    };
    let edge = |maker: BRepBuilderAPIMakeEdge| -> Result<Shape, Failure> {
        Ok(Shape::from(maker.edge()?))
    };
    match kind {
        0 => edge(BRepBuilderAPIMakeEdge::from_lin_params(
            Line3::new(Point3::new(0., 0., 0.), x),
            0.,
            2.,
        )?),
        1 => edge(BRepBuilderAPIMakeEdge::from_circ(Circle3::new(ax2, 2.)?)?),
        2 => edge(BRepBuilderAPIMakeEdge::from_elips(Ellipse3::new(
            ax2, 3., 1.,
        )?)?),
        3 => face(BRepBuilderAPIMakeFace::from_plane_bounds(
            Plane3::new(origin, z)?,
            bounds,
        )?),
        4 => face(BRepBuilderAPIMakeFace::from_cylinder_bounds(
            Cylinder3::new(ax3, 2.)?,
            bounds,
        )?),
        5 => face(BRepBuilderAPIMakeFace::from_cone_bounds(
            Cone3::new(ax3, 0.5, 2.)?,
            bounds,
        )?),
        6 => face(BRepBuilderAPIMakeFace::from_sphere_bounds(
            Sphere3::new(ax3, 2.)?,
            bounds,
        )?),
        7 => face(BRepBuilderAPIMakeFace::from_torus_bounds(
            Torus3::new(ax3, 3., 1.)?,
            bounds,
        )?),
        8 => {
            let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
                Point3::new(2., 0., 0.),
                Direction3::new(1., 0., 1.)?,
            )))
            .into_cast();
            let axis = Axis1::new(Point3::new(0., 0., 0.), z);
            let surface: Handle<dyn GeomSurface> =
                Handle::new(GeomSurfaceOfRevolution::new(&line, axis)?).into_cast();
            face(BRepBuilderAPIMakeFace::from_surface_bounds(
                &surface, bounds, 1.0e-7,
            )?)
        }
        9 => Ok(Shape::from(
            BRepBuilderAPIMakeVertex::new(Point3::new(1., 1., 1.))?.vertex()?,
        )),
        _ => Ok(Shape::default()),
    }
}

/// `v3_stdsel_filter Family Type Kind`.
pub fn filter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (family, type_, kind) = (r.integer(), r.integer(), r.integer());
    let owner: Handle<dyn SelectMgrEntityOwnerSource> = if kind == 11 {
        SelectMgrEntityOwner::new(0).into_cast()
    } else {
        let Ok(shape) = shape(kind) else {
            return end();
        };
        StdSelectBRepOwner::with_shape(&shape, 5, false).into_cast()
    };
    let edge = StdSelectEdgeFilter::new(EDGES[type_.clamp(0, 2) as usize]);
    let face = StdSelectFaceFilter::new(FACES[type_.clamp(0, 6) as usize]);
    let shape_type = StdSelectShapeTypeFilter::new(MODES[type_.clamp(0, 8) as usize]);
    let filter: &dyn SelectMgrFilter = match family {
        0 => &edge,
        1 => &face,
        _ => &shape_type,
    };
    show(filter.is_ok(&owner), flag);
    for mode in MODES {
        flag(filter.acts_on(mode));
    }
    if family == 0 {
        int(edge.type_() as i32);
        edge.set_type(EDGES[((type_ + 1) % 3) as usize]);
        int(edge.type_() as i32);
    }
    if family == 1 {
        int(face.type_() as i32);
        face.set_type(FACES[((type_ + 1) % 7) as usize]);
        int(face.type_() as i32);
    }
    if family == 2 {
        int(shape_type.type_() as i32);
    }
    end();
}
