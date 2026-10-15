//! The operations of the `dsx` probe, the Rust twin of
//! tools/oracle/tkbool_probe_topopebrepds_ops_run.cxx.
use super::{Ds, KINDS, ds_loi, ds_ref};
use crate::bl_probe::common::{attempt, captured, flag, int, real, text};
use rustycad_tkbool::top_ope_brep_ds::export::{
    fds_config3d, fds_edge_is_connex_to_same_domain_faces, fds_getupperlower,
    fds_has_same_domain2d, fds_has_same_domain3d, fds_si_is_gi_of_i_of_sba_of_t_of_i,
    fun_ds_add_se_sdm1d, fun_ds_complete_for_e7, fun_ds_complete_for_se1, fun_ds_complete_for_se2,
    fun_ds_complete_for_se3, fun_ds_complete_for_se4, fun_ds_complete_for_se5,
    fun_ds_complete_for_se6, fun_ds_complete_for_se8, fun_ds_complete_for_se9,
    fun_ds_complete1d_for_se_sdm, fun_ds_feig_b1_to0, fun_ds_fill_sdm_faces, fun_ds_get_vsdm,
    fun_ds_getoov, fun_ds_getoov_hds, fun_ds_has_i2d, fun_ds_one_sd, fun_ds_ori_e_in_f,
    fun_ds_point_to_vertex, fun_ds_purge_for_e9, fun_ds_redusamsha, fun_ds_sdm,
    fun_select_traint_interference,
};
use rustycad_tkbool::top_ope_brep_ds::process_edge_interferences::fun_unkeep_esymetrictransitions;
use rustycad_tkbool::top_ope_brep_ds::process_face_interferences::fun_unkeep_fdouble_gbound_interferences;
use rustycad_tkbool::top_ope_brep_ds::process_interferences_tool::{
    fun_reducedoublons, fun_select_gi_interference, fun_select_gk_interference,
    fun_select_itrasha_interference, fun_select_si_interference, fun_select_sk_interference,
    fun_select_traori_interference, fun_select_trasha_interference, fun_select_traunk_interference,
    fun_select1d_i, fun_select2d_i, fun_select3d_interference, fun_selectpure2d_i,
    fun_unkeep_unknown,
};
use rustycad_tkbool::top_ope_brep_ds::{
    ListOfInterference, MapOfShapeOn1State, TopOpeBRepDSCheck, TopOpeBRepDSDumper, TopOpeBRepDSEIR,
    TopOpeBRepDSFIR, TopOpeBRepDSFilter, TopOpeBRepDSKind, TopOpeBRepDSTKI, TopOpeBRepDSTOOL,
    TopOpeBRepDSTransition, fds_repvg, fdscnx_edge_connexity_same_shape, fdscnx_has_connex_face,
    fdscnx_prepare, fun_ds_get_tr, fun_ds_redu2d1d, fun_recl_se, fun_scanloi,
};
use rustycad_tkbool::top_ope_brep_tool::TopOpeBRepToolShapeClassifier;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

type Loi = ListOfInterference;

/// `bl_dsx_tki`.
fn dsx_tki(l: &Loi) -> Result<(), Failure> {
    for s in 0..2 {
        let mut tki = TopOpeBRepDSTKI::new();
        if s == 0 {
            tki.fill_on_geometry(l)?
        } else {
            tki.fill_on_support(l)?
        }
        tki.init();
        while let Some((k, g, x)) = tki.value() {
            int(k as i32);
            int(g);
            int(x.extent());
            tki.next();
        }
        int(-1);
    }
    Ok(())
}

/// One selector of `bl_dsx_select`.
fn select_one(s: i32, l: &Loi, l1: &mut Loi, l2: &mut Loi, arg: i32) -> Result<i32, Failure> {
    match s {
        0 => fun_select_trasha_interference(
            l1,
            if arg % 2 != 0 {
                ShapeEnum::Edge
            } else {
                ShapeEnum::Face
            },
            l2,
        ),
        1 => fun_select_itrasha_interference(l1, arg, l2),
        2 => fun_select_traunk_interference(l1, l2),
        3 => fun_select_traori_interference(l1, Orientation::ALL[(arg % 4) as usize], l2),
        4 => fun_select_gk_interference(l1, KINDS[(arg % 9) as usize], l2),
        5 => fun_select_sk_interference(l1, KINDS[(arg % 9) as usize], l2),
        6 => fun_select_gi_interference(l1, arg, l2),
        7 => fun_select_si_interference(l1, arg, l2),
        _ => Ok(i32::from(fun_select_traint_interference(l, l2)?)),
    }
}

