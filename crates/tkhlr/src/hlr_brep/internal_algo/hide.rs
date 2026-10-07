//! HLRBRep_InternalAlgo.cxx: Select, ShowAll, HideAll and the Hide members.
use super::InternalAlgo;
use crate::hlr_algo::MinMaxIndices;
use crate::hlr_brep::{Data, FaceData, Hider};
use rustycad_tkernel::standard::Failure;

// An index range of edges or faces.
type Range = (i32, i32);

// The order of the hiding faces: elementary faces first (planes before cylinders,
// cones, spheres and tori), cut, outlined and side faces last.
fn face_value(fd: &FaceData) -> i32 {
    let mut val = if fd.plane() {
        10
    } else if fd.cylinder() {
        9
    } else if fd.cone() {
        8
    } else if fd.sphere() {
        7
    } else if fd.torus() {
        6
    } else {
        0
    };
    if fd.cut() {
        val -= 10;
    }
    if fd.side() {
        val -= 100;
    }
    if fd.with_out_l() {
        val -= 20;
    }
    val
}

// Whether `a` sorts after `b` in the heap of HideSelected: a greater value, or the
// same value and a greater size.
fn after(val: &[i32], size: &[f64], a: usize, b: usize) -> bool {
    val[a] > val[b] || (val[a] == val[b] && size[a] > size[b])
}

/// The native heap sort of HideSelected (on more than two faces): the face indices
/// by decreasing value, then decreasing size; `index[0]` is unused.
fn heap_sort(index: &mut [usize], val: &[i32], size: &[f64]) {
    let nf = index.len() - 1;
    let mut l = (nf >> 1) + 1;
    let mut ir = nf;
    loop {
        let rra;
        if l > 1 {
            l -= 1;
            rra = index[l];
        } else {
            rra = index[ir];
            index[ir] = index[1];
            ir -= 1;
            if ir == 1 {
                index[1] = rra;
                break;
            }
        }
        let mut i = l;
        let mut k = l + l;
        while k <= ir {
            if k < ir && after(val, size, index[k], index[k + 1]) {
                k += 1;
            }
            if after(val, size, rra, index[k]) {
                index[i] = index[k];
                i = k;
                k <<= 1;
            } else {
                k = ir + 1;
            }
        }
        index[i] = rra;
    }
}

// The box test of Hide(I, J) (the native test skips `I.Max[6] - J.Min[6]`).
fn boxes_meet(i: &MinMaxIndices, j: &MinMaxIndices) -> bool {
    let ok = |x: i32, y: i32| (x.wrapping_sub(y) as u32 & 0x8000_8000) == 0;
    (0..6).all(|k| ok(j.max[k], i.min[k]) && ok(i.max[k], j.min[k]))
        && ok(j.max[6], i.min[6])
        && ok(j.max[7], i.min[7])
}

impl InternalAlgo {
    // Applies `edit` to the selection flags of every edge and face.
    fn set_selection(
        &self,
        edges: Option<(i32, i32)>,
        faces: Option<(i32, i32)>,
    ) -> Result<(), Failure> {
        let Some(ds) = self.data() else {
            return Ok(());
        };
        let data = ds.get().ok_or_else(|| Failure::new("no data"))?;
        if let Some((e1, e2)) = edges {
            let mut ed = data.e_data_array();
            for e in 1..=data.nb_edges() {
                ed.change_value(e)
                    .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::Select"))?
                    .set_selected(e >= e1 && e <= e2);
            }
        }
        if let Some((f1, f2)) = faces {
            let mut fd = data.f_data_array();
            for f in 1..=data.nb_faces() {
                fd.change_value(f)
                    .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::Select"))?
                    .set_selected(f >= f1 && f <= f2);
            }
        }
        Ok(())
    }

    // The edge and face ranges of the shape `i` (checked).
    fn ranges(&self, member: &str, i: i32) -> Result<(Range, Range), Failure> {
        let (_, _, e1, e2, f1, f2) = self
            .shape_bounds(i)
            .map_err(|_| super::unknown_shape(member, i))?
            .bounds();
        Ok(((e1, e2), (f1, f2)))
    }

    /// Select(): selects every edge and face; `algo.select()?`.
    pub fn select(&self) -> Result<(), Failure> {
        self.set_selection(Some((1, i32::MAX)), Some((1, i32::MAX)))
    }

    /// Select(I): selects the edges and faces of the shape `i`; `algo.select_shape(1)?`.
    pub fn select_shape(&self, i: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        let (edges, faces) = self.ranges("Select", i)?;
        self.set_selection(Some(edges), Some(faces))
    }

    /// SelectEdge(I): selects the edges of the shape `i`; `algo.select_edge(1)?`.
    pub fn select_edge(&self, i: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        let (edges, _) = self.ranges("SelectEdge", i)?;
        self.set_selection(Some(edges), None)
    }

    /// SelectFace(I): selects the faces of the shape `i`; `algo.select_face(1)?`.
    pub fn select_face(&self, i: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        let (_, faces) = self.ranges("SelectFace", i)?;
        self.set_selection(None, Some(faces))
    }

    // Shows (`show`) or hides every edge, or only the selected ones.
    fn set_status(&self, show: bool, selected_only: bool) -> Result<(), Failure> {
        let Some(ds) = self.data() else {
            return Ok(());
        };
        let data = ds.get().ok_or_else(|| Failure::new("no data"))?;
        let mut ed = data.e_data_array();
        for e in 1..=data.nb_edges() {
            let edge = ed
                .change_value(e)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::ShowAll"))?;
            if selected_only && !edge.selected() {
                continue;
            }
            if show {
                edge.status_mut().show_all();
            } else {
                edge.status_mut().hide_all();
            }
        }
        Ok(())
    }

