//! HLRAlgo value classes, the Rust twin of tools/oracle/tkhlr_probe_hlralgo.cxx: the
//! package box functions, BiPoint, the edge and wire blocks, EdgeStatus with
//! EdgeIterator and the interference records.
use super::common::{Reader, attempt, end, flag, int, real, xy, xyz};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, State};
use rustycad_tkhlr::hlr_algo::{
    self as ha, BiPoint, BiPointFace, BiPointPoints, BiPointRegularity, Coincidence, EdgeIterator,
    EdgeStatus, EdgesBlock, Interference, Intersection, MinMaxIndices, WiresBlock,
};

fn read_minmax(r: &mut Reader) -> MinMaxIndices {
    let mut v = MinMaxIndices::default();
    v.min.iter_mut().for_each(|x| *x = r.integer());
    v.max.iter_mut().for_each(|x| *x = r.integer());
    v
}

fn print_minmax(v: &MinMaxIndices) {
    v.min.iter().chain(v.max.iter()).for_each(|&x| int(x));
}

fn print_parts((s, ts, e, te): (f64, f32, f64, f32)) {
    real(s);
    real(f64::from(ts));
    real(e);
    real(f64::from(te));
}

fn orientation(value: i32) -> Orientation {
    Orientation::ALL[value as usize]
}

fn state(value: i32) -> State {
    State::ALL[value as usize]
}

/// `big tol n (x y z)*n`: see `hl_ha_minmax_case`.
pub fn minmax_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (big, tol, n) = (r.real(), r.real(), r.integer());
    let (mut mn, mut mx) = ([0.0; 16], [0.0; 16]);
    ha::init_min_max(big, &mut mn, &mut mx);
    for _ in 0..n {
        let (x, y, z) = (r.real(), r.real(), r.real());
        ha::update_min_max(x, y, z, &mut mn, &mut mx);
    }
    let print = |mn: &[f64; 16], mx: &[f64; 16]| {
        (0..16).for_each(|i| {
            real(mn[i]);
            real(mx[i]);
        })
    };
    print(&mn, &mx);
    ha::enlarge_min_max(tol, &mut mn, &mut mx);
    print(&mn, &mx);
    end();
}

/// `min max other`: see `hl_ha_encode_case`.
pub fn encode_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let (mn, mx, other) = (
        read_minmax(&mut r),
        read_minmax(&mut r),
        read_minmax(&mut r),
    );
    let (mut code, mut d1, mut d2) = Default::default();
    ha::encode_min_max(&mn, &mx, &mut code);
    print_minmax(&code);
    ha::decode_min_max(&code, &mut d1, &mut d2);
    print_minmax(&d1);
    print_minmax(&d2);
    real(ha::size_box(&mn, &mx));
    let (mut a, mut b) = (mn, mn);
    print_minmax(a.minimize(&other));
    print_minmax(b.maximize(&other));
    let (mut o1, mut o2) = (mn, mx);
    ha::add_min_max(&other, &other, &mut o1, &mut o2);
    print_minmax(&o1);
    print_minmax(&o2);
    ha::copy_min_max(&other, &mn, &mut o1, &mut o2);
    print_minmax(&o1);
    print_minmax(&o2);
    end();
}

fn print_bipoint(bp: &BiPoint) {
    let i = bp.indices();
    let fields = [
        i.shape_index,
        i.face_conex1,
        i.face1_pt1,
        i.face1_pt2,
        i.face_conex2,
        i.face2_pt1,
        i.face2_pt2,
        i.min_seg,
        i.max_seg,
        i.seg_flags,
    ];
    fields.into_iter().for_each(int);
    let p = bp.points();
    [p.pnt1, p.pnt2, p.pnt_p1, p.pnt_p2]
        .into_iter()
        .for_each(xyz);
    xy(p.pnt_p1_2d());
    xy(p.pnt_p2_2d());
    [
        bp.rg1_line(),
        bp.rgn_line(),
        bp.out_line(),
        bp.int_line(),
        bp.hidden(),
    ]
    .into_iter()
    .for_each(flag);
}