/// `bl_dsx_select`.
fn dsx_select(l: &Loi, arg: i32) {
    for s in 0..9 {
        let (mut l1, mut l2) = (l.clone(), Loi::default());
        attempt(|| {
            let n = select_one(s, l, &mut l1, &mut l2, arg)?;
            int(n);
            ds_loi(&l1);
            ds_loi(&l2);
            Ok(())
        });
    }
}

/// `bl_dsx_select3d`.
fn dsx_select3d(d: &Ds, six: i32, l: &Loi) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let mut ds = h.change_ds();
    let mut li = l.clone();
    let (mut lf, mut lfe, mut le) = (Loi::default(), Loi::default(), Loi::default());
    fun_select_trasha_interference(&mut li, ShapeEnum::Face, &mut lf)?;
    fun_select_sk_interference(&mut lf, TopOpeBRepDSKind::Edge, &mut lfe)?;
    fun_select_trasha_interference(&mut li, ShapeEnum::Edge, &mut le)?;
    let (mut lp, mut l2p) = (lfe.clone(), Loi::default());
    attempt(|| {
        int(fun_selectpure2d_i(&lf, &mut lp, &mut l2p)?);
        ds_loi(&lp);
        ds_loi(&l2p);
        Ok(())
    });
    let mut x: [Loi; 5] = Default::default();
    attempt(|| {
        let [r, t3, f3, t3r, t2] = &mut x;
        fun_select3d_interference(six, &mut ds, &mut lf, f3, &mut lfe, r, t3, t3r, t2)?;
        for l in [&lf, &*f3, &lfe, &*r, &*t3, &*t3r, &*t2] {
            ds_loi(l);
        }
        Ok(())
    });
    let mut e1 = Loi::default();
    attempt(|| {
        int(fun_select2d_i(
            six,
            &mut ds,
            ShapeEnum::Edge,
            &mut le,
            &mut e1,
        )?);
        ds_loi(&le);
        ds_loi(&e1);
        Ok(())
    });
    let mut esd = Loi::default();
    attempt(|| {
        int(fun_select1d_i(six, &mut ds, &mut le, &mut esd)?);
        ds_loi(&le);
        ds_loi(&esd);
        Ok(())
    });
    Ok(())
}

/// The list reductions of `bl_dsx_reduce` (doubles to reclSE).
fn reduce_lists(d: &Ds, six: i32, l: &Loi, arg: i32) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let mut ds = h.change_ds();
    let mut a = l.clone();
    attempt(|| {
        fun_reducedoublons(&mut a, &ds, six)?;
        ds_loi(&a);
        Ok(())
    });
    let mut b = l.clone();
    attempt(|| {
        fun_unkeep_unknown(&mut b, &mut ds, six)?;
        ds_loi(&b);
        Ok(())
    });
    let mut c = l.clone();
    attempt(|| {
        fun_unkeep_esymetrictransitions(&mut c, &ds, six)?;
        ds_loi(&c);
        Ok(())
    });
    let mut f = l.clone();
    attempt(|| {
        fun_unkeep_fdouble_gbound_interferences(&mut f)?;
        ds_loi(&f);
        Ok(())
    });
    let (mut e, mut re) = (l.clone(), Loi::default());
    let gt = if arg % 2 != 0 {
        TopOpeBRepDSKind::Vertex
    } else {
        TopOpeBRepDSKind::Point
    };
    attempt(|| {
        fds_repvg(&ds, six, gt, &mut e, &mut re)?;
        ds_loi(&e);
        ds_loi(&re);
        Ok(())
    });
    let (mut g, mut rg) = (l.clone(), Loi::default());
    attempt(|| {
        fun_recl_se(six, &ds, &mut g, &mut rg)?;
        ds_loi(&g);
        ds_loi(&rg);
        Ok(())
    });
    Ok(())
}

