// Rust translation of BRepOffset_Tool::CheckBounds and EnLargeFace with the static MakeFace,
// EnlargeGeometry, UpdatePCurves and CompactUVBounds of
// ModelingAlgorithms/TKOffset/BRepOffset/BRepOffset_Tool.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The enlargement of a face: its surface extended (B-spline, Bezier, extrusion and
//! revolution surfaces) and a face of the enlarged parameter box.
use super::{BRepOffsetTool, THE_INFINI, live};
use crate::brep_offset::BRepOffsetAnalyse;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d;
use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Vertex, Wire};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkfillet::chfi_ds::TypeOfConcavity;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d as Adaptor2dCurve;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{
    GeomBoundedSurface, GeomConicalSurface, GeomCurve, GeomOffsetSurface,
    GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkgeombase::geom_lib::extend_surf_by_length;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Direction2, Point2};
use rustycad_tkshhealing::shape_custom::ShapeCustomCurve2d;

/// The apex tolerance of the face enlargement.
const TOL_APEX: f64 = 1.0e-5;

/// The parameter box of an enlargement `(u1, u2, v1, v2)`.
type Bounds = [f64; 4];

/// The enlargement options of `EnlargeGeometry`.
#[derive(Clone, Copy)]
struct EnlargeOptions {
    /// The face box `(uf1, uf2, vf1, vf2)`.
    face: Bounds,
    coeff: f64,
    enlarge_u: bool,
    enlarge_vfirst: bool,
    enlarge_vlast: bool,
    /// `theLenBeforeUfirst`, `theLenAfterUlast`, `theLenBeforeVfirst`, `theLenAfterVlast`.
    lengths: [f64; 4],
}

/// The flags of a face made by `MakeFace`.
#[derive(Clone, Copy)]
struct FaceFlags {
    uclosed: bool,
    vclosed: bool,
    is_vmin_degen: bool,
    is_vmax_degen: bool,
}

/// `MakeFace(S, Um, UM, Vm, VM, uclosed, vclosed, isVminDegen, isVmaxDegen, F)`: the face of
/// the parameter box on the surface, with iso edges (the apex edges of a cone or of a
/// tiny iso degenerated), one or two wires by the infinite sides.
fn make_face(
    s: &Handle<dyn GeomSurface>,
    [umin, umax, vmin, vmax]: Bounds,
    flags: FaceFlags,
) -> Result<Face, Failure> {
    let umininf = precision::is_negative_infinite(umin);
    let umaxinf = precision::is_positive_infinite(umax);
    let vmininf = precision::is_negative_infinite(vmin);
    let vmaxinf = precision::is_positive_infinite(vmax);
    let (mut vmindegen, mut vmaxdegen) = cone_apex_degeneracy(s, vmin, vmax, flags)?;
    let tol = precision::confusion();
    let builder = BRepBuilder::default();
    let surface = live(s, "surface")?;
    let corner = |u: f64, v: f64| -> Result<Vertex, Failure> {
        let mut vertex = Vertex::default();
        builder.make_vertex_at(&mut vertex, surface.value(u, v)?, tol)?;
        Ok(vertex)
    };
    let (mut v00, mut v10, mut v11, mut v01) = (
        Vertex::default(),
        Vertex::default(),
        Vertex::default(),
        Vertex::default(),
    );
    if !umininf {
        if !vmininf {
            v00 = corner(umin, vmin)?;
        }
        if !vmaxinf {
            v01 = corner(umin, vmax)?;
        }
    }
    if !umaxinf {
        if !vmininf {
            v10 = corner(umax, vmin)?;
        }
        if !vmaxinf {
            v11 = corner(umax, vmax)?;
        }
    }
    if flags.uclosed {
        v10 = v00.clone();
        v11 = v01.clone();
    }
    if flags.vclosed {
        v01 = v00.clone();
        v11 = v10.clone();
    }
    if vmindegen {
        v10 = v00.clone();
    }
    if vmaxdegen {
        v11 = v01.clone();
    }
    let line = |x: f64, y: f64, along_u: bool| -> Result<Handle<dyn Geom2dCurve>, Failure> {
        let d = if along_u {
            Direction2::new(1., 0.)?
        } else {
            Direction2::new(0., 1.)?
        };
        Ok(Handle::new(Geom2dLine::from_point(Point2::new(x, y), d)).cast())
    };
    let lumin = if umininf {
        Handle::default()
    } else {
        line(umin, 0., false)?
    };
    let lumax = if umaxinf {
        Handle::default()
    } else {
        line(umax, 0., false)?
    };
    let lvmin = if vmininf {
        Handle::default()
    } else {
        line(0., vmin, true)?
    };
    let lvmax = if vmaxinf {
        Handle::default()
    } else {
        line(0., vmax, true)?
    };
    let hasiso = s.is_kind_name("Geom_ElementarySurface");
    let isos = iso_curves(
        s,
        hasiso,
        [umin, umax, vmin, vmax],
        [umininf, umaxinf, vmininf, vmaxinf],
        (&mut vmindegen, &mut vmaxdegen),
    )?;
    let mut f = Face::default();
    builder.make_face_on_surface(&mut f, s, tol)?;
    let make_edge = |curve: &Handle<dyn GeomCurve>, use_curve: bool| -> Result<Edge, Failure> {
        let mut e = Edge::default();
        if use_curve {
            builder.make_edge_with_curve(&mut e, curve, tol)?;
        } else {
            builder.make_edge(&mut e)?;
        }
        Ok(e)
    };
    let mut eumin = Edge::default();
    if !umininf {
        eumin = make_edge(&isos[0], hasiso)?;
        if flags.uclosed {
            builder.update_edge_pcurves_on_face(&eumin, (&lumax, &lumin), &f, tol)?;
        } else {
            builder.update_edge_pcurve_on_face(&eumin, &lumin, &f, tol)?;
        }
        add_bounded(
            &builder,
            &mut eumin,
            (&mut v00, !vmininf),
            (&mut v01, !vmaxinf),
            (vmin, vmax),
        )?;
    }
    let mut eumax = Edge::default();
    if !umaxinf {
        if flags.uclosed {
            eumax = eumin.clone();
        } else {
            eumax = make_edge(&isos[1], hasiso)?;
            builder.update_edge_pcurve_on_face(&eumax, &lumax, &f, tol)?;
            add_bounded(
                &builder,
                &mut eumax,
                (&mut v10, !vmininf),
                (&mut v11, !vmaxinf),
                (vmin, vmax),
            )?;
        }
    }
    let mut evmin = Edge::default();
    if !vmininf {
        evmin = make_edge(&isos[2], hasiso && !vmindegen)?;
        if flags.vclosed {
            builder.update_edge_pcurves_on_face(&evmin, (&lvmin, &lvmax), &f, tol)?;
        } else {
            builder.update_edge_pcurve_on_face(&evmin, &lvmin, &f, tol)?;
        }
        add_bounded(
            &builder,
            &mut evmin,
            (&mut v00, !umininf),
            (&mut v10, !umaxinf),
            (umin, umax),
        )?;
        if vmindegen {
            builder.degenerated(&evmin, true)?;
        }
    }
    let mut evmax = Edge::default();
    if !vmaxinf {
        if flags.vclosed {
            evmax = evmin.clone();
        } else {
            evmax = make_edge(&isos[3], hasiso && !vmaxdegen)?;
            builder.update_edge_pcurve_on_face(&evmax, &lvmax, &f, tol)?;
            add_bounded(
                &builder,
                &mut evmax,
                (&mut v01, !umininf),
                (&mut v11, !umaxinf),
                (umin, umax),
            )?;
            if vmaxdegen {
                builder.degenerated(&evmax, true)?;
            }
        }
    }
    eumin.set_orientation(Orientation::Reversed);
    evmax.set_orientation(Orientation::Reversed);
    let infinite = [umininf, umaxinf, vmininf, vmaxinf];
    add_face_wires(
        &builder,
        &mut f,
        [&eumin, &eumax, &evmin, &evmax],
        infinite,
        flags,
    )?;
    Ok(f)
}

