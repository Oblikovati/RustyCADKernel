// Rust translation of GeomPlate_BuildAveragePlane::HalfSpace
// (ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_BuildAveragePlane.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_BuildAveragePlane::HalfSpace`: whether a set of normals lies in one
//! half space, maintained incrementally with the edges (`GeomPlate_Aij`) of its cone.
use super::aij::GeomPlateAij;
use super::build_average_plane::GeomPlateBuildAveragePlane;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::Vector3;

/// The outcome of matching a new cone edge against the kept edges.
enum EdgeMatch {
    /// No kept edge is opposite: the edge is new.
    New,
    /// A kept edge is opposite and was updated (or already uses the new normal).
    Known,
    /// The normals are not in a half space.
    Reject,
}

/// The working state of one `HalfSpace` call.
struct HalfSpaceRun<'a> {
    normals: &'a mut Sequence<Vector3>,
    bset: &'a mut Sequence<GeomPlateAij>,
    save_normals: Sequence<Vector3>,
    save_bset: Sequence<GeomPlateAij>,
    lin_tol: f64,
    ang_tol: f64,
    square_tol: f64,
}

fn at<T: Copy>(seq: &Sequence<T>, index: i32) -> Result<T, Failure> {
    seq.value(index.max(0) as usize).copied()
}

impl HalfSpaceRun<'_> {
    /// Restores the input sequences and answers false.
    fn reject(&mut self) -> bool {
        *self.normals = self.save_normals.clone();
        *self.bset = self.save_bset.clone();
        false
    }

    /// Matches the cone edge `cross` of the new normal `new_normal` (index `ii`)
    /// against the kept edges `b2set` (the native loop over `k`).
    fn match_edge(
        &self,
        b2set: &mut Sequence<GeomPlateAij>,
        cross: Vector3,
        new_normal: Vector3,
        ii: i32,
    ) -> Result<EdgeMatch, Failure> {
        for k in 1..=b2set.len() {
            let edge: GeomPlateAij = *b2set.value(k)?;
            if !edge.vec.is_opposite(-cross, self.ang_tol)? {
                continue;
            }
            let (ind1, ind2) = (edge.ind1, edge.ind2);
            if ind1 == ii || ind2 == ii {
                return Ok(EdgeMatch::Known);
            }
            let (n1, n2) = (at(self.normals, ind1)?, at(self.normals, ind2)?);
            let (cross1, mut cross2) = (n1 ^ new_normal, n2 ^ new_normal);
            if cross1.square_magnitude() <= self.square_tol
                || cross2.square_magnitude() <= self.square_tol
            {
                return Ok(EdgeMatch::Reject);
            }
            if cross1.is_opposite(cross2, self.ang_tol)? {
                cross2 = n1 ^ n2;
                if cross1.is_opposite(cross2, self.ang_tol)? {
                    return Ok(EdgeMatch::Reject);
                }
            } else {
                let slot = b2set.change_value(k)?;
                if new_normal.angle(n1)? > new_normal.angle(n2)? {
                    slot.ind2 = ind1;
                }
                slot.ind1 = ii;
            }
            return Ok(EdgeMatch::Known);
        }
        Ok(EdgeMatch::New)
    }

    /// Starts from the first two new normals when no normal is kept yet; `Some`
    /// answers the call.
    fn start(&mut self, new_normals: &Sequence<Vector3>) -> Result<Option<bool>, Failure> {
        if new_normals.length() == 1 {
            self.normals.append(at(new_normals, new_normals.length())?);
            return Ok(Some(true));
        }
        let (first, second) = (at(new_normals, 1)?, at(new_normals, 2)?);
        let mut cross: Vector3 = first ^ second;
        if cross.square_magnitude() <= self.square_tol {
            return Ok(Some(false));
        }
        cross.normalize()?;
        self.bset.append(GeomPlateAij::with_values(1, 2, cross));
        self.bset.append(GeomPlateAij::with_values(2, 1, -cross));
        self.normals.append(first);
        self.normals.append(second);
        Ok(None)
    }

    /// Adds the new normal `new_normal`; false (after the restore) when the normals
    /// leave a half space.
    fn add(&mut self, new_normal: Vector3) -> Result<bool, Failure> {
        let mut b2set: Sequence<GeomPlateAij> = Sequence::default();
        for edge in self.bset.iter() {
            if edge.vec * new_normal >= -self.lin_tol {
                b2set.append(*edge);
            }
        }
        let Some(b1set) = self.new_edges(&mut b2set, new_normal)? else {
            return Ok(self.reject());
        };
        self.keep_supporting(&b1set, &mut b2set)?;
        if b2set.is_empty() {
            return Ok(self.reject());
        }
        *self.bset = b2set;
        self.normals.append(new_normal);
        self.null_unused()?;
        Ok(true)
    }

    /// The cone edges between the new normal and each kept normal (steps 4 of the
    /// native code); `None` when the normals leave a half space.
    fn new_edges(
        &mut self,
        b2set: &mut Sequence<GeomPlateAij>,
        new_normal: Vector3,
    ) -> Result<Option<Sequence<GeomPlateAij>>, Failure> {
        let mut b1set: Sequence<GeomPlateAij> = Sequence::default();
        let ii: i32 = self.normals.length() + 1;
        for j in 1..ii {
            let normal: Vector3 = at(self.normals, j)?;
            if normal.square_magnitude() == 0.0 {
                continue;
            }
            let mut cross: Vector3 = new_normal ^ normal;
            if cross.square_magnitude() <= self.square_tol {
                return Ok(None);
            }
            cross.normalize()?;
            for _side in 0..2 {
                match self.match_edge(b2set, cross, new_normal, ii)? {
                    EdgeMatch::Reject => return Ok(None),
                    EdgeMatch::New => {
                        b1set.append(GeomPlateAij::with_values(ii, j, cross));
                    }
                    EdgeMatch::Known => {}
                }
                cross.reverse();
            }
        }
        Ok(Some(b1set))
    }

    /// Step 5: keeps the new edges on the non-negative side of every non-null normal.
    fn keep_supporting(
        &self,
        b1set: &Sequence<GeomPlateAij>,
        b2set: &mut Sequence<GeomPlateAij>,
    ) -> Result<(), Failure> {
        for edge in b1set.iter() {
            let supporting = self
                .normals
                .iter()
                .filter(|n| n.square_magnitude() != 0.0)
                .all(|n| edge.vec * *n >= -self.lin_tol);
            if supporting {
                b2set.append(*edge);
            }
        }
        Ok(())
    }

    /// Step 8: nulls the normals that no kept edge uses.
    fn null_unused(&mut self) -> Result<(), Failure> {
        for j in 1..=self.normals.length() {
            if at(self.normals, j)?.square_magnitude() == 0.0 {
                continue;
            }
            let used = self
                .bset
                .iter()
                .any(|edge| edge.ind1 == j || edge.ind2 == j);
            if !used {
                self.normals
                    .set_value(j as usize, Vector3::new(0.0, 0.0, 0.0))?;
            }
        }
        Ok(())
    }
}

