//! gp_Circ/Elips/Hypr/Parab member requests (gpconic3).
use super::{Fields, attempt, ax1, ax2, direction, end, flag, frame3, pnt, real, reals};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{
    Axis1, Axis2, Circle3, Ellipse3, Hyperbola3, Parabola3, Point3, Transform3, Vector3,
};

pub struct Motion3 {
    pub pivot: Point3,
    pub axis: Axis1,
    pub plane: Axis2,
    pub angle: f64,
    pub scale: f64,
    pub shift: Vector3,
    pub transform: Transform3,
}
/// Reads pivot, axis direction, angle, scale and shift, building the native transforms.
pub fn motion3(fields: &mut Fields) -> (Point3, rustycad_tkmath::gp::Xyz, f64, f64, Vector3) {
    let pivot = fields.point();
    let axis = fields.xyz();
    let (angle, scale) = (fields.real(), fields.real());
    (pivot, axis, angle, scale, Vector3::from(fields.xyz()))
}
pub fn build_motion3(
    (pivot, axis, angle, scale, shift): (Point3, rustycad_tkmath::gp::Xyz, f64, f64, Vector3),
) -> Result<Motion3, Failure> {
    let axis = Axis1::new(pivot, direction(axis)?);
    let plane = Axis2::new(pivot, axis.direction())?;
    let rotation = Transform3::rotation(axis, angle)?;
    let transform = rotation.multiplied(Transform3::scaling(pivot, scale)?);
    Ok(Motion3 {
        pivot,
        axis,
        plane,
        angle,
        scale,
        shift,
        transform,
    })
}

/// The shared positional block of the native `el_placed3` template.
macro_rules! placed3 {
    ($c:expr, $k:expr, $p:expr) => {{
        let (c, k, p) = ($c, $k, $p);
        ax1(c.axis());
        ax1(c.x_axis());
        ax1(c.y_axis());
        ax2(c.mirrored_point(k.pivot).position());
        attempt(|| {
            ax2(c.mirrored_axis(k.axis)?.position());
            Ok(())
        });
        attempt(|| {
            ax2(c.mirrored_plane(k.plane)?.position());
            Ok(())
        });
        attempt(|| {
            ax2(c.rotated(k.axis, k.angle)?.position());
            Ok(())
        });
        ax2(c.translated(k.shift).position());
        ax2(c.translated_between(k.pivot, p).position());
        ax2(c.scaled(k.pivot, k.scale).position());
        attempt(|| {
            ax2(c.transformed(k.transform)?.position());
            Ok(())
        });
        let mut moved = c;
        attempt(|| {
            moved.set_axis(k.axis)?;
            ax2(moved.position());
            Ok(())
        });
    }};
}

fn circle3(a: Axis2, r: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let c = Circle3::new(a, r)?;
    reals(&[c.area(), c.length(), c.distance(p), c.square_distance(p)]);
    flag(c.contains(p, 1e-7));
    placed3!(c, k, p);
    real(c.scaled(k.pivot, k.scale).radius());
    attempt(|| {
        real(c.transformed(k.transform)?.radius());
        Ok(())
    });
    Ok(())
}
fn ellipse3(a: Axis2, big: f64, small: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let e = Ellipse3::new(a, big, small)?;
    reals(&[e.area(), e.eccentricity(), e.focal()]);
    pnt(e.focus1());
    pnt(e.focus2());
    real(e.parameter());
    attempt(|| {
        ax1(e.directrix1()?);
        Ok(())
    });
    attempt(|| {
        ax1(e.directrix2()?);
        Ok(())
    });
    placed3!(e, k, p);
    let s = e.scaled(k.pivot, k.scale);
    real(s.major_radius());
    real(s.minor_radius());
    attempt(|| {
        let t = e.transformed(k.transform)?;
        real(t.major_radius());
        real(t.minor_radius());
        Ok(())
    });
    Ok(())
}
fn hyperbola3(a: Axis2, big: f64, small: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let h = Hyperbola3::new(a, big, small)?;
    real(h.focal());
    pnt(h.focus1());
    pnt(h.focus2());
    attempt(|| {
        real(h.eccentricity()?);
        Ok(())
    });
    attempt(|| {
        real(h.parameter()?);
        Ok(())
    });
    for axis in [
        h.asymptote1(),
        h.asymptote2(),
        h.directrix1(),
        h.directrix2(),
    ] {
        attempt(|| {
            ax1(axis?);
            Ok(())
        });
    }
    attempt(|| {
        let b = h.conjugate_branch1()?;
        ax2(b.position());
        real(b.major_radius());
        Ok(())
    });
    attempt(|| {
        ax2(h.conjugate_branch2()?.position());
        Ok(())
    });
    attempt(|| {
        ax2(h.other_branch()?.position());
        Ok(())
    });
    let mut located = h;
    attempt(|| {
        located.set_location(p)?;
        ax2(located.position());
        Ok(())
    });
    placed3!(h, k, p);
    Ok(())
}
fn parabola3(a: Axis2, focal: f64, k: &Motion3, p: Point3) -> Result<(), Failure> {
    let c = Parabola3::new(a, focal)?;
    real(c.parameter());
    pnt(c.focus());
    ax1(c.directrix());
    placed3!(c, k, p);
    real(c.scaled(k.pivot, k.scale).focal());
    attempt(|| {
        let d = Parabola3::from_directrix(k.axis, p)?;
        ax2(d.position());
        real(d.focal());
        Ok(())
    });
    Ok(())
}
pub fn gpconic3_case(values: &[f64]) {
    let mut fields = Fields::new(values);
    let (origin, normal, x) = frame3(&mut fields);
    let (big, small, focal, r) = (fields.real(), fields.real(), fields.real(), fields.real());
    let motion = motion3(&mut fields);
    let p = fields.point();
    attempt(|| {
        let a = Axis2::with_x_direction(origin, direction(normal)?, direction(x)?)?;
        let k = build_motion3(motion)?;
        attempt(|| circle3(a, r, &k, p));
        attempt(|| ellipse3(a, big, small, &k, p));
        attempt(|| hyperbola3(a, big, small, &k, p));
        attempt(|| parabola3(a, focal, &k, p));
        attempt(|| Hyperbola3::default().conjugate_branch1().map(|_| ()));
        Ok(())
    });
    end();
}
