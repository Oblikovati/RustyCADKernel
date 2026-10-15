//! TopOpeBRepDS interference processing probe, the Rust twin of
//! tools/oracle/tkbool_probe_topopebrepds_ops.cxx (the records are documented there).
use super::common::{Reader, attempt, end, flag, int, real};
use super::top_ope_brep_tool_tool::solid;
use rustycad_tkbool::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSCurvePointInterference,
    TopOpeBRepDSEdgeVertexInterference, TopOpeBRepDSFaceEdgeInterference,
    TopOpeBRepDSHDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind, TopOpeBRepDSPoint,
    TopOpeBRepDSShapeShapeInterference, TopOpeBRepDSTransition, fdscnx_close,
};
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::top_abs::{ShapeEnum, State};

#[path = "top_ope_brep_ds_ops_run.rs"]
mod run;

/// The kinds in their native order.
pub(crate) const KINDS: [TopOpeBRepDSKind; 12] = [
    TopOpeBRepDSKind::Point,
    TopOpeBRepDSKind::Curve,
    TopOpeBRepDSKind::Surface,
    TopOpeBRepDSKind::Vertex,
    TopOpeBRepDSKind::Edge,
    TopOpeBRepDSKind::Wire,
    TopOpeBRepDSKind::Face,
    TopOpeBRepDSKind::Shell,
    TopOpeBRepDSKind::Solid,
    TopOpeBRepDSKind::CompSolid,
    TopOpeBRepDSKind::Compound,
    TopOpeBRepDSKind::Unknown,
];

/// The configurations in their native order.
const CONFIGS: [TopOpeBRepDSConfig; 3] = [
    TopOpeBRepDSConfig::UnshGeometry,
    TopOpeBRepDSConfig::SameOriented,
    TopOpeBRepDSConfig::DiffOriented,
];

type H = Handle<TopOpeBRepDSInterference>;

/// `BlDs`.
pub(crate) struct Ds {
    pub(crate) hds: Handle<TopOpeBRepDSHDataStructure>,
    pub(crate) idx: [Vec<i32>; 4],
    pub(crate) np: i32,
}

/// `bl_ds_ref`.
pub(crate) fn ds_ref(d: &Ds, r: i32) -> i32 {
    let (k, n) = (r / 1000, r % 1000);
    if k == 4 {
        return if d.np == 0 { 0 } else { n % d.np + 1 };
    }
    let v = &d.idx[k as usize];
    if v.is_empty() {
        0
    } else {
        v[n as usize % v.len()]
    }
}

/// `bl_ds_type`.
fn ds_type(i: &H) -> i32 {
    let names = [
        "TopOpeBRepDS_CurvePointInterference",
        "TopOpeBRepDS_EdgeVertexInterference",
        "TopOpeBRepDS_FaceEdgeInterference",
        "TopOpeBRepDS_ShapeShapeInterference",
        "TopOpeBRepDS_SurfaceCurveInterference",
        "TopOpeBRepDS_SolidSurfaceInterference",
    ];
    names
        .iter()
        .position(|n| i.is_instance_name(n))
        .unwrap_or(6) as i32
}

/// `bl_ds_i`.
pub(crate) fn ds_i(i: &H) {
    let Some(ii) = i.get() else {
        int(-1);
        return;
    };
    let ty = ds_type(i);
    int(ty);
    let t = ii.transition();
    int(t.before() as i32);
    int(t.after() as i32);
    int(t.shape_before() as i32);
    int(t.shape_after() as i32);
    int(t.index_before());
    int(t.index_after());
    int(ii.support_type() as i32);
    int(ii.support());
    int(ii.geometry_type() as i32);
    int(ii.geometry());
    if ty == 0 {
        let c: Handle<TopOpeBRepDSCurvePointInterference> = i.cast();
        real(c.get().map_or(0., |c| c.parameter()));
    }
    if ty == 1 {
        let e: Handle<TopOpeBRepDSEdgeVertexInterference> = i.cast();
        real(e.get().map_or(0., |e| e.parameter()));
    }
    if (1..=3).contains(&ty) {
        let s: Handle<TopOpeBRepDSShapeShapeInterference> = i.cast();
        if let Some(s) = s.get() {
            flag(s.gbound());
            int(s.config() as i32);
        }
    }
}

/// `bl_ds_loi`.
pub(crate) fn ds_loi(l: &ListOfInterference) {
    int(l.extent());
    for i in l.iter() {
        ds_i(i);
    }
}

/// `bl_ds_dump`.
fn ds_dump(d: &Ds) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let ds = h.ds();
    int(ds.nb_shapes());
    for i in 1..=ds.nb_shapes() {
        let l = ds.shape_interferences_index(i, true)?;
        if l.is_empty() {
            continue;
        }
        int(i);
        ds_loi(l);
    }
    let mut se = Vec::new();
    for i in 1..=ds.nb_section_edges() {
        se.push(ds.shape_index(&ds.section_edge(i, true)?.into_shape(), true));
    }
    se.sort_unstable();
    int(ds.nb_section_edges());
    se.into_iter().for_each(int);
    for i in 1..=ds.nb_points() {
        flag(ds.keep_point(i)?);
    }
    Ok(())
}