/// The degenerated flags at the V bounds: given, or at the apex of a cone.
fn cone_apex_degeneracy(
    s: &Handle<dyn GeomSurface>,
    vmin: f64,
    vmax: f64,
    flags: FaceFlags,
) -> Result<(bool, bool), Failure> {
    let (mut vmindegen, mut vmaxdegen) = (flags.is_vmin_degen, flags.is_vmax_degen);
    let mut the_surf = s.clone();
    if s.is_instance_name("Geom_RectangularTrimmedSurface") {
        the_surf = live(
            &s.clone().cast::<GeomRectangularTrimmedSurface>(),
            "surface",
        )?
        .basis_surface();
    }
    if the_surf.is_instance_name("Geom_ConicalSurface") {
        let the_cone = live(&the_surf.clone().cast::<GeomConicalSurface>(), "cone")?.cone()?;
        let (_, vapex) = elslib::parameters(&the_cone, the_cone.apex());
        if (vmin - vapex).abs() <= precision::confusion() {
            vmindegen = true;
        }
        if (vmax - vapex).abs() <= precision::confusion() {
            vmaxdegen = true;
        }
    }
    Ok((vmindegen, vmaxdegen))
}

/// The iso curves at `umin`, `umax`, `vmin`, `vmax` of an elementary surface (null
/// handles otherwise); a V iso of size within the apex tolerance makes its side
/// degenerated.
fn iso_curves(
    s: &Handle<dyn GeomSurface>,
    hasiso: bool,
    [umin, umax, vmin, vmax]: Bounds,
    [umininf, umaxinf, vmininf, vmaxinf]: [bool; 4],
    (vmindegen, vmaxdegen): (&mut bool, &mut bool),
) -> Result<[Handle<dyn GeomCurve>; 4], Failure> {
    let mut isos: [Handle<dyn GeomCurve>; 4] = Default::default();
    if !hasiso {
        return Ok(isos);
    }
    let surface = live(s, "surface")?;
    if !umininf {
        isos[0] = surface.u_iso(umin)?;
    }
    if !umaxinf {
        isos[1] = surface.u_iso(umax)?;
    }
    if !vmininf {
        isos[2] = surface.v_iso(vmin)?;
        if BRepOffsetTool::gabarit(&isos[2])? <= TOL_APEX {
            *vmindegen = true;
        }
    }
    if !vmaxinf {
        isos[3] = surface.v_iso(vmax)?;
        if BRepOffsetTool::gabarit(&isos[3])? <= TOL_APEX {
            *vmaxdegen = true;
        }
    }
    Ok(isos)
}

