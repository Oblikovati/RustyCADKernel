// The shape digest of the TKPrim probes; the Rust twin is
// crates/tkprim/examples/pr_probe/shape.rs. Every TKPrim result is compared through the
// TKTopAlgo digest `tp_shape` (tktopalgo_probe_breplib_common.cxx): its sub-shapes in
// TopExp::MapShapes order with their types, orientations, flags, children and geometry.
#include <BRep_Builder.hxx>
#include <TopoDS_Vertex.hxx>
// `x y z tol`: the digest of a vertex built with BRep_Builder; it proves that both probes
// print the shared digest identically before any TKPrim class is compared.
void pr_digest_vertex_case(std::istream& input){
 GaReader r{input};gp_Pnt point=r.pnt();double tolerance=r.real();
 TopoDS_Vertex vertex;
 GA_TRY(BRep_Builder().MakeVertex(vertex,point,tolerance);tp_shape(vertex))
 ga_end();}
