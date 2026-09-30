// Rust translation of ModelingAlgorithms/TKTopAlgo/MAT2d/MAT2d_MiniPath.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `MAT2d_MiniPath`: the tree of the shortest connexions that link the lines of a
//! figure, and the path that runs around it.
mod minimum;

use super::Mat2dFigure;
use super::connexion::Mat2dConnexion;
use super::seq;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use std::collections::HashMap;

type Connexions = Sequence<Handle<Mat2dConnexion>>;

/// `MAT2d_MiniPath`: a minimal spanning tree of the lines of a figure (Prim's
/// algorithm on the shortest connexions), from the line `IndStart`, and the path
/// that explores it (each connexion, then its reverse). Repeated `Perform`s add to
/// the tree, as natively.
/// ```
/// use rustycad_tktopalgo::mat2d::Mat2dMiniPath;
/// let path = Mat2dMiniPath::new();
/// assert!(path.path().is_empty() && !path.is_connexions_from(1));
/// ```
pub struct Mat2dMiniPath {
    the_connexions: HashMap<i32, Connexions>,
    the_father: HashMap<i32, Handle<Mat2dConnexion>>,
    the_path: Connexions,
    the_direction: f64,
    ind_start: i32,
}

impl Default for Mat2dMiniPath {
    fn default() -> Self {
        Self::new()
    }
}

// The connexions from `index` (`theConnexions(index)`): Standard_NoSuchObject when
// there are none.
fn connexions_of(map: &HashMap<i32, Connexions>, index: i32) -> Result<&Connexions, Failure> {
    map.get(&index).ok_or_else(|| {
        errors::no_such_object(&format!(
            "MAT2d_MiniPath: no connexion from line {index}; expected a bound line"
        ))
    })
}

