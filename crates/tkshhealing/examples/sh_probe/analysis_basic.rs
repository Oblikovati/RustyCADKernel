//! ShapeAnalysis_Geom, _ShapeTolerance, _Shell, _WireOrder and _FreeBoundData probe
//! operations, the Rust twin of tools/oracle/tkshhealing_probe_analysis_basic.cxx (which
//! documents the requests).
use super::common::{Reader, attempt, dir, end, flag, int, pnt, real, show, xy, xyz};
use super::shapes::{TpShapes, tp_same_index, tp_shape_index, tp_shapes};
use super::wire_data::{sh_shape, sh_wire};
use rustycad_tkbrep::topods::{Shape, ShapeIterator};
use rustycad_tkernel::ncollection::{Array1, HArray2, HSequence};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Xy;
use rustycad_tkshhealing::shape_analysis::{
    ShapeAnalysisFreeBoundData, ShapeAnalysisGeom, ShapeAnalysisShapeTolerance, ShapeAnalysisShell,
    ShapeAnalysisWireOrder,
};

/// A `TopAbs_ShapeEnum` read as its native value.
pub fn shape_enum(value: i32) -> ShapeEnum {
    ShapeEnum::ALL[value as usize]
}

/// `sh_geom`.
pub fn geom_case(values: &[f64]) {
    let mut r = Reader::new(values);
    if r.integer() == 0 {
        let n = r.integer();
        let mut points = Array1::with_bounds(1, n).expect("probe points");
        for i in 1..=n {
            points[i] = r.pnt();
        }
        attempt(|| {
            let found = ShapeAnalysisGeom::nearest_plane(&points)?;
            flag(found.is_some());
            if let Some((plane, dmax)) = found {
                pnt(plane.location());
                dir(plane.axis().direction());
                dir(plane.x_axis().direction());
                real(dmax);
            }
            Ok(())
        });
        end();
        return;
    }
    position_trsf(&mut r);
    end();
}

/// `sh_geom 1`: PositionTrsf.
fn position_trsf(r: &mut Reader<'_>) {
    let null = r.flag();
    let (rl, ru, cl, cu) = (r.integer(), r.integer(), r.integer(), r.integer());
    let matrix = (!null).then(|| HArray2::with_bounds(rl, ru, cl, cu).expect("probe matrix"));
    for i in rl..=ru {
        for j in cl..=cu {
            let v = r.real();
            if let Some(m) = &matrix {
                *m.change_array2().change_value(i, j).expect("probe cell") = v;
            }
        }
    }
    let coefs = matrix.map_or_else(Handle::default, Handle::new);
    let (unit, prec) = (r.real(), r.real());
    attempt(|| {
        let (ok, trsf) = ShapeAnalysisGeom::position_trsf(&coefs, unit, prec)?;
        flag(ok);
        int(trsf.form() as i32);
        real(trsf.scale_factor());
        for i in 1..=3 {
            for j in 1..=4 {
                real(trsf.value(i, j)?);
            }
        }
        Ok(())
    });
}

/// `sh_item`: the IsSame index and the orientation of a shape.
pub fn shape_item(t: &TpShapes, s: &Shape) {
    int(tp_same_index(t, s));
    int(if s.is_null() {
        -1
    } else {
        s.orientation() as i32
    });
}

/// `sh_list`: the length of a list and each shape (`sh_item`).
pub fn shape_list(t: &TpShapes, list: &Handle<HSequence<Shape>>) {
    let items = list.sequence(|s| s.iter().cloned().collect::<Vec<Shape>>());
    int(items.len() as i32);
    for s in &items {
        shape_item(t, s);
    }
}

/// `sh_children`: the count and the items (`sh_item`) of the children of a compound.
fn children(t: &TpShapes, compound: &Shape) -> Result<(), Failure> {
    int(compound.nb_children());
    let mut it = ShapeIterator::new(compound, true, true);
    while it.more() {
        shape_item(t, it.value()?);
        it.next();
    }
    Ok(())
}

/// `sh_st_op`.
fn shapetol_op(r: &mut Reader<'_>, t: &TpShapes, st: &mut ShapeAnalysisShapeTolerance, op: i32) {
    match op {
        0 => {
            let (s, mode, ty) = (
                sh_shape(t, r.integer()),
                r.integer(),
                shape_enum(r.integer()),
            );
            show(st.tolerance(&s, mode, ty), real);
        }
        1 => {
            let (s, v, ty) = (sh_shape(t, r.integer()), r.real(), shape_enum(r.integer()));
            show(st.over_tolerance(&s, v, ty), |l| shape_list(t, &l));
        }
        2 => {
            let (s, a, b) = (sh_shape(t, r.integer()), r.real(), r.real());
            let ty = shape_enum(r.integer());
            show(st.in_tolerance(&s, a, b, ty), |l| shape_list(t, &l));
        }
        3 => st.init_tolerance(),
        4 => {
            let (s, ty) = (sh_shape(t, r.integer()), shape_enum(r.integer()));
            attempt(|| st.add_tolerance(&s, ty));
        }
        _ => {
            for mode in [-1, 0, 1] {
                real(st.global_tolerance(mode));
            }
        }
    }
}

/// `sh_shapetol program nops (op)...`.
pub fn shapetol_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let mut st = ShapeAnalysisShapeTolerance::new();
    for _ in 0..r.integer() {
        let op = r.integer();
        shapetol_op(&mut r, &t, &mut st, op);
    }
    end();
}

