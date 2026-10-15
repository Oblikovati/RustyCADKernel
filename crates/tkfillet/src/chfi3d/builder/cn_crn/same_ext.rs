// Rust translation of the file static PerformTwoCornerSameExt (ChFi3d_Builder_CnCrn.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Two contours whose ends share their common points: the corner is the intersection of
//! the two fillets, kept when it joins those points.
use crate::chfi_ds::{CommonPoint, StripeHandle, SurfDataHandle, lock};
use crate::chfi3d::{
    IntersectionCurves, bound_surf, compute_curves, fil_curve_in_ds, fil_point_in_ds,
    index_point_in_ds, surf_data_at,
};
use rustycad_tkbool::top_ope_brep_ds::{TopOpeBRepDSCurve, TopOpeBRepDSDataStructure};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::{self, Orientation};
use rustycad_tkmath::gp::{Point2, Point3};

/// One contour of the corner: the stripe, its surface data at the vertex and its sense.
type End<'a> = (&'a StripeHandle, i32, i32);

/// The null object failure of the corner.
fn null() -> Failure {
    Failure::with_message("Standard_NullObject", "PerformTwoCornerSameExt")
}

/// The point of the pcurve on the fillet of the side `on_s` at its end.
fn end_uv(fd: &SurfDataHandle, on_s: i32, isfirst: bool) -> Result<Point2, Failure> {
    let g = lock(fd);
    let fi = g.interference(on_s);
    let pc = fi.pcurve_on_surf().clone();
    let u = fi.parameter(isfirst);
    drop(g);
    pc.get().ok_or_else(null)?.value(u)
}

/// The ends of the two fillets on their surfaces (`Pardeb` on S1 and `Parfin` on S2,
/// the sides of the second one swapped when its first common point is not the first
/// one of the first).
fn ends_uv(
    fd: [&SurfDataHandle; 2],
    first: [bool; 2],
    same: bool,
) -> Result<([f64; 4], [f64; 4]), Failure> {
    let (pfi11, pfi12) = (end_uv(fd[0], 1, first[0])?, end_uv(fd[0], 2, first[0])?);
    let (mut pfi21, mut pfi22) = (end_uv(fd[1], 1, first[1])?, end_uv(fd[1], 2, first[1])?);
    if !same {
        std::mem::swap(&mut pfi21, &mut pfi22);
    }
    Ok((
        [pfi11.x(), pfi11.y(), pfi21.x(), pfi21.y()],
        [pfi12.x(), pfi12.y(), pfi22.x(), pfi22.y()],
    ))
}

/// The common points of the end of a contour.
fn common_points(fd: &SurfDataHandle, isfirst: bool) -> (CommonPoint, CommonPoint) {
    let g = lock(fd);
    (g.vertex(isfirst, 1).clone(), g.vertex(isfirst, 2).clone())
}

/// The ends of the 3d curve of the intersection.
fn curve_ends(gc: &IntersectionCurves) -> Result<(Point3, Point3), Failure> {
    let c = gc.c3d.get().ok_or_else(null)?;
    Ok((c.value(c.first_parameter())?, c.value(c.last_parameter())?))
}

/// `PerformTwoCornerSameExt(DStr, stripe1, index1, sens1, stripe2, index2, sens2,
/// trouve)`: the intersection of the two fillets stored when it joins the common points
/// of the first one; whether it does.
pub(super) fn perform_two_corner_same_ext(
    dstr: &mut TopOpeBRepDSDataStructure,
    e1: End<'_>,
    e2: End<'_>,
) -> Result<bool, Failure> {
    let (fd1, fd2) = (surf_data_at(e1.0, e1.1)?, surf_data_at(e2.0, e2.1)?);
    let first = [e1.2 == 1, e2.2 == 1];
    let (com11, com12) = common_points(&fd1, first[0]);
    let com21 = common_points(&fd2, first[1]).0;
    let same = com11.point().distance(&com21.point()) < 1.0e-4;
    let (pardeb, parfin) = ends_uv([&fd1, &fd2], first, same)?;
    let hs1: Handle<dyn Adaptor3dSurface> = bound_surf(dstr, &fd1, 1, 2)?.cast();
    let hs2: Handle<dyn Adaptor3dSurface> = bound_surf(dstr, &fd2, 1, 2)?.cast();
    let mut gc = IntersectionCurves::default();
    if !compute_curves(
        (&hs1, &hs2),
        (pardeb, parfin),
        (1.0e-4, 1.0e-5),
        true,
        &mut gc,
    )? {
        return Ok(false);
    }
    let (p1, p2) = curve_ends(&gc)?;
    let near =
        |cp: &CommonPoint| cp.point().distance(&p1) < 1.0e-4 || cp.point().distance(&p2) < 1.0e-4;
    if !(near(&com11) && near(&com12)) {
        return Ok(false);
    }
    let mut ind = [
        index_point_in_ds(&com11, dstr)?,
        index_point_in_ds(&com12, dstr)?,
    ];
    {
        let mut s = lock(e1.0);
        s.in_ds(first[0], 1);
        s.set_index_point(ind[0], first[0], 1);
        s.set_index_point(ind[1], first[0], 2);
    }
    {
        let mut s = lock(e2.0);
        s.in_ds(first[1], 1);
        let (a, b) = if same {
            (ind[0], ind[1])
        } else {
            (ind[1], ind[0])
        };
        s.set_index_point(a, first[1], 1);
        s.set_index_point(b, first[1], 2);
    }
    let orpcurve = curve_orientation(dstr, &fd1, (p1, p2))?;
    if com11.point().distance(&p1) > 1.0e-4 {
        ind.swap(0, 1);
    }
    store_curve(dstr, (&fd1, &fd2), &gc, ind, orpcurve)?;
    Ok(true)
}

