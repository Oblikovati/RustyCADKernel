// Rust translation of the interference members of TopOpeBRepDS_HDataStructure (the static
// FUN_TopOpeBRepDS_SortOnParameter, SortOnParameter, MinMaxOnParameter, ScanInterfList,
// GetGeometry, StoreInterference(s) and ClearStoreInterferences in
// ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_HDataStructure.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Sorting interferences on their parameters, finding the geometry of a point among
//! interferences, and storing interferences on shapes and on their geometries.
use super::TopOpeBRepDSHDataStructure;
use crate::top_ope_brep_ds::{
    ListOfInterference, TopOpeBRepDSDataStructure, TopOpeBRepDSInterference, TopOpeBRepDSKind,
    TopOpeBRepDSPoint, TopOpeBRepDSPointIterator,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::real::{real_first, real_last};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};

/// Standard_NullObject of a null interference.
fn null_interference() -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        "TopOpeBRepDS_HDataStructure: null interference; expected an interference handle",
        "TopOpeBRepDS_HDataStructure",
    )
}

/// The interference stored on its surface or curve of the data structure (nothing for
/// the other kinds); `store_on_geometry_ds(&mut ds, &i)?`.
pub(crate) fn store_on_geometry_ds(
    ds: &mut TopOpeBRepDSDataStructure,
    i: &Handle<TopOpeBRepDSInterference>,
) -> Result<(), Failure> {
    let ii = i.get().ok_or_else(null_interference)?;
    let (g, gt) = (ii.geometry(), ii.geometry_type());
    match gt {
        TopOpeBRepDSKind::Surface => {
            ds.change_surface_interferences(g).append(i.clone());
        }
        TopOpeBRepDSKind::Curve => {
            ds.change_curve_interferences(g).append(i.clone());
        }
        _ => {}
    }
    Ok(())
}

/// `HDS->StoreInterference(I, S)` on the data structure itself: on a kept shape (and
/// its geometry); Standard_ProgramError for another shape;
/// `store_interference_ds(&mut ds, &i, &face)?`.
pub(crate) fn store_interference_ds(
    ds: &mut TopOpeBRepDSDataStructure,
    i: &Handle<TopOpeBRepDSInterference>,
    s: &Shape,
) -> Result<(), Failure> {
    if !ds.has_shape(s, true) {
        return Err(Failure::with_type(
            "Standard_ProgramError",
            "StoreInterference on shape out of DS",
            "TopOpeBRepDS_HDataStructure",
        ));
    }
    ds.change_shape_interferences(s).append(i.clone());
    store_on_geometry_ds(ds, i)
}

/// `FUN_TopOpeBRepDS_SortOnParameter(List, SList)`: the point and vertex interferences
/// appended by increasing parameter; natively the loop runs once per interference of
/// the list, so each extra round appends the last found again (a null handle when the
/// list has no point); `fun_topopebrepds_sort_on_parameter(&l, &mut sorted)?`.
pub fn fun_topopebrepds_sort_on_parameter(
    list: &ListOfInterference,
    slist: &mut ListOfInterference,
) -> Result<(), Failure> {
    let n_intf = list.len();
    if n_intf == 0 {
        return Ok(());
    }
    let mut taken: Vec<bool> = vec![false; n_intf + 1];
    let (mut intf, mut i_intf): (Handle<TopOpeBRepDSInterference>, usize) = (Handle::default(), 0);
    for _ in 1..=n_intf {
        let mut parmin: f64 = real_last();
        let mut it = TopOpeBRepDSPointIterator::new(list)?;
        let mut itest: usize = 1;
        while it.more() {
            if !taken[itest] {
                let par = it.parameter()?;
                if par < parmin {
                    (parmin, intf, i_intf) = (par, it.value()?.clone(), itest);
                }
            }
            it.next()?;
            itest += 1;
        }
        slist.append(intf.clone());
        taken[i_intf] = true;
    }
    Ok(())
}

