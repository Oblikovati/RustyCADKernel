// BRepOffsetAPI_ThruSections probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/thru_sections.rs.
//
// The sections are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <BRepOffsetAPI_ThruSections.hxx>
// `program solid ruled pres3d check smoothing partype continuity maxdeg mutable n (kind
// index)...`: a loft of the sections (kind 0 AddWire, 1 AddVertex) with
// CheckCompatibility, SetSmoothing, SetParType, SetContinuity, SetMaxDegree (unless -1)
// and SetMutableInput; prints the exception of Build or IsDone and GetStatus, when done
// the shape and the first and last shapes, then for every shape of the program the
// indices (in the map of the result, -1 for none) of Generated and of GeneratedFace.
void of_oa_thru_sections_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 bool solid=r.flag(),ruled=r.flag();double pres3d=r.real();
 BRepOffsetAPI_ThruSections loft(solid,ruled,pres3d);
 loft.CheckCompatibility(r.flag());loft.SetSmoothing(r.flag());
 loft.SetParType((Approx_ParametrizationType)r.integer());loft.SetContinuity((GeomAbs_Shape)r.integer());
 int maxdeg=r.integer();if(maxdeg>=0)loft.SetMaxDegree(maxdeg);
 loft.SetMutableInput(r.flag());
 int n=r.integer();
 for(int k=0;k<n;++k){int kind=r.integer();const TopoDS_Shape& s=t.shapes.at(r.integer());
  if(kind==1)loft.AddVertex(TopoDS::Vertex(s));else loft.AddWire(TopoDS::Wire(s));}
 try{loft.Build();}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 bool done=loft.IsDone();ga_flag(done);ga_int((int)loft.GetStatus());
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(done){GA_TRY(pr_shape(loft.Shape()))GA_TRY(pr_shape(loft.FirstShape()))GA_TRY(pr_shape(loft.LastShape()))
  TopExp::MapShapes(loft.Shape(),map);}
 for(const TopoDS_Shape& s:t.shapes){
  GA_TRY(of_list(map,loft.Generated(s)))
  GA_TRY(const TopoDS_Shape f=loft.GeneratedFace(s);ga_int(f.IsNull()?-1:map.FindIndex(f)))}
 ga_end();}
