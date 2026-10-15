// BRepOffsetAPI_MiddlePath probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/middle_path.rs.
//
// The pipe-like shapes are BRepOffsetAPI_MakePipe sweeps of shape programs
// (tktopalgo_probe_shapes.cxx) or primitives (ot_primitive of
// tkoffset_probe_brepoffset_tool.cxx); the path prints through pr_shape
// (tkprim_probe_shape.cxx).
#include <BRepOffsetAPI_MiddlePath.hxx>
#include <BRepTools.hxx>
// The middle path of a shape between two shapes: prints the exception or IsDone after
// Build, then the shape.
static void of_middle_path(const TopoDS_Shape& shape,const TopoDS_Shape& first,const TopoDS_Shape& last){
 std::unique_ptr<BRepOffsetAPI_MiddlePath> path;
 try{path.reset(new BRepOffsetAPI_MiddlePath(shape,first,last));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 GA_TRY(path->Build())
 ga_flag(path->IsDone());
 if(path->IsDone())GA_TRY(pr_shape(path->Shape()))
 ga_end();}
// `oa_middle program spine profile wires`: BRepOffsetAPI_MakePipe(spine, profile), then
// the middle path of the pipe between its first and last shapes (their outer wires when
// `wires` and they are faces).
void of_oa_middle_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire spine=TopoDS::Wire(t.shapes.at(r.integer()));TopoDS_Shape profile=t.shapes.at(r.integer());
 bool wires=r.flag();
 TopoDS_Shape shape,first,last;
 try{
  BRepOffsetAPI_MakePipe pipe(spine,profile);
  shape=pipe.Shape();first=pipe.FirstShape();last=pipe.LastShape();
  if(wires&&first.ShapeType()==TopAbs_FACE){
   first=BRepTools::OuterWire(TopoDS::Face(first));last=BRepTools::OuterWire(TopoDS::Face(last));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 of_middle_path(shape,first,last);}
// `oa_middle_prim prim start end wires`: the middle path of a primitive between two of its
// faces (explorer indices; their outer wires when `wires`).
void of_oa_middle_prim_case(std::istream& input){
 GaReader r{input};TopoDS_Shape shape,first,last;
 try{
  shape=ot_primitive(r);
  first=ot_sub(shape,TopAbs_FACE,r.integer());last=ot_sub(shape,TopAbs_FACE,r.integer());
  if(r.flag()){first=BRepTools::OuterWire(TopoDS::Face(first));last=BRepTools::OuterWire(TopoDS::Face(last));}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 of_middle_path(shape,first,last);}