/// Adds the bounding vertices (FORWARD first, REVERSED last) present on the edge and
/// sets its range.
fn add_bounded(
    builder: &BRepBuilder,
    e: &mut Edge,
    (first, has_first): (&mut Vertex, bool),
    (last, has_last): (&mut Vertex, bool),
    (f, l): (f64, f64),
) -> Result<(), Failure> {
    if has_first {
        first.set_orientation(Orientation::Forward);
        builder.add(e, first.as_ref())?;
    }
    if has_last {
        last.set_orientation(Orientation::Reversed);
        builder.add(e, last.as_ref())?;
    }
    builder.range(e, f, l, false)
}

/// The wires of `MakeFace`: two wires of the U (or V) edges when the other direction
/// is infinite, else one wire of the existing edges.
fn add_face_wires(
    builder: &BRepBuilder,
    f: &mut Face,
    [eumin, eumax, evmin, evmax]: [&Edge; 4],
    [umininf, umaxinf, vmininf, vmaxinf]: [bool; 4],
    flags: FaceFlags,
) -> Result<(), Failure> {
    let wire_of = |edges: &[&Edge]| -> Result<Wire, Failure> {
        let mut w = Wire::default();
        builder.make_wire(&mut w);
        for e in edges {
            builder.add(&mut w, e.as_ref())?;
        }
        Ok(w)
    };
    if !umininf && !umaxinf && vmininf && vmaxinf {
        builder.add(f, wire_of(&[eumin])?.as_ref())?;
        builder.add(f, wire_of(&[eumax])?.as_ref())?;
        return f.set_closed(flags.uclosed);
    }
    if umininf && umaxinf && !vmininf && !vmaxinf {
        builder.add(f, wire_of(&[evmin])?.as_ref())?;
        builder.add(f, wire_of(&[evmax])?.as_ref())?;
        return f.set_closed(flags.vclosed);
    }
    if umininf && umaxinf && vmininf && vmaxinf {
        return Ok(());
    }
    let mut edges = Vec::new();
    for (e, inf) in [
        (eumin, umininf),
        (evmin, vmininf),
        (eumax, umaxinf),
        (evmax, vmaxinf),
    ] {
        if !inf {
            edges.push(e);
        }
    }
    let w = wire_of(&edges)?;
    builder.add(f, w.as_ref())?;
    w.set_closed(!umininf && !umaxinf && !vmininf && !vmaxinf)?;
    f.set_closed(flags.uclosed && flags.vclosed)
}

/// The length of an iso curve times the coefficient.
fn iso_length(c: &Handle<dyn GeomCurve>, coeff: f64) -> Result<f64, Failure> {
    Ok(AbscissaPoint::length(
        &GeomAdaptorCurve::with_curve(c)? as &dyn rustycad_tkg3d::adaptor3d::Adaptor3dCurve
    )? * coeff)
}

/// A requested length, or the default when it is -1.
fn length_or(requested: f64, default: f64) -> f64 {
    if requested == -1. { default } else { requested }
}

/// `EnlargeGeometry(S, U1, U2, V1, V2, IsV1degen, IsV2degen, ...)`: the surface extended
/// (through a trimmed or an offset surface to its basis) and its new parameter box;
/// whether the surface changed.
fn enlarge_geometry(
    s: &mut Handle<dyn GeomSurface>,
    box_: &mut Bounds,
    degen: (&mut bool, &mut bool),
    o: EnlargeOptions,
) -> Result<bool, Failure> {
    if s.is_instance_name("Geom_RectangularTrimmedSurface") {
        return enlarge_trimmed(s, box_, degen, o);
    }
    if s.is_instance_name("Geom_OffsetSurface") {
        let offset = s.clone().cast::<GeomOffsetSurface>();
        let mut surf = live(&offset, "surface")?.basis_surface();
        let change = enlarge_geometry(&mut surf, box_, degen, o)?;
        live(&offset, "surface")?.set_basis_surface(&surf, false)?;
        return Ok(change);
    }
    if s.is_instance_name("Geom_SurfaceOfLinearExtrusion")
        || s.is_instance_name("Geom_SurfaceOfRevolution")
    {
        return enlarge_swept(s, box_, degen, o);
    }
    if s.is_instance_name("Geom_BezierSurface") || s.is_instance_name("Geom_BSplineSurface") {
        return enlarge_spline(s, box_, degen, o);
    }
    let (uu1, uu2, vv1, vv2) = live(s, "surface")?.bounds();
    box_[0] = uu1.max(box_[0]);
    box_[2] = vv1.max(box_[2]);
    box_[1] = uu2.min(box_[1]);
    box_[3] = vv2.min(box_[3]);
    Ok(false)
}

