// BRepAlgo package probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_algo.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <BRepAlgo.hxx>
// `program wire option angtol`: ConcatenateWire (option as a GeomAbs_Shape value).
void bl_ba_concat_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Wire w=TopoDS::Wire(t.shapes.at(r.integer()));GeomAbs_Shape option=(GeomAbs_Shape)r.integer();double tol=r.real();
 GA_TRY(pr_shape(BRepAlgo::ConcatenateWire(w,option,tol)))
 ga_end();}
// `program wire`: ConcatenateWireC0.
void bl_ba_concat_c0_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Wire w=TopoDS::Wire(t.shapes.at(r.integer()));
 GA_TRY(pr_shape(BRepAlgo::ConcatenateWireC0(w)))
 ga_end();}
// `program face angtol`: ConvertWire of each wire of the face, then ConvertFace.
void bl_ba_convert_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));double tol=r.real();
 for(TopExp_Explorer ex(f,TopAbs_WIRE);ex.More();ex.Next())GA_TRY(pr_shape(BRepAlgo::ConvertWire(TopoDS::Wire(ex.Current()),tol,f)))
 GA_TRY(pr_shape(BRepAlgo::ConvertFace(f,tol)))
 ga_end();}
// `program n arg... result closed geomctrl`: IsValid and IsTopologicallyValid of the
// result, then IsValid of the result of the arguments.
void bl_ba_valid_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 NCollection_List<TopoDS_Shape> args;int n=r.integer();for(int i=0;i<n;i++)args.Append(t.shapes.at(r.integer()));
 const TopoDS_Shape res=t.shapes.at(r.integer());bool closed=r.flag(),geom=r.flag();
 GA_TRY(ga_flag(BRepAlgo::IsValid(res)))
 GA_TRY(ga_flag(BRepAlgo::IsTopologicallyValid(res)))
 GA_TRY(ga_flag(BRepAlgo::IsValid(args,res,closed,geom)))
 ga_end();}