impl Mat2dMiniPath {
    /// `MAT2d_MiniPath()`: direction 1, start 0; `Mat2dMiniPath::new()`.
    pub fn new() -> Self {
        Self {
            the_connexions: HashMap::new(),
            the_father: HashMap::new(),
            the_path: Sequence::default(),
            the_direction: 1.0,
            ind_start: 0,
        }
    }
    /// `Perform(Figure, IndStart, Sense)`: the tree of the shortest connexions from
    /// the line `ind_start` (`sense` true when the circuit turns counterclockwise),
    /// then the path; `path.perform(&figure, 1, true)?`.
    pub fn perform(
        &mut self,
        figure: &Mat2dFigure,
        ind_start: i32,
        sense: bool,
    ) -> Result<(), Failure> {
        let nb_lines: i32 = figure.length();
        self.ind_start = ind_start;
        self.the_direction = if sense { -1. } else { 1. };
        let connexion = minimum::all_connexions(figure, nb_lines)?;
        let mut set1: Vec<i32> = vec![ind_start];
        let mut set2: Vec<i32> = (1..=nb_lines).filter(|&i| i != ind_start).collect();
        let (mut i_suiv, mut min_on_set1, mut min_on_set2) = (0usize, 0, 0);
        while !set2.is_empty() {
            let mut dist_s1s2: f64 = f64::MAX;
            for &line1 in &set1 {
                for (j, &line2) in set2.iter().enumerate() {
                    let distance: f64 = connexion.get(line1, line2)?.distance();
                    if distance < dist_s1s2 {
                        (i_suiv, dist_s1s2, min_on_set1, min_on_set2) =
                            (j + 1, distance, line1, line2);
                    }
                }
            }
            let chosen: i32 = *set2.get(i_suiv.wrapping_sub(1)).ok_or_else(|| {
                errors::out_of_range(&format!("MAT2d_MiniPath::Perform: index {i_suiv}; expected a connexion shorter than RealLast"))
            })?;
            set1.push(chosen);
            set2.remove(i_suiv - 1);
            self.append(&connexion.get(min_on_set1, min_on_set2)?)?;
        }
        self.run_on_connexions()
    }
    // `Append(C)`: `c` joins the connexions from its first line, before the first one
    // after it; it becomes the father of its second line.
    fn append(&mut self, c: &Handle<Mat2dConnexion>) -> Result<(), Failure> {
        let direction: f64 = self.the_direction;
        let seq: &mut Connexions = self.the_connexions.entry(c.index_first_line()).or_default();
        let index_after: Option<usize> = seq.iter().position(|cc| cc.is_after(c, direction));
        match index_after {
            Some(offset) if !seq.is_empty() => {
                seq::insert_before(seq, offset as i32 + 1, c.clone())?
            }
            _ => {
                seq.append(c.clone());
            }
        }
        self.the_father.insert(c.index_second_line(), c.clone());
        Ok(())
    }
    /// `RunOnConnexions()`: the path around the tree from the start line;
    /// Standard_NoSuchObject when no connexion starts there; `path.run_on_connexions()?`.
    pub fn run_on_connexions(&mut self) -> Result<(), Failure> {
        let sc: Connexions = connexions_of(&self.the_connexions, self.ind_start)?.clone();
        self.the_path.clear(None);
        let mut path: Connexions = Sequence::default();
        for c in sc.iter() {
            path.append(c.clone());
            self.explo_sons(&mut path, c)?;
            path.append(c.reverse());
        }
        self.the_path = path;
        Ok(())
    }
    // `ExploSons(CResult, CRef)`: the sons of the second line of `cref`, those after
    // the reverse of `cref` first, then those before it.
    fn explo_sons(
        &self,
        result: &mut Connexions,
        cref: &Handle<Mat2dConnexion>,
    ) -> Result<(), Failure> {
        let Some(sc) = self.the_connexions.get(&cref.index_second_line()) else {
            return Ok(());
        };
        let crr: Handle<Mat2dConnexion> = cref.reverse();
        for c in sc.iter() {
            if c.is_after(&crr, self.the_direction) {
                self.explore(result, c)?;
            }
        }
        for c in sc.iter() {
            if c.is_after(&crr, self.the_direction) {
                break;
            }
            self.explore(result, c)?;
        }
        Ok(())
    }
    // A connexion, its sons, then its reverse.
    fn explore(&self, result: &mut Connexions, c: &Handle<Mat2dConnexion>) -> Result<(), Failure> {
        result.append(c.clone());
        self.explo_sons(result, c)?;
        result.append(c.reverse());
        Ok(())
    }
    /// `Path()`: the connexions of the path; `path.path()`.
    pub fn path(&self) -> &Sequence<Handle<Mat2dConnexion>> {
        &self.the_path
    }
    /// `IsConnexionsFrom(Index)`: true when a connexion starts on the line `index`;
    /// `path.is_connexions_from(1)`.
    pub fn is_connexions_from(&self, index: i32) -> bool {
        self.the_connexions.contains_key(&index)
    }
    /// `ConnexionsFrom(Index)`: the connexions from the line `index`, to edit;
    /// Standard_NoSuchObject when there are none; `path.connexions_from(1)?`.
    pub fn connexions_from(
        &mut self,
        index: i32,
    ) -> Result<&mut Sequence<Handle<Mat2dConnexion>>, Failure> {
        self.the_connexions.get_mut(&index).ok_or_else(|| {
            errors::no_such_object(&format!("MAT2d_MiniPath::ConnexionsFrom: no connexion from line {index}; expected IsConnexionsFrom"))
        })
    }
    /// `IsRoot(Index)`: true for the start line; `path.is_root(1)`.
    pub fn is_root(&self, index: i32) -> bool {
        index == self.ind_start
    }
    /// `Father(Index)`: the connexion that ends on the line `index`;
    /// Standard_NoSuchObject for the root; `path.father(2)?`.
    pub fn father(&self, index: i32) -> Result<Handle<Mat2dConnexion>, Failure> {
        self.the_father.get(&index).cloned().ok_or_else(|| {
            errors::no_such_object(&format!("MAT2d_MiniPath::Father: no connexion ends on line {index}; expected a line that is not the root"))
        })
    }
}