/// The trimmed-surface branch of `EnlargeGeometry`: the basis enlarged, then trimmed
/// again unless both V sides were enlarged.
fn enlarge_trimmed(
    s: &mut Handle<dyn GeomSurface>,
    box_: &mut Bounds,
    degen: (&mut bool, &mut bool),
    o: EnlargeOptions,
) -> Result<bool, Failure> {
    let mut bs = live(
        &s.clone().cast::<GeomRectangularTrimmedSurface>(),
        "surface",
    )?
    .basis_surface();
    enlarge_geometry(&mut bs, box_, degen, o)?;
    if !o.enlarge_vfirst {
        box_[2] = o.face[2];
    }
    if !o.enlarge_vlast {
        box_[3] = o.face[3];
    }
    if !o.enlarge_vfirst || !o.enlarge_vlast {
        let trimmed = GeomRectangularTrimmedSurface::new(
            &bs,
            (box_[0], box_[1]),
            (box_[2], box_[3]),
            (true, true),
        )?;
        *s = Handle::new(trimmed).cast();
    } else {
        *s = bs;
    }
    Ok(true)
}

/// The extrusion and revolution branch of `EnlargeGeometry`.
fn enlarge_swept(
    s: &mut Handle<dyn GeomSurface>,
    box_: &mut Bounds,
    (is_v1degen, is_v2degen): (&mut bool, &mut bool),
    o: EnlargeOptions,
) -> Result<bool, Failure> {
    let [uf1, uf2, vf1, vf2] = o.face;
    let surface = live(s, "surface")?;
    let (mut u1, mut u2, mut v1, mut v2) = surface.bounds();
    let (mut du_first, mut du_last, mut dv_first, mut dv_last) = (0., 0., 0., 0.);
    let (mut enlarge_u, mut enlarge_v) = (o.enlarge_u, true);
    let (mut enlarge_ufirst, mut enlarge_ulast) = (enlarge_u, enlarge_u);
    let (mut enlarge_vfirst, mut enlarge_vlast) = (o.enlarge_vfirst, o.enlarge_vlast);
    if precision::is_infinite(u1) || precision::is_infinite(u2) {
        du_first = uf2 - uf1;
        du_last = du_first;
        u1 = uf1 - du_first;
        u2 = uf2 + du_last;
        enlarge_u = false;
    } else if surface.is_u_closed()? {
        enlarge_u = false;
    } else {
        let du_default = iso_length(&surface.v_iso(vf1)?, o.coeff)?;
        du_first = length_or(o.lengths[0], du_default);
        du_last = length_or(o.lengths[1], du_default);
        enlarge_ufirst &= BRepOffsetTool::gabarit(&surface.u_iso(uf1)?)? > TOL_APEX;
        enlarge_ulast &= BRepOffsetTool::gabarit(&surface.u_iso(uf2)?)? > TOL_APEX;
    }
    if precision::is_infinite(v1) || precision::is_infinite(v2) {
        dv_first = vf2 - vf1;
        dv_last = dv_first;
        v1 = vf1 - dv_first;
        v2 = vf2 + dv_last;
        enlarge_v = false;
    } else if surface.is_v_closed()? {
        enlarge_v = false;
    } else {
        let dv_default = iso_length(&surface.u_iso(uf1)?, o.coeff)?;
        dv_first = length_or(o.lengths[2], dv_default);
        dv_last = length_or(o.lengths[3], dv_default);
        if BRepOffsetTool::gabarit(&surface.v_iso(vf1)?)? <= TOL_APEX {
            enlarge_vfirst = false;
            *is_v1degen = true;
        }
        if BRepOffsetTool::gabarit(&surface.v_iso(vf2)?)? <= TOL_APEX {
            enlarge_vlast = false;
            *is_v2degen = true;
        }
    }
    let mut a_surf: Handle<dyn GeomBoundedSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(s, (u1, u2), (v1, v2), (true, true))?,
    )
    .cast();
    let extensions = [
        (
            enlarge_u && enlarge_ufirst && du_first != 0.,
            du_first,
            true,
            false,
        ),
        (
            enlarge_u && enlarge_ulast && du_last != 0.,
            du_last,
            true,
            true,
        ),
        (
            enlarge_v && enlarge_vfirst && dv_first != 0.,
            dv_first,
            false,
            false,
        ),
        (
            enlarge_v && enlarge_vlast && dv_last != 0.,
            dv_last,
            false,
            true,
        ),
    ];
    extend_sides(&mut a_surf, &extensions)?;
    *s = a_surf.cast();
    let (b0, b1, b2, b3) = live(s, "surface")?.bounds();
    *box_ = [b0, b1, b2, b3];
    Ok(true)
}

