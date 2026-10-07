//! The radius of a fillet guideline as `(abscissa, radius)` pairs.
use super::{FilSpine, not_constant};
use rustycad_tkbrep::topods::{Edge, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkgeomalgo::law::{LawComposite, LawFunction};
use rustycad_tkmath::gp::{Xy, resolution};

impl FilSpine {
    /// `SetRadius(Radius, E)`: a constant `radius` on `edge` (at both its ends);
    /// `spine.set_radius_on_edge(2.0, &edge)?`.
    pub fn set_radius_on_edge(&mut self, radius: f64, edge: &Edge) -> Result<(), Failure> {
        self.spine.set_split_done(false);
        let index = self.spine.index_of_edge(edge);
        self.set_radius_at(Xy::new(0., radius), index)?;
        self.set_radius_at(Xy::new(1., radius), index)
    }

    /// `UnSetRadius(E)`: removes the pairs from the start to the end of `edge`, when
    /// both ends hold a pair; `spine.unset_radius_on_edge(&edge)?`.
    pub fn unset_radius_on_edge(&mut self, edge: &Edge) -> Result<(), Failure> {
        self.spine.set_split_done(false);
        let index = self.spine.index_of_edge(edge);
        let first = self.spine.first_parameter_of(index)?;
        let last = self.spine.last_parameter_of(index)?;
        let (mut i_first, mut i_last) = (0, 0);
        for (offset, pair) in self.par_and_rad.iter().enumerate() {
            if (pair.x() - first).abs() <= resolution() {
                i_first = offset + 1;
            }
            if (pair.x() - last).abs() <= resolution() {
                i_last = offset + 1;
            }
        }
        if i_first != 0 && i_last != 0 {
            remove_range(&mut self.par_and_rad, i_first, i_last)?;
        }
        Ok(())
    }

    /// `SetRadius(Radius, V)`: the radius at the abscissa of `vertex`;
    /// `spine.set_radius_on_vertex(2.0, &vertex)?`.
    pub fn set_radius_on_vertex(&mut self, radius: f64, vertex: &Vertex) -> Result<(), Failure> {
        let abscissa = self.spine.absc_of_vertex(vertex)?;
        self.set_radius_at(Xy::new(abscissa, radius), 0)
    }

    /// `UnSetRadius(V)`: removes the first pair at exactly the abscissa of `vertex`;
    /// `spine.unset_radius_on_vertex(&vertex)?`.
    pub fn unset_radius_on_vertex(&mut self, vertex: &Vertex) -> Result<(), Failure> {
        let abscissa = self.spine.absc_of_vertex(vertex)?;
        if let Some(offset) = self
            .par_and_rad
            .iter()
            .position(|pair| pair.x() == abscissa)
        {
            self.par_and_rad.remove(offset);
        }
        Ok(())
    }

    /// `SetRadius(Radius)`: a constant `radius` on the whole guideline;
    /// `spine.set_radius(2.0)?`.
    pub fn set_radius(&mut self, radius: f64) -> Result<(), Failure> {
        self.par_and_rad.clear();
        let first = Xy::new(self.spine.first_parameter(), radius);
        let last = Xy::new(self.spine.last_parameter()?, radius);
        self.set_radius_at(first, 0)?;
        self.set_radius_at(last, 0)
    }

    /// `SetRadius(UandR, IinC)`: the radius `u_and_r.y()` at the abscissa `u_and_r.x()`
    /// (`index_in_curve == 0`) or at the relative position `u_and_r.x()` of the
    /// `index_in_curve`-th edge, kept sorted; once the guideline is split, the law of the
    /// elementary spine holding the abscissa is computed again; `spine.set_radius_at(uv, 0)?`.
    pub fn set_radius_at(&mut self, u_and_r: Xy, index_in_curve: i32) -> Result<(), Failure> {
        let w = if index_in_curve == 0 {
            u_and_r.x()
        } else {
            let first = self.spine.first_parameter_of(index_in_curve)?;
            let last = self.spine.last_parameter_of(index_in_curve)?;
            first + u_and_r.x() * (last - first)
        };
        if !self.insert_pair(Xy::new(w, u_and_r.y())) {
            self.par_and_rad.push(Xy::new(w, u_and_r.y()));
        }
        if !self.spine.split_done() {
            return Ok(());
        }
        self.recompute_law_at(w)
    }

    /// The loop of `SetRadius(UandR, IinC)`: replaces the radius of a pair at exactly
    /// `pair.x()`, or inserts `pair` before the first larger abscissa; false when every
    /// abscissa is smaller (the pair is then appended).
    fn insert_pair(&mut self, pair: Xy) -> bool {
        for offset in 0..self.par_and_rad.len() {
            let current = self.par_and_rad[offset].x();
            if current == pair.x() {
                self.par_and_rad[offset].set_y(pair.y());
                return true;
            }
            if current > pair.x() {
                self.par_and_rad.insert(offset, pair);
                return true;
            }
        }
        false
    }

    /// `SetRadius(C, IinC)`: natively the law `C` is put into a composite that is then
    /// dropped, so only the split flag and the pairs are reset (the law is lost, a native
    /// quirk kept as is); `spine.set_radius_law(&law, 0)`.
    pub fn set_radius_law(&mut self, law: &Handle<dyn LawFunction>, _index_in_curve: i32) {
        self.spine.set_split_done(false);
        let discarded = LawComposite::new();
        discarded.change_laws(|laws| {
            laws.append(law.clone());
        });
        self.par_and_rad.clear();
    }

    /// `IsConstant()`: whether every radius is within Precision::Confusion() of the first
    /// (false without radius); `spine.is_constant()`.
    pub fn is_constant(&self) -> bool {
        let Some(first) = self.par_and_rad.first() else {
            return false;
        };
        self.par_and_rad[1..]
            .iter()
            .all(|pair| (first.y() - pair.y()).abs() <= precision::confusion())
    }

    /// `IsConstant(IE)`: whether the radius is constant along the `index`-th edge;
    /// `spine.is_constant_on_edge(1)?`.
    pub fn is_constant_on_edge(&self, index: i32) -> Result<bool, Failure> {
        Ok(self.edge_radius(index)?.is_some())
    }

    /// `Radius(IE)`: the constant radius of the `index`-th edge; Standard_DomainError
    /// "Edge is not constant" when it varies; `spine.radius_on_edge_index(1)?`.
    pub fn radius_on_edge_index(&self, index: i32) -> Result<f64, Failure> {
        self.edge_radius(index)?
            .ok_or_else(|| not_constant("Edge is not constant"))
    }

    /// `Radius(E)`: the constant radius of `edge`; `spine.radius_on_edge(&edge)?`.
    pub fn radius_on_edge(&self, edge: &Edge) -> Result<f64, Failure> {
        self.radius_on_edge_index(self.spine.index_of_edge(edge))
    }

    /// `Radius()`: the radius of a constant guideline; Standard_DomainError "Spine is not
    /// constant" otherwise; `spine.radius()?`.
    pub fn radius(&self) -> Result<f64, Failure> {
        if !self.is_constant() {
            return Err(not_constant("Spine is not constant"));
        }
        Ok(self.par_and_rad[0].y())
    }

    /// The shared body of `IsConstant(IE)` and `Radius(IE)`: the start radius of the
    /// edge when the radius stays within Precision::Confusion() of it up to the end.
    fn edge_radius(&self, index: i32) -> Result<Option<f64>, Failure> {
        let first = self.spine.first_parameter_of(index)?;
        let last = self.spine.last_parameter_of(index)?;
        let pairs = &self.par_and_rad;
        let (mut start, mut i) = (0.0, 1);
        while i < pairs.len() {
            let (par, next) = (pairs[i - 1].x(), pairs[i].x());
            let on_start = (first - par).abs() <= resolution();
            if on_start || (par < first && first < next && next - first > resolution()) {
                start = pairs[i - 1].y();
                break;
            }
            i += 1;
        }
        Ok(radius_until(&pairs[i.min(pairs.len())..], start, last))
    }
}

/// The second loop of `IsConstant(IE)` / `Radius(IE)` over the pairs after the start:
/// `Some(start)` when no radius differs before the end `last` of the edge.
fn radius_until(pairs: &[Xy], start: f64, last: f64) -> Option<f64> {
    for pair in pairs {
        if (pair.y() - start).abs() > precision::confusion() {
            return None;
        }
        if (last - pair.x()).abs() <= resolution() || pair.x() > last {
            return Some(start);
        }
    }
    Some(start)
}

/// `NCollection_Sequence::Remove(from, to)` on 1-based indices: Standard_OutOfRange
/// unless `1 <= from <= to <= length`.
fn remove_range(pairs: &mut Vec<Xy>, from: usize, to: usize) -> Result<(), Failure> {
    if from > to || to > pairs.len() {
        return Err(Failure::out_of_range(&format!(
            "ChFiDS_FilSpine::UnSetRadius: range [{from}, {to}]; expected 1 <= from <= to <= {}",
            pairs.len()
        )));
    }
    pairs.drain(from - 1..to);
    Ok(())
}