/// `bl_dsx_reduce`.
fn dsx_reduce(d: &Ds, six: i32, l: &Loi, arg: i32) -> Result<(), Failure> {
    reduce_lists(d, six, l, arg)?;
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let ds = h.ds();
    let mut lx: [Loi; 4] = Default::default();
    attempt(|| {
        let [lf, lr, lin, lex] = &mut lx;
        let (nf, nr, ni, ne) = fun_scanloi(l, lf, lr, lin, lex)?;
        [nf, nr, ni, ne].into_iter().for_each(int);
        ds_loi(lex);
        Ok(())
    });
    attempt(|| {
        let ((sb, ib, bd), (sa, ia, ad)) = fun_ds_get_tr(&ds, six, 0, l)?;
        [sb as i32, ib, bd, sa as i32, ia, ad]
            .into_iter()
            .for_each(int);
        Ok(())
    });
    let mut hl = Loi::default();
    attempt(|| {
        int(fun_ds_has_i2d(six, l, &mut hl)?);
        ds_loi(&hl);
        Ok(())
    });
    let (mut l1d, mut rest) = (Loi::default(), l.clone());
    fun_select_trasha_interference(&mut rest, ShapeEnum::Edge, &mut l1d)?;
    for i in l.iter() {
        let mut t = TopOpeBRepDSTransition::default();
        attempt(|| {
            flag(fun_ds_redu2d1d(&ds, six, i, &l1d, &mut t)?);
            [
                t.before() as i32,
                t.after() as i32,
                t.index_before(),
                t.index_after(),
            ]
            .into_iter()
            .for_each(int);
            Ok(())
        });
    }
    Ok(())
}

/// The shapes `six` and `other` of the structure.
fn two_shapes(d: &Ds, six: i32, other: i32) -> Result<(Shape, Shape), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let ds = h.ds();
    Ok((ds.shape(six, true).clone(), ds.shape(other, true).clone()))
}

/// The same domain queries of `bl_dsx_query`.
fn query_sdm(d: &Ds, six: i32, s: &Shape, o: &Shape) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    attempt(|| {
        let ds = h.ds();
        let (mut l3, mut l2) = (List::default(), List::default());
        flag(fds_has_same_domain3d(&ds, s, Some(&mut l3)));
        int(l3.extent());
        flag(fds_has_same_domain2d(&ds, s, Some(&mut l2)));
        int(l2.extent());
        Ok(())
    });
    attempt(|| {
        let oov = fun_ds_getoov(s, &h.ds());
        flag(oov.is_some());
        int(h.ds().shape_index(&oov.unwrap_or_default(), true));
        flag(fun_ds_getoov_hds(s, &d.hds)?.is_some());
        Ok(())
    });
    attempt(|| {
        let ds = h.ds();
        let iv = fun_ds_get_vsdm(&ds, six);
        flag(iv.is_some());
        int(iv.unwrap_or(0));
        flag(fun_ds_sdm(&ds, s, o));
        Ok(())
    });
    Ok(())
}

/// The geometric queries of `bl_dsx_query`.
fn query_geometry(d: &Ds, six: i32, s: &Shape, o: &Shape) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    attempt(|| {
        let c = fds_config3d(s, o)?;
        flag(c.is_some());
        int(c.map_or(0, |c| c as i32));
        Ok(())
    });
    attempt(|| {
        flag(fds_edge_is_connex_to_same_domain_faces(s, &d.hds)?);
        Ok(())
    });
    attempt(|| {
        let (k, ori) = fun_ds_ori_e_in_f(&h.ds(), &topods::edge(s)?, o)?;
        int(k);
        int(ori as i32);
        Ok(())
    });
    attempt(|| {
        let ie = fun_ds_one_sd(&h.ds(), six, o)?;
        flag(ie.is_some());
        int(ie.unwrap_or(0));
        Ok(())
    });
    Ok(())
}

/// `bl_dsx_query`.
fn dsx_query(d: &Ds, six: i32, other: i32) -> Result<(), Failure> {
    let (s, o) = two_shapes(d, six, other)?;
    query_sdm(d, six, &s, &o)?;
    query_geometry(d, six, &s, &o)?;
    attempt(|| {
        let mut ls = List::default();
        int(TopOpeBRepDSTOOL::e_share_g(
            &d.hds,
            &topods::edge(&s)?,
            &mut ls,
        )?);
        Ok(())
    });
    attempt(|| {
        let ie = TopOpeBRepDSTOOL::get_esd(&d.hds, &o, six)?;
        flag(ie.is_some());
        int(ie.unwrap_or(0));
        flag(TopOpeBRepDSTOOL::share_g(&d.hds, six, other)?);
        Ok(())
    });
    attempt(|| {
        int(fdscnx_edge_connexity_same_shape(&s, &d.hds)?.extent());
        flag(fdscnx_has_connex_face(&s, &d.hds)?);
        Ok(())
    });
    attempt(|| {
        let (p1, p2) = fds_getupperlower(&d.hds, six, 0.5)?;
        real(p1);
        real(p2);
        Ok(())
    });
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let l = h.ds().shape_interferences_index(six, true)?.clone();
    for i in l.iter() {
        attempt(|| {
            flag(fds_si_is_gi_of_i_of_sba_of_t_of_i(&h.ds(), other, i)?);
            Ok(())
        });
    }
    Ok(())
}