impl TopOpeBRepDSHDataStructure {
    /// `SortOnParameter(L1, L2)`: the sorted interferences rotated to start at the first
    /// FORWARD one (seen from IN); `hds.sort_on_parameter(&l, &mut sorted)?`.
    pub fn sort_on_parameter(
        &self,
        list: &ListOfInterference,
        slist: &mut ListOfInterference,
    ) -> Result<(), Failure> {
        fun_topopebrepds_sort_on_parameter(list, slist)?;
        let (mut l1, mut l2) = (ListOfInterference::default(), ListOfInterference::default());
        let mut found = false;
        for i in slist.iter() {
            if !found {
                let o = i
                    .get()
                    .ok_or_else(null_interference)?
                    .transition()
                    .orientation(State::In, ShapeEnum::Face);
                if o == Orientation::Forward {
                    found = true;
                    l1.append(i.clone());
                } else {
                    l2.append(i.clone());
                }
            } else {
                l1.append(i.clone());
            }
        }
        slist.clear();
        for i in l1.iter().chain(l2.iter()) {
            slist.append(i.clone());
        }
        Ok(())
    }

    /// `SortOnParameter(L)`: the list sorted in place when it has a point or vertex
    /// interference; `hds.sort_on_parameter_in_place(&mut l)?`.
    pub fn sort_on_parameter_in_place(&self, list: &mut ListOfInterference) -> Result<(), Failure> {
        if TopOpeBRepDSPointIterator::new(list)?.more() {
            let mut slist = ListOfInterference::default();
            self.sort_on_parameter(list, &mut slist)?;
            *list = slist;
        }
        Ok(())
    }

    /// `MinMaxOnParameter(L, Min, Max)`: the extreme parameters of the point and vertex
    /// interferences (RealLast and RealFirst without one); `None` for an empty list
    /// (the outputs untouched natively); `hds.min_max_on_parameter(&l)?`.
    pub fn min_max_on_parameter(
        &self,
        list: &ListOfInterference,
    ) -> Result<Option<(f64, f64)>, Failure> {
        if list.is_empty() {
            return Ok(None);
        }
        let (mut parmin, mut parmax) = (real_last(), real_first());
        let mut it = TopOpeBRepDSPointIterator::new(list)?;
        while it.more() {
            let parline = it.parameter()?;
            parmin = if parline < parmin { parline } else { parmin };
            parmax = if parmax < parline { parline } else { parmax };
            it.next()?;
        }
        Ok(Some((parmin, parmax)))
    }

    /// `ScanInterfList(IT, PDS)`: from `*position`, the first interference whose point
    /// (or vertex) equals `pds`, the position left on it; `*position` past the end when
    /// none; `hds.scan_interf_list(&l, &mut pos, &p)?`.
    pub fn scan_interf_list(
        &self,
        list: &ListOfInterference,
        position: &mut usize,
        pds: &TopOpeBRepDSPoint,
    ) -> Result<bool, Failure> {
        let items: Vec<&Handle<TopOpeBRepDSInterference>> = list.iter().collect();
        while let Some(i) = items.get(*position) {
            let i = i.get().ok_or_else(null_interference)?;
            let (gt, g) = (i.geometry_type(), i.geometry());
            let other = match gt {
                TopOpeBRepDSKind::Point => Some(*self.ds().point(g)?),
                TopOpeBRepDSKind::Vertex => Some(TopOpeBRepDSPoint::from_shape(
                    &self.ds().shape(g, true).clone(),
                )?),
                _ => None,
            };
            if other.is_some_and(|o| pds.is_equal(&o)) {
                return Ok(true);
            }
            *position += 1;
        }
        Ok(false)
    }

    /// `GetGeometry(IT, PDS, G, K)`: the geometry index and kind of the interference
    /// found by `ScanInterfList`; `hds.get_geometry(&l, &mut pos, &p)?`.
    pub fn get_geometry(
        &self,
        list: &ListOfInterference,
        position: &mut usize,
        pds: &TopOpeBRepDSPoint,
    ) -> Result<Option<(i32, TopOpeBRepDSKind)>, Failure> {
        if !self.scan_interf_list(list, position, pds)? {
            return Ok(None);
        }
        let i = list
            .iter()
            .nth(*position)
            .and_then(|i| i.get())
            .ok_or_else(null_interference)?;
        Ok(Some((i.geometry(), i.geometry_type())))
    }

