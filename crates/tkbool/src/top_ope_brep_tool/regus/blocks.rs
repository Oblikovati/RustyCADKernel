//! TopOpeBRepTool_REGUS.cxx: the walk of the faces of a shell into connected blocks
//! (new shells), the next face across an edge being the nearest by matter angle.
use super::TopOpeBRepToolREGUS;
use crate::top_ope_brep_tool::project::fun_tool_bounds;
use crate::top_ope_brep_tool::tool::TopOpeBRepToolTOOL;
use crate::top_ope_brep_tool::topology::{fun_tool_getxx, fun_tool_nggeom_f_on_edge};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Face, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Direction3;

/// `FUN_update(fcur, edstoconnect)` (static): the oriented non-closing edges of an
/// oriented face toggled in the edges to connect.
fn fun_update(fcur: &Shape, edstoconnect: &mut Map<Shape, ShapeMapHasher>) -> Result<(), Failure> {
    if matches!(
        fcur.orientation(),
        Orientation::Internal | Orientation::External
    ) {
        return Ok(());
    }
    let f = topods::face(fcur)?;
    for e in Explorer::new(fcur, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if matches!(
            e.orientation(),
            Orientation::Internal | Orientation::External
        ) {
            continue;
        }
        if TopOpeBRepToolTOOL::is_closing_e(&topods::edge(&e)?, &f)? {
            continue;
        }
        if edstoconnect.contains(&e) {
            edstoconnect.remove(&e);
        } else {
            edstoconnect.add(e);
        }
    }
    Ok(())
}

/// `FUN_vectors(f, e, pare, nt, xx, tola, approx)` (static): the oriented normal of the
/// face at the edge parameter (approximated beside the edge with `approx`) and the
/// vector inside the face; `None` when one fails.
fn fun_vectors(
    f: &Face,
    e: &Edge,
    pare: f64,
    tola: f64,
    approx: bool,
) -> Result<Option<(Direction3, Direction3)>, Failure> {
    let mut nt = if approx {
        match TopOpeBRepToolTOOL::try_ng_app(pare, e, f, tola)? {
            Some(n) => n,
            None => return Ok(None),
        }
    } else {
        match fun_tool_nggeom_f_on_edge(pare, e, f)? {
            Some(tmp) => Direction3::from_xyz(*tmp.xyz())?,
            None => return Ok(None),
        }
    };
    if f.orientation() == Orientation::Reversed {
        nt.reverse();
    }
    Ok(fun_tool_getxx(f, e, pare)?.map(|xx| (nt, xx)))
}

/// Standard_ProgramError of a native loop that never ends.
fn endless(member: &str) -> Failure {
    Failure::with_type(
        "Standard_ProgramError",
        &format!(
            "TopOpeBRepTool_REGUS::{member}: no face can be added and the block is open; the native loop does not end"
        ),
        "TopOpeBRepTool_REGUS",
    )
}

impl TopOpeBRepToolREGUS {
    /// The block closed: a new shell of its faces; false when the shell is unchanged.
    fn close_block(&mut self, splits: &mut List<Shape>) -> Result<bool, Failure> {
        let n_fcur = self.myl_fin_block.extent();
        if n_fcur == self.myoldn_f && self.myn_f == self.myoldn_f {
            return Ok(false);
        }
        splits.append(TopOpeBRepToolTOOL::mk_shell(&self.myl_fin_block)?);
        self.myl_fin_block.clear();
        Ok(true)
    }

    /// The current face taken off the faces of its edges and added to the block.
    fn take_face(&mut self) -> Result<(), Failure> {
        let myf = self.myf.clone();
        for e in Explorer::new(&myf, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if let Some(lof) = self.mymape_fs.change_seek(&e) {
                TopOpeBRepToolTOOL::remove(lof, &myf);
            }
        }
        self.myl_fin_block.append(myf);
        Ok(())
    }

    /// `REGU()`: the faces walked into blocks connected by their edges (a face across
    /// each edge to connect), each closed block giving a new shell stored for the
    /// shell; false when nothing is to regularize, the shell is unchanged or the walk
    /// fails; `r.regu()?`.
    pub fn regu(&mut self) -> Result<bool, Failure> {
        let mut splits: List<Shape> = List::default();
        if self.mymapemult.is_empty() && self.myn_f == self.myoldn_f {
            return Ok(false);
        }
        self.myl_fin_block.clear();
        let mut nite = 0;
        let mut stalls = 0;
        while nite <= self.myn_f {
            let mut start_block = self.myl_fin_block.is_empty();
            let end_block = self.myedstoconnect.is_empty() && !start_block;
            if end_block {
                if !self.close_block(&mut splits)? {
                    return Ok(false);
                }
                start_block = true;
            }
            if nite == self.myn_f {
                break;
            }
            let advance = if start_block || end_block {
                if !self.init_block()? {
                    return Ok(false);
                }
                true
            } else {
                self.nextin_block()?
            };
            fun_update(&self.myf.clone(), &mut self.myedstoconnect)?;
            if !advance {
                if !(self.myedstoconnect.is_empty() && !start_block) {
                    return Ok(false);
                }
                stalls += 1;
                if stalls > 2 {
                    return Err(endless("REGU"));
                }
                continue;
            }
            stalls = 0;
            self.take_face()?;
            nite += 1;
        }
        let s = self.my_s.clone();
        self.my_osh_nsh.bind(s, splits);
        Ok(true)
    }

