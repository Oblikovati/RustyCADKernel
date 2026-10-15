//! BRepOffset_Inter3d probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffset_inter3d.cxx (the records are documented there).
use super::common::{Reader, flag, int};
use super::shape::shape;
use super::tool::{finish, primitive};
use rustycad_tkbool::brep_algo::{BRepAlgoAsDes, BRepAlgoImage};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{ShapeEnum, State};
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkoffset::brep_offset::{
    BRepOffsetAnalyse, BRepOffsetInter3d, BRepOffsetOffset, OffsetFaces, ShapeShapeMap,
};

/// The descendants of a shape in the AsDes.
fn descendants(as_des: &BRepAlgoAsDes, s: &Shape) -> Result<(), Failure> {
    let l: Vec<Shape> = as_des.descendant(s).collect();
    int(l.len() as i32);
    for d in &l {
        int(d.orientation() as i32);
        shape(d)?;
    }
    Ok(())
}

/// `oi_inter`.
pub fn inter_case(values: &[f64]) {
    let mut r = Reader::new(values);
    finish((|| {
        let p = primitive(&mut r)?;
        let off = r.real();
        let angle = r.real();
        let op = r.integer();
        let planar = r.flag();
        let mut a = BRepOffsetAnalyse::new();
        a.set_offset_value(off);
        a.perform(&p, angle)?;
        let mut map_sf = OffsetFaces::default();
        let mut image = BRepAlgoImage::new();
        let mut offset_faces = List::default();
        for fs in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)? {
            let o = BRepOffsetOffset::from_face(
                &topods::face(&fs)?,
                off,
                true,
                JoinType::Intersection,
            )?;
            image.bind(&fs, o.face().as_ref())?;
            offset_faces.append(o.face().as_ref().clone());
            map_sf.bind(fs, o);
        }
        let as_des = Handle::new(BRepAlgoAsDes::new());
        let side = if off > 0. { State::Out } else { State::In };
        let mut inter = BRepOffsetInter3d::new(&as_des, side, 1e-7);
        let (mut mes, mut build) = (ShapeShapeMap::default(), ShapeShapeMap::default());
        let mut failed = List::default();
        let range = ProgressRange::default();
        if op == 1 {
            inter.complet_int(&offset_faces, &image, &range)?;
        } else {
            inter.connex_int_by_int(
                &p,
                &map_sf,
                &a,
                &mut mes,
                &mut build,
                &mut failed,
                &range,
                planar,
            )?;
        }
        if op == 2 {
            let mut ctx = IndexedMapOfShape::default();
            if let Some(first) =
                Iterator::next(&mut Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)?)
            {
                ctx.add(first);
            }
            inter.context_int_by_int(
                &ctx,
                planar,
                &map_sf,
                &a,
                &mut mes,
                &mut build,
                &mut failed,
                &range,
                false,
            )?;
        }
        let ad = as_des.get().expect("AsDes");
        for fs in Explorer::new(&p, ShapeEnum::Face, ShapeEnum::Shape)? {
            let of = map_sf.seek(&fs).expect("offset").face().as_ref().clone();
            let nf = mes.seek(&of).cloned();
            flag(nf.is_some());
            if let Some(nf) = nf {
                shape(&nf)?;
                descendants(ad, &nf)?;
            }
            descendants(ad, &of)?;
            descendants(ad, &fs)?;
        }
        let mut ve = IndexedMapOfShape::default();
        topexp::map_shapes_of_type(&p, ShapeEnum::Edge, &mut ve)?;
        topexp::map_shapes_of_type(&p, ShapeEnum::Vertex, &mut ve)?;
        for i in 1..=ve.extent() {
            let b = build.seek(ve.find_key(i)?).cloned();
            flag(b.is_some());
            if let Some(b) = b {
                shape(&b)?;
            }
        }
        int(failed.len() as i32);
        for s in failed.iter() {
            int(ve.find_index(s) as i32);
        }
        int(inter.new_edges().extent() as i32);
        int(inter.touched_faces().extent() as i32);
        Ok(())
    })());
}
