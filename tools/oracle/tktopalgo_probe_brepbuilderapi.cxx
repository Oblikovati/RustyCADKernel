// BRepBuilderAPI_MakeVertex probe operation; the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_builder_api.rs. The other BRepBuilderAPI
// operations are in tktopalgo_probe_brepbuilderapi_makers.cxx and
// tktopalgo_probe_brepbuilderapi_modify.cxx, which need the BRepLib readers.
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
// `point`: BRepBuilderAPI_MakeVertex(P), then IsDone, BRep_Tool::Pnt and Tolerance.
void tp_make_vertex_case(std::istream& input){
 GaReader r{input};gp_Pnt point=r.pnt();
 BRepBuilderAPI_MakeVertex builder(point);
 ga_flag(builder.IsDone());
 GA_TRY(ga_pnt(BRep_Tool::Pnt(builder.Vertex()));ga_real(BRep_Tool::Tolerance(builder.Vertex())))
 ga_end();}
