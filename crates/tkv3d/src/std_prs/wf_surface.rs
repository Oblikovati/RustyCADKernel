//! `StdPrs_WFSurface` and `StdPrs_WFDeflectionSurface`: the free boundaries and the U and V
//! isolines of an adaptor surface over its (limited) parameter range.
use super::{StdPrsCurve, StdPrsDeflectionCurve};
use crate::prs3d::Prs3dDrawer;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_min;
use rustycad_tkg3d::adaptor3d::{Adaptor3dIsoCurve, Adaptor3dSurface};
use rustycad_tkgeombase::bnd_lib::AddSurface;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::geom_abs::IsoType;
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::aspect::AspectTypeOfDeflection;
use rustycad_tkservice::graphic3d::*;

/// `FindLimits`: the natural parameter bounds; an infinite end is brought in by halving
/// steps from twice the limit until the end points are within `limit` (U first, at the
/// middle or finite V, then V at the middle U). Returns `[u1, u2, v1, v2]`.
#[allow(clippy::neg_cmp_op_on_partial_ord)] // A NaN distance ends the native loops.
fn find_limits(surface: &dyn Adaptor3dSurface, limit: f64) -> Result<[f64; 4], Failure> {
    let mut u_first = surface.first_u_parameter()?;
    let mut u_last = surface.last_u_parameter()?;
    let mut v_first = surface.first_v_parameter()?;
    let mut v_last = surface.last_v_parameter()?;
    let u_first_inf = precision::is_negative_infinite(u_first);
    let u_last_inf = precision::is_positive_infinite(u_last);
    let v_first_inf = precision::is_negative_infinite(v_first);
    let v_last_inf = precision::is_positive_infinite(v_last);
    let at = |u: f64, v: f64| surface.value(u, v);
    if u_first_inf || u_last_inf {
        let v = match (v_first_inf, v_last_inf) {
            (true, true) => 0.,
            (true, false) => v_last,
            (false, true) => v_first,
            (false, false) => (v_first + v_last) / 2.,
        };
        let mut delta = limit * 2.;
        if u_first_inf && u_last_inf {
            loop {
                delta /= 2.;
                u_first = -delta;
                u_last = delta;
                if !(at(u_first, v)?.distance(&at(u_last, v)?) > limit) {
                    break;
                }
            }
        } else if u_first_inf {
            let p2 = at(u_last, v)?;
            loop {
                delta /= 2.;
                u_first = u_last - delta;
                if !(at(u_first, v)?.distance(&p2) > limit) {
                    break;
                }
            }
        } else {
            let p1 = at(u_first, v)?;
            loop {
                delta /= 2.;
                u_last = u_first + delta;
                if !(p1.distance(&at(u_last, v)?) > limit) {
                    break;
                }
            }
        }
    }
    if v_first_inf || v_last_inf {
        let u = (u_first + u_last) / 2.;
        let mut delta = limit * 2.;
        if v_first_inf && v_last_inf {
            loop {
                delta /= 2.;
                v_first = -delta;
                v_last = delta;
                if !(at(u, v_first)?.distance(&at(u, v_last)?) > limit) {
                    break;
                }
            }
        } else if v_first_inf {
            let p2 = at(u, v_last)?;
            loop {
                delta /= 2.;
                v_first = v_last - delta;
                if !(at(u, v_first)?.distance(&p2) > limit) {
                    break;
                }
            }
        } else {
            let p1 = at(u, v_first)?;
            loop {
                delta /= 2.;
                v_last = v_first + delta;
                if !(p1.distance(&at(u, v_last)?) > limit) {
                    break;
                }
            }
        }
    }
    Ok([u_first, u_last, v_first, v_last])
}

/// The isos to draw: `(kind, parameter, range)` of the free boundaries, the U isos and the
/// V isos (`nb` isos over `nb` steps on a closed direction, `nb + 1` otherwise).
type Iso = (IsoType, f64, (f64, f64));