    /// `InitBlock()`: the first face of an edge still holding faces (the edges without
    /// face unbound); false when the edges to connect are not empty or no face is left;
    /// `r.init_block()?`.
    pub fn init_block(&mut self) -> Result<bool, Failure> {
        if self.myedstoconnect.extent() != 0 {
            return Ok(false);
        }
        let eds: Vec<Shape> = self.mymape_fs.items().map(|(k, _)| k.clone()).collect();
        for e in eds {
            let first = self.mymape_fs.seek(&e).and_then(|l| l.first().cloned());
            match first {
                None => {
                    self.mymape_fs.unbind(&e);
                }
                Some(f) => {
                    self.myf = f;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// The face across the edge: its single face, else the nearest by matter angle
    /// from `myf`; `None` when the edge is spent (removed) and `Some(false)` when the
    /// nearest face is not found.
    fn face_across(&mut self, e: &Shape, need_ref: bool) -> Result<Option<bool>, Failure> {
        let Some(lof) = self.mymape_fs.seek(e).cloned() else {
            self.myedstoconnect.remove(e);
            return Ok(None);
        };
        if lof.is_empty() {
            self.myedstoconnect.remove(e);
            self.mymape_fs.unbind(e);
            return Ok(None);
        }
        if lof.extent() == 1 {
            self.myf = lof.first().cloned().unwrap_or_default();
            return Ok(Some(true));
        }
        if need_ref {
            let Some(fref) = self.block_face_of(e) else {
                return Ok(Some(false));
            };
            self.myf = fref.into_shape();
        }
        match self.nearest_f(&topods::edge(e)?, &lof)? {
            Some(ffound) => {
                self.myf = ffound.into_shape();
                Ok(Some(true))
            }
            None => Ok(Some(false)),
        }
    }

    /// The first original face of the edge already in the block.
    fn block_face_of(&self, e: &Shape) -> Option<Face> {
        let mut mapf: Map<Shape, ShapeMapHasher> = Map::default();
        for f in self.myl_fin_block.iter() {
            mapf.add(f.clone());
        }
        let lofc = self.mymape_fsstatic.seek(e)?;
        lofc.iter()
            .find(|fc| mapf.contains(fc))
            .and_then(|fc| topods::face(fc).ok())
    }

    /// `NextinBlock()`: the next face across an edge of the current face to connect,
    /// else across any edge to connect; false when none is found;
    /// `r.nextin_block()?`.
    pub fn nextin_block(&mut self) -> Result<bool, Failure> {
        let myf = self.myf.clone();
        let mut eds: Vec<Shape> = Vec::new();
        for e in Explorer::new(&myf, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if self.myedstoconnect.contains(&e) {
                eds.push(e);
            }
        }
        let need_ref = eds.is_empty();
        if need_ref {
            // Natively the map is changed while it is iterated: the remaining edges
            // are taken from a snapshot.
            eds = self.myedstoconnect.iter().cloned().collect();
        }
        for e in &eds {
            match self.face_across(e, need_ref)? {
                None => continue,
                Some(found) => return Ok(found),
            }
        }
        Ok(false)
    }

    /// The matter angle from the reference face to a face across the edge (0 for
    /// tangent faces resolved by `MatterKPtg`); `Some(None)` when the faces are not
    /// opposite and `None` when a vector fails.
    fn face_angle(
        &self,
        (fref, x, y): (&Face, Direction3, Direction3),
        f: &Face,
        e: &Edge,
        pare: f64,
        tola: f64,
    ) -> Result<Option<Option<f64>>, Failure> {
        let Some((ntf, xxf)) = fun_vectors(f, e, pare, tola, false)? else {
            return Ok(None);
        };
        let Some(mut ang) = TopOpeBRepToolTOOL::matter_dirs(x, y, xxf, ntf, tola)? else {
            return Ok(Some(None));
        };
        if ang < tola {
            match TopOpeBRepToolTOOL::matter_k_ptg(fref, f, e)? {
                Some(a) => ang = a,
                None => return Ok(None),
            }
        }
        Ok(Some(Some(ang)))
    }

    /// `NearestF(e, lof, ffound)`: the face of the list across the edge making the
    /// smallest matter angle with the current face (the first opposite face, then the
    /// smaller or equal angles after it); `None` when none is opposite or a vector
    /// fails; `r.nearest_f(&e, &faces)?`.
    pub fn nearest_f(&self, e: &Edge, lof: &List<Shape>) -> Result<Option<Face>, Failure> {
        let fref = topods::face(&self.myf)?;
        let (f, l) = fun_tool_bounds(e)?;
        let eps: f64 = 0.45678;
        let pare = (1. - eps) * f + eps * l;
        let tola = precision::angular() * 1.0e3;
        let Some((y, x)) = fun_vectors(&fref, e, pare, tola, false)? else {
            return Ok(None);
        };
        let reference = (&fref, x, y);
        let mut found: Option<(Face, f64)> = None;
        for fs in lof.iter() {
            let fi = topods::face(fs)?;
            let Some(ang) = self.face_angle(reference, &fi, e, pare, tola)? else {
                return Ok(None);
            };
            let Some(angi) = ang else {
                continue;
            };
            match &found {
                None => found = Some((fi, angi)),
                Some((_, angfound)) if angi <= *angfound => found = Some((fi, angi)),
                _ => {}
            }
        }
        Ok(found.map(|(f, _)| f))
    }
}
