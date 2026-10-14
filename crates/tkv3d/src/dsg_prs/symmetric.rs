//! `DsgPrs_SymmetricPresentation`: symmetry relations about an axis between two edges, two
//! arcs or two vertices, with the "-=-" symbol and the axis cursor.
use super::dimension_line::{DsgPrsLineExtent, line_extent};
use super::support::*;
use crate::prs3d::{Prs3dDimensionAspect, Prs3dDrawer};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis2, Circle3, Direction3, Line3, Point3, Vector3};
use rustycad_tkservice::graphic3d::{Graphic3dArrayOfPolylines, Graphic3dGroup};
use std::f64::consts::PI;

/// `DsgPrs_SymmetricPresentation`.
pub struct DsgPrsSymmetricPresentation;

fn aspect_ref(aspect: &Handle<Prs3dDimensionAspect>) -> Result<&Prs3dDimensionAspect, Failure> {
    aspect
        .get()
        .ok_or_else(|| null_object("DsgPrs: the dimension aspect is null"))
}

/// `gp_Vec(Vector.Divided(Vector.Magnitude()).Multiplied(Length))`.
fn scaled_unit(vector: Vector3, length: f64) -> Vector3 {
    (vector / vector.magnitude()) * length
}

/// The two arrows of the dimension line, inward unless outside or too short.
fn draw_arrows(
    group: &Handle<dyn Graphic3dGroup>,
    la: &Prs3dDimensionAspect,
    l3: &Line3,
    extent: &DsgPrsLineExtent,
    p1: &Point3,
    p2: &Point3,
) -> Result<(), Failure> {
    let arrdir = super::dimension_line::arrow_direction(l3, extent, la);
    draw_arrow(group, la, p1, &arrdir)?;
    draw_arrow(group, la, p2, &arrdir.reversed())
}

/// The "-=-" symbol at `pOff` and the axis cursor at `pm` (the shared tail of every
/// overload): three bounds of two points, a five-point cursor bound and an axis bound.
fn symmetry_marks(
    group: &Handle<dyn Graphic3dGroup>,
    vec_a: Vector3,
    axis: &Line3,
    p_off: &Point3,
    pm: &Point3,
    dist: f64,
) -> Result<(), Failure> {
    let vaxe = Vector3::from(axis.direction());
    let mut vec_axe = vec_a * 0.7;
    let prims = Graphic3dArrayOfPolylines::new(13, 5, 0, 0)?;
    add_pair(
        &prims,
        p_off.translated(vec_axe),
        p_off.translated(vec_axe.reversed()),
    )?;
    let vec1 = vec_axe * 0.6;
    vec_axe = vaxe * vec_axe.magnitude();
    let vec2 = vec_axe * 0.4;
    add_pair(
        &prims,
        p_off.translated(vec1 + vec2),
        p_off.translated(vec1.reversed() + vec2),
    )?;
    add_pair(
        &prims,
        p_off.translated(vec1 + vec2.reversed()),
        p_off.translated(vec1.reversed() + vec2.reversed()),
    )?;
    let vs = Vector3::from(axis.direction());
    let mut vsym = Vector3::from(*scaled_unit(vs, dist).xyz());
    let vsymper = Vector3::new(vsym.y(), -vsym.x(), vsym.z());
    prims.add_bound(5)?;
    let mut pm1 = pm.translated(vsym + vsymper);
    prims.add_vertex_pnt(&pm1)?;
    pm1 = pm1.translated(vsym.reversed() * 2.0 + vsymper.reversed() * 2.0);
    prims.add_vertex_pnt(&pm1)?;
    pm1 = pm1.translated(vsymper * 2.0);
    prims.add_vertex_pnt(&pm1)?;
    pm1 = pm1.translated(vsym * 2.0 + vsymper.reversed() * 2.0);
    prims.add_vertex_pnt(&pm1)?;
    pm1 = pm1.translated(vsymper * 2.0);
    prims.add_vertex_pnt(&pm1)?;
    vsym *= 4.0;
    add_pair(&prims, pm.translated(vsym), pm.translated(vsym.reversed()))?;
    add_array(group, &Handle::new(prims));
    Ok(())
}

fn add_pair(
    prims: &Graphic3dArrayOfPolylines,
    first: Point3,
    second: Point3,
) -> Result<(), Failure> {
    prims.add_bound(2)?;
    prims.add_vertex_pnt(&first)?;
    prims.add_vertex_pnt(&second)?;
    Ok(())
}

