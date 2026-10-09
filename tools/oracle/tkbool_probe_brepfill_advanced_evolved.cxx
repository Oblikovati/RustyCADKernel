// BRepFill_AdvancedEvolved probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_advanced_evolved.rs.
//
// The spine and the profile are wires of a shape program (tktopalgo_probe_shapes.cxx);
// shapes print through pr_shape (tkprim_probe_shape.cxx).
#include <BRepFill_AdvancedEvolved.hxx>
// `program spine profile tol solid parallel`: Perform of the wires; prints IsDone with
// its error code and the result.
void bl_bf_advanced_evolved_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Wire spine=TopoDS::Wire(t.shapes.at(r.integer())),profile=TopoDS::Wire(t.shapes.at(r.integer()));
 double tol=r.real();bool solid=r.flag(),parallel=r.flag();
 BRepFill_AdvancedEvolved ae;ae.SetParallelMode(parallel);
 try{ae.Perform(spine,profile,tol,solid);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 unsigned int code=0;ga_flag(ae.IsDone(&code));ga_int((int)code);
 GA_TRY(pr_shape(ae.Shape()))
 ga_end();}