    /// The interference stored on its surface or curve (nothing for the other kinds).
    fn store_on_geometry(&self, i: &Handle<TopOpeBRepDSInterference>) -> Result<(), Failure> {
        store_on_geometry_ds(&mut self.change_ds(), i)
    }

    /// `StoreInterference(I, LI, str)`: appended to `li`, and stored on its surface or
    /// curve; `hds.store_interference_in(&i, &mut list)?`.
    pub fn store_interference_in(
        &self,
        i: &Handle<TopOpeBRepDSInterference>,
        li: &mut ListOfInterference,
    ) -> Result<(), Failure> {
        li.append(i.clone());
        self.store_on_geometry(i)
    }

    /// `StoreInterference(I, S, str)`: on a kept shape of the structure (and its
    /// geometry); Standard_ProgramError for another shape; `hds.store_interference(&i, &face)?`.
    pub fn store_interference(
        &self,
        i: &Handle<TopOpeBRepDSInterference>,
        s: &Shape,
    ) -> Result<(), Failure> {
        store_interference_ds(&mut self.change_ds(), i, s)
    }

    /// `StoreInterference(I, IS, str)`: on the shape of index `is`;
    /// Standard_ProgramError out of 1..=NbShapes; `hds.store_interference_index(&i, 1)?`.
    pub fn store_interference_index(
        &self,
        i: &Handle<TopOpeBRepDSInterference>,
        is: i32,
    ) -> Result<(), Failure> {
        let n = self.ds().nb_shapes();
        if is < 1 || is > n {
            return Err(Failure::with_type(
                "Standard_ProgramError",
                &format!("StoreInterference on index out of DS: {is}; expected 1..={n}"),
                "TopOpeBRepDS_HDataStructure",
            ));
        }
        self.change_ds()
            .change_shape_interferences_index(is)?
            .append(i.clone());
        self.store_on_geometry(i)
    }

    /// `StoreInterferences(LI, S, str)`: each interference stored on the shape (on the
    /// shared empty list of an unknown shape, as natively); `hds.store_interferences(&l, &face)?`.
    pub fn store_interferences(&self, li: &ListOfInterference, s: &Shape) -> Result<(), Failure> {
        for i in li.iter() {
            self.change_ds()
                .change_shape_interferences(s)
                .append(i.clone());
            self.store_on_geometry(i)?;
        }
        Ok(())
    }

    /// `StoreInterferences(LI, IS, str)`: Standard_OutOfRange for an unknown index;
    /// `hds.store_interferences_index(&l, 1)?`.
    pub fn store_interferences_index(
        &self,
        li: &ListOfInterference,
        is: i32,
    ) -> Result<(), Failure> {
        self.change_ds().change_shape_interferences_index(is)?;
        for i in li.iter() {
            self.change_ds()
                .change_shape_interferences_index(is)?
                .append(i.clone());
            self.store_on_geometry(i)?;
        }
        Ok(())
    }

    /// `ClearStoreInterferences(LI, S, str)`: the interferences of the shape replaced
    /// by those of `li` (also stored on their geometries);
    /// `hds.clear_store_interferences(&l, &face)?`.
    pub fn clear_store_interferences(
        &self,
        li: &ListOfInterference,
        s: &Shape,
    ) -> Result<(), Failure> {
        self.change_ds().change_shape_interferences(s).clear();
        self.store_interferences(li, s)
    }

    /// `ClearStoreInterferences(LI, IS, str)`; `hds.clear_store_interferences_index(&l, 1)?`.
    pub fn clear_store_interferences_index(
        &self,
        li: &ListOfInterference,
        is: i32,
    ) -> Result<(), Failure> {
        self.change_ds()
            .change_shape_interferences_index(is)?
            .clear();
        self.store_interferences_index(li, is)
    }
}
