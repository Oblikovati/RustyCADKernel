//! HLRBRep_ShapeToHLR and HLRBRep_Data, the Rust twin of
//! tools/oracle/tkhlr_probe_hlrbrep_data.cxx.
use super::common::{Reader, attempt, end, flag, int, real};
use super::hlrbrep_fixtures::projector;
use super::hlrbrep_records::edge_flags;
use super::hlrbrep_shapes::shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkhlr::hlr_algo::{
    Interference, MinMaxIndices, add_min_max, copy_min_max, decode_min_max, encode_min_max,
};
use rustycad_tkhlr::hlr_brep::{Data, EdgeData, FaceData, LocalGeometry2D, ShapeToHLR};
use rustycad_tkhlr::hlr_topo_brep::{MapOfShapeTool, OutLiner};

fn minmax(m: &MinMaxIndices) {
    m.min.iter().chain(m.max.iter()).for_each(|v| int(*v));
}

fn data_edge(d: &EdgeData, boxes: bool) {
    edge_flags(d, true);
    if boxes {
        minmax(d.min_max());
    }
}

fn data_face(f: &FaceData) -> Result<(), Failure> {
    for b in [
        f.selected(),
        f.back(),
        f.side(),
        f.closed(),
        f.hiding(),
        f.simple(),
        f.cut(),
        f.with_out_l(),
        f.plane(),
        f.cylinder(),
        f.cone(),
        f.sphere(),
        f.torus(),
    ] {
        flag(b);
    }
    real(f.size());
    int(f.orientation() as i32);
    real(f64::from(f.tolerance()));
    let wb = f.wires().get().ok_or_else(|| Failure::new("no wires"))?;
    int(wb.nb_wires());
    minmax(&wb.min_max());
    for w in 1..=wb.nb_wires() {
        let eb = wb.wire(w)?;
        let eb = eb.get().ok_or_else(|| Failure::new("no edges"))?;
        int(eb.nb_edges());
        minmax(&eb.min_max());
        for e in 1..=eb.nb_edges() {
            int(eb.edge(e)?);
            int(eb.orientation(e)? as i32);
            for b in [
                eb.out_line(e)?,
                eb.internal(e)?,
                eb.double(e)?,
                eb.iso_line(e)?,
            ] {
                flag(b);
            }
        }
    }
    Ok(())
}

fn local2d((t, n, c): LocalGeometry2D) {
    real(t.x());
    real(t.y());
    real(n.x());
    real(n.y());
    real(c);
}

fn interference(ds: &Data, i: &Interference) {
    let x = i.intersection();
    int(x.orientation() as i32);
    int(x.level());
    int(x.seg_index());
    int(x.index());
    real(x.parameter());
    real(f64::from(x.tolerance()));
    int(x.state() as i32);
    let (fe, p) = i.boundary().value_2d();
    int(fe);
    real(p);
    if x.state() == State::On {
        let (a, b) = i.boundary().state_3d();
        int(a as i32);
        int(b as i32);
    }
    int(i.orientation() as i32);
    int(i.transition() as i32);
    int(i.boundary_transition() as i32);
    attempt(|| {
        local2d(ds.local_le_geometry_2d(x.parameter())?);
        Ok(())
    });
    attempt(|| {
        local2d(ds.local_fe_geometry_2d(fe, p)?);
        Ok(())
    });
}

fn hide_edge(ds: &Data, e: i32) -> Result<(), Failure> {
    let mut il = List::default();
    ds.init_interference()?;
    while ds.more_interference() {
        let rej = ds.rejected_interference()?;
        flag(rej);
        flag(ds.above_interference());
        if !rej {
            let i = ds.interference();
            interference(ds, &i);
            il.append(i);
        }
        ds.next_interference()?;
    }
    int(ds.compare(e)? as i32);
    let (a, _, b, _) = ds
        .e_data_array()
        .value(e)
        .map_err(|x| x.into_failure("probe"))?
        .status()
        .bounds();
    attempt(|| {
        let (state, level) = ds.classify(e, true, (a + b) / 2.)?;
        int(state as i32);
        int(level);
        Ok(())
    });
    attempt(|| {
        int(ds.simpl_classify(e, 3, a, b)? as i32);
        Ok(())
    });
    attempt(|| {
        int(ds.hiding_start_level(e, &il)?);
        Ok(())
    });
    let eda = ds.e_data_array();
    flag(ds.edge_of_the_hiding_face(e, eda.value(e).map_err(|x| x.into_failure("probe"))?));
    Ok(())
}

