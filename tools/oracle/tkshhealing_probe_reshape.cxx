// ShapeBuild_ReShape probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/reshape.rs.
//
// `sh_reshape program nreq (req)... napply (apply)...` over the shapes of a TKTopAlgo shape
// program (-1: a null shape). The requests record: 0 s t: Replace(s, t); 1 s: Remove(s);
// 2 mode: ModeConsiderLocation() = mode. Each apply prints:
//  0 s until: Apply(s, until), its digest and the 19 ShapeExtend status flags;
//  1 s until mode: Apply(s, until, buildmode) and its digest (the corpus keeps to the
//  types the member handles: its "NOT YET IMPLEMENTED" line goes to the standard output);
//  2 s last: Status(s, newsh, last) and the digest of newsh; 3 s: IsRecorded, Value.
// A result is printed as its IsSame index in the table and orientation, then tp_shape.
#include <ShapeBuild_ReShape.hxx>
void sh_rs_result(const TpShapes& t,const TopoDS_Shape& s){
 ga_int(tp_same_index(t,s));ga_int(s.IsNull()?-1:int(s.Orientation()));tp_shape(s);}
void sh_rs_request(GaReader& r,const TpShapes& t,const occ::handle<ShapeBuild_ReShape>& rs){
 int op=r.integer();
 if(op==0){TopoDS_Shape s=sh_shape(t,r.integer());rs->Replace(s,sh_shape(t,r.integer()));}
 else if(op==1)rs->Remove(sh_shape(t,r.integer()));
 else rs->ModeConsiderLocation()=r.flag();}
void sh_rs_apply(GaReader& r,const TpShapes& t,const occ::handle<ShapeBuild_ReShape>& rs){
 int op=r.integer();TopoDS_Shape s=sh_shape(t,r.integer());
 if(op==0){TopAbs_ShapeEnum until=TopAbs_ShapeEnum(r.integer());GA_TRY(sh_rs_result(t,rs->Apply(s,until)))
  for(int k=0;k<19;++k)ga_flag(rs->Status(ShapeExtend_Status(k)));}
 else if(op==1){TopAbs_ShapeEnum until=TopAbs_ShapeEnum(r.integer());int mode=r.integer();GA_TRY(sh_rs_result(t,rs->Apply(s,until,mode)))}
 else if(op==2){bool last=r.flag();TopoDS_Shape newsh;GA_TRY(ga_int(rs->Status(s,newsh,last));sh_rs_result(t,newsh))}
 else{ga_flag(rs->IsRecorded(s));GA_TRY(sh_rs_result(t,rs->Value(s)))}}
void sh_reshape_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);occ::handle<ShapeBuild_ReShape> rs=new ShapeBuild_ReShape();
 int nreq=r.integer();for(int k=0;k<nreq;++k)GA_TRY(sh_rs_request(r,t,rs))
 int napply=r.integer();for(int k=0;k<napply;++k)sh_rs_apply(r,t,rs);
 ga_flag(rs->IsKind(STANDARD_TYPE(BRepTools_ReShape)));ga_end();}
