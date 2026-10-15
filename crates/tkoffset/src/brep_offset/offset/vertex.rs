// Rust translation of BRepOffset_Offset::Init(Vertex, LEdge, Offset, Polynomial, TolApp,
// Conti) with the static GetFarestCorner of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Offset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The sphere face of a vertex bounded by the given edges: the sphere centred on the
//! vertex, framed by the barycentre and the farthest corner of the edges, the edges
//! projected on it.
use super::{BRepOffsetOffset, PipeOptions, update_edge_pcurve};
use crate::brep_offset::BRepOffsetStatus;
use crate::brep_offset::tool::live;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::update_face;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Face, Shape, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSphericalSurface, GeomSurface, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gce::MakePln;
use rustycad_tkgeombase::geom_convert::ApproxSurface;
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{
    Axis2d, Axis3, Direction2, Direction3, Point2, Point3, Vector2, Vector3,
};
use rustycad_tkmath::toploc::Location;
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::brep_gprop::BRepGProp;
use rustycad_tktopalgo::brep_lib::{self, BRepLibMakeWire};
use std::f64::consts::{FRAC_PI_2, PI};

/// `GetFarestCorner(aWire)`: the first point of the farthest pair of vertices.
fn get_farest_corner(a_wire: &Wire) -> Result<Point3, Failure> {
    let mut vertices = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(a_wire.as_ref(), ShapeEnum::Vertex, &mut vertices)?;
    let mut max_dist = 0.;
    let mut the_point = Point3::default();
    for i in 1..=vertices.extent() {
        for j in 1..=vertices.extent() {
            let p1 = BRepTool::pnt(&topods::vertex(vertices.find_key(i)?)?)?;
            let p2 = BRepTool::pnt(&topods::vertex(vertices.find_key(j)?)?)?;
            let a_dist = p1.square_distance(&p2);
            if a_dist > max_dist {
                max_dist = a_dist;
                the_point = p1;
            }
        }
    }
    Ok(the_point)
}

impl BRepOffsetOffset {
    /// `Init(Vertex, LEdge, Offset, Polynomial, TolApp, Conti)`: the sphere of radius
    /// |Offset| at the vertex (approximated when polynomial) bounded by the edges, which
    /// get their p-curves on it; reversed for a negative offset;
    /// `o.init_vertex(&v, &edges, 1.0, PipeOptions::default())?`.
    pub fn init_vertex(
        &mut self,
        vertex: &Vertex,
        l_edge: &List<Shape>,
        offset: f64,
        options: PipeOptions,
    ) -> Result<(), Failure> {
        let PipeOptions {
            polynomial,
            tol: tol_app,
            conti,
        } = options;
        self.my_status = BRepOffsetStatus::Good;
        self.my_shape = vertex.as_ref().clone();
        let origin = BRepTool::pnt(vertex)?;

        // The axis of the sphere, which keeps the degenerated and seam edges out of
        // the face.
        let mut mw = BRepLibMakeWire::new();
        mw.add_list(l_edge)?;
        let the_wire = mw.wire()?;
        let fixer = ShapeFixShape::new(the_wire.into_shape());
        fixer.perform()?;
        let the_wire = topods::wire(&fixer.shape())?;
        let mut global_props = GProps::new();
        BRepGProp::linear_properties(the_wire.as_ref(), &mut global_props, false, false)?;
        let bary_center = global_props.centre_of_mass();
        let xdir = Vector3::between(bary_center, origin);
        let farest_corner = get_farest_corner(&the_wire)?;
        let the_plane = MakePln::through_points(origin, bary_center, farest_corner)?.value()?;
        let vdir = the_plane.axis().direction();
        let axis = Axis3::new(origin, vdir, Direction3::from_vector(xdir)?)?;
        let s: Handle<dyn GeomSurface> =
            Handle::new(GeomSphericalSurface::new(axis, offset.abs())?).cast();

        let tol = BRepTool::vertex_tolerance(vertex)?;
        let builder = BRepBuilder::default();
        let mut my_face = Face::default();
        builder.make_face(&mut my_face);
        let mut ss = s.clone();
        // Polynomially, the surface is approximated as F(u,v): no change of parameter,
        // so the projection on the sphere stays valid.
        if polynomial {
            let approx = ApproxSurface::from_surface(&s, tol_app, (conti, conti), (10, 10), 10, 1)?;
            if approx.is_done() {
                ss = approx.surface().cast();
            }
        }
        builder.update_face(&my_face, &ss, &Location::default(), tol)?;

        let mut w = Wire::default();
        builder.make_wire(&mut w);
        let mut w_shape = w.into_shape();
        for shape in l_edge.iter() {
            let e = topods::edge(shape)?;
            let mut found = BRepTool::curve_with_location(&e)?;
            if found.curve.is_null() {
                let (continuity, max_degree, max_segment) = brep_lib::BUILD_CURVE3D_DEFAULTS;
                brep_lib::build_curve3d(
                    &e,
                    BRepTool::edge_tolerance(&e)?,
                    continuity,
                    max_degree,
                    max_segment,
                )?;
                found = BRepTool::curve_with_location(&e)?;
            }
            let (f, l) = (found.first, found.last);
            let c: Handle<dyn GeomCurve> =
                Handle::new(GeomTrimmedCurve::new(&found.curve, f, l, true, true)?).cast();
            live(&c, "curve")?.transform(found.location.transformation())?;
            let pcurve = geom_proj_lib::curve2d(&c, &s)?;
            // A first point of the p-curve outside the canonical bounds of the sphere
            // is moved: U' = U + PI + 2kPI, V' = +/-PI + 2k'PI.
            let pc = live(&pcurve, "pcurve")?;
            let p2d = pc.value(f)?;
            let mut is_to_adjust = false;
            if p2d.y() < -FRAC_PI_2 {
                is_to_adjust = true;
                pc.mirror_axis(Axis2d::new(Point2::new(0., -FRAC_PI_2), Direction2::X))?;
            } else if p2d.y() > FRAC_PI_2 {
                is_to_adjust = true;
                pc.mirror_axis(Axis2d::new(Point2::new(0., FRAC_PI_2), Direction2::X))?;
            }
            if is_to_adjust {
                // The first u in [0, 2PI].
                let mut tr = Vector2::new(PI, 0.);
                if p2d.x() > PI {
                    tr.reverse();
                }
                pc.translate(tr)?;
            }
            update_edge_pcurve(&e, &pcurve, &my_face, tol)?;
            builder.range_on_face(&e, &my_face, f, l)?;
            builder.add(&mut w_shape, e.as_ref())?;
        }
        let mut face_shape = my_face.as_ref().clone();
        if offset < 0. {
            builder.add(&mut face_shape, &w_shape.oriented(Orientation::Reversed))?;
            my_face.reverse();
        } else {
            builder.add(&mut face_shape, &w_shape)?;
        }
        update_face(&my_face)?;
        self.my_face = my_face;
        Ok(())
    }
}