/// The symbol vector, its axis offset and the symbol point at the offset point.
fn symbol_frame(vec: Vector3, axis: &Line3, offset: &Point3) -> (Vector3, Point3) {
    let vec_a = vec * 0.1;
    let mut vec_b = Vector3::from(axis.direction()) * vec_a.magnitude();
    vec_b *= 0.5;
    (vec_a, offset.translated(vec_b))
}

/// The state of the edges overload while the dimension line is fitted.
struct EdgeLayout {
    p1: Point3,
    p2: Point3,
    l3: Line3,
    extent: DsgPrsLineExtent,
}

impl DsgPrsSymmetricPresentation {
    /// `Add(prs, drawer, A1, A2, Direction1, Axis, OffsetPoint)`: two symmetric edges.
    /// Example: `DsgPrsSymmetricPresentation::add_edges(&prs, &drawer, &a1, &a2, &d1, &axis, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_edges(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        a1: &Point3,
        a2: &Point3,
        direction1: &Direction3,
        axis: &Line3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        current_line_group(prs, la)?;
        let proj_offset = project_on_line(axis, offset)?;
        let pj_attach1 = project_on_line(axis, a1)?;
        let direction_axis = axis.direction();
        let h = (proj_offset.distance(&pj_attach1) / direction_axis.angle(direction1).cos()).abs();
        let mut vl1 = Vector3::from(*direction1);
        let vla = Vector3::between(pj_attach1, proj_offset);
        if vl1.dot(vla) < 0.0 {
            vl1.reverse();
        }
        vl1 *= h;
        let perpendicular = vla.dot(vl1) == 0.0;
        let (p1, p2) = if perpendicular {
            (
                a1.translated(vla),
                proj_offset.translated(Vector3::between(pj_attach1, *a2)),
            )
        } else {
            let p1 = a1.translated(vl1);
            (
                p1,
                proj_offset.translated(Vector3::between(p1, proj_offset)),
            )
        };
        let l3 = make_lin(&p1, &p2)?;
        let extent = line_extent(&l3, &p1, &p2, offset)?;
        let mut layout = EdgeLayout { p1, p2, l3, extent };
        let d1 = axis.distance(*a1);
        fit_edge_layout(
            prs,
            la,
            axis,
            a1,
            a2,
            direction1,
            offset,
            &proj_offset,
            &pj_attach1,
            d1,
            &mut layout,
        )?;
        let group = new_line_group(prs, la)?;
        draw_arrows(
            &group,
            la,
            &layout.l3,
            &layout.extent,
            &layout.p1,
            &layout.p2,
        )?;
        let vvar = Vector3::between(layout.p1, layout.p2);
        let vec = scaled_unit(vvar, axis.distance(*a1) + axis.distance(*a2));
        let (vec_a, p_off) = symbol_frame(vec, axis, offset);
        let pm = if perpendicular {
            layout.p1.translated(Vector3::between(
                layout.p1,
                project_on_line(axis, &layout.p1)?,
            ))
        } else {
            layout.p1.translated(vvar * 0.5)
        };
        let dist = (axis.distance(*a1) + axis.distance(*a2)) / 75.0;
        symmetry_marks(&new_line_group(prs, la)?, vec_a, axis, &p_off, &pm, dist)
    }
}

// "To avoid crossing" and the fractures of the call traits (the three layout cases).
#[allow(clippy::too_many_arguments)] // The native local state shared by the cases.
fn fit_edge_layout(
    prs: &DsgPrsPresentation,
    la: &Prs3dDimensionAspect,
    axis: &Line3,
    a1: &Point3,
    a2: &Point3,
    direction1: &Direction3,
    offset: &Point3,
    proj_offset: &Point3,
    pj_attach1: &Point3,
    d1: f64,
    layout: &mut EdgeLayout,
) -> Result<(), Failure> {
    let coeff = 0.5;
    let p1_previous = layout.p1;
    let mut cross = false;
    let attach_to_proj = Vector3::between(*a1, *pj_attach1);
    if Vector3::between(layout.p1, *proj_offset).is_opposite(attach_to_proj, confusion())? {
        cross = true;
        std::mem::swap(&mut layout.p1, &mut layout.p2);
    }
    let factor = if axis.distance(layout.p1) > d1 * (1.0 + coeff) && !cross {
        1.0 + coeff
    } else if axis.distance(layout.p1) < d1 * (1.0 - coeff) || cross {
        1.0 - coeff
    } else {
        let group = new_line_group(prs, la)?;
        let e = &layout.extent;
        add_array(
            &group,
            &segments(
                6,
                &[e.point_min, e.point_max, *a1, layout.p1, *a2, layout.p2],
            )?,
        );
        return Ok(());
    };
    let group = new_line_group(prs, la)?;
    let pj_p1 = project_on_line(axis, &layout.p1)?;
    let vfix = scaled_unit(Vector3::between(pj_p1, layout.p1), d1 * factor);
    layout.p1 = pj_p1.translated(vfix);
    layout.p2 = pj_p1.translated(vfix.reversed());
    layout.l3 = make_lin(&layout.p1, &layout.p2)?;
    layout.extent = line_extent(&layout.l3, &layout.p1, &layout.p2, offset)?;
    let alpha = axis.direction().angle(direction1);
    let b = (coeff * d1) / alpha.sin();
    let pint = a1.translated(scaled_unit(Vector3::between(*a1, p1_previous), b));
    let pj_pint = project_on_line(axis, &pint)?;
    let sym_pint = pj_pint.translated(Vector3::between(pint, pj_pint));
    let prims = Graphic3dArrayOfPolylines::new(8, 3, 0, 0)?;
    add_pair(&prims, layout.extent.point_min, layout.extent.point_max)?;
    prims.add_bound(3)?;
    for point in [*a1, pint, layout.p1] {
        prims.add_vertex_pnt(&point)?;
    }
    prims.add_bound(3)?;
    for point in [*a2, sym_pint, layout.p2] {
        prims.add_vertex_pnt(&point)?;
    }
    add_array(&group, &Handle::new(prims));
    Ok(())
}

