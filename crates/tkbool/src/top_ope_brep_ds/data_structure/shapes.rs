// Rust translation of the shape members of TopOpeBRepDS_DataStructure (AddShape, the keep
// flags, the section edges, the shape interferences, the same domain lists and the same
// domain reference, orientation, index and ancestor rank in
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_DataStructure.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The shapes of the data structure and their data. The index members ignore an index
//! out of range and the shape members a null or unknown shape, as natively.
use super::TopOpeBRepDSDataStructure;
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSConfig, TopOpeBRepDSInterference, TopOpeBRepDSShapeData,
};
use rustycad_tkbrep::topods::{Edge, Shape};
use rustycad_tkbrep::toptools::ShapeMapHasher;
use rustycad_tkernel::ncollection::{IndexedDataMap, List};
use rustycad_tkernel::standard::{Failure, Handle};

impl TopOpeBRepDSDataStructure {
    /// The data of the shape of index `i`, when in range.
    fn shape_data_index(&self, i: i32) -> Option<&TopOpeBRepDSShapeData> {
        let n = self.my_shapes.len() as i32;
        if !(1..=n).contains(&i) {
            return None;
        }
        self.my_shapes.find_from_index(i as usize).ok()
    }

    /// The mutable data of the shape of index `i`, when in range.
    fn shape_data_index_mut(&mut self, i: i32) -> Option<&mut TopOpeBRepDSShapeData> {
        let n = self.my_shapes.len() as i32;
        if !(1..=n).contains(&i) {
            return None;
        }
        self.my_shapes.change_from_index(i as usize).ok()
    }

    /// The data of a non-null known shape.
    fn shape_data(&self, s: &Shape) -> Option<&TopOpeBRepDSShapeData> {
        if s.is_null() {
            return None;
        }
        self.my_shapes.seek(s)
    }

    /// The mutable data of a non-null known shape.
    fn shape_data_mut(&mut self, s: &Shape) -> Option<&mut TopOpeBRepDSShapeData> {
        if s.is_null() {
            return None;
        }
        self.my_shapes.change_seek(s)
    }

    /// `AddShape(S)`: the index of the shape, added as its own same oriented reference
    /// when new; `ds.add_shape(&face)`.
    pub fn add_shape(&mut self, s: &Shape) -> i32 {
        let mut i_s = self.my_shapes.find_index(s) as i32;
        if i_s == 0 {
            i_s = self
                .my_shapes
                .add(s.clone(), TopOpeBRepDSShapeData::default()) as i32;
            self.set_same_domain_ref_index(i_s, i_s);
            self.set_same_domain_ori_index(i_s, TopOpeBRepDSConfig::SameOriented);
        }
        i_s
    }

    /// `AddShape(S, Ianc)`: `AddShape(S)` with the ancestor rank of a new shape;
    /// `ds.add_shape_with_rank(&face, 1)`.
    pub fn add_shape_with_rank(&mut self, s: &Shape, ianc: i32) -> i32 {
        let mut i_s = self.my_shapes.find_index(s) as i32;
        if i_s == 0 {
            i_s = self.add_shape(s);
            self.set_ancestor_rank_index(i_s, ianc);
        }
        i_s
    }