fn free_boundaries(bounds: [f64; 4], closed: [bool; 2]) -> Vec<Iso> {
    let [u1, u2, v1, v2] = bounds;
    let mut isos = Vec::new();
    if !closed[0] {
        isos.push((IsoType::IsoU, u1, (v1, v2)));
        isos.push((IsoType::IsoU, u2, (v1, v2)));
    }
    if !closed[1] {
        isos.push((IsoType::IsoV, v1, (u1, u2)));
        isos.push((IsoType::IsoV, v2, (u1, u2)));
    }
    isos
}

fn u_isos(bounds: [f64; 4], closed: bool, count: i32) -> Vec<Iso> {
    let [u1, u2, v1, v2] = bounds;
    let du = if closed {
        (u2 - u1) / f64::from(count)
    } else {
        (u2 - u1) / f64::from(1 + count)
    };
    (1..=count)
        .map(|i| (IsoType::IsoU, u1 + du * f64::from(i), (v1, v2)))
        .collect()
}

fn v_isos(bounds: [f64; 4], closed: bool, count: i32) -> Vec<Iso> {
    let [u1, u2, v1, v2] = bounds;
    let dv = if closed {
        (v2 - v1) / f64::from(count)
    } else {
        (v2 - v1) / f64::from(1 + count)
    };
    (1..=count)
        .map(|i| (IsoType::IsoV, v1 + dv * f64::from(i), (u1, u2)))
        .collect()
}

/// `StdPrs_WFSurface`: the free boundaries and isolines sampled with the drawer
/// discretisation, gathered into one polyline array per family in new groups.
/// Example: `StdPrsWfSurface::add(&prs, &surface, &drawer)?`.
pub struct StdPrsWfSurface;

impl StdPrsWfSurface {
    /// `Add`: the U isos, V isos and free boundaries (an open direction contributes its two
    /// boundaries), each family in a new group with its aspect. Native quirks kept: the
    /// current group aspect is set before sampling, and the vertex capacity of an array
    /// accumulates the vertices of the families before it.
    /// Example: `StdPrsWfSurface::add(&prs, &surface, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &Handle<dyn Adaptor3dSurface>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let adaptor = surface.get().ok_or_else(null_surface)?;
        let bounds = find_limits(adaptor, drawer.maximal_parameter_value())?;
        let closed = [adaptor.is_u_closed()?, adaptor.is_v_closed()?];
        let iso = Adaptor3dIsoCurve::with_surface(surface);
        let sample = |isos: Vec<Iso>| -> Result<Vec<Vec<Point3>>, Failure> {
            let mut curves = Vec::new();
            for (kind, parameter, range) in isos {
                iso.load_iso_range(kind, parameter, range)?;
                let mut points = Sequence::default();
                StdPrsCurve::add_points(prs, &iso, drawer, &mut points, false)?;
                curves.push(points.iter().copied().collect());
            }
            Ok(curves)
        };
        let mut free = Vec::new();
        if !(closed[0] && closed[1]) {
            prs.current_group()?
                .set_primitives_aspect(&drawer.free_boundary_aspect().aspect().cast());
            free = sample(free_boundaries(bounds, closed))?;
        }
        let mut u_curves = Vec::new();
        let count = drawer.u_iso_aspect().number();
        if count != 0 {
            prs.current_group()?
                .set_primitives_aspect(&drawer.u_iso_aspect().aspect().cast());
            u_curves = sample(u_isos(bounds, closed[0], count))?;
        }
        let mut v_curves = Vec::new();
        let count = drawer.v_iso_aspect().number();
        if count != 0 {
            prs.current_group()?
                .set_primitives_aspect(&drawer.v_iso_aspect().aspect().cast());
            v_curves = sample(v_isos(bounds, closed[1], count))?;
        }
        let mut nb_vertices = 0;
        for (curves, aspect) in [
            (u_curves, drawer.u_iso_aspect().aspect()),
            (v_curves, drawer.v_iso_aspect().aspect()),
            (free, drawer.free_boundary_aspect().aspect()),
        ] {
            if curves.is_empty() {
                continue;
            }
            nb_vertices += curves.iter().map(|c| c.len() as i32).sum::<i32>();
            let array = Handle::new(Graphic3dArrayOfPrimitives::new(
                Graphic3dTypeOfPrimitiveArray::Polylines,
                nb_vertices,
                curves.len() as i32,
                0,
                0,
            )?);
            for points in &curves {
                array.add_bound(points.len() as i32)?;
                for point in points {
                    array.add_vertex_pnt(point)?;
                }
            }
            let group = prs.new_group()?;
            group.set_primitives_aspect(&aspect.cast());
            group.add_primitive_array(&array, true);
        }
        Ok(())
    }
}