fn build_bipoint(r: &mut Reader, variant: i32) -> BiPoint {
    let mut c = [0.0; 12];
    c.iter_mut().for_each(|x| *x = r.real());
    let points = BiPointPoints::from_coordinates(c);
    let index = r.integer();
    let face1 = BiPointFace {
        conex: r.integer(),
        pt1: r.integer(),
        pt2: r.integer(),
    };
    let face2 = BiPointFace {
        conex: r.integer(),
        pt1: r.integer(),
        pt2: r.integer(),
    };
    let bits = r.integer();
    let regularity = BiPointRegularity {
        reg1: bits & 1 != 0,
        regn: bits & 2 != 0,
        outl: bits & 4 != 0,
        intl: bits & 8 != 0,
    };
    match variant {
        0 => BiPoint::with_regularity(points, index, regularity),
        1 => BiPoint::with_flags(points, index, bits),
        2 => BiPoint::with_face_regularity(points, index, face1, regularity),
        3 => BiPoint::with_face_flags(points, index, face1, bits),
        4 => BiPoint::with_faces_regularity(points, index, [face1, face2], regularity),
        5 => BiPoint::with_faces_flags(points, index, [face1, face2], bits),
        _ => BiPoint::default(),
    }
}

/// `variant c[12] index faces bits n (which value)*n`: see `hl_ha_bipoint_case`.
pub fn bipoint_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let variant = r.integer();
    let mut bp = build_bipoint(&mut r, variant);
    for _ in 0..r.integer() {
        let (which, v) = (r.integer(), r.flag());
        match which {
            0 => bp.set_rg1_line(v),
            1 => bp.set_rgn_line(v),
            2 => bp.set_out_line(v),
            3 => bp.set_int_line(v),
            _ => bp.set_hidden(v),
        }
    }
    print_bipoint(&bp);
    end();
}

fn read_edge_block(eb: &EdgesBlock, i: i32) {
    attempt(|| {
        int(eb.edge(i)?);
        Ok(())
    });
    attempt(|| {
        int(eb.orientation(i)? as i32);
        Ok(())
    });
    attempt(|| {
        flag(eb.out_line(i)?);
        Ok(())
    });
    attempt(|| {
        flag(eb.internal(i)?);
        Ok(())
    });
    attempt(|| {
        flag(eb.double(i)?);
        Ok(())
    });
    attempt(|| {
        flag(eb.iso_line(i)?);
        Ok(())
    });
}

fn edit_edge_block(eb: &EdgesBlock, op: i32, i: i32, v: i32) -> Result<(), Failure> {
    match op {
        0 => eb.set_edge(i, v),
        1 => eb.set_orientation(i, orientation(v)),
        2 => eb.set_out_line(i, v != 0),
        3 => eb.set_internal(i, v != 0),
        4 => eb.set_double(i, v != 0),
        _ => eb.set_iso_line(i, v != 0),
    }
}

/// `nbEdges n ops`: see `hl_ha_edges_case`.
pub fn edges_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let nb = r.integer();
    let Ok(eb) = EdgesBlock::new(nb) else {
        return end();
    };
    int(eb.nb_edges());
    for _ in 0..r.integer() {
        let op = r.integer();
        if op == 7 {
            eb.update_min_max(&read_minmax(&mut r));
            print_minmax(&eb.min_max());
            continue;
        }
        let i = r.integer();
        if op == 6 {
            read_edge_block(&eb, i);
            continue;
        }
        let v = r.integer();
        attempt(|| edit_edge_block(&eb, op, i, v));
    }
    end();
}

/// `nbWires n ops`: see `hl_ha_wires_case`.
pub fn wires_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let Ok(wb) = WiresBlock::new(r.integer()) else {
        return end();
    };
    int(wb.nb_wires());
    for _ in 0..r.integer() {
        let op = r.integer();
        if op == 2 {
            wb.update_min_max(&read_minmax(&mut r));
            print_minmax(&wb.min_max());
            continue;
        }
        let i = r.integer();
        if op == 0 {
            let v = r.integer();
            attempt(|| wb.set(i, Handle::new(EdgesBlock::new(v)?)));
        } else {
            attempt(|| {
                let wire = wb.wire(i)?;
                int(wire.get().map_or(-1, EdgesBlock::nb_edges));
                Ok(())
            });
        }
    }
    end();
}

