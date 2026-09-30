//! HLRAlgo_PolyInternalData, the Rust twin of tools/oracle/tkhlr_probe_hlralgo_internal.cxx:
//! UpdateLinks, AddNode, the split UpdateLinks, the Inc/Dec members and Dump.
use super::common::{Reader, attempt, captured, end, flag, int, real, xy, xyz};
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkhlr::hlr_algo::{
    NodeArray, PolyInternalData, SegmentArray, TriangleArray, TriangleData,
};
use rustycad_tkmath::gp::Xy;

fn read_xy(r: &mut Reader) -> Xy {
    let x = r.real();
    Xy::new(x, r.real())
}

fn print_internal(pid: &PolyInternalData) -> Result<(), Failure> {
    [pid.nb_pi_nod(), pid.nb_pi_seg(), pid.nb_t_data()]
        .into_iter()
        .for_each(int);
    flag(pid.planar());
    flag(pid.int_out_l());
    let (t, s, n) = (pid.t_data(), pid.pi_seg(), pid.pi_nod());
    let missing = || Failure::new("probe: null array");
    let (t, s, n) = (
        t.get().ok_or_else(missing)?,
        s.get().ok_or_else(missing)?,
        n.get().ok_or_else(missing)?,
    );
    let (t, s, n) = (t.array1(), s.array1(), n.array1());
    [t.upper(), s.upper(), n.upper()].into_iter().for_each(int);
    let fail = |e: rustycad_tkernel::ncollection::ArrayError| e.into_failure("probe");
    for i in 1..=pid.nb_pi_seg() {
        let v = s.value(i).map_err(fail)?;
        [
            v.lst_sg1, v.lst_sg2, v.nxt_sg1, v.nxt_sg2, v.conex1, v.conex2,
        ]
        .into_iter()
        .for_each(int);
    }
    for i in 1..=pid.nb_t_data() {
        let v = t.value(i).map_err(fail)?;
        [v.node1, v.node2, v.node3, v.flags]
            .into_iter()
            .for_each(int);
    }
    for i in 1..=pid.nb_pi_nod() {
        let node = n.value(i).map_err(fail)?;
        let node = node.get().ok_or_else(missing)?;
        let ni = *node.indices();
        [ni.nd_sg, ni.flag, ni.edg1, ni.edg2]
            .into_iter()
            .for_each(int);
        let d = *node.data();
        xyz(d.point);
        xyz(d.normal);
        xy(d.uv);
        [d.p_cu1, d.p_cu2, d.scal].into_iter().for_each(real);
    }
    drop((t, s, n));
    captured(|console| pid.dump(console));
    Ok(())
}

fn node_data(
    pid: &PolyInternalData,
    i: i32,
) -> Result<rustycad_tkhlr::hlr_algo::NodeData, Failure> {
    let nodes = pid.pi_nod();
    let nodes = nodes
        .get()
        .ok_or_else(|| Failure::new("probe: null array"))?
        .array1();
    let node = nodes.value(i).map_err(|e| e.into_failure("probe"))?;
    let data = *node
        .get()
        .ok_or_else(|| Failure::new("probe: null node"))?
        .data();
    Ok(data)
}

fn internal_op(r: &mut Reader, pid: &PolyInternalData, op: i32) -> Result<(), Failure> {
    let (mut t1, mut t2): (TriangleArray, TriangleArray) = (pid.t_data(), pid.t_data());
    let (mut s1, mut s2): (SegmentArray, SegmentArray) = (pid.pi_seg(), pid.pi_seg());
    let (mut n1, mut n2): (NodeArray, NodeArray) = (pid.pi_nod(), pid.pi_nod());
    match op {
        0 => {
            let (i, j, c) = (r.integer(), r.integer(), r.real());
            let (x, y, z) = (r.real(), r.real(), r.real());
            let (a, b) = (node_data(pid, i)?, node_data(pid, j)?);
            int(pid.add_node(&a, &b, &mut n1, &mut n2, c, x, y, z)?);
        }
        1 => {
            let (a, b, c) = (r.integer(), r.integer(), r.integer());
            pid.update_links_split(
                a, b, c, &mut t1, &mut t2, &mut s1, &mut s2, &mut n1, &mut n2,
            )?;
        }
        2 => pid.set_planar(r.flag()),
        3 => pid.set_int_out_l(r.flag()),
        _ => {
            pid.dec_pi_seg();
            pid.inc_pi_seg(&mut s1, &mut s2)?;
        }
    }
    Ok(())
}

