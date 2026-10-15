//! The geometry helpers of the `DsgPrs` package class: length, angle, radius and fillet
//! layouts and the ellipse apex distance. Native output arguments are `&mut` structs whose
//! untouched fields keep the caller's values, as the native references do.
use super::support::{angular, confusion, direction_between, make_lin, project_on_line};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::{GeomCircle, GeomCurve, GeomLine, GeomSurface};
use rustycad_tkgeomalgo::geom_api::{ExtremaCurveCurve, ProjectPointOnCurve, ProjectPointOnSurf};
use rustycad_tkgeombase::extrema::ExtAlgo;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Circle3, Direction3, Ellipse3, Line3, Plane3, Point3, Vector3,
};
use std::f64::consts::PI;

/// `DsgPrs`: the package class.
pub struct DsgPrs;

/// The outputs of `ComputePlanarFacesLengthPresentation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DsgPrsPlanarLength {
    /// `EndOfArrow1`.
    pub end_of_arrow1: Point3,
    /// `EndOfArrow2`.
    pub end_of_arrow2: Point3,
    /// `DirOfArrow1`.
    pub dir_of_arrow1: Direction3,
}

/// The in/out arguments of `ComputeCurvilinearFacesLengthPresentation`.
#[derive(Clone, Default)]
pub struct DsgPrsCurvilinearLength {
    /// `EndOfArrow2`.
    pub end_of_arrow2: Point3,
    /// `DirOfArrow1`.
    pub dir_of_arrow1: Direction3,
    /// `VCurve`.
    pub v_curve: Handle<dyn GeomCurve>,
    /// `UCurve`.
    pub u_curve: Handle<dyn GeomCurve>,
    /// `FirstU`.
    pub first_u: f64,
    /// `deltaU`.
    pub delta_u: f64,
    /// `FirstV`.
    pub first_v: f64,
    /// `deltaV`.
    pub delta_v: f64,
}

/// The inputs of `ComputeFacesAnglePresentation`.
#[derive(Clone, Copy, Debug)]
pub struct DsgPrsFacesAngleInputs {
    /// `ArrowLength`.
    pub arrow_length: f64,
    /// `Value`.
    pub value: f64,
    /// `CenterPoint`.
    pub center_point: Point3,
    /// `AttachmentPoint1`.
    pub attachment_point1: Point3,
    /// `AttachmentPoint2`.
    pub attachment_point2: Point3,
    /// `dir1`.
    pub dir1: Direction3,
    /// `dir2`.
    pub dir2: Direction3,
    /// `axisdir`.
    pub axis_dir: Direction3,
    /// `isPlane`.
    pub is_plane: bool,
    /// `AxisOfSurf`.
    pub axis_of_surf: Axis1,
    /// `OffsetPoint`.
    pub offset_point: Point3,
}

/// The in/out arguments of `ComputeFacesAnglePresentation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DsgPrsFacesAngle {
    /// `AngleCirc`.
    pub angle_circ: Circle3,
    /// `FirstParAngleCirc`.
    pub first_par_angle_circ: f64,
    /// `LastParAngleCirc`.
    pub last_par_angle_circ: f64,
    /// `EndOfArrow1`.
    pub end_of_arrow1: Point3,
    /// `EndOfArrow2`.
    pub end_of_arrow2: Point3,
    /// `DirOfArrow1`.
    pub dir_of_arrow1: Direction3,
    /// `DirOfArrow2`.
    pub dir_of_arrow2: Direction3,
    /// `ProjAttachPoint2`.
    pub proj_attach_point2: Point3,
    /// `AttachCirc`.
    pub attach_circ: Circle3,
    /// `FirstParAttachCirc`.
    pub first_par_attach_circ: f64,
    /// `LastParAttachCirc`.
    pub last_par_attach_circ: f64,
}