/// Extends the bounded surface by each enabled `(enabled, length, in_u, after)` side.
fn extend_sides(
    a_surf: &mut Handle<dyn GeomBoundedSurface>,
    extensions: &[(bool, f64, bool, bool)],
) -> Result<(), Failure> {
    for &(enabled, length, in_u, after) in extensions {
        if enabled {
            extend_surf_by_length(a_surf, length, 1, in_u, after)?;
        }
    }
    Ok(())
}

/// The B-spline and Bezier branch of `EnlargeGeometry`.
fn enlarge_spline(
    s: &mut Handle<dyn GeomSurface>,
    box_: &mut Bounds,
    (is_v1degen, is_v2degen): (&mut bool, &mut bool),
    o: EnlargeOptions,
) -> Result<bool, Failure> {
    let [uf1, uf2, vf1, vf2] = o.face;
    let surface = live(s, "surface")?;
    let mut enlarge_u = o.enlarge_u && !surface.is_u_closed()?;
    let mut enlarge_v = !surface.is_v_closed()?;
    let (mut enlarge_ufirst, mut enlarge_ulast) = (o.enlarge_u, o.enlarge_u);
    let (mut enlarge_vfirst, mut enlarge_vlast) = (o.enlarge_vfirst, o.enlarge_vlast);
    let (duf, dvf) = (uf2 - uf1, vf2 - vf1);
    let (u1, u2, v1, v2) = surface.bounds();
    let (uiso1, viso1) = (surface.u_iso(u1)?, surface.v_iso(v1)?);
    let gabarit_uiso1 = BRepOffsetTool::gabarit(&uiso1)?;
    let gabarit_uiso2 = BRepOffsetTool::gabarit(&surface.u_iso(u2)?)?;
    let gabarit_viso1 = BRepOffsetTool::gabarit(&viso1)?;
    let gabarit_viso2 = BRepOffsetTool::gabarit(&surface.v_iso(v2)?)?;
    if gabarit_viso1 <= TOL_APEX || gabarit_viso2 <= TOL_APEX {
        enlarge_u = false;
    }
    if gabarit_uiso1 <= TOL_APEX || gabarit_uiso2 <= TOL_APEX {
        enlarge_v = false;
    }
    let (mut du_first, mut du_last, mut dv_first, mut dv_last) = (0., 0., 0., 0.);
    if enlarge_u {
        let du_default = iso_length(&viso1, o.coeff)?;
        du_first = length_or(o.lengths[0], du_default);
        du_last = length_or(o.lengths[1], du_default);
        enlarge_ufirst &= gabarit_uiso1 > TOL_APEX;
        enlarge_ulast &= gabarit_uiso2 > TOL_APEX;
    }
    if enlarge_v {
        let dv_default = iso_length(&uiso1, o.coeff)?;
        dv_first = length_or(o.lengths[2], dv_default);
        dv_last = length_or(o.lengths[3], dv_default);
        if gabarit_viso1 <= TOL_APEX {
            enlarge_vfirst = false;
            *is_v1degen = true;
        }
        if gabarit_viso2 <= TOL_APEX {
            enlarge_vlast = false;
            *is_v2degen = true;
        }
    }
    let mut a_surf: Handle<dyn GeomBoundedSurface> = s.clone().cast();
    let extensions = [
        (
            enlarge_u && enlarge_ufirst && uf1 - u1 < duf && du_first != 0.,
            du_first,
            true,
            false,
        ),
        (
            enlarge_u && enlarge_ulast && u2 - uf2 < duf && du_last != 0.,
            du_last,
            true,
            true,
        ),
        (
            enlarge_v && enlarge_vfirst && vf1 - v1 < dvf && dv_first != 0.,
            dv_first,
            false,
            false,
        ),
        (
            enlarge_v && enlarge_vlast && v2 - vf2 < dvf && dv_last != 0.,
            dv_last,
            false,
            true,
        ),
    ];
    extend_sides(&mut a_surf, &extensions)?;
    *s = a_surf.cast();
    let (b0, b1, b2, b3) = live(s, "surface")?.bounds();
    *box_ = [b0, b1, b2, b3];
    Ok(true)
}

/// `UpdatePCurves(F, BF)`: the p-curves of the edges of `F` moved onto the surface of
/// `BF` (both FORWARD).
fn update_pcurves(f: &Face, bf: &Face) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    let mut emap = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(f.as_ref(), ShapeEnum::Edge, &mut emap)?;
    let null_pcurve: Handle<dyn Geom2dCurve> = Handle::default();
    for i in 1..=emap.extent() {
        let mut ce = topods::edge(emap.find_key(i)?)?;
        ce.set_orientation(Orientation::Forward);
        let pc = BRepTool::curve_on_face(&ce, f)?;
        if pc.pcurve.is_null() {
            continue;
        }
        let tol = BRepTool::edge_tolerance(&ce)?;
        let (mut first, mut last) = (pc.first, pc.last);
        if BRepTool::is_closed_on_face(&ce, f)? {
            ce.reverse();
            let pcr = BRepTool::curve_on_face(&ce, f)?;
            (first, last) = (pcr.first, pcr.last);
            builder.update_edge_pcurves_on_face(&ce, (&null_pcurve, &null_pcurve), f, tol)?;
            builder.update_edge_pcurves_on_face(&ce, (&pc.pcurve, &pcr.pcurve), bf, tol)?;
        } else {
            builder.update_edge_pcurve_on_face(&ce, &null_pcurve, f, tol)?;
            builder.update_edge_pcurve_on_face(&ce, &pc.pcurve, bf, tol)?;
        }
        builder.range(&ce, first, last, false)?;
    }
    Ok(())
}

