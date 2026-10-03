//! Creation of the one-patch periodic grid passed to ComposeShell.
use super::degenerate_seam::create_for_single_open_wire;
use super::layout::order_and_align;
use super::plan::MissingSeamPlan;
use crate::shape_extend::{ShapeExtendCompositeSurface, ShapeExtendParametrisation, SurfaceGrid};
use rustycad_tkernel::ncollection::HArray2;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomRectangularTrimmedSurface, GeomSurface};

pub(super) struct PreparedMissingSeam {
    pub(super) face: rustycad_tkbrep::topods::Face,
    pub(super) grid: Handle<ShapeExtendCompositeSurface>,
}

pub(super) fn prepare(
    plan: &mut MissingSeamPlan,
    fix_precision: f64,
) -> Result<Option<PreparedMissingSeam>, Failure> {
    if !create_for_single_open_wire(plan)? {
        return Ok(None);
    }
    let Some(ordered) = order_and_align(plan, fix_precision)? else {
        return Ok(None);
    };
    let grid = trimmed_grid(plan, ordered.u_split, ordered.v_split)?;
    Ok(Some(PreparedMissingSeam {
        face: ordered.face,
        grid: Handle::new(grid),
    }))
}

fn trimmed_grid(
    plan: &MissingSeamPlan,
    u_first: f64,
    v_first: f64,
) -> Result<ShapeExtendCompositeSurface, Failure> {
    let surface = rectangular_patch(plan, u_first, v_first)?;
    let grid = one_patch_grid(surface)?;
    ShapeExtendCompositeSurface::with_grid(&grid, ShapeExtendParametrisation::Natural)
}

fn rectangular_patch(
    plan: &MissingSeamPlan,
    u_first: f64,
    v_first: f64,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let bounds = plan.surface_bounds;
    let surface = GeomRectangularTrimmedSurface::new(
        &plan.surface,
        (u_first, u_first + (bounds.1 - bounds.0)),
        (v_first, v_first + (bounds.3 - bounds.2)),
        (true, true),
    )?;
    Ok(Handle::new(surface).cast())
}

fn one_patch_grid(surface: Handle<dyn GeomSurface>) -> Result<SurfaceGrid, Failure> {
    let cells = HArray2::with_value(1, 1, 1, 1, Handle::default()).map_err(invalid_grid)?;
    cells
        .change_array2()
        .set_value(1, 1, surface)
        .map_err(invalid_grid)?;
    Ok(Handle::new(cells))
}

fn invalid_grid(_: rustycad_tkernel::ncollection::ArrayError) -> Failure {
    Failure::with_message(
        "Standard_DimensionError",
        "ShapeFix_Face::FixMissingSeam: grid bounds (1,1,1,1); expected one valid surface patch",
    )
}

#[cfg(test)]
mod tests {
    use super::prepare;
    use crate::shape_fix::face::missing_seam::plan::analyze;
    use crate::shape_fix::{FixRoot, ShapeFixFace};
    use rustycad_tkbrep::brep::BRepBuilder;
    use rustycad_tkbrep::brep_tools::ReShape;
    use rustycad_tkbrep::topexp::Explorer;
    use rustycad_tkbrep::topods::Face;
    use rustycad_tkernel::precision;
    use rustycad_tkernel::standard::Handle;
    use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
    use rustycad_tkg3d::geom::{
        GeomCircle, GeomCylindricalSurface, GeomSphericalSurface, GeomSurface, GeomToroidalSurface,
    };
    use rustycad_tkg3d::top_abs::ShapeEnum;
    use rustycad_tkmath::gp::{
        Axis2, Axis3, Direction2, Point2, Point3, StandardDirection2, StandardDirection3,
    };
    use std::f64::consts::TAU;

    #[test]
    fn sphere_missing_seam_becomes_a_one_patch_periodic_grid() {
        let (face, surface) = sphere_with_open_u_wire().unwrap();
        let mut plan = analyze(&face, &surface).unwrap().unwrap();
        let prepared = prepare(&mut plan, precision::confusion())
            .unwrap()
            .expect("native sphere branch creates a degenerate seam");
        let patch = prepared.grid.patch(1, 1).unwrap();
        let bounds = patch.get().unwrap().bounds();
        assert_eq!(prepared.grid.nb_u_patches().unwrap(), 1);
        assert_eq!(prepared.grid.nb_v_patches().unwrap(), 1);
        assert!((bounds.0 - 0.0).abs() < precision::p_confusion());
        assert!((bounds.1 - TAU).abs() < precision::p_confusion());
        assert!((bounds.2 + std::f64::consts::FRAC_PI_2).abs() < precision::p_confusion());
        assert!((bounds.3 - std::f64::consts::FRAC_PI_2).abs() < precision::p_confusion());
    }