/// The inputs of `ComputeFilletRadiusPresentation`.
#[derive(Clone, Copy, Debug)]
pub struct DsgPrsFilletInputs {
    /// `ArrowLength` (unused natively).
    pub arrow_length: f64,
    /// `Value`.
    pub value: f64,
    /// `Position`.
    pub position: Point3,
    /// `NormalDir`.
    pub normal_dir: Direction3,
    /// `FirstPoint`.
    pub first_point: Point3,
    /// `SecondPoint`.
    pub second_point: Point3,
    /// `Center`.
    pub center: Point3,
    /// `BasePnt`.
    pub base_pnt: Point3,
    /// `drawRevers`.
    pub draw_revers: bool,
}

/// The in/out arguments of `ComputeFilletRadiusPresentation`.
#[derive(Clone, Copy, Debug, Default)]
pub struct DsgPrsFillet {
    /// `SpecCase`.
    pub spec_case: bool,
    /// `FilletCirc`.
    pub fillet_circ: Circle3,
    /// `FirstParCirc`.
    pub first_par_circ: f64,
    /// `LastParCirc`.
    pub last_par_circ: f64,
    /// `EndOfArrow`.
    pub end_of_arrow: Point3,
    /// `DirOfArrow`.
    pub dir_of_arrow: Direction3,
    /// `DrawPosition`.
    pub draw_position: Point3,
}

impl DsgPrs {
    /// `ComputePlanarFacesLengthPresentation`: the arrow ends on the two attachment lines at
    /// the offset point and the first arrow direction (the plane normal for a null length).
    /// Example: `DsgPrs::compute_planar_faces_length_presentation(1.0, 1.0, &a1, &a2, &dir, &offset, &plane, &mut out)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn compute_planar_faces_length_presentation(
        first_arrow_length: f64,
        second_arrow_length: f64,
        attachment_point1: &Point3,
        attachment_point2: &Point3,
        dir_attach: &Direction3,
        offset_point: &Point3,
        plane_of_faces: &Plane3,
        out: &mut DsgPrsPlanarLength,
    ) -> Result<(), Failure> {
        let first_lin = Line3::new(*attachment_point1, *dir_attach);
        let second_lin = Line3::new(*attachment_point2, *dir_attach);
        out.end_of_arrow1 = project_on_line(&first_lin, offset_point)?;
        out.end_of_arrow2 = project_on_line(&second_lin, offset_point)?;
        let square = out.end_of_arrow1.square_distance(&out.end_of_arrow2);
        if square > precision::square_confusion() {
            let length_dir = direction_between(&out.end_of_arrow1, &out.end_of_arrow2)?;
            let sum = first_arrow_length + second_arrow_length;
            out.dir_of_arrow1 = if sum * sum < square {
                -length_dir
            } else {
                length_dir
            };
        } else {
            out.dir_of_arrow1 = plane_of_faces.axis().direction();
        }
        Ok(())
    }

    /// `ComputeCurvilinearFacesLengthPresentation`: the second arrow end on the second
    /// surface and the iso curves joining it to the second attachment point.
    /// Example: `DsgPrs::compute_curvilinear_faces_length_presentation(1.0, 1.0, &surface, &a1, &a2, &dir, &mut out)?`.
    #[allow(clippy::too_many_arguments)] // The native static member's argument list.
    pub fn compute_curvilinear_faces_length_presentation(
        first_arrow_length: f64,
        second_arrow_length: f64,
        second_surf: &Handle<dyn GeomSurface>,
        attachment_point1: &Point3,
        attachment_point2: &Point3,
        dir_attach: &Direction3,
        out: &mut DsgPrsCurvilinearLength,
    ) -> Result<(), Failure> {
        let mut projector = ProjectPointOnSurf::new();
        projector.init(*attachment_point1, second_surf, ExtAlgo::Grad)?;
        let index = nearest_parallel_projection(
            &projector,
            second_surf,
            attachment_point1,
            dir_attach,
            out,
        )?;
        out.end_of_arrow2 = projector.point(index)?;
        let (_, v1) = projector.parameters(index)?;
        let sum = first_arrow_length + second_arrow_length;
        out.dir_of_arrow1 = if sum * sum < attachment_point1.square_distance(&out.end_of_arrow2) {
            -*dir_attach
        } else {
            *dir_attach
        };
        if out.end_of_arrow2.square_distance(attachment_point2) > precision::square_confusion() {
            iso_curve_arcs(second_surf, attachment_point2, v1, &mut projector, out)?;
        }
        Ok(())
    }
}