/// `CompactUVBounds(F, UMin, UMax, VMin, VMax)`: the box of 33 samples of each p-curve
/// of the face (so that it covers at most one period), or the surface bounds.
fn compact_uv_bounds(f: &Face) -> Result<Bounds, Failure> {
    let n = 33.;
    let mut b = BoundingBox2::default();
    let mut exp = Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
    while exp.more() {
        let e = topods::edge(exp.current())?;
        exp.next()?;
        let c = BRepAdaptorCurve2d::with_edge_on_face(&e, f)?;
        let (u1, u2) = BRepTool::range(&e)?;
        let du = (u2 - u1) / (n - 1.);
        let mut u = u1;
        let mut j = 1.;
        while j < n {
            b.add_point(c.value(u)?);
            u += du;
            j += 1.;
        }
        b.add_point(c.value(u2)?);
    }
    if !b.is_void() {
        let [umin, vmin, umax, vmax] = b.get()?;
        return Ok([umin, umax, vmin, vmax]);
    }
    let (umin, umax, vmin, vmax) = live(&BRepTool::surface(f)?, "surface")?.bounds();
    Ok([umin, umax, vmin, vmax])
}

/// The basis of a rectangular trimmed surface, any other surface unchanged.
fn untrimmed(s: Handle<dyn GeomSurface>) -> Result<Handle<dyn GeomSurface>, Failure> {
    if s.is_instance_name("Geom_RectangularTrimmedSurface") {
        return Ok(live(
            &s.clone().cast::<GeomRectangularTrimmedSurface>(),
            "surface",
        )?
        .basis_surface());
    }
    Ok(s)
}

/// The line of a p-curve: the basis of a trimmed curve when it is a line, or the line a
/// Bezier or B-spline p-curve converts to.
fn pcurve_line(an_edge: &Edge, f: &Face) -> Result<Option<Handle<Geom2dLine>>, Failure> {
    let pc = BRepTool::curve_on_face(an_edge, f)?;
    let mut a_curve = pc.pcurve;
    if a_curve.is_instance_name("Geom2d_TrimmedCurve") {
        a_curve = live(&a_curve.clone().cast::<Geom2dTrimmedCurve>(), "pcurve")?.basis_curve();
    }
    if a_curve.is_instance_name("Geom2d_Line") {
        return Ok(Some(a_curve.cast()));
    }
    if a_curve.is_instance_name("Geom2d_BezierCurve")
        || a_curve.is_instance_name("Geom2d_BSplineCurve")
    {
        let converted = ShapeCustomCurve2d::convert_to_line2d(
            &a_curve,
            pc.first,
            pc.last,
            precision::confusion(),
        )?;
        return Ok(converted.map(|c| c.line));
    }
    Ok(None)
}

/// The bounds read by `CheckBounds` from the tangent or degenerated iso edges.
struct IsoBounds {
    ubound: i32,
    vbound: i32,
    ufirst: f64,
    ulast: f64,
    vfirst: f64,
    vlast: f64,
}

impl BRepOffsetTool {
    /// `CheckBounds(F, Analyse, enlargeU, enlargeVfirst, enlargeVlast)`: which sides of
    /// an extrusion, revolution or spline face may be enlarged, refused when two
    /// tangent iso edges bound it at its compact box in that direction (and the V side
    /// of a degenerated iso); `let (u, vf, vl) = BRepOffsetTool::check_bounds(&face, &analyse)?;`.
    pub fn check_bounds(
        f: &Face,
        analyse: &BRepOffsetAnalyse,
    ) -> Result<(bool, bool, bool), Failure> {
        let (mut enlarge_u, mut enlarge_vfirst, mut enlarge_vlast) = (true, true, true);
        let mut iso = IsoBounds {
            ubound: 0,
            vbound: 0,
            ufirst: f64::MAX,
            ulast: f64::MIN,
            vfirst: f64::MAX,
            vlast: f64::MIN,
        };
        let [uf1, uf2, vf1, vf2] = compact_uv_bounds(f)?;
        let the_surf = untrimmed(BRepTool::surface(f)?)?;
        let swept = [
            "Geom_SurfaceOfLinearExtrusion",
            "Geom_SurfaceOfRevolution",
            "Geom_BezierSurface",
            "Geom_BSplineSurface",
        ];
        if swept.iter().any(|name| the_surf.is_instance_name(name)) {
            let mut explo = Explorer::new(f.as_ref(), ShapeEnum::Edge, ShapeEnum::Shape)?;
            while explo.more() {
                let an_edge = topods::edge(explo.current())?;
                explo.next()?;
                read_iso_bound(
                    f,
                    &an_edge,
                    analyse,
                    vf1,
                    (&mut iso, &mut enlarge_vfirst, &mut enlarge_vlast),
                )?;
            }
        }
        if iso.ubound >= 2
            && (uf1 - iso.ufirst).abs() <= precision::confusion()
            && (uf2 - iso.ulast).abs() <= precision::confusion()
        {
            enlarge_u = false;
        }
        if iso.vbound >= 2
            && (vf1 - iso.vfirst).abs() <= precision::confusion()
            && (vf2 - iso.vlast).abs() <= precision::confusion()
        {
            enlarge_vfirst = false;
            enlarge_vlast = false;
        }
        Ok((enlarge_u, enlarge_vfirst, enlarge_vlast))
    }
}