/// The orientation of the pcurve of the intersection on the first fillet: its transition
/// on its first face, reversed unless the curve runs along the trace on that face.
fn curve_orientation(
    dstr: &TopOpeBRepDSDataStructure,
    fd1: &SurfDataHandle,
    (p1, p2): (Point3, Point3),
) -> Result<Orientation, Failure> {
    let g = lock(fd1);
    let face = dstr.shape(g.index_of_s1(), true).clone();
    let mut trafil1 = top_abs::compose(face.orientation(), g.orientation());
    trafil1 = top_abs::compose(
        top_abs::reverse(g.interference_on_s1().transition()),
        trafil1,
    );
    let fi11 = g.interference_on_s1();
    let pc = fi11.pcurve_on_face().clone();
    let (fpar, lpar) = (fi11.first_parameter(), fi11.last_parameter());
    drop(g);
    let stemp = BRepTool::surface(&topods::face(&face)?)?;
    let (s, pc) = (stemp.get().ok_or_else(null)?, pc.get().ok_or_else(null)?);
    let uv4 = pc.value(lpar)?;
    let p4 = s.value(uv4.x(), uv4.y())?;
    let uv3 = pc.value(fpar)?;
    let p3 = s.value(uv3.x(), uv3.y())?;
    Ok(if p1.distance(&p4) < 1.0e-4 || p2.distance(&p3) < 1.0e-4 {
        trafil1
    } else {
        top_abs::reverse(trafil1)
    })
}

/// The intersection stored: the curve, its points and its pcurves on the two fillets
/// (reversed on the second when the fillets have the same orientation).
fn store_curve(
    dstr: &mut TopOpeBRepDSDataStructure,
    (fd1, fd2): (&SurfDataHandle, &SurfDataHandle),
    gc: &IntersectionCurves,
    ind: [i32; 2],
    mut orpcurve: Orientation,
) -> Result<(), Failure> {
    let (indic1, orsurf1) = {
        let g = lock(fd1);
        (g.surf(), g.orientation())
    };
    let (indic2, orsurf2) = {
        let g = lock(fd2);
        (g.surf(), g.orientation())
    };
    let indcurve = dstr.add_curve(&mut TopOpeBRepDSCurve::new(&gc.c3d, gc.tolreached, false));
    let c = gc.c3d.get().ok_or_else(null)?;
    let (f, l) = (c.first_parameter(), c.last_parameter());
    let i1 = fil_point_in_ds(Orientation::Forward, indcurve, ind[0], f, false);
    let i2 = fil_point_in_ds(Orientation::Reversed, indcurve, ind[1], l, false);
    dstr.change_curve_interferences(indcurve).append(i1);
    dstr.change_curve_interferences(indcurve).append(i2);
    let ic = fil_curve_in_ds(indcurve, indic1, &gc.pc1, orpcurve);
    dstr.change_surface_interferences(indic1).append(ic);
    if orsurf1 == orsurf2 {
        orpcurve = top_abs::reverse(orpcurve);
    }
    let ic = fil_curve_in_ds(indcurve, indic2, &gc.pc2, orpcurve);
    dstr.change_surface_interferences(indic2).append(ic);
    Ok(())
}