// The hiding walk of one face, as the Hider drives it, without the edge building.
fn hide(ds: &Data, f: i32, mst: &mut MapOfShapeTool) -> Result<(), Failure> {
    ds.init_edge(f, mst)?;
    flag(ds.is_bad_face()?);
    flag(ds.hiding_the_face());
    flag(ds.simple_hiding_face());
    let mut n = 0;
    while ds.more_edge()? && n < 200 {
        let e = ds.edge()?;
        int(e);
        attempt(|| hide_edge(ds, e));
        ds.next_edge(true)?;
        n += 1;
    }
    flag(ds.more_edge()?);
    Ok(())
}

// The box of the shape: the union of the edge boxes and the face boxes.
fn shape_box(ds: &Data, ne: i32, nf: i32) -> Result<MinMaxIndices, Failure> {
    let (mut smin, mut smax) = (MinMaxIndices::default(), MinMaxIndices::default());
    let (mut tmin, mut tmax) = (MinMaxIndices::default(), MinMaxIndices::default());
    let ed = ds.e_data_array();
    for e in 1..=ne {
        decode_min_max(
            ed.value(e).map_err(|x| x.into_failure("probe"))?.min_max(),
            &mut tmin,
            &mut tmax,
        );
        if e == 1 {
            copy_min_max(&tmin, &tmax, &mut smin, &mut smax);
        } else {
            add_min_max(&tmin, &tmax, &mut smin, &mut smax);
        }
    }
    drop(ed);
    let fd = ds.f_data_array();
    for f in 1..=nf {
        let wb = fd
            .value(f)
            .map_err(|x| x.into_failure("probe"))?
            .wires()
            .get()
            .ok_or_else(|| Failure::new("no wires"))?
            .min_max();
        decode_min_max(&wb, &mut tmin, &mut tmax);
        add_min_max(&tmin, &tmax, &mut smin, &mut smax);
    }
    let mut tot = MinMaxIndices::default();
    encode_min_max(&smin, &smax, &mut tot);
    Ok(tot)
}

fn data_body(r: &mut Reader, p: &rustycad_tkhlr::hlr_algo::Projector) -> Result<(), Failure> {
    let s = shape(r)?;
    let nb_iso = r.integer();
    let o = Handle::new(OutLiner::with_original(&s));
    let mut mst = MapOfShapeTool::default();
    let ds = ShapeToHLR::load(&o, p, &mut mst, nb_iso)?;
    let ds = ds.get().ok_or_else(|| Failure::new("no data"))?;
    let (ne, nf) = (ds.nb_edges(), ds.nb_faces());
    int(ds.nb_vertices());
    int(ne);
    int(nf);
    for e in 1..=ne {
        data_edge(&ds.e_data_array()[e], false);
    }
    ds.update(p)?;
    for e in 1..=ne {
        data_edge(&ds.e_data_array()[e], true);
    }
    for f in 1..=nf {
        data_face(&ds.f_data_array()[f])?;
    }
    if ne == 0 {
        return Ok(());
    }
    ds.init_bound_sort(&shape_box(ds, ne, nf)?, 1, ne)?;
    for f in 1..=nf {
        if ds.f_data_array()[f].hiding() {
            attempt(|| hide(ds, f, &mut mst));
        }
    }
    Ok(())
}

/// `projector shape nbIso`: see `hb_data_case`.
pub fn data_case(values: &[f64]) {
    let mut r = Reader::new(values);
    attempt(|| {
        let p = projector(&mut r)?;
        attempt(|| data_body(&mut r, &p));
        Ok(())
    });
    end();
}
