//! Intf_Tool probe operations, the Rust twin of tools/oracle/tkgeomalgo_probe_intf_tool.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use rustycad_tkgeomalgo::intf::Tool;
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::gp::{
    Axis2, Axis2d, Hyperbola2, Hyperbola3, Line2, Line3, Parabola2, Parabola3,
};

/// `ga_box2d`: void, whole, or bounds with a gap and open sides.
fn box2d(r: &mut Reader) -> BoundingBox2 {
    let mut bounds = BoundingBox2::default();
    let mode = r.integer();
    if mode == 1 {
        bounds.set_whole();
    }
    if mode != 2 {
        return bounds;
    }
    let (x0, y0, x1, y1) = (r.real(), r.real(), r.real(), r.real());
    bounds.update_bounds([x0, y0], [x1, y1]);
    bounds.set_gap(r.real());
    let open = r.integer();
    let sides: [fn(&mut BoundingBox2); 4] = [
        BoundingBox2::open_xmin,
        BoundingBox2::open_xmax,
        BoundingBox2::open_ymin,
        BoundingBox2::open_ymax,
    ];
    for (bit, side) in sides.iter().enumerate() {
        if open & (1 << bit) != 0 {
            side(&mut bounds);
        }
    }
    bounds
}

/// `ga_box3d`: void, whole, or bounds with a gap and open sides.
fn box3d(r: &mut Reader) -> BoundingBox3 {
    let mut bounds = BoundingBox3::default();
    let mode = r.integer();
    if mode == 1 {
        bounds.set_whole();
    }
    if mode != 2 {
        return bounds;
    }
    let low = [r.real(), r.real(), r.real()];
    let high = [r.real(), r.real(), r.real()];
    bounds.update_bounds(low, high);
    bounds.set_gap(r.real());
    let open = r.integer();
    let sides: [fn(&mut BoundingBox3); 6] = [
        BoundingBox3::open_xmin,
        BoundingBox3::open_xmax,
        BoundingBox3::open_ymin,
        BoundingBox3::open_ymax,
        BoundingBox3::open_zmin,
        BoundingBox3::open_zmax,
    ];
    for (bit, side) in sides.iter().enumerate() {
        if open & (1 << bit) != 0 {
            side(&mut bounds);
        }
    }
    bounds
}

fn box2d_print(bounds: &BoundingBox2) {
    for value in [
        bounds.is_void(),
        bounds.is_whole(),
        bounds.is_open_xmin(),
        bounds.is_open_xmax(),
        bounds.is_open_ymin(),
        bounds.is_open_ymax(),
    ] {
        flag(value);
    }
    real(bounds.gap());
    if let Ok(limits) = bounds.get() {
        limits.into_iter().for_each(real);
    }
}

fn box3d_print(bounds: &BoundingBox3) {
    for value in [
        bounds.is_void(),
        bounds.is_whole(),
        bounds.is_open_xmin(),
        bounds.is_open_xmax(),
        bounds.is_open_ymin(),
        bounds.is_open_ymax(),
        bounds.is_open_zmin(),
        bounds.is_open_zmax(),
    ] {
        flag(value);
    }
    real(bounds.gap());
    if let Ok(limits) = bounds.get() {
        limits.into_iter().for_each(real);
    }
}

fn segments_print(tool: &Tool) {
    int(tool.nb_segments());
    for i in 1..=tool.nb_segments() {
        real(tool.begin_param(i).expect("probe segment"));
        real(tool.end_param(i).expect("probe segment"));
    }
    attempt(|| tool.begin_param(0).map(real));
    attempt(|| tool.end_param(tool.nb_segments() + 1).map(real));
}

/// Calls on one tool: Lin2dBox (0), Hypr2dBox (1), Parab2dBox (2), each with a domain.
pub fn tool2d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut tool = Tool::new();
    for _ in 0..r.integer() {
        let kind = r.integer();
        let (p, d) = (r.pnt2d(), r.dir2d());
        let mut result = BoundingBox2::default();
        if kind == 0 {
            let domain = box2d(&mut r);
            tool.lin2d_box(Line2::new(p, d), &domain, &mut result);
        } else if kind == 1 {
            let (sense, major, minor) = (r.flag(), r.real(), r.real());
            let domain = box2d(&mut r);
            attempt(|| {
                let hyperbola = Hyperbola2::from_x_axis(Axis2d::new(p, d), major, minor, sense)?;
                tool.hypr2d_box(hyperbola, &domain, &mut result)
            });
        } else {
            let (sense, focal) = (r.flag(), r.real());
            let domain = box2d(&mut r);
            attempt(|| {
                let parabola = Parabola2::from_mirror_axis(Axis2d::new(p, d), focal, sense)?;
                tool.parab2d_box(parabola, &domain, &mut result)
            });
        }
        segments_print(&tool);
        box2d_print(&result);
    }
    end();
}

/// Calls on one tool: LinBox (0), HyprBox (1), ParabBox (2), each with a domain.
pub fn tool3d_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut tool = Tool::new();
    for _ in 0..r.integer() {
        let kind = r.integer();
        let p = r.pnt();
        let mut result = BoundingBox3::default();
        if kind == 0 {
            let d = r.dir();
            let domain = box3d(&mut r);
            tool.lin_box(Line3::new(p, d), &domain, &mut result);
        } else if kind == 1 {
            let (n, x, major, minor) = (r.dir(), r.dir(), r.real(), r.real());
            let domain = box3d(&mut r);
            attempt(|| {
                let hyperbola = Hyperbola3::new(Axis2::with_x_direction(p, n, x)?, major, minor)?;
                tool.hypr_box(hyperbola, &domain, &mut result)
            });
        } else {
            let (n, x, focal) = (r.dir(), r.dir(), r.real());
            let domain = box3d(&mut r);
            attempt(|| {
                let parabola = Parabola3::new(Axis2::with_x_direction(p, n, x)?, focal)?;
                tool.parab_box(parabola, &domain, &mut result)
            });
        }
        segments_print(&tool);
        box3d_print(&result);
    }
    end();
}
