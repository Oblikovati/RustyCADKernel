// Rust translation of the static GetSurface (face, shell, edge and wire) and GetCurve of
// ModelingAlgorithms/TKShHealing/ShapeAnalysis/ShapeAnalysis_CanonicalRecognition.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The analytic surfaces and curves under faces, shells, edges and wires.
use super::params::{compare_surf_params, deviation_surf_params, set_surf_params, surface_nb_pars};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Face, ShapeIterator, Shell, Wire};
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use rustycad_tkgeombase::geom_convert::{ConvType, CurveToAnaCurve, SurfToAnaSurf};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::Axis3;

type SurfaceCandidates = Vec<(Handle<dyn GeomSurface>, f64)>;

/// The in/out arguments `thePos`, `theParams`, `theGap` and `theStatus` of the edge and
/// wire members.
pub(super) struct SurfFit<'a> {
    pub pos: &'a mut Axis3,
    pub params: &'a mut [f64],
    pub gap: &'a mut f64,
    pub status: &'a mut i32,
}

/// `GetSurface(theFace, ...)`: the analytic surface of the face (moved by its location)
/// and its gap; status 1 for a face without surface.
pub(super) fn face_surface(
    face: &Face,
    tol: f64,
    conv_type: ConvType,
    target: SurfaceType,
    gap: &mut f64,
    status: &mut i32,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    *status = 0;
    let (surface, location) = BRepTool::surface_with_location(face)?;
    if surface.is_null() {
        *status = 1;
        return Ok(surface);
    }
    let mut conv = SurfToAnaSurf::with_surface(&surface);
    conv.set_conv_type(conv_type);
    conv.set_target(target);
    let ana = conv.convert_to_analytical(tol)?;
    let Some(ana_surface) = ana.get() else {
        return Ok(ana);
    };
    if !location.is_identity() {
        ana_surface.transform(location.transformation())?;
    }
    *gap = conv.gap();
    Ok(ana)
}

/// `GetSurface(theShell, ...)`: the analytic surface of the first face when every face
/// has one. Natively each later face is compared with the parameters of the first
/// surface itself (`SetSurfParams(theTarget, anElemSurf, ...)`), so the comparison always
/// holds.
pub(super) fn shell_surface(
    shell: &Shell,
    tol: f64,
    conv_type: ConvType,
    target: SurfaceType,
    gap: &mut f64,
    status: &mut i32,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut it = ShapeIterator::new(shell, true, true);
    if !it.more() {
        *status = 1;
        return Ok(Handle::default());
    }
    let nb = surface_nb_pars(target).max(1);
    let (mut pos1, mut params1) = (Axis3::default(), vec![0.0; nb]);
    let first = topods::face(it.value()?)?;
    let elem = face_surface(&first, tol, conv_type, target, gap, status)?;
    if elem.is_null() {
        return Ok(elem);
    }
    set_surf_params(target, &elem, &mut pos1, &mut params1)?;
    it.next();
    let (mut pos, mut params) = (Axis3::default(), vec![0.0; nb]);
    while it.more() {
        let face = topods::face(it.value()?)?;
        let other = face_surface(&face, tol, conv_type, target, gap, status)?;
        if other.is_null() {
            return Ok(other);
        }
        set_surf_params(target, &elem, &mut pos, &mut params)?;
        if !compare_surf_params(target, tol, &pos1, &params1, &pos, &params)? {
            return Ok(Handle::default());
        }
        it.next();
    }
    Ok(elem)
}

/// The analytic surfaces of the curves on surfaces of an edge, with their gaps.
fn edge_surfaces(
    edge: &Edge,
    tol: f64,
    conv_type: ConvType,
    target: SurfaceType,
) -> Result<SurfaceCandidates, Failure> {
    let mut found = Vec::new();
    let mut j = 0;
    loop {
        j += 1;
        let on_surface = BRepTool::curve_on_surface_by_index(edge, j)?;
        if on_surface.surface.is_null() {
            break;
        }
        let mut conv = SurfToAnaSurf::with_surface(&on_surface.surface);
        conv.set_conv_type(conv_type);
        conv.set_target(target);
        let ana = conv.convert_to_analytical(tol)?;
        let Some(ana_surface) = ana.get() else {
            continue;
        };
        if !on_surface.location.is_identity() {
            ana_surface.transform(on_surface.location.transformation())?;
        }
        found.push((ana.clone(), conv.gap()));
    }
    Ok(found)
}

