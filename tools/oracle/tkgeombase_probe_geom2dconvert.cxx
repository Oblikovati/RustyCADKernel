// TKGeomBase oracle: the Geom2dConvert package (tkgeombase_probe.cxx). Curves are read
// with the LProp builders (tkgeombase_probe_lprop_build.cxx); each request calls one
// class or package member and prints every result, or the raised exception type. The
// Rust twin is crates/tkgeombase/examples/gb_probe/geom2d_convert.rs.
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2dConvert.hxx>
#include <Geom2dConvert_ApproxArcsSegments.hxx>
#include <Geom2dConvert_ApproxCurve.hxx>
#include <Geom2dConvert_BSplineCurveKnotSplitting.hxx>
#include <Geom2dConvert_BSplineCurveToBezierCurve.hxx>
#include <Geom2dConvert_CompCurveToBSplineCurve.hxx>
#include <Geom2dConvert_PPoint.hxx>
// A B-spline: -1 when null, else degree, periodic and rational flags, the poles with
// their weights and the knots with their multiplicities.
void gb_g2c_bspline(const occ::handle<Geom2d_BSplineCurve>& c){
 if(c.IsNull()){gb_int(-1);return;}
 gb_int(c->Degree());gb_flag(c->IsPeriodic());gb_flag(c->IsRational());gb_int(c->NbPoles());
 for(int i=1;i<=c->NbPoles();++i){gb_pnt2d(c->Pole(i));gb_real(c->Weight(i));}
 gb_int(c->NbKnots());for(int i=1;i<=c->NbKnots();++i){gb_real(c->Knot(i));gb_int(c->Multiplicity(i));}
}
void gb_g2c_bezier(const occ::handle<Geom2d_BezierCurve>& c){
 if(c.IsNull()){gb_int(-1);return;}
 gb_int(c->Degree());gb_flag(c->IsRational());
 for(int i=1;i<=c->NbPoles();++i){gb_pnt2d(c->Pole(i));gb_real(c->Weight(i));}
}
// An element of ApproxArcsSegments: -1 null; 1 trimmed line or 2 trimmed circle with
// its bounds, end points and basis; 0 anything else.
void gb_g2c_element(const occ::handle<Geom2d_Curve>& c){
 if(c.IsNull()){gb_int(-1);return;}
 occ::handle<Geom2d_TrimmedCurve> t=occ::down_cast<Geom2d_TrimmedCurve>(c);
 if(t.IsNull()){gb_int(0);return;}
 occ::handle<Geom2d_Line> l=occ::down_cast<Geom2d_Line>(t->BasisCurve());
 occ::handle<Geom2d_Circle> ci=occ::down_cast<Geom2d_Circle>(t->BasisCurve());
 if(!l.IsNull()){gb_int(1);gb_lin2d(l->Lin2d());}else if(!ci.IsNull()){gb_int(2);gb_circ2d(ci->Circ2d());}else{gb_int(0);return;}
 gb_real(t->FirstParameter());gb_real(t->LastParameter());gb_pnt2d(t->StartPoint());gb_pnt2d(t->EndPoint());
}
occ::handle<Geom2d_BSplineCurve> gb_g2c_read_bspline(GbReader& r){return occ::down_cast<Geom2d_BSplineCurve>(gb_lp_curve2d(r));}
// g2c_to_bspline Parameterisation Spec: CurveToBSplineCurve.
void gb_g2c_to_bspline_case(std::istream& in){
 GbReader r{in};const Convert_ParameterisationType p=Convert_ParameterisationType(r.integer());
 GB_TRY(occ::handle<Geom2d_Curve> c=gb_lp_curve2d(r);gb_g2c_bspline(Geom2dConvert::CurveToBSplineCurve(c,p)));gb_end();
}
// g2c_split Mode From To Tol Same Spec: SplitBSplineCurve by knot indices (mode 0) or
// by parameters (mode 1).
void gb_g2c_split_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double a=r.real(),b=r.real(),tol=r.real();const bool same=r.flag();
 GB_TRY(occ::handle<Geom2d_BSplineCurve> c=gb_g2c_read_bspline(r);
  if(mode==0)gb_g2c_bspline(Geom2dConvert::SplitBSplineCurve(c,int(a),int(b),same));
  else gb_g2c_bspline(Geom2dConvert::SplitBSplineCurve(c,a,b,tol,same)));
 gb_end();
}
// g2c_knot_split Range Spec: NbSplits, SplitValue(0 .. NbSplits + 1), Splitting.
void gb_g2c_knot_split_case(std::istream& in){
 GbReader r{in};const int range=r.integer();
 GB_TRY(Geom2dConvert_BSplineCurveKnotSplitting s(gb_g2c_read_bspline(r),range);const int n=s.NbSplits();gb_int(n);
  for(int i=0;i<=n+1;++i){GB_TRY(gb_int(s.SplitValue(i)));}
  NCollection_Array1<int> v(1,n);s.Splitting(v);for(int i=1;i<=n;++i)gb_int(v(i)));
 gb_end();
}
// g2c_bezier Mode U1 U2 Tol Spec: the whole curve (mode 0) or [U1, U2]; NbArcs, Knots,
// Arc(0 .. NbArcs + 1), then Arcs.
void gb_g2c_bezier_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),tol=r.real();
 GB_TRY(occ::handle<Geom2d_BSplineCurve> c=gb_g2c_read_bspline(r);
  std::unique_ptr<Geom2dConvert_BSplineCurveToBezierCurve> s(mode==0?new Geom2dConvert_BSplineCurveToBezierCurve(c):new Geom2dConvert_BSplineCurveToBezierCurve(c,u1,u2,tol));
  const int n=s->NbArcs();gb_int(n);NCollection_Array1<double> k(1,n+1);s->Knots(k);for(int i=1;i<=n+1;++i)gb_real(k(i));
  for(int i=0;i<=n+1;++i){GB_TRY(gb_g2c_bezier(s->Arc(i)));}
  NCollection_Array1<occ::handle<Geom2d_BezierCurve>> arcs(1,n);s->Arcs(arcs);for(int i=1;i<=n;++i)gb_g2c_bezier(arcs(i)));
 gb_end();
}
// g2c_comp Parameterisation WithFirst N (Tol After Spec)...: the first curve through the
// constructor (WithFirst) or Add, the others through Add (each result printed), the
// B-spline, then Clear.
void gb_g2c_comp_case(std::istream& in){
 GbReader r{in};const Convert_ParameterisationType p=Convert_ParameterisationType(r.integer());const bool withFirst=r.flag();const int n=r.integer();
 GB_TRY(
  std::unique_ptr<Geom2dConvert_CompCurveToBSplineCurve> comp;
  for(int i=0;i<n;++i){
   const double tol=r.real();const bool after=r.flag();occ::handle<Geom2d_BoundedCurve> c=occ::down_cast<Geom2d_BoundedCurve>(gb_lp_curve2d(r));
   if(i==0&&withFirst){comp.reset(new Geom2dConvert_CompCurveToBSplineCurve(c,p));continue;}
   if(!comp)comp.reset(new Geom2dConvert_CompCurveToBSplineCurve(p));
   GB_TRY(gb_flag(comp->Add(c,tol,after)));
  }
  gb_g2c_bspline(comp->BSplineCurve());comp->Clear();gb_flag(comp->BSplineCurve().IsNull()));
 gb_end();
}
// g2c_concat Kind Closed ClosedTol AngTol NTol Tol... NCurves Spec...: ConcatG1 (kind 0),
// ConcatC1 (1) or ConcatC1 with the angular tolerance (2); then the closed flag, the
// indices, the concatenated curves and the input array, which the members rotate and
// whose weights they normalize.
void gb_g2c_concat_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();bool closed=r.flag();const double ctol=r.real(),ang=r.real();
 const int nt=r.integer();NCollection_Array1<double> tols(0,nt-1);for(int i=0;i<nt;++i)tols(i)=r.real();
 const int nc=r.integer();NCollection_Array1<occ::handle<Geom2d_BSplineCurve>> curves(0,nc-1);for(int i=0;i<nc;++i)curves(i)=gb_g2c_read_bspline(r);
 occ::handle<NCollection_HArray1<int>> idx;occ::handle<NCollection_HArray1<occ::handle<Geom2d_BSplineCurve>>> res;
 GB_TRY(if(kind==0)Geom2dConvert::ConcatG1(curves,tols,res,closed,ctol);else if(kind==1)Geom2dConvert::ConcatC1(curves,tols,idx,res,closed,ctol);
  else Geom2dConvert::ConcatC1(curves,tols,idx,res,closed,ctol,ang));
 gb_flag(closed);
 if(idx.IsNull())gb_int(-1);else{gb_int(idx->Lower());gb_int(idx->Upper());for(int i=idx->Lower();i<=idx->Upper();++i)gb_int(idx->Value(i));}
 if(res.IsNull())gb_int(-1);else{gb_int(res->Lower());gb_int(res->Upper());for(int i=res->Lower();i<=res->Upper();++i)gb_g2c_bspline(res->Value(i));}
 for(int i=0;i<nc;++i)gb_g2c_bspline(curves(i));
 gb_end();
}
// g2c_c0c1 Mode Tol Ang Spec: C0BSplineToC1BSplineCurve (mode 0), then
// C0BSplineToArrayOfC1BSplineCurve with Tol (1) or with Ang and Tol (2).
void gb_g2c_c0c1_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double tol=r.real(),ang=r.real();
 GB_TRY(occ::handle<Geom2d_BSplineCurve> c=gb_g2c_read_bspline(r);
  if(mode==0){Geom2dConvert::C0BSplineToC1BSplineCurve(c,tol);gb_g2c_bspline(c);}
  else{occ::handle<NCollection_HArray1<occ::handle<Geom2d_BSplineCurve>>> t;
   GB_TRY(if(mode==1)Geom2dConvert::C0BSplineToArrayOfC1BSplineCurve(c,t,tol);else Geom2dConvert::C0BSplineToArrayOfC1BSplineCurve(c,t,ang,tol));
   if(t.IsNull())gb_int(-1);else{gb_int(t->Lower());gb_int(t->Upper());for(int i=t->Lower();i<=t->Upper();++i)gb_g2c_bspline(t->Value(i));}});
 gb_end();
}
// g2c_approx Adaptor Ranged U1 U2 Tol Order MaxSeg MaxDeg Spec: the curve or adaptor
// constructor; IsDone, HasResult, MaxError and the curve.
void gb_g2c_approx_case(std::istream& in){
 GbReader r{in};const bool viaAdaptor=r.flag(),ranged=r.flag();const double u1=r.real(),u2=r.real(),tol=r.real();
 const GeomAbs_Shape order=GeomAbs_Shape(r.integer());const int seg=r.integer(),deg=r.integer();
 GB_TRY(occ::handle<Geom2d_Curve> c=gb_lp_curve2d(r);std::unique_ptr<Geom2dConvert_ApproxCurve> a;
  if(viaAdaptor){occ::handle<Geom2dAdaptor_Curve> h=ranged?new Geom2dAdaptor_Curve(c,u1,u2):new Geom2dAdaptor_Curve(c);a.reset(new Geom2dConvert_ApproxCurve(occ::handle<Adaptor2d_Curve2d>(h),tol,order,seg,deg));}
  else a.reset(new Geom2dConvert_ApproxCurve(c,tol,order,seg,deg));
  gb_flag(a->IsDone());gb_flag(a->HasResult());gb_real(a->MaxError());gb_g2c_bspline(a->Curve()));
 gb_end();
}
// g2c_arcs Ranged U1 U2 Tol AngTol Spec: ApproxArcsSegments on the adapted curve; the
// element count and each element.
void gb_g2c_arcs_case(std::istream& in){
 GbReader r{in};const bool ranged=r.flag();const double u1=r.real(),u2=r.real(),tol=r.real(),ang=r.real();
 GB_TRY(occ::handle<Geom2d_Curve> c=gb_lp_curve2d(r);Geom2dAdaptor_Curve a;if(ranged)a.Load(c,u1,u2);else a.Load(c);
  Geom2dConvert_ApproxArcsSegments s(a,tol,ang);const NCollection_Sequence<occ::handle<Geom2d_Curve>>& seq=s.GetResult();
  gb_int(seq.Length());for(int i=1;i<=seq.Length();++i)gb_g2c_element(seq(i)));
 gb_end();
}
void gb_g2c_ppoint(const Geom2dConvert_PPoint& p){gb_real(p.Parameter());gb_xy(p.Point());gb_xy(p.D1());}
// g2c_ppoint U1 U2 X Y DX DY Spec: the points of the curve at U1 and U2, the empty point,
// a point from values, their distances and both comparisons, then SetD1.
void gb_g2c_ppoint_case(std::istream& in){
 GbReader r{in};const double u1=r.real(),u2=r.real();const gp_XY xy=r.xy(),d=r.xy();
 GB_TRY(occ::handle<Geom2d_Curve> c=gb_lp_curve2d(r);Geom2dAdaptor_Curve a(c);
  Geom2dConvert_PPoint p1(u1,a),p2(u2,a),e,v(u2,xy,d);gb_g2c_ppoint(p1);gb_g2c_ppoint(p2);gb_g2c_ppoint(e);gb_g2c_ppoint(v);
  gb_real(p1.Dist(p2));gb_real(p1.Dist(e));gb_flag(p1==p2);gb_flag(p1!=p2);gb_flag(p2==v);gb_flag(p2!=v);gb_flag(e==e);gb_flag(e!=e);
  p1.SetD1(d);gb_g2c_ppoint(p1));
 gb_end();
}