impl DsgPrsSymmetricPresentation {
    /// `Add(prs, drawer, A1, A2, aCircle1, Axis, OffsetPoint)`: two symmetric arcs.
    /// Example: `DsgPrsSymmetricPresentation::add_arcs(&prs, &drawer, &a1, &a2, &circle, &axis, &o)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn add_arcs(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        a1: &Point3,
        a2: &Point3,
        circle1: &Circle3,
        axis: &Line3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        let mut offset_pnt = *offset;
        let center1 = circle1.location();
        let mut proj_offset = project_on_line(axis, &offset_pnt)?;
        let proj_center1 = project_on_line(axis, &center1)?;
        let mut vp = Vector3::between(proj_center1, center1);
        if vp.magnitude() <= confusion() {
            vp = Vector3::from(axis.direction()) ^ Vector3::from(circle1.position().direction());
        }
        let mut dt = proj_center1.distance(&proj_offset);
        let r = circle1.radius();
        if dt > 0.999 * r {
            dt = 0.999 * r;
            proj_offset = proj_center1
                .translated(scaled_unit(Vector3::between(proj_center1, proj_offset), dt));
            offset_pnt = proj_offset;
        }
        let h = (r * r - dt * dt).sqrt();
        let p1 = proj_offset.translated(vp + scaled_unit(vp, h));
        let p2 = proj_offset.translated(Vector3::between(p1, proj_offset));
        let l3 = make_lin(&p1, &p2)?;
        let extent = line_extent(&l3, &p1, &p2, &offset_pnt)?;
        add_array(&group, &segments(2, &[extent.point_min, extent.point_max])?);
        add_attachment_arc(&group, circle1, &p1, a1, false)?;
        let center2 = proj_center1.translated(vp.reversed());
        let circle2 = Circle3::new(
            Axis2::new(center2, circle1.axis().direction())?,
            circle1.radius(),
        )?;
        add_attachment_arc(&group, &circle2, &p2, a2, true)?;
        let group = new_line_group(prs, la)?;
        draw_arrows(&group, la, &l3, &extent, &p1, &p2)?;
        let vvar = Vector3::between(p1, p2);
        let vec = scaled_unit(vvar, 2.0 * axis.distance(center1));
        let (vec_a, p_off) = symbol_frame(vec, axis, &offset_pnt);
        let pm = p1.translated(vvar * 0.5);
        symmetry_marks(
            &new_line_group(prs, la)?,
            vec_a,
            axis,
            &p_off,
            &pm,
            axis.distance(center1) / 37.0,
        )
    }
}