impl GeomPlateBuildAveragePlane {
    /// Adds `new_normals` to `normals` while all of them lie in one half space
    /// (`HalfSpace`): `bset` holds the edges of the cone of the normals, and a normal
    /// inside the cone is nulled. False, with `normals` and `bset` restored, as soon as
    /// two normals are parallel within `lin_tol` or the half space breaks.
    /// Standard_OutOfRange for an empty `new_normals` with no normal kept.
    /// `GeomPlateBuildAveragePlane::half_space(&new, &mut normals, &mut bset, 1e-3, 1e-3)?`.
    pub fn half_space(
        new_normals: &Sequence<Vector3>,
        normals: &mut Sequence<Vector3>,
        bset: &mut Sequence<GeomPlateAij>,
        lin_tol: f64,
        ang_tol: f64,
    ) -> Result<bool, Failure> {
        let mut run = HalfSpaceRun {
            save_normals: normals.clone(),
            save_bset: bset.clone(),
            normals,
            bset,
            lin_tol,
            ang_tol,
            square_tol: lin_tol * lin_tol,
        };
        let mut i: i32 = 1;
        if run.normals.is_empty() {
            if let Some(answer) = run.start(new_normals)? {
                return Ok(answer);
            }
            i = 3;
        }
        for index in i..=new_normals.length() {
            if !run.add(at(new_normals, index)?)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
