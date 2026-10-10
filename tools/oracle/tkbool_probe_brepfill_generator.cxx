// BRepFill_Generator probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_generator.rs.
//
// The sections are wires of a shape program (tktopalgo_probe_shapes.cxx). The result shell
// prints through pr_shape (tkprim_probe_shape.cxx); generated and result shapes print as
// their index in TopExp::MapShapes of the shell and their orientation, or (index 0, a shape
// outside the shell) their digest.
#include <BRepFill_Generator.hxx>
#include <TopExp.hxx>
#include <TopoDS_Wire.hxx>
// A shape against the sub-shapes of the result: -1 for a null shape, else its index (0 when
// it is not a sub-shape, then its digest) and its orientation.
void bl_indexed(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& result,const TopoDS_Shape& s){
 if(s.IsNull()){ga_int(-1);return;}
 int i=result.FindIndex(s);ga_int(i);ga_int(s.Orientation());
 if(i==0)pr_shape(s);}
// A list of shapes: its extent, then each shape.
void bl_list(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& result,const NCollection_List<TopoDS_Shape>& l){
 ga_int(l.Extent());for(const TopoDS_Shape& s:l)bl_indexed(result,s);}
// `program n (wire)... mutable`: a generator on the wires of the program (table indices);
// prints IsMutableInput, GetStatus and Shell after Perform, Generated().Extent() and
// GeneratedShapes of a null shape (the map is keyed by the explorer's edge after its
// advance, a null edge for the last face), then GeneratedShapes and ResultShape of every
// shape of the program.
void bl_bf_generator_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 BRepFill_Generator generator;
 int n=r.integer();
 for(int k=0;k<n;++k)generator.AddWire(TopoDS::Wire(t.shapes.at(r.integer())));
 generator.SetMutableInput(r.flag());
 ga_flag(generator.IsMutableInput());
 try{generator.Perform();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_int(generator.GetStatus());
 const TopoDS_Shell& shell=generator.Shell();
 GA_TRY(pr_shape(shell))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> result;
 if(!shell.IsNull())TopExp::MapShapes(shell,result);
 ga_int(generator.Generated().Extent());
 GA_TRY(bl_list(result,generator.GeneratedShapes(TopoDS_Shape())))
 for(const TopoDS_Shape& s:t.shapes){
  GA_TRY(bl_list(result,generator.GeneratedShapes(s)))
  GA_TRY(bl_indexed(result,generator.ResultShape(s)))}
 ga_end();}