// The arc from an attachment point towards the dimension end on its circle; the second arc
// widens a null span to 1e-5. The step sign follows the native table: positive when the end
// parameter is below the attachment's and the span is reflex, or above it and not reflex.
// A null first span gives `0/0` natively (an undefined int conversion); Rust gives 0 points.
fn add_attachment_arc(
    group: &Handle<dyn Graphic3dGroup>,
    circle: &Circle3,
    end: &Point3,
    attach: &Point3,
    widen_null: bool,
) -> Result<(), Failure> {
    let param_end = elclib::parameter(circle, *end)?;
    let param_attach = elclib::parameter(circle, *attach)?;
    let mut alpha = (param_end - param_attach).abs();
    if widen_null && alpha <= confusion() {
        alpha = 1.0e-5;
    }
    let reflex = alpha > PI;
    if reflex {
        alpha = (2.0 * PI) - alpha;
    }
    let nbp = (alpha / (alpha * 0.02)).trunc() as i32;
    let step = alpha / (nbp - 1) as f64;
    let dalpha = if (param_end < param_attach) == reflex {
        step
    } else {
        -step
    };
    let prims = Graphic3dArrayOfPolylines::new(nbp, 0, 0, 0)?;
    prims.add_vertex_pnt(attach)?;
    let mut alpha_iter = dalpha;
    for _ in 2..=nbp {
        prims.add_vertex_pnt(&elclib::value(param_attach + alpha_iter, circle))?;
        alpha_iter += dalpha;
    }
    add_array(group, &Handle::new(prims));
    Ok(())
}

impl DsgPrsSymmetricPresentation {
    /// `Add(prs, drawer, A1, A2, Axis, OffsetPoint)`: two symmetric vertices (a fixed symbol
    /// when they coincide). Example: `DsgPrsSymmetricPresentation::add_vertices(&prs, &drawer, &a1, &a2, &axis, &o)?`.
    pub fn add_vertices(
        prs: &DsgPrsPresentation,
        drawer: &Handle<Prs3dDrawer>,
        a1: &Point3,
        a2: &Point3,
        axis: &Line3,
        offset: &Point3,
    ) -> Result<(), Failure> {
        let aspect = dimension_aspect(drawer)?;
        let la = aspect_ref(&aspect)?;
        let group = current_line_group(prs, la)?;
        if a1.is_equal(a2, confusion()) {
            return coincident_vertices(prs, la, &group, a1, offset);
        }
        let proj_offset = project_on_line(axis, offset)?;
        let proj_a1 = project_on_line(axis, a1)?;
        let to_a1 = Vector3::between(proj_a1, *a1);
        let p1 = proj_offset.translated(to_a1);
        let p2 = proj_offset.translated(to_a1.reversed());
        let l3 = make_lin(&p1, &p2)?;
        let extent = line_extent(&l3, &p1, &p2, offset)?;
        let group = new_line_group(prs, la)?;
        add_array(
            &group,
            &segments(6, &[extent.point_min, extent.point_max, *a1, p1, *a2, p2])?,
        );
        draw_arrows(&group, la, &l3, &extent, &p1, &p2)?;
        add_ring_point(prs, la, a1)?;
        let marker = ring_marker(la)?;
        let group = new_line_group(prs, la)?;
        if let Some(value) = group.get() {
            value.set_primitives_aspect(&marker.cast());
        }
        add_array(&group, &points(&[*a2])?);
        let vec = Vector3::between(p1, p2);
        let (vec_a, p_off) = symbol_frame(vec, axis, offset);
        let pm = p1.translated(vec * 0.5);
        symmetry_marks(
            &new_line_group(prs, la)?,
            vec_a,
            axis,
            &p_off,
            &pm,
            p1.distance(&p2) / 75.0,
        )
    }
}

// A ring marker at the vertex and a short symbol along the offset direction.
fn coincident_vertices(
    prs: &DsgPrsPresentation,
    la: &Prs3dDimensionAspect,
    group: &Handle<dyn Graphic3dGroup>,
    attach: &Point3,
    offset: &Point3,
) -> Result<(), Failure> {
    let marker = ring_marker(la)?;
    if let Some(value) = group.get() {
        value.set_primitives_aspect(&marker.cast());
    }
    add_array(group, &points(&[*attach])?);
    let group = new_line_group(prs, la)?;
    let vao = Vector3::between(*attach, *offset);
    let u_vao = vao / vao.magnitude();
    let p_daxe = offset.translated(u_vao * 3.0);
    let p_faxe = p_daxe.translated(u_vao * 12.0);
    let n_vao = Vector3::new(-u_vao.y(), u_vao.x(), u_vao.z());
    let sg_p11 = p_daxe.translated(u_vao * 2.0 + n_vao * 2.0);
    let sg_p12 = sg_p11.translated(u_vao * 8.0);
    let n_vao_r = n_vao.reversed();
    let sg_p21 = p_daxe.translated(u_vao * 2.0 + n_vao_r * 2.0);
    let sg_p22 = sg_p21.translated(u_vao * 8.0);
    add_array(
        &group,
        &segments(
            8,
            &[
                *attach, *offset, p_daxe, p_faxe, sg_p11, sg_p12, sg_p21, sg_p22,
            ],
        )?,
    );
    Ok(())
}