/// `GetSurface(theEdge, ...)`: among the analytic surfaces of the edge's curves on
/// surfaces, the one closest to the given frame and parameters (the only one when there is
/// one); its frame and parameters and gap are written; status 1 when there is none.
pub(super) fn edge_surface(
    edge: &Edge,
    tol: f64,
    conv_type: ConvType,
    target: SurfaceType,
    fit: &mut SurfFit<'_>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let surfs = edge_surfaces(edge, tol, conv_type, target)?;
    if surfs.is_empty() {
        *fit.status = 1;
        return Ok(Handle::default());
    }
    let (mut pos, mut params) = (Axis3::default(), vec![0.0; surface_nb_pars(target).max(1)]);
    let mut ifit: Option<usize> = (surfs.len() == 1).then_some(0);
    if surfs.len() > 1 {
        let mut min_dev = real_last();
        for (i, (surface, _)) in surfs.iter().enumerate() {
            set_surf_params(target, surface, &mut pos, &mut params)?;
            let dev = deviation_surf_params(target, fit.pos, fit.params, &pos, &params)?;
            if dev < min_dev {
                min_dev = dev;
                ifit = Some(i);
            }
        }
    }
    let Some(i) = ifit else {
        *fit.status = 1;
        return Ok(Handle::default());
    };
    set_surf_params(target, &surfs[i].0, fit.pos, fit.params)?;
    *fit.gap = surfs[i].1;
    Ok(surfs[i].0.clone())
}

/// `GetSurface(theWire, ...)`: the surface of the first edge (seeded by the given frame
/// and parameters) when every other edge (seeded by the first result) gives a surface of
/// the same parameters; the frame, parameters and gap of the first edge are written.
pub(super) fn wire_surface(
    wire: &Wire,
    tol: f64,
    conv_type: ConvType,
    target: SurfaceType,
    fit: &mut SurfFit<'_>,
) -> Result<Handle<dyn GeomSurface>, Failure> {
    let mut it = ShapeIterator::new(wire, true, true);
    if !it.more() {
        *fit.status = 1;
        return Ok(Handle::default());
    }
    let nb = surface_nb_pars(target);
    let mut pos1 = *fit.pos;
    let mut params1 = vec![0.0; nb.max(1)];
    params1[..nb].copy_from_slice(&fit.params[..nb]);
    let mut gap1 = 0.0;
    let first = topods::edge(it.value()?)?;
    let mut first_fit = SurfFit {
        pos: &mut pos1,
        params: &mut params1,
        gap: &mut gap1,
        status: fit.status,
    };
    let elem1 = edge_surface(&first, tol, conv_type, target, &mut first_fit)?;
    if *fit.status != 0 || elem1.is_null() {
        return Ok(Handle::default());
    }
    it.next();
    while it.more() {
        let mut pos = pos1;
        let mut params = vec![0.0; nb.max(1)];
        params[..nb].copy_from_slice(&params1[..nb]);
        let mut gap = 0.0;
        let edge = topods::edge(it.value()?)?;
        let mut edge_fit = SurfFit {
            pos: &mut pos,
            params: &mut params,
            gap: &mut gap,
            status: fit.status,
        };
        let elem = edge_surface(&edge, tol, conv_type, target, &mut edge_fit)?;
        if *fit.status != 0 || elem.is_null() {
            return Ok(Handle::default());
        }
        if !compare_surf_params(target, tol, &pos1, &params1, &pos, &params)? {
            return Ok(Handle::default());
        }
        it.next();
    }
    set_surf_params(target, &elem1, fit.pos, fit.params)?;
    *fit.gap = gap1;
    Ok(elem1)
}

/// `GetCurve(theEdge, ...)`: the analytic curve of the edge's 3D curve on its range
/// (moved by its location) and its gap; status 1 for an edge without 3D curve.
pub(super) fn edge_curve(
    edge: &Edge,
    tol: f64,
    conv_type: ConvType,
    target: CurveType,
    gap: &mut f64,
    status: &mut i32,
) -> Result<Handle<dyn GeomCurve>, Failure> {
    *status = 0;
    let curve3d = BRepTool::curve_with_location(edge)?;
    if curve3d.curve.is_null() {
        *status = 1;
        return Ok(curve3d.curve);
    }
    let mut conv = CurveToAnaCurve::with_curve(&curve3d.curve);
    conv.set_conv_type(conv_type);
    conv.set_target(target);
    let mut ana: Handle<dyn GeomCurve> = Handle::default();
    // The native `nf` and `nl` are uninitialized; they only receive the new range.
    let mut new_range = (0.0, 0.0);
    conv.convert_to_analytical(tol, &mut ana, (curve3d.first, curve3d.last), &mut new_range)?;
    let Some(ana_curve) = ana.get() else {
        return Ok(ana);
    };
    if !curve3d.location.is_identity() {
        ana_curve.transform(curve3d.location.transformation())?;
    }
    *gap = conv.gap();
    Ok(ana)
}
