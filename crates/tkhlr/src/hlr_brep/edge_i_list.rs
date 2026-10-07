//! HLRBRep_EdgeIList.hxx/.cxx, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::EdgeInterferenceTool;
use crate::hlr_algo::Interference;
use crate::top_cnx::EdgeFaceTransition;
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;

/// HLRBRep_EdgeIList: the sorted lists of interferences of an edge: insertion by
/// parameter and the merge of the interferences at one vertex into one complex
/// transition. The members work on `List<Interference>` as the native ones; the
/// Hider keeps its lists as vectors and uses the `_in` forms.
///
/// ```
/// use rustycad_tkernel::ncollection::List;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkhlr::hlr_algo::Interference;
/// use rustycad_tkhlr::hlr_brep::{Data, EdgeIList, EdgeInterferenceTool};
/// let tool = EdgeInterferenceTool::new(&Handle::new(Data::new(0, 0, 0)?));
/// let mut list = List::default();
/// let mut late = Interference::new();
/// late.change_intersection().set_parameter(2.0);
/// EdgeIList::add_interference(&mut list, &late, &tool);
/// EdgeIList::add_interference(&mut list, &Interference::new(), &tool);
/// assert_eq!(list.first().unwrap().intersection().parameter(), 0.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub struct EdgeIList;

// The angular tolerance of the complex transitions.
const TOL_ANG: f64 = 0.0001;

impl EdgeIList {
    /// AddInterference(IL, I, T): inserts `i` before the first interference of
    /// greater parameter; `EdgeIList::add_interference(&mut list, &i, &tool)`.
    pub fn add_interference(
        il: &mut List<Interference>,
        i: &Interference,
        t: &EdgeInterferenceTool,
    ) {
        let mut v: Vec<Interference> = il.iter().copied().collect();
        Self::add_interference_in(&mut v, i, t);
        *il = v.into_iter().collect();
    }

    /// AddInterference on a vector; `EdgeIList::add_interference_in(&mut v, &i, &tool)`.
    pub fn add_interference_in(
        il: &mut Vec<Interference>,
        i: &Interference,
        t: &EdgeInterferenceTool,
    ) {
        let p = t.parameter_of_interference(i);
        let at = il
            .iter()
            .position(|x| p < t.parameter_of_interference(x))
            .unwrap_or(il.len());
        il.insert(at, *i);
    }

    /// ProcessComplex(IL, T): merges each run of interferences at one vertex into
    /// the first of the run, with the cumulated transitions of the edge through the
    /// face edges there; `EdgeIList::process_complex(&mut list, &tool)?`.
    pub fn process_complex(
        il: &mut List<Interference>,
        t: &EdgeInterferenceTool,
    ) -> Result<(), Failure> {
        let mut v: Vec<Interference> = il.iter().copied().collect();
        let result = Self::process_complex_in(&mut v, t);
        *il = v.into_iter().collect();
        result
    }

    /// ProcessComplex on a vector; `EdgeIList::process_complex_in(&mut v, &tool)?`.
    pub fn process_complex_in(
        il: &mut Vec<Interference>,
        t: &EdgeInterferenceTool,
    ) -> Result<(), Failure> {
        let mut trans_tool = EdgeFaceTransition::new();
        let mut i1 = 0;
        while i1 < il.len() {
            let i2 = i1 + 1;
            if i2 < il.len() && t.same_interferences(&il[i1], &il[i2]) {
                let (tgt_e, norm_e, curv_e) =
                    t.edge_geometry(t.parameter_of_interference(&il[i1]))?;
                trans_tool.reset(tgt_e, norm_e, curv_e);
                Self::add_to(&mut trans_tool, &il[i1], t)?;
                while i2 < il.len() && t.same_interferences(&il[i1], &il[i2]) {
                    Self::add_to(&mut trans_tool, &il[i2], t)?;
                    il.remove(i2);
                }
                let first = &mut il[i1];
                first.set_transition(trans_tool.transition());
                first.set_boundary_transition(trans_tool.boundary_transition());
            }
            i1 += 1;
        }
        Ok(())
    }

    fn add_to(
        tool: &mut EdgeFaceTransition,
        i: &Interference,
        t: &EdgeInterferenceTool,
    ) -> Result<(), Failure> {
        let (tgt_i, norm_i, curv_i) = t.interference_boundary_geometry(i)?;
        tool.add_interference(
            TOL_ANG,
            tgt_i,
            norm_i,
            curv_i,
            i.orientation(),
            i.transition(),
            i.boundary_transition(),
        );
        Ok(())
    }
}