/// `bl_ds_make`.
fn ds_make(d: &Ds, r: &mut Reader<'_>) -> Result<(i32, H), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let target = ds_ref(d, r.integer());
    let ty = r.integer();
    let (sb, sa) = (
        State::ALL[r.integer() as usize],
        State::ALL[r.integer() as usize],
    );
    let (shb, sha) = (
        ShapeEnum::ALL[r.integer() as usize],
        ShapeEnum::ALL[r.integer() as usize],
    );
    let (ib, ia) = (ds_ref(d, r.integer()), ds_ref(d, r.integer()));
    let sk = KINDS[r.integer() as usize];
    let s = ds_ref(d, r.integer());
    let gk = KINDS[r.integer() as usize];
    let g = ds_ref(d, r.integer());
    let t = r.real();
    let gb = r.integer() != 0;
    let cf = CONFIGS[r.integer() as usize];
    let ts = h.ds().shape(target, true).clone();
    let mut p = t;
    if !ts.is_null() && ts.shape_type()? == ShapeEnum::Edge {
        let (f, l) = BRepTool::range(&topods::edge(&ts)?)?;
        p = f + t * (l - f);
    }
    let mut tr = TopOpeBRepDSTransition::new(sb, sa, shb, sha);
    tr.set_index_before(ib);
    tr.set_index_after(ia);
    let i: H = match ty {
        0 => Handle::new(TopOpeBRepDSCurvePointInterference::new(
            &tr, sk, s, gk, g, p,
        ))
        .cast(),
        1 => Handle::new(TopOpeBRepDSEdgeVertexInterference::new(
            &tr, sk, s, g, gb, cf, p,
        ))
        .cast(),
        2 => Handle::new(TopOpeBRepDSFaceEdgeInterference::new(&tr, s, g, gb, cf)).cast(),
        _ => Handle::new(TopOpeBRepDSShapeShapeInterference::new(
            &tr, sk, s, gk, g, gb, cf,
        ))
        .cast(),
    };
    Ok((target, i))
}

/// The solids, faces, edges and vertices of the two solids added with their ranks.
fn add_shapes(d: &mut Ds, s: &[Shape; 2]) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let mut ds = h.change_ds();
    for (k, sk) in s.iter().enumerate() {
        d.idx[0].push(ds.add_shape_with_rank(sk, k as i32 + 1));
    }
    for (k, sk) in s.iter().enumerate() {
        for (j, ty) in [ShapeEnum::Face, ShapeEnum::Edge, ShapeEnum::Vertex]
            .into_iter()
            .enumerate()
        {
            for x in Explorer::new(sk, ty, ShapeEnum::Shape)? {
                let had = ds.has_shape(&x, true);
                let i = ds.add_shape_with_rank(&x, k as i32 + 1);
                if !had {
                    d.idx[j + 1].push(i);
                }
            }
        }
    }
    Ok(())
}

/// `bl_ds_build`.
fn ds_build(r: &mut Reader<'_>) -> Result<Ds, Failure> {
    let mut d = Ds {
        hds: Handle::new(TopOpeBRepDSHDataStructure::new()),
        idx: Default::default(),
        np: 0,
    };
    let s = [solid(r)?, solid(r)?];
    add_shapes(&mut d, &s)?;
    let h = d.hds.clone();
    let h = h.get().ok_or_else(|| Failure::new("null"))?;
    let np = r.integer();
    for _ in 0..np {
        let e = ds_ref(&d, r.integer());
        let (t, tol) = (r.real(), r.real());
        let mut ds = h.change_ds();
        let bc = BRepAdaptorCurve::with_edge(&topods::edge(ds.shape(e, true))?)?;
        let (f, l) = (bc.first_parameter()?, bc.last_parameter()?);
        ds.add_point(&TopOpeBRepDSPoint::new(bc.value(f + t * (l - f))?, tol));
    }
    d.np = np;
    for _ in 0..r.integer() {
        let (a, b) = (ds_ref(&d, r.integer()), ds_ref(&d, r.integer()));
        let mut ds = h.change_ds();
        let (sa, sb) = (ds.shape(a, true).clone(), ds.shape(b, true).clone());
        ds.fill_shapes_same_domain(&sa, &sb, true)?;
    }
    for _ in 0..r.integer() {
        let e = ds_ref(&d, r.integer());
        let mut ds = h.change_ds();
        let se = topods::edge(ds.shape(e, true))?;
        ds.add_section_edge(&se);
    }
    for _ in 0..r.integer() {
        let (target, i) = ds_make(&d, r)?;
        let mut ds = h.change_ds();
        let ts = ds.shape(target, true).clone();
        ds.change_shape_interferences(&ts).append(i);
    }
    Ok(d)
}

/// `dsx solids... op target arg`.
pub fn dsx_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let d = match ds_build(&mut r) {
        Ok(d) => d,
        Err(e) => {
            print!("{} ", e.exception_type());
            end();
            return;
        }
    };
    let (op, target, arg) = (r.integer(), ds_ref(&d, r.integer()), r.integer());
    attempt(|| run::dsx_run(&d, op, target, arg));
    attempt(|| ds_dump(&d));
    fdscnx_close();
    end();
}