/// One edge of `CheckBounds`: a tangent (or degenerated) edge whose p-curve is a U or V
/// iso line counts as a bound.
fn read_iso_bound(
    f: &Face,
    an_edge: &Edge,
    analyse: &BRepOffsetAnalyse,
    vf1: f64,
    (iso, enlarge_vfirst, enlarge_vlast): (&mut IsoBounds, &mut bool, &mut bool),
) -> Result<(), Failure> {
    let l = analyse.type_of(an_edge)?;
    let degenerated = BRepTool::degenerated(an_edge)?;
    if l.is_empty() && !degenerated {
        return Ok(());
    }
    let first = l.first().ok_or_else(|| {
        Failure::with_message(
            "Standard_NoSuchObject",
            "BRepOffset_Tool::CheckBounds: empty interval list",
        )
    })?;
    if first.get_type()? != TypeOfConcavity::Tangential && !degenerated {
        return Ok(());
    }
    let Some(the_line) = pcurve_line(an_edge, f)? else {
        return Ok(());
    };
    let line = live(&the_line, "line")?;
    let (dir, loc) = (line.direction(), line.location());
    if dir.is_parallel(Direction2::new(1., 0.)?, precision::angular()) {
        iso.vbound += 1;
        if degenerated {
            if (loc.y() - vf1).abs() <= precision::confusion() {
                *enlarge_vfirst = false;
            } else {
                *enlarge_vlast = false;
            }
        } else {
            iso.vfirst = iso.vfirst.min(loc.y());
            iso.vlast = iso.vlast.max(loc.y());
        }
    } else if dir.is_parallel(Direction2::new(0., 1.)?, precision::angular()) {
        iso.ubound += 1;
        iso.ufirst = iso.ufirst.min(loc.x());
        iso.ulast = iso.ulast.max(loc.x());
    }
    Ok(())
}

/// The options of `EnLargeFace` past the face: whether the surface may be extended,
/// the p-curves updated, each side enlarged, the extension mode and side lengths.
#[derive(Clone, Copy, Debug)]
pub struct EnlargeFaceOptions {
    /// `CanExtentSurface`.
    pub can_extent_surface: bool,
    /// `UpdatePCurve`.
    pub update_pcurve: bool,
    /// `theEnlargeU`.
    pub enlarge_u: bool,
    /// `theEnlargeVfirst`.
    pub enlarge_vfirst: bool,
    /// `theEnlargeVlast`.
    pub enlarge_vlast: bool,
    /// `theExtensionMode`: 1 enlarges to `TheInfini`, any other value ten face sizes.
    pub extension_mode: i32,
    /// `theLenBeforeUfirst`, `theLenAfterUlast`, `theLenBeforeVfirst`,
    /// `theLenAfterVlast` (-1 for the default length).
    pub lengths: [f64; 4],
}

impl EnlargeFaceOptions {
    /// The native defaults of `EnLargeFace(F, NF, ChangeGeom)`: no p-curve update, every
    /// side enlarged, mode 1, default lengths.
    pub fn with_change_geom(can_extent_surface: bool) -> Self {
        Self {
            can_extent_surface,
            update_pcurve: false,
            enlarge_u: true,
            enlarge_vfirst: true,
            enlarge_vlast: true,
            extension_mode: 1,
            lengths: [-1.; 4],
        }
    }
}

