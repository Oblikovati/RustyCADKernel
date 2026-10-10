// BRepFill_CompatibleWires probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_compatible.rs.
//
// The sections are wires of a shape program (tktopalgo_probe_shapes.cxx). The result
// sections print through pr_shape; the generated edges as their index in the
// TopExp::MapShapes map of all result sections (bl_indexed of
// tkbool_probe_brepfill_generator.cxx).
#include <BRepFill_CompatibleWires.hxx>
// `program n (wire)... percent rotation`: Init, SetPercent, Perform(rotation); prints
// GetStatus, IsDone, the degenerated flags, the result sections, Generated().Extent() and
// GeneratedShapes of every edge of the program.
void bl_bf_compatible_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 NCollection_Sequence<TopoDS_Shape> sections;
 int n=r.integer();
 for(int k=0;k<n;++k)sections.Append(t.shapes.at(r.integer()));
 BRepFill_CompatibleWires compatible(sections);
 compatible.SetPercent(r.real());
 bool rotation=r.flag();
 try{compatible.Perform(rotation);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_int(compatible.GetStatus());ga_flag(compatible.IsDone());
 ga_flag(compatible.IsDegeneratedFirstSection());ga_flag(compatible.IsDegeneratedLastSection());
 const NCollection_Sequence<TopoDS_Shape>& result=compatible.Shape();
 ga_int(result.Length());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> all;
 for(const TopoDS_Shape& s:result){GA_TRY(pr_shape(s)) if(!s.IsNull())TopExp::MapShapes(s,all);}
 ga_int(compatible.Generated().Extent());
 for(const TopoDS_Shape& s:t.shapes){
  if(s.IsNull()||s.ShapeType()!=TopAbs_EDGE)continue;
  GA_TRY(bl_list(all,compatible.GeneratedShapes(TopoDS::Edge(s))))}
 ga_end();}