    /// ShowAll(): every edge visible; `algo.show_all()?`.
    pub fn show_all(&self) -> Result<(), Failure> {
        self.set_status(true, false)
    }

    /// ShowAll(I): the edges of the shape `i` visible (and selected);
    /// `algo.show_all_shape(1)?`.
    pub fn show_all_shape(&self, i: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        self.check("ShowAll", i)?;
        self.select_shape(i)?;
        self.set_status(true, true)
    }

    /// HideAll(): every edge hidden; `algo.hide_all()?`.
    pub fn hide_all(&self) -> Result<(), Failure> {
        self.set_status(false, false)
    }

    /// HideAll(I): the edges of the shape `i` hidden (and selected);
    /// `algo.hide_all_shape(1)?`.
    pub fn hide_all_shape(&self, i: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        self.check("HideAll", i)?;
        self.select_shape(i)?;
        self.set_status(false, true)
    }

    /// PartialHide(): each shape hidden by its own faces; `algo.partial_hide()?`.
    pub fn partial_hide(&self) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        for i in 1..=self.nb_shapes() {
            self.hide_shape(i)?;
        }
        self.select()
    }

    /// Hide(): each shape hidden by its own faces, then by the faces of every other
    /// shape; `algo.hide()?`.
    pub fn hide(&self) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        let n = self.nb_shapes();
        for i in 1..=n {
            self.hide_shape(i)?;
        }
        for i in 1..=n {
            for j in 1..=n {
                if i != j {
                    self.hide_shapes(i, j)?;
                }
            }
        }
        self.select()
    }

    /// Hide(I): the shape `i` hidden by its own faces; `algo.hide_shape(1)?`.
    pub fn hide_shape(&self, i: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        self.check("Hide", i)?;
        self.select_shape(i)?;
        self.init_edge_status()?;
        self.hide_selected(i, true)
    }

    /// Hide(I, J): the edges of the shape `i` hidden by the faces of the shape `j`
    /// when their boxes meet; `algo.hide_shapes(1, 2)?`.
    pub fn hide_shapes(&self, i: i32, j: i32) -> Result<(), Failure> {
        if self.data().is_none() {
            return Ok(());
        }
        let n = self.nb_shapes();
        if i <= 0 || i > n || j <= 0 || j > n {
            return Err(Failure::with_message(
                "Standard_OutOfRange",
                &format!(
                    "HLRBRep_InternalAlgo::Hide : unknown Shapes {i} {j}; expected 1 to NbShapes"
                ),
            ));
        }
        if i == j {
            return self.hide_shape(i);
        }
        let (bi, bj) = (
            *self.shape_bounds(i)?.min_max(),
            *self.shape_bounds(j)?.min_max(),
        );
        if boxes_meet(&bi, &bj) {
            self.select_edge(i)?;
            self.select_face(j)?;
            self.hide_selected(i, false)?;
        }
        Ok(())
    }

    // HideSelected(I, SideFace): the edges of the shape `i` hidden by the selected
    // hiding faces, in the order of `face_value` and size.
    fn hide_selected(&self, i: i32, side_face: bool) -> Result<(), Failure> {
        let sb = self.shape_bounds(i)?;
        let (_, _, e1, e2, _, _) = sb.bounds();
        if e2 < e1 {
            return Ok(());
        }
        let ds = self.data_structure();
        let data = ds
            .get()
            .ok_or_else(|| Failure::new("HLRBRep_InternalAlgo::HideSelected: no data structure"))?;
        data.init_bound_sort(sb.min_max(), e1, e2)?;
        let cache = Hider::new(&ds);
        let nf = data.nb_faces();
        if nf == 0 {
            return Ok(());
        }
        let order = Self::hiding_order(data, side_face, &cache)?;
        let mut mst = self.mst.write();
        for fi in order {
            let hiding = {
                let fd = data.f_data_array();
                let fd = fd
                    .value(fi)
                    .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::HideSelected"))?;
                fd.selected() && fd.hiding()
            };
            if hiding {
                cache.hide(fi, &mut mst)?;
            }
        }
        Ok(())
    }

    // The faces in hiding order (OwnHiding of the selected side faces first).
    fn hiding_order(data: &Data, side_face: bool, cache: &Hider) -> Result<Vec<i32>, Failure> {
        let fd = data.f_data_array();
        let nf = data.nb_faces() as usize;
        let face = |f: usize| {
            fd.value(f as i32)
                .map_err(|x| x.into_failure("HLRBRep_InternalAlgo::HideSelected"))
        };
        if side_face {
            for f in 1..=nf {
                let fd = face(f)?;
                if fd.selected() && fd.side() {
                    cache.own_hiding(f as i32);
                }
            }
        }
        let (mut val, mut size) = (vec![0; nf + 1], vec![0.; nf + 1]);
        for f in 1..=nf {
            val[f] = face_value(face(f)?);
            size[f] = face(f)?.size();
        }
        let mut index: Vec<usize> = (0..=nf).collect();
        if nf > 2 {
            heap_sort(&mut index, &val, &size);
        }
        Ok(index.into_iter().skip(1).map(|f| f as i32).collect())
    }
}