impl BRepOffsetTool {
    /// `EnLargeFace(F, BF, CanExtentSurface, UpdatePCurve, theEnlargeU, ..., theLenAfterVlast)`:
    /// the face of the enlarged parameter box (at most a tenth of the missing period
    /// beyond a periodic face, stopped at the apex of a cone), with the p-curves of the
    /// face moved to the extended surface on request; whether the surface changed;
    /// `BRepOffsetTool::en_large_face(&face, &mut bf, EnlargeFaceOptions::with_change_geom(true))?`.
    pub fn en_large_face(f: &Face, bf: &mut Face, o: EnlargeFaceOptions) -> Result<bool, Failure> {
        let (mut s, l) = BRepTool::surface_with_location(f)?;
        let surface = live(&s, "surface")?;
        let face_box = if surface.is_u_periodic() || surface.is_v_periodic() {
            compact_uv_bounds(f)?
        } else {
            let (u1, u2, v1, v2) = brep_tools::uv_bounds(f)?;
            [u1, u2, v1, v2]
        };
        let (us1, us2, vs1, vs2) = surface.bounds();
        let (mut box_, coeff) = initial_box(face_box, o.extension_mode);
        let (mut is_vv1degen, mut is_vv2degen) = (false, false);
        let mut surface_change = false;
        if o.can_extent_surface {
            let options = EnlargeOptions {
                face: face_box,
                coeff,
                enlarge_u: o.enlarge_u,
                enlarge_vfirst: o.enlarge_vfirst,
                enlarge_vlast: o.enlarge_vlast,
                lengths: o.lengths,
            };
            surface_change = enlarge_geometry(
                &mut s,
                &mut box_,
                (&mut is_vv1degen, &mut is_vv2degen),
                options,
            )?;
        } else {
            box_ = [
                us1.max(box_[0]),
                box_[1].min(us2),
                vs1.max(box_[2]),
                box_[3].min(vs2),
            ];
        }
        let (uperiodic, vperiodic) = periodic_box(&s, face_box, &mut box_)?;
        cone_apex_limit(&s, face_box, &mut box_)?;
        if !o.enlarge_u {
            box_[0] = face_box[0];
            box_[1] = face_box[1];
        }
        if !o.enlarge_vfirst {
            box_[2] = face_box[2];
        }
        if !o.enlarge_vlast {
            box_[3] = face_box[3];
        }
        let (mut uclosed, mut vclosed) = brep_tools::detect_closedness(f)?;
        if uclosed && !uperiodic && (o.lengths[0] != 0. || o.lengths[1] != 0.) {
            uclosed = false;
        }
        if vclosed && !vperiodic && (o.lengths[2] != 0. && o.lengths[3] != 0.) {
            vclosed = false;
        }
        let flags = FaceFlags {
            uclosed,
            vclosed,
            is_vmin_degen: is_vv1degen,
            is_vmax_degen: is_vv2degen,
        };
        *bf = make_face(&s, box_, flags)?;
        bf.set_location(&l, true)?;
        if surface_change && o.update_pcurve {
            update_pcurves(&topods::face(&f.oriented(Orientation::Forward))?, bf)?;
            BRepBuilder::default().update_face(f, &s, &l, BRepTool::face_tolerance(f)?)?;
        }
        bf.set_orientation(f.orientation());
        Ok(surface_change)
    }
}

/// The initial enlarged box and coefficient of an extension mode.
fn initial_box([uf1, uf2, vf1, vf2]: Bounds, extension_mode: i32) -> (Bounds, f64) {
    if extension_mode == 1 {
        return ([-THE_INFINI, THE_INFINI, -THE_INFINI, THE_INFINI], 0.25);
    }
    let (face_du, face_dv) = (uf2 - uf1, vf2 - vf1);
    (
        [
            uf1 - 10. * face_du,
            uf2 + 10. * face_du,
            vf1 - 10. * face_dv,
            vf2 + 10. * face_dv,
        ],
        1.,
    )
}

/// The periodic directions of the surface bounded to the face box plus a tenth of the
/// missing period on each side, at most one period; whether U and V are periodic.
fn periodic_box(
    s: &Handle<dyn GeomSurface>,
    [uf1, uf2, vf1, vf2]: Bounds,
    box_: &mut Bounds,
) -> Result<(bool, bool), Failure> {
    let surface = live(s, "surface")?;
    let alpha = 0.1;
    let uperiodic = surface.is_u_periodic();
    if uperiodic {
        let period = surface.u_period()?;
        let delta = period - (uf2 - uf1);
        box_[0] = uf1 - alpha * delta;
        box_[1] = uf2 + alpha * delta;
        if box_[1] - box_[0] > period {
            box_[1] = box_[0] + period;
        }
    }
    let vperiodic = surface.is_v_periodic();
    if vperiodic {
        let period = surface.v_period()?;
        let delta = period - (vf2 - vf1);
        box_[2] = vf1 - alpha * delta;
        box_[3] = vf2 + alpha * delta;
        if box_[3] - box_[2] > period {
            box_[3] = box_[2] + period;
        }
    }
    Ok((uperiodic, vperiodic))
}

/// The V box of a cone stopped at its apex on the side of the face.
fn cone_apex_limit(
    s: &Handle<dyn GeomSurface>,
    [_, _, vf1, vf2]: Bounds,
    box_: &mut Bounds,
) -> Result<(), Failure> {
    let the_surf = untrimmed(s.clone())?;
    if !the_surf.is_instance_name("Geom_ConicalSurface") {
        return Ok(());
    }
    let the_cone = live(&the_surf.clone().cast::<GeomConicalSurface>(), "cone")?.cone()?;
    let (_, vapex) = elslib::parameters(&the_cone, the_cone.apex());
    if box_[2] < vapex && vapex < box_[3] {
        if vapex - vf1 >= TOL_APEX || vapex - vf2 >= TOL_APEX {
            box_[3] = vapex;
        } else {
            box_[2] = vapex;
        }
    }
    Ok(())
}
