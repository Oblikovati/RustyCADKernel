use rustycad_tkbo::int_tools::{Context, WLineTool};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkgeomalgo::{
    geom_int::GeomIntLineConstructor,
    int_patch::{IntPatchPointLine, IntPatchWLType, IntPatchWLine},
    int_surf::{IntSurfLineOn2S, IntSurfPntOn2S},
};
fn parameters(pattern: i32) -> Vec<f64> {
    let tau = std::f64::consts::TAU;
    match pattern {
        0 => vec![],
        1 => vec![0., 0., 0.],
        2 => vec![0.1, 0.2, 0.3],
        3 => vec![0., 0.1, 0.2, tau],
        4 => vec![0.1, 0.2, 0., tau - 0.2, tau - 0.1],
        5 => vec![0., 0.1, tau],
        6 => vec![tau, tau - 0.1, tau - 0.2, 0.],
        7 => vec![0., 1e-10, 0.1, 0.2, tau - 1e-10, tau],
        8 => vec![0.1, 0.1, 0., 0.2, 0.2],
        9 => vec![0., 0.1, 0.2, 0.3],
        10 => vec![0.1, 0.2, 0.3, 0.],
        11 => vec![-0.1, -0.2, 0., 0.1, 0.2],
        12 => vec![1e-6, 0.1, 0.2, tau + 1e-6],
        13 => vec![-1e-6, 0.1, 0.2, -1e-6],
        _ => vec![0.1, 0.2, 1e-6, 0.3, 0.4],
    }
}
pub fn walking(
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    v: &[f64],
) -> Result<Handle<IntPatchWLine>, Failure> {
    let points = Handle::new(IntSurfLineOn2S::new(None));
    for parameter in parameters(v[2] as i32) {
        let point = walking_point(surface, v, parameter)?;
        points.add(&point);
    }
    let line = Handle::new(IntPatchWLine::new_undecided(&points, false));
    line.set_creating_way_info(IntPatchWLType::ImpImp);
    Ok(line)
}
pub fn constructor(
    line: &Handle<IntPatchWLine>,
    a: &Handle<GeomAdaptorSurface>,
    b: &Handle<GeomAdaptorSurface>,
    mode: i32,
) -> Result<GeomIntLineConstructor, Failure> {
    let mut constructor = GeomIntLineConstructor::new();
    if mode < 2 {
        return Ok(constructor);
    }
    add_vertices(line, mode)?;
    let d1 = rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool::with_surface(&a.clone().into_cast())?;
    let d2 = rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool::with_surface(&b.clone().into_cast())?;
    constructor.load(std::rc::Rc::new(d1), std::rc::Rc::new(d2), a, b);
    constructor.perform(&line.clone().into_cast())?;
    Ok(constructor)
}
fn add_vertices(line: &Handle<IntPatchWLine>, mode: i32) -> Result<(), Failure> {
    let count = line.nb_pnts()?;
    let indices = match mode {
        2 => vec![1, count],
        3 => (1..=count).collect(),
        _ => vec![2, count - 1],
    };
    for index in indices {
        if index < 1 || index > count {
            continue;
        }
        let mut point = rustycad_tkgeomalgo::int_patch::IntPatchPoint::new();
        point.set_pnt_on_2s(&line.point(index)?);
        point.set_parameter(f64::from(index));
        line.add_vertex(&point, false);
    }
    Ok(())
}

pub fn decompose(
    line: &Handle<IntPatchWLine>,
    surfaces: [&Handle<GeomAdaptorSurface>; 2],
    faces: [&rustycad_tkbrep::topods::Face; 2],
    constructor: &GeomIntLineConstructor,
    avoid: bool,
    output: &mut Vec<Handle<dyn rustycad_tkgeomalgo::int_patch::IntPatchLine>>,
    context: &Handle<Context>,
) -> Result<bool, Failure> {
    WLineTool::decomposition_of_wline(
        line,
        surfaces[0],
        surfaces[1],
        faces[0],
        faces[1],
        constructor,
        avoid,
        1e-6,
        output,
        context,
    )
}

fn walking_point(
    surface: &Handle<dyn rustycad_tkg3d::geom::GeomSurface>,
    v: &[f64],
    parameter: f64,
) -> Result<IntSurfPntOn2S, Failure> {
    let vertical = v.get(6).copied().unwrap_or(0.) != 0.;
    let (u, w) = if vertical {
        (v[3], parameter)
    } else {
        (parameter, v[3])
    };
    let p = surface.eval_d0(u, w)?;
    let mut point = IntSurfPntOn2S::new();
    let (x, y) = second_parameters(p, (u, w), v, vertical);
    point.set_value_all(p, u, w, x, y);
    Ok(point)
}

fn second_parameters(
    point: rustycad_tkmath::gp::Point3,
    (u, v): (f64, f64),
    request: &[f64],
    vertical: bool,
) -> (f64, f64) {
    if request[1] == 0. {
        return (point.x(), point.y());
    }
    if vertical {
        return (u, v + request[4]);
    }
    (u + request[4], v)
}
