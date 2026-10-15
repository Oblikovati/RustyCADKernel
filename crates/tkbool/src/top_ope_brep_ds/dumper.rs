// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepDS/TopOpeBRepDS_Dumper.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `TopOpeBRepDS_Dumper`: the texts naming the shapes of the data structure with their
//! rank, keep flag and same domain reference.
use super::export::href;
use super::{
    TopOpeBRepDSHDataStructure, TopOpeBRepDSKind, is_topology, kind_to_shape, shape_to_kind,
    sprint_config, sprint_kind_index,
};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::{Failure, Handle};

/// The dumper of a data structure; `let d = TopOpeBRepDSDumper::new(&hds);`.
pub struct TopOpeBRepDSDumper {
    my_hds: Handle<TopOpeBRepDSHDataStructure>,
}

impl TopOpeBRepDSDumper {
    /// `TopOpeBRepDS_Dumper(HDS)`; `TopOpeBRepDSDumper::new(&hds)`.
    pub fn new(hds: &Handle<TopOpeBRepDSHDataStructure>) -> Self {
        Self {
            my_hds: hds.clone(),
        }
    }

    /// `SDumpRefOri(K, I)`: "(reference,configuration)" of a topological shape of the
    /// kind (empty otherwise); `d.sdump_ref_ori(TopOpeBRepDSKind::Face, 1)?`.
    pub fn sdump_ref_ori(&self, k: TopOpeBRepDSKind, i: i32) -> Result<String, Failure> {
        let h = href(&self.my_hds)?;
        if !is_topology(k) {
            return Ok(String::new());
        }
        let s = h.ds().shape(i, false).clone();
        if s.shape_type()? != kind_to_shape(k) {
            return Ok(String::new());
        }
        let r = h.same_domain_reference(&s);
        let o = h.same_domain_orientation(&s);
        Ok(format!(
            "({},{})",
            self.sprint_shape_index(r)?,
            sprint_config(o)
        ))
    }

    /// `SDumpRefOri(S)`; `d.sdump_ref_ori_of(&face)?`.
    pub fn sdump_ref_ori_of(&self, s: &Shape) -> Result<String, Failure> {
        let k = shape_to_kind(s.shape_type()?)?;
        let i = href(&self.my_hds)?.shape_index(s, false);
        self.sdump_ref_ori(k, i)
    }

    /// `SPrintShape(IS)`: the name of the shape of index `is` (empty out of range);
    /// `d.sprint_shape_index(1)?`.
    pub fn sprint_shape_index(&self, is: i32) -> Result<String, Failure> {
        let s = {
            let ds = href(&self.my_hds)?.ds();
            if is < 1 || is > ds.nb_shapes() {
                return Ok(String::new());
            }
            ds.shape(is, true).clone()
        };
        self.sprint_shape(&s)
    }

    /// `SPrintShape(S)`: the kind and index, marked `*` (kept) or `~` before for rank 1
    /// and after for rank 2; `d.sprint_shape(&face)?`.
    pub fn sprint_shape(&self, s: &Shape) -> Result<String, Failure> {
        let ds = href(&self.my_hds)?.ds();
        let is = ds.shape_index(s, true);
        let rank_is = ds.ancestor_rank_index(is);
        let mark = if ds.keep_shape_index(is, true) {
            "*"
        } else {
            "~"
        };
        let s1 = if rank_is == 1 { mark } else { "" };
        let s2 = if rank_is == 2 { mark } else { "" };
        Ok(sprint_kind_index(
            shape_to_kind(s.shape_type()?)?,
            is,
            s1,
            s2,
        ))
    }

    /// `SPrintShapeRefOri(S, astr)`: `astr`, the name and the reference;
    /// `d.sprint_shape_ref_ori(&face, "")?`.
    pub fn sprint_shape_ref_ori(&self, s: &Shape, astr: &str) -> Result<String, Failure> {
        Ok(format!(
            "{astr}{} {}",
            self.sprint_shape(s)?,
            self.sdump_ref_ori_of(s)?
        ))
    }

    /// `SPrintShapeRefOri(L, astr)`: one line per shape, the first after `astr`, the
    /// others indented as long; `d.sprint_shape_ref_ori_list(&l, "")?`.
    pub fn sprint_shape_ref_ori_list(
        &self,
        l: &List<Shape>,
        astr: &str,
    ) -> Result<String, Failure> {
        if l.is_empty() {
            return Ok(String::new());
        }
        let bst = " ".repeat(astr.len());
        let mut ss = astr.to_string();
        for (il, s) in l.iter().enumerate() {
            let line = self.sprint_shape_ref_ori(s, "")?;
            if il > 0 {
                ss.push_str(&bst);
            }
            ss.push_str(&line);
            ss.push('\n');
        }
        Ok(ss)
    }
}
