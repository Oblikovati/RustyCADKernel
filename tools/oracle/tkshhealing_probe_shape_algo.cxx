// ShapeAlgo_AlgoContainer probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/shape_algo.rs.
//
// `sh_algo_geom op ...` on one ShapeAlgo_AlgoContainer:
//  0 (curve record): ApproxBSplineCurve of the curve as a Geom_BSplineCurve (-1 when it
//    is not one): the count, then per curve whether it is the input, its digest and its
//    B-spline data;
//  1 (2d curve record): the same in 2D;
//  2 (curve record): C0BSplineToSequenceOfC1BSplineCurve (-1 when not a B-spline): the
//    flag, then for true the count and each curve;
//  3 (2d curve record): the same in 2D;
//  4 (curve record) first last tol continuity maxseg maxdeg: ConvertCurveToBSpline;
//  5 (surface as in sh_construct_pkg) uf ul vf vl: ConvertSurfaceToBSpline;
//  6 (surface as in sh_construct_pkg): ConvertToPeriodic, the digest and B-spline data.
// `sh_algo_shape (prelude) program nops (op)...` on the shapes of a prelude and a shape
// program (tkshhealing_probe_shcommon.cxx):
//  0 w1 w2: HomoWires(w1, w2): the flag and, for true, the digests of both wires;
//  1 w1 w2 prec maxtol: ConnectNextWire of a ShapeAnalysis_Wire loaded with the wire data
//    of w1 (precision prec) and the wire data of w2: the flag, distmin, revsewd,
//    revnextsewd and the digests of both wires after;
//  2 f: OuterWire and GetFaceUVBounds of the face;
//  3 s tol: C0ShapeToC1Shape(s, tol).
#include <ShapeAlgo_AlgoContainer.hxx>
#include <ShapeAnalysis_Wire.hxx>
#include <ShapeExtend_WireData.hxx>
void sa_spline3d(const occ::handle<Geom_Curve>& c,const occ::handle<Geom_Curve>& input){
 ga_flag(c==input);shc_curve(c);sh_construct_spline3d(c);}
void sa_spline2d(const occ::handle<Geom2d_Curve>& c,const occ::handle<Geom2d_Curve>& input){
 ga_flag(c==input);shc_curve2d(c);sh_construct_spline2d(c);}
void sa_approx(GaReader& r,const ShapeAlgo_AlgoContainer& a){
 occ::handle<Geom_BSplineCurve> b=occ::down_cast<Geom_BSplineCurve>(tp_geom_curve(r));
 if(b.IsNull()){ga_int(-1);return;}
 GA_TRY(NCollection_Sequence<occ::handle<Geom_Curve>> seq;a.ApproxBSplineCurve(b,seq);ga_int(seq.Length());
  for(int i=1;i<=seq.Length();++i)sa_spline3d(seq(i),b))}
void sa_approx2d(GaReader& r,const ShapeAlgo_AlgoContainer& a){
 occ::handle<Geom2d_BSplineCurve> b=occ::down_cast<Geom2d_BSplineCurve>(tp_geom2d_curve(r));
 if(b.IsNull()){ga_int(-1);return;}
 GA_TRY(NCollection_Sequence<occ::handle<Geom2d_Curve>> seq;a.ApproxBSplineCurve(b,seq);ga_int(seq.Length());
  for(int i=1;i<=seq.Length();++i)sa_spline2d(seq(i),b))}
void sa_split(GaReader& r,const ShapeAlgo_AlgoContainer& a){
 occ::handle<Geom_BSplineCurve> b=occ::down_cast<Geom_BSplineCurve>(tp_geom_curve(r));
 if(b.IsNull()){ga_int(-1);return;}
 GA_TRY(occ::handle<NCollection_HSequence<occ::handle<Geom_BoundedCurve>>> seq;
  bool ok=a.C0BSplineToSequenceOfC1BSplineCurve(b,seq);ga_flag(ok);if(!ok)return;ga_int(seq->Length());
  for(int i=1;i<=seq->Length();++i)sa_spline3d(seq->Value(i),b))}
void sa_split2d(GaReader& r,const ShapeAlgo_AlgoContainer& a){
 occ::handle<Geom2d_BSplineCurve> b=occ::down_cast<Geom2d_BSplineCurve>(tp_geom2d_curve(r));
 if(b.IsNull()){ga_int(-1);return;}
 GA_TRY(occ::handle<NCollection_HSequence<occ::handle<Geom2d_BoundedCurve>>> seq;
  bool ok=a.C0BSplineToSequenceOfC1BSplineCurve(b,seq);ga_flag(ok);if(!ok)return;ga_int(seq->Length());
  for(int i=1;i<=seq->Length();++i)sa_spline2d(seq->Value(i),b))}