fn build_internal(r: &mut Reader) -> Result<PolyInternalData, Failure> {
    let (nn, nt) = (r.integer(), r.integer());
    let pid = PolyInternalData::new(nn, nt)?;
    for i in 1..=nn {
        let (point, normal, uv, scal) = (r.xyz(), r.xyz(), read_xy(r), r.real());
        let nodes = pid.pi_nod();
        let nodes = nodes
            .get()
            .ok_or_else(|| Failure::new("probe: null array"))?
            .array1();
        let node = nodes.value(i).map_err(|e| e.into_failure("probe"))?;
        let node = node.get().ok_or_else(|| Failure::new("probe: null node"))?;
        let mut data = node.data();
        (data.point, data.normal, data.uv, data.scal) = (point, normal, uv, scal);
    }
    let triangles = pid.t_data();
    let triangles = triangles
        .get()
        .ok_or_else(|| Failure::new("probe: null array"))?;
    for i in 1..=nt {
        let t = TriangleData {
            node1: r.integer(),
            node2: r.integer(),
            node3: r.integer(),
            flags: r.integer(),
        };
        triangles
            .change_array1()
            .set_value(i, t)
            .map_err(|e| e.into_failure("probe"))?;
    }
    Ok(pid)
}

fn internal_body(r: &mut Reader) -> Result<(), Failure> {
    let pid = build_internal(r)?;
    {
        let (t, s, n) = (pid.t_data(), pid.pi_seg(), pid.pi_nod());
        let missing = || Failure::new("probe: null array");
        let (t, s, n) = (
            t.get().ok_or_else(missing)?,
            s.get().ok_or_else(missing)?,
            n.get().ok_or_else(missing)?,
        );
        pid.update_links(&mut t.change_array1(), &mut s.change_array1(), &n.array1())?;
    }
    print_internal(&pid)?;
    for _ in 0..r.integer() {
        let op = r.integer();
        internal_op(r, &pid, op)?;
        print_internal(&pid)?;
    }
    Ok(())
}

/// `nbNod nbTri nodes triangles n ops`: see `hl_ha_internal_case`.
pub fn internal_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| internal_body(&mut r));
    end();
}

fn same_array<T: Send + Sync + 'static>(a: &Handle<HArray1<T>>, b: &Handle<HArray1<T>>) -> bool {
    a.object_address() == b.object_address()
}

/// The array `0..=0` a second pointer names when it differs from the first.
fn empty_copy<T: Default + Send + Sync + 'static>() -> Result<Handle<HArray1<T>>, Failure> {
    let array = HArray1::with_bounds(0, 0).map_err(|e| e.into_failure("probe"))?;
    Ok(Handle::new(array))
}

fn grow_step(pid: &PolyInternalData, which: i32, same: bool) -> Result<(), Failure> {
    match which {
        0 => {
            let mut a = pid.t_data();
            let mut b = if same { a.clone() } else { empty_copy()? };
            pid.inc_t_data(&mut a, &mut b)?;
            flag(same_array(&a, &pid.t_data()));
            flag(same_array(&b, &pid.t_data()));
        }
        1 => {
            let mut a = pid.pi_seg();
            let mut b = if same { a.clone() } else { empty_copy()? };
            pid.inc_pi_seg(&mut a, &mut b)?;
            flag(same_array(&a, &pid.pi_seg()));
            flag(same_array(&b, &pid.pi_seg()));
        }
        _ => {
            let mut a = pid.pi_nod();
            let mut b = if same { a.clone() } else { empty_copy()? };
            pid.inc_pi_nod(&mut a, &mut b)?;
            flag(same_array(&a, &pid.pi_nod()));
            flag(same_array(&b, &pid.pi_nod()));
        }
    }
    Ok(())
}

fn uppers(pid: &PolyInternalData) {
    let upper = |u: Option<i32>| int(u.unwrap_or(-1));
    upper(pid.t_data().get().map(|a| a.array1().upper()));
    upper(pid.pi_seg().get().map(|a| a.array1().upper()));
    upper(pid.pi_nod().get().map(|a| a.array1().upper()));
}

fn grow_body(r: &mut Reader) -> Result<(), Failure> {
    let (nn, nt) = (r.integer(), r.integer());
    let pid = PolyInternalData::new(nn, nt)?;
    uppers(&pid);
    for _ in 0..r.integer() {
        let (which, same) = (r.integer(), r.flag());
        grow_step(&pid, which, same)?;
        [pid.nb_t_data(), pid.nb_pi_seg(), pid.nb_pi_nod()]
            .into_iter()
            .for_each(int);
        uppers(&pid);
    }
    pid.dec_t_data();
    pid.dec_pi_seg();
    pid.dec_pi_nod();
    [pid.nb_t_data(), pid.nb_pi_seg(), pid.nb_pi_nod()]
        .into_iter()
        .for_each(int);
    Ok(())
}

/// `nbNod nbTri n (which same)*n`: see `hl_ha_grow_case`.
pub fn grow_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| grow_body(&mut r));
    end();
}
