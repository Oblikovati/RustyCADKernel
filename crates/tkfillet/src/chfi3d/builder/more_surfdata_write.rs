// Rust translation of the data structure part of ChFi3d_Builder::PerformMoreSurfdata
// (ChFi3d_Builder_C1.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of one contour on two surface data written: the last surface data
//! removed, its cut kept on the previous one, and the curves closing the hole on the
//! fillets and the faces.
use super::ChFi3dBuilder;
use super::more_surfdata::{MoreSurfdata, TOL3D, ends, sd_at};
use crate::chfi3d::{fil_curve_in_ds, fil_point_in_ds, index_point_in_ds};
use crate::chfi_ds::lock;
use rustycad_tkbool::top_ope_brep_ds::{
    TopOpeBRepDSCurve, TopOpeBRepDSKind, TopOpeBRepDSPoint, TopOpeBRepDSSolidSurfaceInterference,
    TopOpeBRepDSTransition,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, compose, reverse};
use rustycad_tkgeombase::geom_proj_lib;
use rustycad_tkmath::gp::Point3;

/// A curve of the hole: its curve, the points at its ends, its pcurve on the surface it
/// is put on and the transition there.
struct HoleCurve<'a> {
    c: &'a Handle<dyn GeomCurve>,
    tol: f64,
    points: (i32, i32),
}

/// The two ends of the curves are not joined (neither the first to the last of the
/// other nor the last to the first): the orientation is reversed.
fn not_chained(a: &Handle<dyn GeomCurve>, b: &Handle<dyn GeomCurve>) -> Result<bool, Failure> {
    let ((p1, p2), (p3, p4)) = (ends(a)?, ends(b)?);
    Ok(p1.distance(&p4) > TOL3D && p2.distance(&p3) > TOL3D)
}

impl ChFi3dBuilder {
    /// The curve added with its ends: index of the curve.
    fn add_hole_curve(&mut self, h: &HoleCurve<'_>) -> Result<i32, Failure> {
        let ic = self.ds.add_curve(&mut TopOpeBRepDSCurve::new(h.c, h.tol, false));
        let g = h.c.get().ok_or_else(|| Failure::new("PerformMoreSurfdata: null curve"))?;
        let li = self.ds.change_curve_interferences(ic);
        li.append(fil_point_in_ds(Orientation::Forward, ic, h.points.0, g.first_parameter(), false));
        li.append(fil_point_in_ds(Orientation::Reversed, ic, h.points.1, g.last_parameter(), false));
        Ok(ic)
    }

    /// The curve `ic` put on the fillet surface `isurf`.
    fn put_on_surface(&mut self, ic: i32, isurf: i32, pc: &Handle<dyn Geom2dCurve>, or: Orientation) {
        let interf = fil_curve_in_ds(ic, isurf, pc, or);
        self.ds.change_surface_interferences(isurf).append(interf);
    }

    /// The curve `ic` put on the face of index `ishape`.
    fn put_on_face(&mut self, ic: i32, ishape: i32, pc: &Handle<dyn Geom2dCurve>, or: Orientation) -> Result<(), Failure> {
        let interf = fil_curve_in_ds(ic, ishape, pc, or);
        self.ds.change_shape_interferences_index(ishape)?.append(interf);
        Ok(())
    }