// The index of the closest projection whose surface normal is parallel to the attachment
// direction (1 when none is); `EndOfArrow2` is overwritten by each D1 evaluation as natively.
fn nearest_parallel_projection(
    projector: &ProjectPointOnSurf,
    surface: &Handle<dyn GeomSurface>,
    attachment_point1: &Point3,
    dir_attach: &Direction3,
    out: &mut DsgPrsCurvilinearLength,
) -> Result<i32, Failure> {
    let Some(surface) = surface.get() else {
        return Err(super::support::null_object("DsgPrs: the surface is null"));
    };
    let (mut index, mut min_dist) = (1, rustycad_tkernel::standard::real::real_last());
    for i in 1..=projector.nb_points()? {
        let (u, v) = projector.parameters(i)?;
        let d1 = surface.d1(u, v)?;
        out.end_of_arrow2 = d1.point;
        let square_tolerance = precision::square_confusion();
        let local_dir = if d1.d1u.square_magnitude() <= square_tolerance
            || d1.d1v.square_magnitude() <= square_tolerance
        {
            direction_between(attachment_point1, &projector.point(i)?)?
        } else {
            Direction3::from_vector(d1.d1u ^ d1.d1v)?
        };
        if dir_attach.is_parallel(local_dir, angular()) && projector.distance(i)? < min_dist {
            index = i;
            min_dist = projector.distance(i)?;
        }
    }
    Ok(index)
}

fn iso_curve_arcs(
    surface: &Handle<dyn GeomSurface>,
    attachment_point2: &Point3,
    v1: f64,
    projector: &mut ProjectPointOnSurf,
    out: &mut DsgPrsCurvilinearLength,
) -> Result<(), Failure> {
    let Some(geometry) = surface.get() else {
        return Err(super::support::null_object("DsgPrs: the surface is null"));
    };
    out.v_curve = geometry.v_iso(v1)?;
    let mut on_curve = ProjectPointOnCurve::with_curve(out.end_of_arrow2, &out.v_curve)?;
    out.first_u = on_curve.lower_distance_parameter()?;
    projector.init(*attachment_point2, surface, ExtAlgo::Grad)?;
    let (u2, _) = projector.lower_distance_parameters()?;
    out.u_curve = geometry.u_iso(u2)?;
    on_curve.init(*attachment_point2, &out.u_curve)?;
    let last_v = on_curve.lower_distance_parameter()?;
    let intersection = geometry.value(u2, v1)?;
    on_curve.init(intersection, &out.v_curve)?;
    let last_u = on_curve.lower_distance_parameter()?;
    on_curve.init(intersection, &out.u_curve)?;
    out.first_v = on_curve.lower_distance_parameter()?;
    out.delta_u = periodic_delta(&out.v_curve, last_u - out.first_u)?;
    out.delta_v = periodic_delta(&out.u_curve, last_v - out.first_v)?;
    Ok(())
}

// The shorter way round a periodic curve when the delta exceeds half a period.
fn periodic_delta(curve: &Handle<dyn GeomCurve>, delta: f64) -> Result<f64, Failure> {
    let Some(curve) = curve.get() else {
        return Ok(delta);
    };
    if !curve.is_periodic() || delta.abs() <= curve.period()? / 2.0 {
        return Ok(delta);
    }
    let sign = if delta > 0.0 { -1.0 } else { 1.0 };
    Ok((curve.period()? - delta.abs()) * sign)
}

