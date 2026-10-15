//! BRepOffsetAPI_DraftAngle and Draft_Modification probe operations, the Rust twin of
//! tools/oracle/tkoffset_probe_brepoffsetapi_draft_angle.cxx (the records are documented
//! there).
use super::common::{Reader, attempt, end, flag, int};
use super::make_offset::failed;
use super::shape::shape;
use super::tool::{primitive, sub};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Face};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Plane3;
use rustycad_tkoffset::brep_offset_api::BRepOffsetAPIDraftAngle;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

/// `oa_draft_angle prim nadd (face dx dy dz angle px py pz nx ny nz flag)... remove`.
pub fn draft_angle_case(values: &[f64]) {
    let mut r = Reader::new(values);
    let prim = match primitive(&mut r) {
        Ok(prim) => prim,
        Err(error) => return failed(error),
    };
    let mut draft = match BRepOffsetAPIDraftAngle::with_shape(&prim) {
        Ok(draft) => draft,
        Err(error) => return failed(error),
    };
    let nadd = r.integer();
    let mut first = Face::default();
    for _ in 0..nadd {
        let index = r.integer();
        let (d, angle, p, n, fl) = (r.dir(), r.real(), r.pnt(), r.dir(), r.flag());
        attempt(|| {
            let f = topods::face(&sub(&prim, ShapeEnum::Face, index)?)?;
            if first.is_null() {
                first = f.clone();
            }
            draft.add(&f, d, angle, &Plane3::new(p, n)?, fl)?;
            flag(draft.add_done()?);
            Ok(())
        });
    }
    let removed = r.integer();
    if removed >= 0 {
        attempt(|| {
            draft.remove(&topods::face(&sub(&prim, ShapeEnum::Face, removed)?)?)?;
            flag(draft.add_done()?);
            Ok(())
        });
    }
    attempt(|| draft.build(&ProgressRange::default()));
    flag(draft.is_done());
    attempt(|| {
        int(draft.status()? as i32);
        let mut map = IndexedMapOfShape::default();
        topexp::map_shapes(&prim, &mut map, true, true);
        let bad = draft.problematic_shape()?;
        if bad.is_null() {
            int(-1);
        } else {
            int(bad.shape_type()? as i32);
            int(map.find_index(&bad) as i32);
        }
        Ok(())
    });
    if draft.is_done() {
        attempt(|| draft.shape().and_then(shape));
    }
    for f in Explorer::new(&prim, ShapeEnum::Face, ShapeEnum::Shape)
        .map(|e| e.collect::<Vec<_>>())
        .unwrap_or_default()
    {
        attempt(|| draft.generated(&f).map(|l| int(l.len() as i32)));
        attempt(|| draft.modified(&f).map(|l| int(l.len() as i32)));
    }
    attempt(|| draft.modified_faces().map(|l| int(l.len() as i32)));
    if !first.is_null() {
        attempt(|| draft.connected_faces(&first).map(|l| int(l.len() as i32)));
    }
    end();
}