    /// The corner written (the second half of `PerformMoreSurfdata`).
    pub(super) fn write_more_surfdata(&mut self, ms: MoreSurfdata) -> Result<(), Failure> {
        let sd_prev = sd_at(&ms.seq, ms.ind_prev)?;
        let surf_prev_sd = self.ds.surface(lock(&sd_prev).surf()).surface().clone();
        let pc_cracc_s = geom_proj_lib::curve2d(&ms.tr_cracc, &surf_prev_sd)?;
        let side_prev = if ms.is2nd_cp1_on_arc { 1 } else { 2 };
        let (fi_prev, ind_shape, or_sd2) = {
            let g = lock(&sd_prev);
            (g.interference(side_prev).clone(), g.index(side_prev), g.orientation())
        };
        if ind_shape <= 0 {
            return Ok(());
        }
        let mut cur_orient = self.ds.shape(ind_shape, true).orientation();
        cur_orient = compose(cur_orient, or_sd2);
        cur_orient = compose(reverse(fi_prev.transition()), cur_orient);
        let sd = sd_at(&ms.seq, ms.ind)?;
        let (ind_surf1, or_sd1) = {
            let g = lock(&sd);
            (g.surf(), g.orientation())
        };
        let ind_cp1_on_arc = self.ds.add_point(&TopOpeBRepDSPoint::new(ms.cp1_on_arc.point(), ms.cp1_on_arc.tolerance()));
        let arc1 = ms.cp1_on_arc.arc()?.clone();
        let ind_arc1 = self.ds.add_shape(arc1.as_ref());
        let ind_sol = lock(&ms.stripe).solid_index();
        let interfp = fil_point_in_ds(
            ms.cp1_on_arc.transition_on_arc()?,
            ind_arc1,
            ind_cp1_on_arc,
            ms.cp1_on_arc.parameter_on_arc()?,
            false,
        );
        self.ds.change_shape_interferences(arc1.as_ref()).append(interfp);
        let ssi = TopOpeBRepDSSolidSurfaceInterference::new(
            &TopOpeBRepDSTransition::from_orientation(or_sd1),
            TopOpeBRepDSKind::Solid,
            ind_sol,
            TopOpeBRepDSKind::Surface,
            ind_surf1,
        );
        self.ds.change_shape_interferences_index(ind_sol)?.append(Handle::new(ssi).cast());
        let mut ind = ms.ind;
        lock(&ms.seq).remove((ind - 1) as usize);
        if !ms.isfirst {
            ind -= 1;
        }
        let sd_cur = sd_at(&ms.seq, ind)?;
        let (ind_cp2_on_arc, ind_cp2_not_on_arc) = self.more_surfdata_points(&ms)?;
        let side = if ms.is2nd_cp1_on_arc { 1 } else { 2 };
        let (fi, ind_shape) = {
            let g = lock(&sd_cur);
            (g.interference(side).clone(), g.index(side))
        };
        let face_shape = topods::face(self.ds.shape(ind_shape, true))?;
        let stemp2 = BRepTool::surface(&face_shape)?;
        let s2 = stemp2.get().ok_or_else(|| Failure::new("PerformMoreSurfdata: null surface"))?;
        let pof = fi.pcurve_on_face().clone();
        let uv = pof.value(fi.last_parameter())?;
        let point2 = s2.value(uv.x(), uv.y())?;
        let uv = pof.value(fi.first_parameter())?;
        let point1 = s2.value(uv.x(), uv.y())?;
        let mut or_surf = cur_orient;
        let ind_a_face = self.ds.add_shape(ms.face.as_ref());
        let cur_surf = lock(&sd_cur).surf();
        let mut ind_point = ind_cp2_on_arc;
        if let Some((pext, cint2)) = &ms.second {
            ind_point = self.ds.add_point(&TopOpeBRepDSPoint::new(*pext, cint2.tol));
            let (e1, e2) = ends(&cint2.c)?;
            let points = if e1.distance(pext) <= TOL3D { (ind_point, ind_cp2_on_arc) } else { (ind_cp2_on_arc, ind_point) };
            let ic = self.add_hole_curve(&HoleCurve { c: &cint2.c, tol: cint2.tol, points })?;
            if e1.distance(&point2) > TOL3D && e2.distance(&point1) > TOL3D {
                or_surf = reverse(or_surf);
            }
            self.put_on_surface(ic, cur_surf, &cint2.pc2, or_surf);
            let or_face = if ms.face.orientation() == or_sd2 { reverse(or_surf) } else { or_surf };
            self.put_on_face(ic, ind_a_face, &cint2.pc1, or_face)?;
        }
        let (e1, e2) = ends(&ms.tr_cracc)?;
        let points = if e1.distance(&ms.cp2_not_on_arc.point()) <= TOL3D {
            (ind_cp2_not_on_arc, ind_point)
        } else {
            (ind_point, ind_cp2_not_on_arc)
        };
        let to_reverse = match &ms.second {
            Some((_, cint2)) => not_chained(&ms.tr_cracc, &cint2.c)?,
            None => e1.distance(&point2) > TOL3D && e2.distance(&point1) > TOL3D,
        };
        if to_reverse {
            or_surf = reverse(or_surf);
        }
        let ic = self.add_hole_curve(&HoleCurve { c: &ms.tr_cracc, tol: ms.cracc.tolreached, points })?;
        self.put_on_surface(ic, cur_surf, &pc_cracc_s, or_surf);
        lock(&ms.stripe).in_ds(ms.isfirst, 1);
        if or_sd1 == or_sd2 {
            or_surf = reverse(or_surf);
        }
        self.put_on_surface(ic, ind_surf1, &ms.cracc.pcurv, or_surf);
        self.write_cint1_and_line(&ms, (ind_cp1_on_arc, ind_point, ind_cp2_not_on_arc), (ind_surf1, ind_a_face, or_sd1), or_surf)
    }