impl DsgPrs {
    /// `ComputeFacesAnglePresentation`: the angle arc and its arrows, and the projection of
    /// the second attachment point (a line or an arc around the surface axis).
    /// Example: `DsgPrs::compute_faces_angle_presentation(&inputs, &mut out)?`.
    pub fn compute_faces_angle_presentation(
        input: &DsgPrsFacesAngleInputs,
        out: &mut DsgPrsFacesAngle,
    ) -> Result<(), Failure> {
        if input.value > angular() && (PI - input.value).abs() > angular() {
            angle_arc(input, out)?;
        } else {
            let arrow_dir = input.axis_dir.crossed(input.dir1)?;
            out.dir_of_arrow1 = arrow_dir;
            out.dir_of_arrow2 = -arrow_dir;
            let dir_line = Line3::new(input.attachment_point1, input.dir1);
            out.end_of_arrow1 = project_on_line(&dir_line, &input.offset_point)?;
            out.end_of_arrow2 = out.end_of_arrow1;
        }
        project_second_attachment(input, out)
    }
}

// The arc of the angle between dir1 and dir2 at the offset radius, and its two arrows.
fn angle_arc(input: &DsgPrsFacesAngleInputs, out: &mut DsgPrsFacesAngle) -> Result<(), Failure> {
    let center = input.center_point;
    out.angle_circ
        .set_position(Axis2::with_x_direction(center, input.axis_dir, input.dir1)?);
    out.angle_circ
        .set_radius(center.distance(&input.offset_point))?;
    let radius = out.angle_circ.radius();
    let vec1 = Vector3::from(input.dir1) * radius;
    let vec2 = Vector3::from(input.dir2) * radius;
    let (p1, p2) = (center.translated(vec1), center.translated(vec2));
    let mut par1 = 0.0;
    let mut par2 = elclib::parameter(&out.angle_circ, p2)?;
    let par0 = elclib::parameter(&out.angle_circ, input.offset_point)?;
    let pos_vec = Vector3::between(center, input.offset_point);
    let normal_of_plane = vec1 ^ vec2;
    let sign1 = if pos_vec * (normal_of_plane ^ vec1) >= 0.0 {
        1
    } else {
        -1
    };
    let sign2 = if pos_vec * (normal_of_plane ^ vec2) >= 0.0 {
        1
    } else {
        -1
    };
    match (sign1, sign2) {
        (1, -1) => (out.first_par_angle_circ, out.last_par_angle_circ) = (par1, par2),
        (1, 1) => (out.first_par_angle_circ, out.last_par_angle_circ) = (par1, par0),
        (-1, 1) => {
            par1 += PI;
            par2 += PI;
            (out.first_par_angle_circ, out.last_par_angle_circ) = (par1, par2);
        }
        _ => {
            out.angle_circ.set_position(Axis2::with_x_direction(
                center,
                input.axis_dir,
                Direction3::from_vector(pos_vec)?,
            )?);
            par1 = elclib::parameter(&out.angle_circ, p1)?;
            par2 = elclib::parameter(&out.angle_circ, p2)?;
            (out.first_par_angle_circ, out.last_par_angle_circ) = (0.0, par2);
        }
    }
    angle_arrows(input.arrow_length, par1, par2, out)
}

fn angle_arrows(
    arrow_length: f64,
    par1: f64,
    par2: f64,
    out: &mut DsgPrsFacesAngle,
) -> Result<(), Failure> {
    out.end_of_arrow1 = elclib::value(par1, &out.angle_circ);
    out.end_of_arrow2 = elclib::value(par2, &out.angle_circ);
    let mut beta = 0.0;
    if out.angle_circ.radius() > confusion() {
        beta = arrow_length / out.angle_circ.radius();
    }
    let origin1 = elclib::value(par1 + beta, &out.angle_circ);
    let origin2 = elclib::value(par2 - beta, &out.angle_circ);
    out.dir_of_arrow1 = direction_between(&origin1, &out.end_of_arrow1)?;
    out.dir_of_arrow2 = direction_between(&origin2, &out.end_of_arrow2)?;
    let twice = arrow_length + arrow_length;
    if out.end_of_arrow1.square_distance(&out.end_of_arrow2) <= twice * twice {
        out.dir_of_arrow1.reverse();
        out.dir_of_arrow2.reverse();
    }
    Ok(())
}

