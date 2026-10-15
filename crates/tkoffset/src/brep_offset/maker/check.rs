// Rust translation of BRepOffset_MakeOffset::CheckInputData and IsPlanar of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_MakeOffset.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The checks of the input of the offset algorithm, and the linearization of the
//! nearly planar faces.
use super::BRepOffsetMakeOffset;
use super::by_arc::entries;
use super::statics::{self, check_single_point};
use crate::brep_offset::BRepOffsetError;
use crate::brep_offset::tool::live;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::brep_tools::uv_bounds;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face, Shape, ShapeIterator, TShape};
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::geom_lib::IsPlanarSurface;
use rustycad_tkmath::geom_abs::{Shape as Continuity, SurfaceType};
use rustycad_tkmath::gp::Axis1;

/// The middle of a parameter range, one inside a half-infinite range, zero for an
/// infinite one.
fn middle(p1: f64, p2: f64) -> f64 {
    let (inf1, inf2) = (precision::is_infinite(p1), precision::is_infinite(p2));
    match (inf1, inf2) {
        (false, false) => (p1 + p2) / 2.,
        (true, false) => p2 - 1.,
        (false, true) => p1 + 1.,
        (true, true) => 0.,
    }
}

impl BRepOffsetMakeOffset {
    /// `CheckInputData(Range)`: a non-null offset (or a face offset), a connected shell,
    /// non-C0 surfaces without bad normals on a 21x21 grid and at the vertices (the
    /// degenerated points excepted); false with the error set otherwise;
    /// `m.check_input_data(&range)?`.
    pub fn check_input_data(&mut self, the_range: &ProgressRange) -> Result<bool, Failure> {
        // The initial error state.
        self.my_error = BRepOffsetError::NoError;
        self.my_bad_shape = Shape::default();
        let a_ps = ProgressScope::unnamed(the_range, 1., false);
        // A non-null offset.
        if self.my_offset.abs() <= self.my_tol {
            let is_found = entries(&self.my_face_offset)?
                .iter()
                .any(|(_, v)| v.abs() > self.my_tol);
            if !is_found {
                // No face with a non-null offset.
                self.my_error = BRepOffsetError::NullOffset;
                return Ok(false);
            }
        }
        // The connectivity of the input shape.
        if !statics::is_connected_shell(&self.my_face_comp)? {
            self.my_error = BRepOffsetError::NotConnectedShell;
            return Ok(false);
        }
        // The normals and the continuity: 21 points in each direction.
        const A_PNT_PER_DIM: i32 = 20;
        let mut a_presence: Vec<Handle<dyn TShape>> = Vec::new();
        for fs in Explorer::new(&self.my_face_comp, ShapeEnum::Face, ShapeEnum::Shape)? {
            if !a_ps.more() {
                self.my_error = BRepOffsetError::UserBreak;
                return Ok(false);
            }
            let a_f = topods::face(&fs)?;
            // The partner shapes share their geometry: no computations for them.
            let tshape = fs.tshape().clone();
            if a_presence.contains(&tshape) {
                continue;
            }
            a_presence.push(tshape);
            let (a_surf, _l) = BRepTool::surface_with_location(&a_f)?;
            let (a_umin, a_umax, a_vmin, a_vmax) = uv_bounds(&a_f)?;
            // The continuity.
            if live(&a_surf, "surface")?.continuity() == Continuity::C0 {
                self.my_error = BRepOffsetError::C0Geometry;
                return Ok(false);
            }
            // The degenerated points, not checked.
            let mut a_bad3d_pnts = Vec::new();
            for es in Explorer::new(&fs, ShapeEnum::Edge, ShapeEnum::Shape)? {
                let a_e = topods::edge(&es)?;
                if BRepTool::degenerated(&a_e)? {
                    a_bad3d_pnts.push(BRepTool::pnt(&topexp::first_vertex(&a_e, false))?);
                }
            }
            // The geometry grid.
            for i in 0..=A_PNT_PER_DIM {
                let a_u_param =
                    a_umin + (a_umax - a_umin) * f64::from(i) / f64::from(A_PNT_PER_DIM);
                for j in 0..=A_PNT_PER_DIM {
                    let a_v_param =
                        a_vmin + (a_vmax - a_vmin) * f64::from(j) / f64::from(A_PNT_PER_DIM);
                    self.my_error =
                        check_single_point(a_u_param, a_v_param, &a_surf, &a_bad3d_pnts)?;
                    if self.my_error != BRepOffsetError::NoError {
                        return Ok(false);
                    }
                }
            }
            // The vertices.
            for vs in Explorer::new(&fs, ShapeEnum::Vertex, ShapeEnum::Shape)? {
                let a_pnt2d = BRepTool::parameters(&topods::vertex(&vs)?, &a_f)?;
                self.my_error =
                    check_single_point(a_pnt2d.x(), a_pnt2d.y(), &a_surf, &a_bad3d_pnts)?;
                if self.my_error != BRepOffsetError::NoError {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// `IsPlanar()`: whether every face is a plane; with linearization allowed, the
    /// nearly planar faces are replaced by planar ones (oriented as the face).
    pub(super) fn is_planar(&mut self) -> Result<bool, Failure> {
        let mut a_is_non_planar_found = false;
        let builder = BRepBuilder::default();
        for fs in Explorer::new(&self.my_shape, ShapeEnum::Face, ShapeEnum::Shape)? {
            let a_f = topods::face(&fs)?;
            let a_bas = BRepAdaptorSurface::with_face(&a_f, false)?;
            if a_bas.get_type()? == SurfaceType::Plane {
                continue;
            }
            if !self.my_is_linearization_allowed {
                a_is_non_planar_found = true;
                continue;
            }
            // The tolerance.
            let a_tol_for_face = BRepTool::face_tolerance(&a_f)?;
            // The linearization.
            let a_surf = BRepTool::surface(&a_f)?;
            let a_planarity_checker = IsPlanarSurface::new(&a_surf, precision::confusion())?;
            if !a_planarity_checker.is_planar() {
                a_is_non_planar_found = true;
                continue;
            }
            let mut a_pln = a_planarity_checker.plan()?;
            let (u1, u2, v1, v2) = live(&a_surf, "surface")?.bounds();
            let (um, vm) = (middle(u1, u2), middle(v1, v2));
            let d1 = a_bas.d1(um, vm)?;
            let a_norm = d1.d1u.crossed(d1.d1v);
            let mut a_pln_norm = a_pln.position().direction();
            if a_norm.dot(a_pln_norm.into()) < 0. {
                a_pln_norm.reverse();
                let an_ax = Axis1::new(a_pln.position().location(), a_pln_norm);
                a_pln.set_axis(an_ax)?;
            }
            let a_plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::from_pln(a_pln)).cast();
            let mut a_planar_face = Face::default();
            builder.make_face_on_surface(&mut a_planar_face, &a_plane, a_tol_for_face)?;
            let a_face_forward = fs.oriented(Orientation::Forward);
            let mut pf = a_planar_face.as_ref().clone();
            let mut an_it_face = ShapeIterator::new(&a_face_forward, true, true);
            while an_it_face.more() {
                builder.add(&mut pf, an_it_face.value()?)?;
                an_it_face.next();
            }
            statics::remove_seam_and_degenerated_edges(
                &a_planar_face,
                &topods::face(&a_face_forward)?,
            )?;
            self.my_face_planface_map
                .bind(fs.clone(), a_planar_face.as_ref().clone());
            if self.my_faces.contains(&fs) {
                self.my_faces.remove_key(&fs);
                self.my_faces.add(a_planar_face.as_ref().clone());
            }
        }
        Ok(!a_is_non_planar_found)
    }
}