fn print_status(st: &EdgeStatus) -> Result<(), Failure> {
    print_parts(st.bounds());
    flag(st.all_hidden());
    flag(st.all_visible());
    let nb = st.nb_visible_part();
    int(nb);
    for i in 1..=nb {
        print_parts(st.visible_part(i)?);
    }
    let mut it = EdgeIterator::new();
    it.init_visible(st);
    while it.more_visible() {
        print_parts(it.visible()?);
        it.next_visible();
    }
    if nb == 0 && !st.all_hidden() {
        int(-1);
        return Ok(());
    }
    it.init_hidden(st)?;
    while it.more_hidden() {
        print_parts(it.hidden());
        it.next_hidden()?;
    }
    int(-2);
    Ok(())
}

fn read_bounds(r: &mut Reader) -> (f64, f32, f64, f32) {
    let (s, ts) = (r.real(), r.real() as f32);
    let e = r.real();
    (s, ts, e, r.real() as f32)
}

fn edit_status(r: &mut Reader, st: &mut EdgeStatus, op: i32) {
    match op {
        0 | 1 => {
            let (s, ts, e, te) = read_bounds(r);
            if op == 0 {
                st.initialize(s, ts, e, te);
            } else {
                let on_face = r.flag();
                st.hide(s, ts, e, te, on_face, r.flag());
            }
        }
        2 => st.hide_all(),
        3 => st.show_all(),
        4 => st.set_all_hidden(r.flag()),
        _ => st.set_all_visible(r.flag()),
    }
}

/// `mode [s ts e te] n ops`: see `hl_ha_status_case`.
pub fn status_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let mut st = EdgeStatus::new();
    if r.integer() == 1 {
        let (s, ts, e, te) = read_bounds(&mut r);
        st = EdgeStatus::with_bounds(s, ts, e, te);
    }
    attempt(|| print_status(&st));
    for _ in 0..r.integer() {
        let op = r.integer();
        edit_status(&mut r, &mut st, op);
        attempt(|| print_status(&st));
    }
    end();
}

fn print_intersection(x: &Intersection, with_enums: bool) {
    if with_enums {
        int(x.orientation() as i32);
        int(x.state() as i32);
    }
    int(x.level());
    int(x.seg_index());
    int(x.index());
    real(x.parameter());
    real(f64::from(x.tolerance()));
}

fn print_coincidence(c: &Coincidence) {
    let (fe, p) = c.value_2d();
    int(fe);
    real(p);
    let (a, b) = c.state_3d();
    int(a as i32);
    int(b as i32);
}

fn print_interference_enums(f: &Interference) {
    int(f.orientation() as i32);
    int(f.transition() as i32);
    int(f.boundary_transition() as i32);
}

/// `o lev seg ind p tol s fe param sb sa or tr btr x`: see `hl_ha_records_case`.
pub fn records_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let o = orientation(r.integer());
    let (lev, seg, ind, p, tol) = (
        r.integer(),
        r.integer(),
        r.integer(),
        r.real(),
        r.real() as f32,
    );
    let s = state(r.integer());
    let (fe, param, sb, sa) = (
        r.integer(),
        r.real(),
        state(r.integer()),
        state(r.integer()),
    );
    let (orr, tr, btr) = (
        orientation(r.integer()),
        orientation(r.integer()),
        orientation(r.integer()),
    );
    let x = r.real();
    let (d, dc) = (Intersection::new(), Coincidence::new());
    print_intersection(&d, false);
    print_coincidence(&dc);
    let v = Intersection::with_values(o, lev, seg, ind, p, tol, s);
    print_intersection(&v, true);
    let mut w = Intersection::new();
    w.set_orientation(orr);
    w.set_level(ind);
    w.set_seg_index(lev);
    w.set_index(seg);
    w.set_parameter(x);
    w.set_tolerance(tol * 2.0);
    w.set_state(sb);
    print_intersection(&w, true);
    let mut c = Coincidence::new();
    c.set_2d(fe, param);
    c.set_state_3d(sb, sa);
    print_coincidence(&c);
    let mut f = Interference::with_values(v, c, orr, tr, btr);
    print_intersection(f.intersection(), true);
    print_coincidence(f.boundary());
    print_interference_enums(&f);
    f.change_intersection().set_parameter(x);
    f.change_boundary().set_2d(fe + 1, x);
    f.set_orientation(tr);
    f.set_transition(btr);
    f.set_boundary_transition(orr);
    f.set_intersection(w);
    print_intersection(f.intersection(), true);
    print_coincidence(f.boundary());
    f.set_boundary(dc);
    print_coincidence(f.boundary());
    print_interference_enums(&f);
    end();
}