// "Line or arc from AttachmentPoint2 to its projection" on the second direction line.
fn project_second_attachment(
    input: &DsgPrsFacesAngleInputs,
    out: &mut DsgPrsFacesAngle,
) -> Result<(), Failure> {
    let second_lin = Line3::new(input.center_point, input.dir2);
    if second_lin.contains(input.attachment_point2, confusion()) {
        out.proj_attach_point2 = input.attachment_point2;
        return Ok(());
    }
    if input.is_plane {
        out.proj_attach_point2 = project_on_line(&second_lin, &input.attachment_point2)?;
        return Ok(());
    }
    let line_of_axis = Line3::from_axis(input.axis_of_surf);
    let center_of_arc = project_on_line(&line_of_axis, &input.attachment_point2)?;
    let x_dir = direction_between(&center_of_arc, &input.attachment_point2)?;
    out.attach_circ.set_position(Axis2::with_x_direction(
        center_of_arc,
        input.axis_of_surf.direction(),
        x_dir,
    )?);
    out.attach_circ
        .set_radius(center_of_arc.distance(&input.attachment_point2))?;
    let circle: Handle<dyn GeomCurve> = Handle::new(GeomCircle::from_circ(out.attach_circ)).cast();
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_lin(second_lin)).cast();
    let intersection = ExtremaCurveCurve::with_curves(&circle, &line)?;
    out.proj_attach_point2 = intersection.nearest_points()?.1;
    let u2 = elclib::parameter(&out.attach_circ, out.proj_attach_point2)?;
    (out.first_par_attach_circ, out.last_par_attach_circ) =
        if u2 <= PI { (0.0, u2) } else { (u2, 2.0 * PI) };
    Ok(())
}

impl DsgPrs {
    /// `ComputeFilletRadiusPresentation`: the fillet arc, the arrow end and direction and the
    /// text position (reflected through the arrow end when `drawRevers`).
    /// Example: `DsgPrs::compute_fillet_radius_presentation(&inputs, &mut out)?`.
    pub fn compute_fillet_radius_presentation(
        input: &DsgPrsFilletInputs,
        out: &mut DsgPrsFillet,
    ) -> Result<(), Failure> {
        let dir1 = direction_between(&input.center, &input.first_point)?;
        let dir2 = direction_between(&input.center, &input.second_point)?;
        let angle = dir1.angle(&dir2);
        out.spec_case =
            angle <= angular() || (PI - angle) <= angular() || input.value <= confusion();
        if !out.spec_case {
            fillet_arc(input, dir1, dir2, out)?;
        } else {
            out.draw_position = input.position;
            out.end_of_arrow = input.base_pnt;
        }
        if input.draw_revers {
            let reversal = Vector3::between(out.draw_position, out.end_of_arrow);
            out.draw_position.translate(reversal * 2.0);
        }
        // `DirOfArrow.SetXYZ(gp_Dir(...).XYZ())` normalizes the unit vector a second time.
        let unit = direction_between(&out.draw_position, &out.end_of_arrow)?;
        out.dir_of_arrow = Direction3::from_xyz(unit.xyz())?;
        Ok(())
    }
}