    #[test]
    fn sphere_missing_seam_matches_native_face_edge_result() {
        let (face, _) = sphere_with_open_u_wire().unwrap();
        let input = face.into_shape();
        let fixer = ShapeFixFace::new(input.clone());
        assert!(fixer.fix_missing_seam().unwrap());
        let result = fixer.result();
        assert_context_maps_input(&fixer, &input, &result);
        let faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
        let face_count = faces.count();
        let edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        let degenerated_count = edges
            .filter(|edge| {
                let edge = rustycad_tkbrep::topods::edge(edge).unwrap();
                rustycad_tkbrep::brep::BRepTool::degenerated(&edge).unwrap()
            })
            .count();
        let actual = format!(
            "sphere_open_u changed=1 faces={face_count} degenerated_edges={degenerated_count}"
        );
        assert_eq!(
            actual,
            include_str!("../../../../../../tools/oracle/shape_fix_face_missing_seam.expected")
                .lines()
                .next()
                .unwrap()
        );
    }

    #[test]
    fn cylinder_two_open_wires_matches_native_seam_result() {
        let face = cylinder_with_two_open_u_wires().unwrap();
        let input = face.into_shape();
        let fixer = ShapeFixFace::new(input.clone());
        assert!(fixer.fix_missing_seam().unwrap());
        let result = fixer.result();
        assert_context_maps_input(&fixer, &input, &result);
        let faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
        let face_count = faces.clone().count();
        let face = rustycad_tkbrep::topods::face(faces.current()).unwrap();
        let edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        let edge_count = edges.clone().count();
        let seam_count = edges
            .filter(|edge| {
                let edge = rustycad_tkbrep::topods::edge(edge).unwrap();
                rustycad_tkbrep::brep::BRepTool::is_closed_on_face(&edge, &face).unwrap()
            })
            .count();
        let actual = format!(
            "cylinder_two_open_u changed=1 faces={face_count} edges={edge_count} seam_edges={seam_count}"
        );
        assert_eq!(
            actual,
            include_str!("../../../../../../tools/oracle/shape_fix_face_missing_seam.expected")
                .lines()
                .nth(1)
                .unwrap()
        );
    }

    #[test]
    fn spindle_torus_single_open_wire_matches_native_result() {
        let face = spindle_torus_with_open_u_wire().unwrap();
        let input = face.into_shape();
        let fixer = ShapeFixFace::new(input.clone());
        assert!(fixer.fix_missing_seam().unwrap());
        let result = fixer.result();
        assert_context_maps_input(&fixer, &input, &result);
        let faces = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
        let face_count = faces.count();
        let edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        let edge_count = edges.clone().count();
        let degenerated_count = edges
            .filter(|edge| {
                let edge = rustycad_tkbrep::topods::edge(edge).unwrap();
                rustycad_tkbrep::brep::BRepTool::degenerated(&edge).unwrap()
            })
            .count();
        let actual = format!(
            "spindle_torus_open_u changed=1 faces={face_count} edges={edge_count} degenerated_edges={degenerated_count}"
        );
        assert_eq!(
            actual,
            include_str!("../../../../../../tools/oracle/shape_fix_face_missing_seam.expected")
                .lines()
                .nth(2)
                .unwrap()
        );
    }

    #[test]
    fn periodic_bspline_sphere_open_wire_matches_native_result() {
        let face = bspline_sphere_with_open_u_wire().unwrap();
        let input = face.into_shape();
        let fixer = ShapeFixFace::new(input.clone());
        assert!(fixer.fix_missing_seam().unwrap());
        let result = fixer.result();
        assert_context_maps_input(&fixer, &input, &result);
        let face_count = Explorer::new(&result, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .count();
        let edges = Explorer::new(&result, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
        let edge_count = edges.clone().count();
        let degenerated_count = edges
            .filter(|edge| {
                let edge = rustycad_tkbrep::topods::edge(edge).unwrap();
                rustycad_tkbrep::brep::BRepTool::degenerated(&edge).unwrap()
            })
            .count();
        let actual = format!(
            "periodic_bspline_sphere_open_u changed=1 faces={face_count} edges={edge_count} degenerated_edges={degenerated_count}"
        );
        assert_eq!(
            actual,
            include_str!("../../../../../../tools/oracle/shape_fix_face_missing_seam.expected")
                .lines()
                .nth(3)
                .unwrap()
        );
    }

    fn assert_context_maps_input(
        fixer: &ShapeFixFace,
        input: &rustycad_tkbrep::topods::Shape,
        result: &rustycad_tkbrep::topods::Shape,
    ) {
        let context = fixer.context();
        assert!(!context.is_null());
        assert!(
            context
                .apply(input, ShapeEnum::Shape)
                .unwrap()
                .is_same(result)
        );
    }

    fn cylinder_with_two_open_u_wires() -> Result<Face, rustycad_tkernel::standard::Failure> {
        let surface: Handle<dyn GeomSurface> =
            Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0)?).cast();
        let mut face = Face::default();
        let builder = BRepBuilder::default();
        builder.make_face_on_surface(&mut face, &surface, precision::confusion())?;
        add_cylinder_wire(&builder, &mut face, -1.0, false)?;
        add_cylinder_wire(&builder, &mut face, 1.0, true)?;
        Ok(face)
    }