/// `bl_dsx_check`.
fn dsx_check(d: &Ds) -> Result<(), Failure> {
    let mut c = TopOpeBRepDSCheck::with_hds(&d.hds);
    attempt(|| {
        flag(c.chk_intg()?);
        Ok(())
    });
    attempt(|| {
        flag(c.chk_intg_sam_dom()?);
        Ok(())
    });
    attempt(|| {
        flag(c.one_vertex_on_pnt()?);
        Ok(())
    });
    captured(|console| {
        let mut out = Vec::new();
        let r = c.print_intg(&mut out);
        console.push_str(&String::from_utf8_lossy(&out));
        r
    });
    let du = TopOpeBRepDSDumper::new(&d.hds);
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let n = h.ds().nb_shapes();
    for i in 1..=n {
        attempt(|| {
            text(du.sprint_shape_index(i)?.as_bytes());
            Ok(())
        });
        let s = h.ds().shape(i, true).clone();
        attempt(|| {
            text(du.sprint_shape_ref_ori(&s, "")?.as_bytes());
            Ok(())
        });
    }
    Ok(())
}

/// The FUN_ds_* completion `k`.
fn complete(d: &Ds, k: i32, m: &MapOfShapeOn1State) -> Result<(), Failure> {
    let h = &d.hds;
    match k {
        0 => fun_ds_purge_for_e9(h),
        1 => fun_ds_complete_for_se1(h),
        2 => fun_ds_complete_for_se2(h),
        3 => fun_ds_complete_for_se3(h),
        4 => fun_ds_complete_for_se4(h),
        5 => fun_ds_complete_for_se5(h),
        6 => fun_ds_complete_for_se6(h),
        7 => fun_ds_complete_for_e7(h),
        8 => fun_ds_complete_for_se8(h),
        9 => fun_ds_complete_for_se9(h),
        10 => fun_ds_point_to_vertex(h),
        11 => fun_ds_redusamsha(h),
        12 => fun_ds_fill_sdm_faces(h),
        13 => fun_ds_add_se_sdm1d(h),
        14 => fun_ds_feig_b1_to0(h, m),
        _ => fun_ds_complete1d_for_se_sdm(h),
    }
}

/// `bl_dsx_run`.
pub(crate) fn dsx_run(d: &Ds, op: i32, six: i32, arg: i32) -> Result<(), Failure> {
    let h = d.hds.get().ok_or_else(|| Failure::new("null"))?;
    let (s1, s2) = (h.ds().shape(1, true).clone(), h.ds().shape(2, true).clone());
    fdscnx_prepare(&s1, &s2, &d.hds)?;
    let l = h.ds().shape_interferences_index(six, true)?.clone();
    let m = MapOfShapeOn1State::default();
    let mut sc = TopOpeBRepToolShapeClassifier::new();
    match op {
        0 => dsx_tki(&l),
        1 => {
            dsx_select(&l, arg);
            Ok(())
        }
        2 => dsx_select3d(d, six, &l),
        3 => dsx_reduce(d, six, &l, arg),
        4 => TopOpeBRepDSEIR::new(&d.hds).process_edge_interferences_index(six),
        5 => TopOpeBRepDSEIR::new(&d.hds).process_edge_interferences(),
        6 => TopOpeBRepDSFIR::new(&d.hds).process_face_interferences_index(six, &m),
        7 => TopOpeBRepDSFIR::new(&d.hds).process_face_interferences(&m),
        8 => TopOpeBRepDSFilter::new(&d.hds, Some(&mut sc)).process_edge_interferences_index(six),
        9 => {
            TopOpeBRepDSFilter::new(&d.hds, Some(&mut sc)).process_face_interferences_index(six, &m)
        }
        10 => TopOpeBRepDSFilter::new(&d.hds, Some(&mut sc)).process_interferences(),
        11 => dsx_query(d, six, ds_ref(d, arg)),
        12 => dsx_check(d),
        _ => complete(d, op - 13, &m),
    }
}