fn fillet_arc(
    input: &DsgPrsFilletInputs,
    dir1: Direction3,
    dir2: Direction3,
    out: &mut DsgPrsFillet,
) -> Result<(), Failure> {
    let center = input.center;
    out.fillet_circ
        .set_position(Axis2::with_x_direction(center, input.normal_dir, dir1)?);
    out.fillet_circ
        .set_radius(center.distance(&input.first_point))?;
    let radius = out.fillet_circ.radius();
    let (vec1, vec2) = (Vector3::from(dir1) * radius, Vector3::from(dir2) * radius);
    let mut pos_vec = Vector3::default();
    if !center.is_equal(&input.position, confusion()) {
        pos_vec.set_xyz(*Vector3::between(center, input.position).xyz());
    } else {
        pos_vec.set_xyz(*(vec1 + vec2).xyz());
    }
    let normal_of_plane = vec1 ^ vec2;
    let sign1 = if pos_vec * (normal_of_plane ^ vec1) >= 0.0 {
        1
    } else {
        -1
    };
    let sign2 = if pos_vec * (normal_of_plane ^ vec2) >= 0.0 {
        1
    } else {
        -1
    };
    if sign1 != sign2 {
        out.draw_position = input.position;
        let direction = Direction3::from_vector(pos_vec)?;
        let mut angle = dir1.angle(&direction);
        if dir1.crossed(direction)? * input.normal_dir < 0.0 {
            angle = -angle;
        }
        if sign1 == -1 {
            angle += PI;
        }
        out.end_of_arrow = elclib::value(angle, &out.fillet_circ);
    } else {
        nearest_side(input, dir1, dir2, out)?;
    }
    if dir1.crossed(dir2)?.is_opposite(input.normal_dir, angular()) {
        out.fillet_circ.set_position(Axis2::with_x_direction(
            center,
            input.normal_dir.reversed(),
            dir1,
        )?);
    }
    out.first_par_circ = elclib::parameter(&out.fillet_circ, input.first_point)?;
    out.last_par_circ = elclib::parameter(&out.fillet_circ, input.second_point)?;
    Ok(())
}

// The arrow on the fillet end whose side line is nearer the position.
fn nearest_side(
    input: &DsgPrsFilletInputs,
    dir1: Direction3,
    dir2: Direction3,
    out: &mut DsgPrsFillet,
) -> Result<(), Failure> {
    let l1 = Line3::new(input.center, dir1);
    let l2 = Line3::new(input.center, dir2);
    if l1.distance(input.position) < l2.distance(input.position) {
        out.end_of_arrow = input.first_point;
        out.draw_position = project_on_line(&l1, &input.position)?;
    } else {
        out.end_of_arrow = input.second_point;
        out.draw_position = project_on_line(&l2, &input.position)?;
    }
    Ok(())
}

impl DsgPrs {
    /// `ComputeRadiusLine`: the radius line ends, from the center or from the position.
    /// Example: `DsgPrs::compute_radius_line(&center, &end, &position, true)?`.
    pub fn compute_radius_line(
        center: &Point3,
        end_of_arrow: &Point3,
        position: &Point3,
        draw_from_center: bool,
    ) -> Result<(Point3, Point3), Failure> {
        if !draw_from_center {
            return Ok((*position, *end_of_arrow));
        }
        let radius_line = make_lin(center, end_of_arrow)?;
        let pos_par = elclib::parameter(&radius_line, *position)?;
        let end_par = elclib::parameter(&radius_line, *end_of_arrow)?;
        Ok(if pos_par < 0.0 {
            (*position, *end_of_arrow)
        } else if pos_par > end_par {
            (*position, *center)
        } else {
            (*center, *end_of_arrow)
        })
    }

    /// `DistanceFromApex`: the parameter distance from an ellipse apex (on an axis) to `par`.
    /// Example: `DsgPrs::distance_from_apex(&ellipse, &apex, 1.0)?`.
    pub fn distance_from_apex(ellipse: &Ellipse3, apex: &Point3, par: f64) -> Result<f64, Failure> {
        let par_apex = elclib::parameter(ellipse, *apex)?;
        if par_apex == 0.0 {
            return Ok(if par < PI { par } else { 2.0 * PI - par });
        }
        if par_apex == PI {
            return Ok(if par < PI { PI - par } else { par - PI });
        }
        if par_apex == PI / 2.0 {
            return Ok(if par <= par_apex + PI && par > par_apex {
                par - par_apex
            } else if par > par_apex + PI {
                2.0 * PI - par + par_apex
            } else {
                par_apex - par
            });
        }
        Ok(if par <= par_apex && par >= PI / 2.0 {
            par_apex - par
        } else if par > par_apex {
            par - par_apex
        } else {
            par + PI / 2.0
        })
    }
}