fn null_surface() -> Failure {
    Failure::with_message("Standard_NullObject", "Adaptor3d_Surface handle is null")
}

/// `StdPrs_WFDeflectionSurface`: the free boundaries and isolines drawn with a deflection
/// into the current group.
/// Example: `StdPrsWfDeflectionSurface::add(&prs, &surface, &drawer)?`.
pub struct StdPrsWfDeflectionSurface;

impl StdPrsWfDeflectionSurface {
    /// `Add`: a relative deflection is the deviation coefficient of the smallest closed
    /// extent of the surface box (at most the chordial deviation over the coefficient),
    /// an absolute one the chordial deviation; every iso is drawn with an angle of 0.2.
    /// Example: `StdPrsWfDeflectionSurface::add(&prs, &surface, &drawer)?`.
    pub fn add(
        prs: &Handle<dyn Graphic3dStructureSource>,
        surface: &Handle<dyn Adaptor3dSurface>,
        drawer: &Handle<Prs3dDrawer>,
    ) -> Result<(), Failure> {
        let adaptor = surface.get().ok_or_else(null_surface)?;
        let limit = drawer.maximal_parameter_value();
        let bounds = find_limits(adaptor, limit)?;
        let closed = [adaptor.is_u_closed()?, adaptor.is_v_closed()?];
        let deflection = if drawer.type_of_deflection() == AspectTypeOfDeflection::Relative {
            let mut total = BoundingBox3::default();
            AddSurface::add_range(adaptor, bounds, 0., &mut total)?;
            let mut m = drawer.maximal_chordial_deviation() / drawer.deviation_coefficient();
            let [x_min, y_min, z_min, x_max, y_max, z_max] = total.get()?;
            if !(total.is_open_xmin() || total.is_open_xmax()) {
                m = std_min(m, (x_max - x_min).abs());
            }
            if !(total.is_open_ymin() || total.is_open_ymax()) {
                m = std_min(m, (y_max - y_min).abs());
            }
            if !(total.is_open_zmin() || total.is_open_zmax()) {
                m = std_min(m, (z_max - z_min).abs());
            }
            m * drawer.deviation_coefficient()
        } else {
            drawer.maximal_chordial_deviation()
        };
        let iso = Adaptor3dIsoCurve::with_surface(surface);
        let draw = |isos: Vec<Iso>| -> Result<(), Failure> {
            for (kind, parameter, range) in isos {
                iso.load_iso_range(kind, parameter, range)?;
                StdPrsDeflectionCurve::add_with_deflection(
                    prs, &iso, deflection, limit, 0.2, true,
                )?;
            }
            Ok(())
        };
        if !(closed[0] && closed[1]) {
            prs.current_group()?
                .set_primitives_aspect(&drawer.free_boundary_aspect().aspect().cast());
            draw(free_boundaries(bounds, closed))?;
        }
        let count = drawer.u_iso_aspect().number();
        if count != 0 {
            prs.current_group()?
                .set_primitives_aspect(&drawer.u_iso_aspect().aspect().cast());
            draw(u_isos(bounds, closed[0], count))?;
        }
        let count = drawer.v_iso_aspect().number();
        if count != 0 {
            prs.current_group()?
                .set_primitives_aspect(&drawer.v_iso_aspect().aspect().cast());
            draw(v_isos(bounds, closed[1], count))?;
        }
        Ok(())
    }
}
