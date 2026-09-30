// BRepBuilderAPI probe operations; the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_builder_api.rs. The Rust side is still the
// STUB MakeVertex (crates/tktopalgo/src/brep_builder_api), so only the vertex it
// builds is compared: its point and its tolerance.
#include <BRep_Tool.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
// `point`: BRepBuilderAPI_MakeVertex(P), then IsDone, BRep_Tool::Pnt and Tolerance.
void tp_make_vertex_case(std::istream& input){
 GaReader r{input};gp_Pnt point=r.pnt();
 BRepBuilderAPI_MakeVertex builder(point);
 ga_flag(builder.IsDone());
 GA_TRY(ga_pnt(BRep_Tool::Pnt(builder.Vertex()));ga_real(BRep_Tool::Tolerance(builder.Vertex())))
 ga_end();}