    /// `KeepShape(I, FindKeep)` (FindKeep natively defaulted to true): a shape in range,
    /// kept when `find_keep`; `ds.keep_shape_index(1, true)`.
    pub fn keep_shape_index(&self, i: i32, find_keep: bool) -> bool {
        let n = self.my_shapes.len() as i32;
        if !(1..=n).contains(&i) {
            return false;
        }
        match self.my_shapes.find_key(i as usize) {
            Ok(s) if find_keep => self.keep_shape(s, true),
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// `KeepShape(S, FindKeep)`: a known non-null shape, kept when `find_keep`;
    /// `ds.keep_shape(&face, true)`.
    pub fn keep_shape(&self, s: &Shape, find_keep: bool) -> bool {
        self.shape_data(s).is_some_and(|sd| !find_keep || sd.keep())
    }

    /// `ChangeKeepShape(I, FindKeep)`; `ds.change_keep_shape_index(1, false)`.
    pub fn change_keep_shape_index(&mut self, i: i32, find_keep: bool) {
        if let Some(sd) = self.shape_data_index_mut(i) {
            sd.change_keep(find_keep);
        }
    }

    /// `ChangeKeepShape(S, FindKeep)`; `ds.change_keep_shape(&face, false)`.
    pub fn change_keep_shape(&mut self, s: &Shape, find_keep: bool) {
        if let Some(sd) = self.shape_data_mut(s) {
            sd.change_keep(find_keep);
        }
    }

    /// `InitSectionEdges()`; `ds.init_section_edges()`.
    pub fn init_section_edges(&mut self) {
        self.my_section_edges.clear(false);
    }

    /// `AddSectionEdge(E)`: its section index; `ds.add_section_edge(&e)`.
    pub fn add_section_edge(&mut self, e: &Edge) -> i32 {
        let i_e = self.my_section_edges.find_index(e.as_ref());
        if i_e != 0 {
            return i_e as i32;
        }
        self.my_section_edges.add(e.as_ref().clone()) as i32
    }

    /// `ChangeShapes()`; `ds.change_shapes()`.
    pub fn change_shapes(
        &mut self,
    ) -> &mut IndexedDataMap<Shape, TopOpeBRepDSShapeData, ShapeMapHasher> {
        &mut self.my_shapes
    }

    /// `ShapeInterferences(S, FindKeep)` (FindKeep natively defaulted to true): the
    /// interferences of a kept shape, else the shared empty list;
    /// `ds.shape_interferences(&face, true)`.
    pub fn shape_interferences(&self, s: &Shape, find_keep: bool) -> &ListOfInterference {
        if self.has_shape(s, find_keep)
            && let Some(sd) = self.my_shapes.seek(s)
        {
            return &sd.my_interferences;
        }
        &self.my_empty_list_of_interference
    }

    /// `ChangeShapeInterferences(S)`: the interferences of a kept shape, else the shared
    /// empty list (what is appended to it stays visible, as natively);
    /// `ds.change_shape_interferences(&face).append(i)`.
    pub fn change_shape_interferences(&mut self, s: &Shape) -> &mut ListOfInterference {
        if !self.has_shape(s, true) {
            return &mut self.my_empty_list_of_interference;
        }
        match self.my_shapes.change_seek(s) {
            Some(sd) => &mut sd.my_interferences,
            None => &mut self.my_empty_list_of_interference,
        }
    }

    /// `ShapeInterferences(I, FindKeep)`: the interferences of the shape of index `i`;
    /// the shared empty list when it is not kept; Standard_OutOfRange out of range
    /// without `find_keep`; `ds.shape_interferences_index(1, true)?`.
    pub fn shape_interferences_index(
        &self,
        i: i32,
        find_keep: bool,
    ) -> Result<&ListOfInterference, Failure> {
        if find_keep && !self.keep_shape_index(i, true) {
            return Ok(&self.my_empty_list_of_interference);
        }
        Ok(&self
            .my_shapes
            .find_from_index(usize::try_from(i).unwrap_or(0))?
            .my_interferences)
    }

    /// `ChangeShapeInterferences(I)`; Standard_OutOfRange out of range;
    /// `ds.change_shape_interferences_index(1)?.append(i)`.
    pub fn change_shape_interferences_index(
        &mut self,
        i: i32,
    ) -> Result<&mut ListOfInterference, Failure> {
        Ok(&mut self
            .my_shapes
            .change_from_index(usize::try_from(i).unwrap_or(0))?
            .my_interferences)
    }

    /// `ShapeSameDomain(S)`: the same domain shapes, empty for a null or unknown shape;
    /// `ds.shape_same_domain(&face)`.
    pub fn shape_same_domain(&self, s: &Shape) -> &List<Shape> {
        self.shape_data(s)
            .map_or(&self.my_empty_list_of_shape, |sd| &sd.my_same_domain)
    }

    /// `ChangeShapeSameDomain(S)`: Standard_NoSuchObject for an unknown shape;
    /// `ds.change_shape_same_domain(&face)?.append(other)`.
    pub fn change_shape_same_domain(&mut self, s: &Shape) -> Result<&mut List<Shape>, Failure> {
        Ok(&mut self.my_shapes.change_from_key(s)?.my_same_domain)
    }

    /// `ShapeSameDomain(I)`: empty out of range; `ds.shape_same_domain_index(1)`.
    pub fn shape_same_domain_index(&self, i: i32) -> &List<Shape> {
        self.shape_data_index(i)
            .map_or(&self.my_empty_list_of_shape, |sd| &sd.my_same_domain)
    }

    /// `ChangeShapeSameDomain(I)`: Standard_OutOfRange out of range;
    /// `ds.change_shape_same_domain_index(1)?`.
    pub fn change_shape_same_domain_index(&mut self, i: i32) -> Result<&mut List<Shape>, Failure> {
        Ok(&mut self
            .my_shapes
            .change_from_index(usize::try_from(i).unwrap_or(0))?
            .my_same_domain)
    }

    /// `AddShapeSameDomain(S, SSD)`: appended unless a same shape is there;
    /// `ds.add_shape_same_domain(&f1, &f2)?`.
    pub fn add_shape_same_domain(&mut self, s: &Shape, ssd: &Shape) -> Result<(), Failure> {
        if self.shape_same_domain(s).iter().any(|its| its.is_same(ssd)) {
            return Ok(());
        }
        self.change_shape_same_domain(s)?.append(ssd.clone());
        Ok(())
    }

    /// `RemoveShapeSameDomain(S, SSD)`: every same shape removed;
    /// `ds.remove_shape_same_domain(&f1, &f2)?`.
    pub fn remove_shape_same_domain(&mut self, s: &Shape, ssd: &Shape) -> Result<(), Failure> {
        self.change_shape_same_domain(s)?
            .retain(|its| !its.is_same(ssd));
        Ok(())
    }

    /// `SameDomainRef(I)`: 0 out of range; `ds.same_domain_ref_index(1)`.
    pub fn same_domain_ref_index(&self, i: i32) -> i32 {
        self.shape_data_index(i)
            .map_or(0, |sd| sd.my_same_domain_ref)
    }

    /// `SameDomainRef(S)`; `ds.same_domain_ref(&face)`.
    pub fn same_domain_ref(&self, s: &Shape) -> i32 {
        self.shape_data(s).map_or(0, |sd| sd.my_same_domain_ref)
    }

    /// `SameDomainRef(I, Ref)`; `ds.set_same_domain_ref_index(1, 2)`.
    pub fn set_same_domain_ref_index(&mut self, i: i32, r: i32) {
        if let Some(sd) = self.shape_data_index_mut(i) {
            sd.my_same_domain_ref = r;
        }
    }

    /// `SameDomainRef(S, Ref)`; `ds.set_same_domain_ref(&face, 2)`.
    pub fn set_same_domain_ref(&mut self, s: &Shape, r: i32) {
        if let Some(sd) = self.shape_data_mut(s) {
            sd.my_same_domain_ref = r;
        }
    }

    /// `SameDomainOri(I)`: UNSHGEOMETRY out of range; `ds.same_domain_ori_index(1)`.
    pub fn same_domain_ori_index(&self, i: i32) -> TopOpeBRepDSConfig {
        self.shape_data_index(i)
            .map_or(TopOpeBRepDSConfig::UnshGeometry, |sd| sd.my_same_domain_ori)
    }

    /// `SameDomainOri(S)`; `ds.same_domain_ori(&face)`.
    pub fn same_domain_ori(&self, s: &Shape) -> TopOpeBRepDSConfig {
        self.shape_data(s)
            .map_or(TopOpeBRepDSConfig::UnshGeometry, |sd| sd.my_same_domain_ori)
    }

    /// `SameDomainOri(I, Ori)`; `ds.set_same_domain_ori_index(1, TopOpeBRepDSConfig::DiffOriented)`.
    pub fn set_same_domain_ori_index(&mut self, i: i32, ori: TopOpeBRepDSConfig) {
        if let Some(sd) = self.shape_data_index_mut(i) {
            sd.my_same_domain_ori = ori;
        }
    }

    /// `SameDomainOri(S, Ori)`; `ds.set_same_domain_ori(&face, TopOpeBRepDSConfig::DiffOriented)`.
    pub fn set_same_domain_ori(&mut self, s: &Shape, ori: TopOpeBRepDSConfig) {
        if let Some(sd) = self.shape_data_mut(s) {
            sd.my_same_domain_ori = ori;
        }
    }

    /// `SameDomainInd(I)`: 0 out of range; `ds.same_domain_ind_index(1)`.
    pub fn same_domain_ind_index(&self, i: i32) -> i32 {
        self.shape_data_index(i)
            .map_or(0, |sd| sd.my_same_domain_ind)
    }

    /// `SameDomainInd(S)`; `ds.same_domain_ind(&face)`.
    pub fn same_domain_ind(&self, s: &Shape) -> i32 {
        self.shape_data(s).map_or(0, |sd| sd.my_same_domain_ind)
    }

    /// `SameDomainInd(I, Ind)`; `ds.set_same_domain_ind_index(1, 2)`.
    pub fn set_same_domain_ind_index(&mut self, i: i32, ind: i32) {
        if let Some(sd) = self.shape_data_index_mut(i) {
            sd.my_same_domain_ind = ind;
        }
    }

    /// `SameDomainInd(S, Ind)`; `ds.set_same_domain_ind(&face, 2)`.
    pub fn set_same_domain_ind(&mut self, s: &Shape, ind: i32) {
        if let Some(sd) = self.shape_data_mut(s) {
            sd.my_same_domain_ind = ind;
        }
    }

    /// `AncestorRank(I)`: 0 out of range; `ds.ancestor_rank_index(1)`.
    pub fn ancestor_rank_index(&self, i: i32) -> i32 {
        self.shape_data_index(i).map_or(0, |sd| sd.my_ancestor_rank)
    }

    /// `AncestorRank(S)`; `ds.ancestor_rank(&face)`.
    pub fn ancestor_rank(&self, s: &Shape) -> i32 {
        self.shape_data(s).map_or(0, |sd| sd.my_ancestor_rank)
    }

    /// `AncestorRank(I, Ianc)`; `ds.set_ancestor_rank_index(1, 2)`.
    pub fn set_ancestor_rank_index(&mut self, i: i32, ianc: i32) {
        if let Some(sd) = self.shape_data_index_mut(i) {
            sd.my_ancestor_rank = ianc;
        }
    }

    /// `AncestorRank(S, Ianc)`; `ds.set_ancestor_rank(&face, 2)`.
    pub fn set_ancestor_rank(&mut self, s: &Shape, ianc: i32) {
        if let Some(sd) = self.shape_data_mut(s) {
            sd.my_ancestor_rank = ianc;
        }
    }

    /// `AddShapeInterference(S, I)`: appended to the interferences of the shape (to the
    /// shared empty list for an unknown shape, as natively);
    /// `ds.add_shape_interference(&face, &i)`.
    pub fn add_shape_interference(&mut self, s: &Shape, i: &Handle<TopOpeBRepDSInterference>) {
        self.change_shape_interferences(s).append(i.clone());
    }
}
