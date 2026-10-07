//! HLRBRep_Data.cxx: Write, the copy of another data structure into this one.
use super::{Data, data_array};
use crate::hlr_brep::{EdgeData, FaceData};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Failure;

const MEMBER: &str = "HLRBRep_Data::Write";

// The records and map keys of the source, copied before this structure is locked, so
// that the source may be any data structure.
struct Source {
    edges: Vec<(EdgeData, Shape)>,
    faces: Vec<(FaceData, Shape)>,
}

fn source(ds: &Data) -> Result<Source, Failure> {
    let (ed, em) = (ds.e_data_array(), ds.edge_map());
    let mut edges = Vec::new();
    for i in 1..=ds.nb_edges() {
        let e = ed.value(i).map_err(data_array(MEMBER))?.try_clone()?;
        edges.push((e, em.find_key(i as usize)?.clone()));
    }
    drop((ed, em));
    let (fd, fm) = (ds.f_data_array(), ds.face_map());
    let mut faces = Vec::new();
    for i in 1..=ds.nb_faces() {
        let f = fd.value(i).map_err(data_array(MEMBER))?.try_clone()?;
        faces.push((f, fm.find_key(i as usize)?.clone()));
    }
    Ok(Source { edges, faces })
}

// Shifts the edge indices of the wires of a copied face by `de`; the wire blocks
// are shared with the source face, as the native handle copy.
fn shift_wire_edges(fd: &FaceData, de: i32) -> Result<(), Failure> {
    let wb = fd.wires().get().ok_or_else(|| {
        Failure::with_message(
            "Standard_NullObject",
            "HLRBRep_Data::Write: a face has no wires; expected Set",
        )
    })?;
    for iw in 1..=wb.nb_wires() {
        let eb = wb.wire(iw)?;
        let eb = eb.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "HLRBRep_Data::Write: a wire has no edges; expected SetWire",
            )
        })?;
        for ie in 1..=eb.nb_edges() {
            eb.set_edge(ie, eb.edge(ie)? + de)?;
        }
    }
    Ok(())
}

impl Data {
    /// Write(DS, dv, de, df): copies the edges of `ds` after the edge `de` (their
    /// vertices shifted by `dv`) and its faces after the face `df` (their wire edges
    /// shifted by `de`), and adds their shapes to the maps;
    /// `ds.write(&part, 0, 0, 0)?`.
    pub fn write(&self, ds: &Data, dv: i32, de: i32, df: i32) -> Result<(), Failure> {
        let src = source(ds)?;
        {
            let mut ed = self.e_data_array();
            let mut em = self.edge_map();
            for (k, (mut e, key)) in src.edges.into_iter().enumerate() {
                if dv != 0 {
                    e.set_v_sta(e.v_sta() + dv);
                    e.set_v_end(e.v_end() + dv);
                }
                *ed.change_value(de + 1 + k as i32)
                    .map_err(data_array(MEMBER))? = e;
                em.add(key);
            }
        }
        let mut fd = self.f_data_array();
        let mut fm = self.face_map();
        for (k, (f, key)) in src.faces.into_iter().enumerate() {
            if de != 0 {
                shift_wire_edges(&f, de)?;
            }
            *fd.change_value(df + 1 + k as i32)
                .map_err(data_array(MEMBER))? = f;
            fm.add(key);
        }
        Ok(())
    }
}
