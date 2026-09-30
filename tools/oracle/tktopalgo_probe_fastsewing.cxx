// BRepBuilderAPI_FastSewing probe operation; the Rust twin is
// crates/tktopalgo/examples/tp_probe/fast_sewing.rs. Surfaces come from the face reader
// (tp_surface of tktopalgo_probe_face.cxx), shapes from the shape reader
// (tktopalgo_probe_shape.cxx); the result is printed with the BRepLib digest.
//
// `bba_fastsew tol settol perform n (item)...`: BRepBuilderAPI_FastSewing(tol), then
// SetTolerance(settol) when settol >= 0, then each item: kind 0 Add(surface record),
// 1 Add(shape record), 2 Add of a null shape, 3 Add of a null surface (the result of each
// Add is printed); Perform when `perform`. Then GetTolerance, GetStatuses(nullptr),
// GetStatuses(stream) with the stream text, and the digest of GetResult.
#include <BRepBuilderAPI_FastSewing.hxx>
void tp_fastsew_item(BRepBuilderAPI_FastSewing& sew,GaReader& r){
 int kind=r.integer();
 if(kind==0){occ::handle<Geom_Surface> s=tp_surface(r);ga_flag(sew.Add(s));return;}
 if(kind==1){TpShape s=tp_shape(r);ga_flag(sew.Add(s.shape));return;}
 if(kind==2){ga_flag(sew.Add(TopoDS_Shape()));return;}
 ga_flag(sew.Add(occ::handle<Geom_Surface>()));}
void tp_bba_fastsew_case(std::istream& input){
 GaReader r{input};
 try{double tol=r.real(),settol=r.real();bool perform=r.flag();int n=r.integer();
  occ::handle<BRepBuilderAPI_FastSewing> sew=new BRepBuilderAPI_FastSewing(tol);
  if(settol>=0)sew->SetTolerance(settol);
  for(int i=0;i<n;++i)tp_fastsew_item(*sew,r);
  if(perform)sew->Perform();
  ga_real(sew->GetTolerance());ga_int(int(sew->GetStatuses()));
  std::ostringstream text;ga_int(int(sew->GetStatuses(&text)));ga_text(text.str());
  tp_shape(sew->GetResult());
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