    /// The indices of the points on the arc and off it, set on the contour.
    fn more_surfdata_points(&mut self, ms: &MoreSurfdata) -> Result<(i32, i32), Failure> {
        let i_on = index_point_in_ds(&ms.cp2_on_arc, &mut self.ds)?;
        let i_off = index_point_in_ds(&ms.cp2_not_on_arc, &mut self.ds)?;
        let (s_on, s_off) = if ms.is2nd_cp1_on_arc { (1, 2) } else { (2, 1) };
        let mut s = lock(&ms.stripe);
        s.set_index_point(i_on, ms.isfirst, s_on);
        s.set_index_point(i_off, ms.isfirst, s_off);
        Ok((s.index_point(ms.isfirst, s_on), s.index_point(ms.isfirst, s_off)))
    }

    /// The intersection of the face with the last fillet and its line on the neighbour.
    fn write_cint1_and_line(
        &mut self,
        ms: &MoreSurfdata,
        (ind_cp1_on_arc, ind_point, ind_cp2_not_on_arc): (i32, i32, i32),
        (ind_surf1, ind_a_face, or_sd1): (i32, i32, Orientation),
        mut or_surf: Orientation,
    ) -> Result<(), Failure> {
        let (e1, _) = ends(&ms.cint1.c)?;
        let points = if e1.distance(&ms.cp1_on_arc.point()) <= TOL3D { (ind_cp1_on_arc, ind_point) } else { (ind_point, ind_cp1_on_arc) };
        if not_chained(&ms.cint1.c, &ms.tr_cracc)? {
            or_surf = reverse(or_surf);
        }
        let ic = self.add_hole_curve(&HoleCurve { c: &ms.cint1.c, tol: ms.cint1.tol, points })?;
        self.put_on_surface(ic, ind_surf1, &ms.cint1.pc2, or_surf);
        let or_face = if ms.face.orientation() == or_sd1 { reverse(or_surf) } else { or_surf };
        self.put_on_face(ic, ind_a_face, &ms.cint1.pc1, or_face)?;
        let ind_line = ms.fi_line.line_index();
        let cline = self.ds.curve(ind_line).curve().clone();
        let g = cline.get().ok_or_else(|| Failure::new("PerformMoreSurfdata: null line"))?;
        let tr_cline: Handle<dyn GeomCurve> =
            Handle::new(GeomTrimmedCurve::new(&cline, g.first_parameter(), g.last_parameter(), true, true)?).cast();
        let tolerance = self.ds.curve(ind_line).tolerance();
        let (l1, _): (Point3, Point3) = ends(&tr_cline)?;
        let points = if l1.distance(&ms.cp1_on_arc.point()) < TOL3D {
            (ind_cp1_on_arc, ind_cp2_not_on_arc)
        } else {
            (ind_cp2_not_on_arc, ind_cp1_on_arc)
        };
        if not_chained(&tr_cline, &ms.cint1.c)? {
            or_surf = reverse(or_surf);
        }
        let ic = self.add_hole_curve(&HoleCurve { c: &tr_cline, tol: tolerance, points })?;
        self.put_on_surface(ic, ind_surf1, ms.fi_line.pcurve_on_surf(), or_surf);
        let ind_shape = self.ds.add_shape(ms.neighbor.as_ref());
        let or_face = if ms.neighbor.orientation() == or_sd1 { reverse(or_surf) } else { or_surf };
        self.put_on_face(ic, ind_shape, ms.fi_line.pcurve_on_face(), or_face)?;
        Ok(())
    }
}