    fn spindle_torus_with_open_u_wire() -> Result<Face, rustycad_tkernel::standard::Failure> {
        let surface: Handle<dyn GeomSurface> =
            Handle::new(GeomToroidalSurface::new(Axis3::default(), 1.0, 2.0)?).cast();
        let mut face = Face::default();
        let builder = BRepBuilder::default();
        builder.make_face_on_surface(&mut face, &surface, precision::confusion())?;
        let circle_axis = Axis2::with_standard_frame(
            Point3::default(),
            StandardDirection3::Z,
            StandardDirection3::X,
        );
        let circle: Handle<dyn rustycad_tkg3d::geom::GeomCurve> =
            Handle::new(GeomCircle::new(circle_axis, 3.0)?).cast();
        let edge =
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve(&circle)?
                .edge()?;
        add_periodic_boundary(&builder, &mut face, &edge, 0.0, Direction2::X)?;
        Ok(face)
    }

    fn bspline_sphere_with_open_u_wire() -> Result<Face, rustycad_tkernel::standard::Failure> {
        let sphere: Handle<dyn GeomSurface> =
            Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.0)?).cast();
        let bspline = rustycad_tkgeombase::geom_convert::surface_to_bspline_surface(&sphere)?;
        let surface: Handle<dyn GeomSurface> = bspline.cast();
        let mut face = Face::default();
        let builder = BRepBuilder::default();
        builder.make_face_on_surface(&mut face, &surface, precision::confusion())?;
        let circle: Handle<dyn rustycad_tkg3d::geom::GeomCurve> =
            Handle::new(GeomCircle::new(Axis2::default(), 1.0)?).cast();
        let edge =
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve(&circle)?
                .edge()?;
        add_periodic_boundary(&builder, &mut face, &edge, 0.0, Direction2::X)?;
        Ok(face)
    }

    fn add_cylinder_wire(
        builder: &BRepBuilder,
        face: &mut Face,
        z: f64,
        reverse: bool,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        let axis = Axis2::with_standard_frame(
            Point3::new(0.0, 0.0, z),
            StandardDirection3::Z,
            StandardDirection3::X,
        );
        let circle: Handle<dyn rustycad_tkg3d::geom::GeomCurve> =
            Handle::new(GeomCircle::new(axis, 1.0)?).cast();
        let edge =
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve(&circle)?
                .edge()?;
        let direction = if reverse {
            Direction2::from_standard(StandardDirection2::NegX)
        } else {
            Direction2::X
        };
        add_periodic_boundary(builder, face, &edge, z, direction)
    }

    fn add_periodic_boundary(
        builder: &BRepBuilder,
        face: &mut Face,
        edge: &rustycad_tkbrep::topods::Edge,
        v: f64,
        direction: Direction2,
    ) -> Result<(), rustycad_tkernel::standard::Failure> {
        builder.range(edge, 0.0, TAU, true)?;
        let pcurve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::from_point(Point2::new(0.0, v), direction)).cast();
        builder.update_edge_pcurve_on_face(edge, &pcurve, face, precision::confusion())?;
        let (support, location) = rustycad_tkbrep::brep::BRepTool::surface_with_location(face)?;
        builder.range_on_surface(edge, &support, &location, 0.0, TAU)?;
        let mut wire = rustycad_tkbrep::topods::Wire::default();
        rustycad_tkbrep::topods::Builder.make_wire(&mut wire);
        rustycad_tkbrep::topods::Builder.add(&mut wire, &edge.clone().into_shape())?;
        rustycad_tkbrep::topods::Builder.add(face, &wire.into_shape())?;
        Ok(())
    }

    fn sphere_with_open_u_wire()
    -> Result<(Face, Handle<dyn GeomSurface>), rustycad_tkernel::standard::Failure> {
        let sphere = GeomSphericalSurface::new(Axis3::default(), 1.0)?;
        let surface: Handle<dyn GeomSurface> = Handle::new(sphere).cast();
        let mut face = Face::default();
        let builder = BRepBuilder::default();
        builder.make_face_on_surface(&mut face, &surface, precision::confusion())?;
        let circle: Handle<dyn rustycad_tkg3d::geom::GeomCurve> =
            Handle::new(GeomCircle::new(Axis2::default(), 1.0)?).cast();
        let edge =
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve(&circle)?
                .edge()?;
        builder.range(&edge, 0.0, TAU, true)?;
        let pcurve: Handle<dyn Geom2dCurve> =
            Handle::new(Geom2dLine::from_point(Point2::default(), Direction2::X)).cast();
        builder.update_edge_pcurve_on_face(&edge, &pcurve, &face, precision::confusion())?;
        let (support, location) = rustycad_tkbrep::brep::BRepTool::surface_with_location(&face)?;
        builder.range_on_surface(&edge, &support, &location, 0.0, TAU)?;
        let mut wire = rustycad_tkbrep::topods::Wire::default();
        rustycad_tkbrep::topods::Builder.make_wire(&mut wire);
        rustycad_tkbrep::topods::Builder.add(&mut wire, &edge.into_shape())?;
        rustycad_tkbrep::topods::Builder.add(&mut face, &wire.into_shape())?;
        Ok((face, surface))
    }
}