/// `sh_shell_op`.
fn shell_op(r: &mut Reader<'_>, t: &TpShapes, sh: &mut ShapeAnalysisShell, op: i32) {
    match op {
        0 => {
            let s = sh_shape(t, r.integer());
            attempt(|| sh.load_shells(&s));
        }
        1 => {
            let (s, f, internal) = (sh_shape(t, r.integer()), r.flag(), r.flag());
            show(sh.check_oriented_shells(&s, f, internal), flag);
        }
        2 => sh.clear(),
        3 => flag(sh.is_loaded(&sh_shape(t, r.integer()))),
        _ => show(sh.loaded(r.integer()), |s| int(tp_shape_index(t, &s))),
    }
}

/// `sh_shell program nops (op)...`.
pub fn shell_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let mut sh = ShapeAnalysisShell::new();
    for _ in 0..r.integer() {
        let op = r.integer();
        shell_op(&mut r, &t, &mut sh, op);
    }
    let digest = || -> Result<(), Failure> {
        let n = sh.nb_loaded();
        int(n);
        for i in 1..=n {
            int(tp_shape_index(&t, &sh.loaded(i)?));
        }
        flag(sh.has_bad_edges());
        children(&t, &sh.bad_edges()?.into_shape())?;
        flag(sh.has_free_edges());
        children(&t, &sh.free_edges()?.into_shape())?;
        flag(sh.has_connected_edges());
        Ok(())
    };
    digest().expect("probe shell digest");
    end();
}

fn read_xy(r: &mut Reader<'_>) -> Xy {
    r.pnt2d().xy()
}

/// `sh_wo_op`.
fn wireorder_op(r: &mut Reader<'_>, wo: &mut ShapeAnalysisWireOrder, op: i32) {
    match op {
        0 => {
            let a = r.xyz();
            wo.add_3d(a, r.xyz());
        }
        1 => {
            let a = read_xy(r);
            wo.add_2d(a, read_xy(r));
        }
        2 => {
            let (a, b) = (r.xyz(), r.xyz());
            let c = read_xy(r);
            wo.add_both(a, b, c, read_xy(r));
        }
        3 => {
            let closed = r.flag();
            attempt(|| wo.perform(closed));
        }
        4 => {
            let (m, tol) = (r.flag(), r.real());
            wo.set_mode(m, tol, r.flag());
        }
        5 => wo.clear(),
        6 => {
            let gap = r.real();
            attempt(|| wo.set_chains(gap));
        }
        7 => wo.set_couples(r.real()),
        _ => wo.set_keep_loops_mode(r.flag()),
    }
}

/// `sh_wo_digest`.
fn wireorder_digest(wo: &ShapeAnalysisWireOrder) {
    real(wo.tolerance());
    let n = wo.nb_edges();
    int(n);
    flag(wo.is_done());
    int(wo.status());
    for i in 0..=n + 1 {
        show(wo.ordered(i), int);
    }
    for i in 1..=n {
        attempt(|| {
            let o = wo.ordered(i)?;
            let (a, b) = wo.xyz(o)?;
            xyz(a);
            xyz(b);
            let (c, d) = wo.xy(o)?;
            xy(c);
            xy(d);
            Ok(())
        });
    }
    attempt(|| wo.xyz(0).map(|_| ()));
    for i in 0..=n {
        show(wo.gap(i), real);
    }
    let nc = wo.nb_chains();
    int(nc);
    for i in -1..=nc + 1 {
        show(wo.chain(i), |(n1, n2)| {
            int(n1);
            int(n2);
        });
    }
    int(wo.nb_couples());
    let (n1, n2) = wo.couple(1);
    int(n1);
    int(n2);
}

/// `sh_wireorder mode3d tol modeboth nops (op)...`.
pub fn wireorder_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (m3, tol, both) = (r.flag(), r.real(), r.flag());
    let mut wo = ShapeAnalysisWireOrder::with_mode(m3, tol, both);
    for _ in 0..r.integer() {
        let op = r.integer();
        wireorder_op(&mut r, &mut wo, op);
    }
    flag(wo.keep_loops_mode());
    wireorder_digest(&wo);
    end();
}

/// The ops of `sh_fbdata`.
fn fbdata_op(r: &mut Reader<'_>, t: &TpShapes, d: &ShapeAnalysisFreeBoundData, op: i32) {
    match op {
        0..=3 => {
            let v = r.real();
            match op {
                0 => d.set_area(v),
                1 => d.set_perimeter(v),
                2 => d.set_ratio(v),
                _ => d.set_width(v),
            }
        }
        4 => {
            let n = sh_wire(t, r.integer()).expect("probe notch");
            d.add_notch(&n, r.real());
        }
        5 => {
            let kept = d.notches();
            d.clear();
            int(kept.sequence(|s| s.length()));
        }
        _ => d.set_free_bound(&sh_wire(t, r.integer()).expect("probe bound")),
    }
}

/// `sh_fbdata program w nops (op)... probe`.
pub fn fbdata_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let t = tp_shapes(&mut r).expect("probe shapes");
    let w = r.integer();
    let d = if w == -2 {
        ShapeAnalysisFreeBoundData::new()
    } else {
        ShapeAnalysisFreeBoundData::with_bound(&sh_wire(&t, w).expect("probe bound"))
    };
    for _ in 0..r.integer() {
        let op = r.integer();
        fbdata_op(&mut r, &t, &d, op);
    }
    let probe = sh_wire(&t, r.integer()).expect("probe wire");
    int(tp_shape_index(&t, &d.free_bound()));
    for v in [d.area(), d.perimeter(), d.ratio(), d.width()] {
        real(v);
    }
    let n = d.nb_notches();
    int(n);
    shape_list(&t, &d.notches());
    for i in 0..=n + 1 {
        attempt(|| {
            int(tp_shape_index(&t, &d.notch(i)?.into_shape()));
            real(d.notch_width(i)?);
            Ok(())
        });
    }
    show(d.notch_width_of(&probe), real);
    end();
}