void sa_curve(GaReader& r,const ShapeAlgo_AlgoContainer& a){
 occ::handle<Geom_Curve> c=tp_geom_curve(r);double first=r.real(),last=r.real(),tol=r.real();
 GeomAbs_Shape cont=GeomAbs_Shape(r.integer());int seg=r.integer(),deg=r.integer();
 GA_TRY(occ::handle<Geom_BSplineCurve> b=a.ConvertCurveToBSpline(c,first,last,tol,cont,seg,deg);
  sa_spline3d(b,c))}
void sa_surface(GaReader& r,const ShapeAlgo_AlgoContainer& a,bool periodic){
 occ::handle<Geom_Surface> s;GA_TRY(s=scp_surface(r))
 double uf=0.,ul=0.,vf=0.,vl=0.;
 if(!periodic){uf=r.real();ul=r.real();vf=r.real();vl=r.real();}
 if(s.IsNull())return;
 if(periodic){GA_TRY(occ::handle<Geom_Surface> p=a.ConvertToPeriodic(s);ga_flag(p==s);shc_surface(p);
  scp_bspline_surface(occ::down_cast<Geom_BSplineSurface>(p)))return;}
 GA_TRY(occ::handle<Geom_BSplineSurface> b=a.ConvertSurfaceToBSpline(s,uf,ul,vf,vl);
  shc_surface(b);scp_bspline_surface(b))}
void sh_algo_geom_case(std::istream& input){
 GaReader r{input};int op=r.integer();ShapeAlgo_AlgoContainer a;
 if(op==0)sa_approx(r,a);else if(op==1)sa_approx2d(r,a);else if(op==2)sa_split(r,a);
 else if(op==3)sa_split2d(r,a);else if(op==4)sa_curve(r,a);else sa_surface(r,a,op==6);
 ga_end();}
void sa_homo(GaReader& r,const TpShapes& t,const ShapeAlgo_AlgoContainer& a){
 const TopoDS_Shape& w1=t.shapes.at(r.integer());const TopoDS_Shape& w2=t.shapes.at(r.integer());
 GA_TRY(TopoDS_Wire o1,o2;bool ok=a.HomoWires(TopoDS::Wire(w1),TopoDS::Wire(w2),o1,o2,false);ga_flag(ok);
  if(ok){tp_shape(o1);tp_shape(o2);})}
void sa_connect(GaReader& r,const TpShapes& t,const ShapeAlgo_AlgoContainer& a){
 const TopoDS_Shape& w1=t.shapes.at(r.integer());const TopoDS_Shape& w2=t.shapes.at(r.integer());
 double prec=r.real(),maxtol=r.real();
 GA_TRY(occ::handle<ShapeExtend_WireData> sewd=new ShapeExtend_WireData(TopoDS::Wire(w1));
  occ::handle<ShapeExtend_WireData> next=new ShapeExtend_WireData(TopoDS::Wire(w2));
  occ::handle<ShapeAnalysis_Wire> saw=new ShapeAnalysis_Wire;saw->Load(sewd);saw->SetPrecision(prec);
  double dist=-1.;bool rev=false,revnext=false;
  ga_flag(a.ConnectNextWire(saw,next,maxtol,dist,rev,revnext));ga_real(dist);ga_flag(rev);ga_flag(revnext);
  tp_shape(sewd->Wire());tp_shape(next->Wire()))}
void sa_face(GaReader& r,const TpShapes& t,const ShapeAlgo_AlgoContainer& a){
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));
 GA_TRY(tp_shape(a.OuterWire(f)))
 GA_TRY(double u1=0.,u2=0.,v1=0.,v2=0.;a.GetFaceUVBounds(f,u1,u2,v1,v2);ga_real(u1);ga_real(u2);ga_real(v1);ga_real(v2))}
void sa_c1(GaReader& r,const TpShapes& t,const ShapeAlgo_AlgoContainer& a){
 const TopoDS_Shape& s=t.shapes.at(r.integer());double tol=r.real();
 GA_TRY(tp_shape(a.C0ShapeToC1Shape(s,tol)))}
void sh_algo_shape_case(std::istream& input){
 GaReader r{input};TpShapes t=shc_shapes(r);ShapeAlgo_AlgoContainer a;int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  if(op==0)sa_homo(r,t,a);else if(op==1)sa_connect(r,t,a);else if(op==2)sa_face(r,t,a);else sa_c1(r,t,a);}
 ga_end();}
