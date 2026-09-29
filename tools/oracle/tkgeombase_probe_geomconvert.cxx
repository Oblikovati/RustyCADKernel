// TKGeomBase oracle: the GeomConvert conversions, splitting, Bezier cutting,
// composition, concatenation and approximation (tkgeombase_probe.cxx). Geometry is read
// with the GeomTools builders (tkgeombase_probe_geomtools_build.cxx: gt_curve,
// gt_curve2d, gt_surface); each request calls one class or package member and prints
// every result, or the raised exception type. The printers below are shared by the
// GeomConvert recognizer and GeomLib probes. The Rust twin is
// crates/tkgeombase/examples/gb_probe/geom_convert/.
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <GeomConvert.hxx>
#include <GeomConvert_ApproxCurve.hxx>
#include <GeomConvert_ApproxSurface.hxx>
#include <GeomConvert_BSplineCurveKnotSplitting.hxx>
#include <GeomConvert_BSplineCurveToBezierCurve.hxx>
#include <GeomConvert_BSplineSurfaceKnotSplitting.hxx>
#include <GeomConvert_BSplineSurfaceToBezierSurface.hxx>
#include <GeomConvert_CompBezierSurfacesToBSplineSurface.hxx>
#include <GeomConvert_CompCurveToBSplineCurve.hxx>
#include <gp_Sphere.hxx>
#include <gp_Torus.hxx>
// A B-spline curve: -1 when null, else degree, periodic and rational flags, the poles
// with their weights and the knots with their multiplicities.
void gb_gcv_bspline(const occ::handle<Geom_BSplineCurve>& c){
 if(c.IsNull()){gb_int(-1);return;}
 gb_int(c->Degree());gb_flag(c->IsPeriodic());gb_flag(c->IsRational());gb_int(c->NbPoles());
 for(int i=1;i<=c->NbPoles();++i){gb_pnt(c->Pole(i));gb_real(c->Weight(i));}
 gb_int(c->NbKnots());for(int i=1;i<=c->NbKnots();++i){gb_real(c->Knot(i));gb_int(c->Multiplicity(i));}
}
void gb_gcv_bezier(const occ::handle<Geom_BezierCurve>& c){
 if(c.IsNull()){gb_int(-1);return;}
 gb_int(c->Degree());gb_flag(c->IsRational());gb_int(c->NbPoles());
 for(int i=1;i<=c->NbPoles();++i){gb_pnt(c->Pole(i));gb_real(c->Weight(i));}
}
// A B-spline surface: -1 when null, else degrees, periodic and rational flags, the pole
// grid with its weights and both knot sequences with their multiplicities.
void gb_gcv_bspline_surface(const occ::handle<Geom_BSplineSurface>& s){
 if(s.IsNull()){gb_int(-1);return;}
 gb_int(s->UDegree());gb_int(s->VDegree());gb_flag(s->IsUPeriodic());gb_flag(s->IsVPeriodic());
 gb_flag(s->IsURational());gb_flag(s->IsVRational());gb_int(s->NbUPoles());gb_int(s->NbVPoles());
 for(int i=1;i<=s->NbUPoles();++i)for(int j=1;j<=s->NbVPoles();++j){gb_pnt(s->Pole(i,j));gb_real(s->Weight(i,j));}
 gb_int(s->NbUKnots());for(int i=1;i<=s->NbUKnots();++i){gb_real(s->UKnot(i));gb_int(s->UMultiplicity(i));}
 gb_int(s->NbVKnots());for(int i=1;i<=s->NbVKnots();++i){gb_real(s->VKnot(i));gb_int(s->VMultiplicity(i));}
}
void gb_gcv_bezier_surface(const occ::handle<Geom_BezierSurface>& s){
 if(s.IsNull()){gb_int(-1);return;}
 gb_int(s->UDegree());gb_int(s->VDegree());gb_flag(s->IsURational());gb_flag(s->IsVRational());
 for(int i=1;i<=s->NbUPoles();++i)for(int j=1;j<=s->NbVPoles();++j){gb_pnt(s->Pole(i,j));gb_real(s->Weight(i,j));}
}
void gb_gcv_sphere(const gp_Sphere& s){gb_ax3(s.Position());gb_real(s.Radius());}
void gb_gcv_torus(const gp_Torus& t){gb_ax3(t.Position());gb_real(t.MajorRadius());gb_real(t.MinorRadius());}
// Any Geom curve: the class name, then its data (trimmed bounds and basis, gp element,
// poles, offset and basis); nothing more for another class.
void gb_gcv_curve(const occ::handle<Geom_Curve>& c){
 gb_name(c);if(c.IsNull())return;
 if(auto t=occ::down_cast<Geom_TrimmedCurve>(c)){gb_real(t->FirstParameter());gb_real(t->LastParameter());gb_gcv_curve(t->BasisCurve());return;}
 if(auto x=occ::down_cast<Geom_Line>(c)){gb_lin(x->Lin());return;}
 if(auto x=occ::down_cast<Geom_Circle>(c)){gb_circ(x->Circ());return;}
 if(auto x=occ::down_cast<Geom_Ellipse>(c)){gb_elips(x->Elips());return;}
 if(auto x=occ::down_cast<Geom_Hyperbola>(c)){gb_hypr(x->Hypr());return;}
 if(auto x=occ::down_cast<Geom_Parabola>(c)){gb_parab(x->Parab());return;}
 if(auto x=occ::down_cast<Geom_BSplineCurve>(c)){gb_gcv_bspline(x);return;}
 if(auto x=occ::down_cast<Geom_BezierCurve>(c)){gb_gcv_bezier(x);return;}
 if(auto x=occ::down_cast<Geom_OffsetCurve>(c)){gb_real(x->Offset());gb_dir(x->Direction());gb_gcv_curve(x->BasisCurve());}
}
void gb_gcv_curve2d(const occ::handle<Geom2d_Curve>& c){
 gb_name(c);if(c.IsNull())return;
 if(auto t=occ::down_cast<Geom2d_TrimmedCurve>(c)){gb_real(t->FirstParameter());gb_real(t->LastParameter());gb_gcv_curve2d(t->BasisCurve());return;}
 if(auto x=occ::down_cast<Geom2d_Line>(c)){gb_lin2d(x->Lin2d());return;}
 if(auto x=occ::down_cast<Geom2d_Circle>(c)){gb_circ2d(x->Circ2d());return;}
 if(auto x=occ::down_cast<Geom2d_Ellipse>(c)){gb_elips2d(x->Elips2d());return;}
 if(auto x=occ::down_cast<Geom2d_Hyperbola>(c)){gb_hypr2d(x->Hypr2d());return;}
 if(auto x=occ::down_cast<Geom2d_Parabola>(c)){gb_parab2d(x->Parab2d());return;}
 if(auto x=occ::down_cast<Geom2d_BSplineCurve>(c)){gb_g2c_bspline(x);return;}
 if(auto x=occ::down_cast<Geom2d_BezierCurve>(c)){gb_g2c_bezier(x);return;}
 if(auto x=occ::down_cast<Geom2d_OffsetCurve>(c)){gb_real(x->Offset());gb_gcv_curve2d(x->BasisCurve());}
}
// Any Geom surface: the class name, then its data (trimmed bounds and basis, gp
// element, poles, swept axis or direction and curve, offset and basis).
void gb_gcv_surface(const occ::handle<Geom_Surface>& s){
 gb_name(s);if(s.IsNull())return;
 if(auto t=occ::down_cast<Geom_RectangularTrimmedSurface>(s)){
  double u1,u2,v1,v2;t->Bounds(u1,u2,v1,v2);gb_real(u1);gb_real(u2);gb_real(v1);gb_real(v2);gb_gcv_surface(t->BasisSurface());return;}
 if(auto x=occ::down_cast<Geom_Plane>(s)){gb_pln(x->Pln());return;}
 if(auto x=occ::down_cast<Geom_CylindricalSurface>(s)){gb_cylinder(x->Cylinder());return;}
 if(auto x=occ::down_cast<Geom_ConicalSurface>(s)){gb_cone(x->Cone());return;}
 if(auto x=occ::down_cast<Geom_SphericalSurface>(s)){gb_gcv_sphere(x->Sphere());return;}
 if(auto x=occ::down_cast<Geom_ToroidalSurface>(s)){gb_gcv_torus(x->Torus());return;}
 if(auto x=occ::down_cast<Geom_BSplineSurface>(s)){gb_gcv_bspline_surface(x);return;}
 if(auto x=occ::down_cast<Geom_BezierSurface>(s)){gb_gcv_bezier_surface(x);return;}
 if(auto x=occ::down_cast<Geom_SurfaceOfRevolution>(s)){gb_ax1(x->Axis());gb_gcv_curve(x->BasisCurve());return;}
 if(auto x=occ::down_cast<Geom_SurfaceOfLinearExtrusion>(s)){gb_dir(x->Direction());gb_gcv_curve(x->BasisCurve());return;}
 if(auto x=occ::down_cast<Geom_OffsetSurface>(s)){gb_real(x->Offset());gb_gcv_surface(x->BasisSurface());}
}
occ::handle<Geom_BSplineCurve> gb_gcv_read_bspline(GbReader& r){return occ::down_cast<Geom_BSplineCurve>(gt_curve(r));}
occ::handle<Geom_BSplineSurface> gb_gcv_read_bspline_surface(GbReader& r){return occ::down_cast<Geom_BSplineSurface>(gt_surface(r));}
// gcv_to_bspline Parameterisation Curve: CurveToBSplineCurve.
void gb_gcv_to_bspline_case(std::istream& in){
 GbReader r{in};const Convert_ParameterisationType p=Convert_ParameterisationType(r.integer());
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);gb_gcv_bspline(GeomConvert::CurveToBSplineCurve(c,p)));gb_end();
}
// gcv_surf_to_bspline Surface: SurfaceToBSplineSurface.
void gb_gcv_surf_to_bspline_case(std::istream& in){
 GbReader r{in};GB_TRY(occ::handle<Geom_Surface> s=gt_surface(r);gb_gcv_bspline_surface(GeomConvert::SurfaceToBSplineSurface(s)));gb_end();
}
// gcv_split Mode From To Tol Same Curve: SplitBSplineCurve by knot indices (mode 0) or
// by parameters (mode 1).
void gb_gcv_split_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double a=r.real(),b=r.real(),tol=r.real();const bool same=r.flag();
 GB_TRY(occ::handle<Geom_BSplineCurve> c=gb_gcv_read_bspline(r);
  if(mode==0)gb_gcv_bspline(GeomConvert::SplitBSplineCurve(c,int(a),int(b),same));
  else gb_gcv_bspline(GeomConvert::SplitBSplineCurve(c,a,b,tol,same)));
 gb_end();
}
// gcv_split_surface Mode A B C D Tol USplit SameU SameV Surface: knot indices in both
// directions (0) or one (1), parameters in both directions (2) or one (3).
void gb_gcv_split_surface_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double a=r.real(),b=r.real(),c=r.real(),d=r.real(),tol=r.real();
 const bool usplit=r.flag(),su=r.flag(),sv=r.flag();
 GB_TRY(occ::handle<Geom_BSplineSurface> s=gb_gcv_read_bspline_surface(r);occ::handle<Geom_BSplineSurface> x;
  if(mode==0)x=GeomConvert::SplitBSplineSurface(s,int(a),int(b),int(c),int(d),su,sv);
  else if(mode==1)x=GeomConvert::SplitBSplineSurface(s,int(a),int(b),usplit,su);
  else if(mode==2)x=GeomConvert::SplitBSplineSurface(s,a,b,c,d,tol,su,sv);
  else x=GeomConvert::SplitBSplineSurface(s,a,b,usplit,tol,su);
  gb_gcv_bspline_surface(x));
 gb_end();
}
// gcv_knot_split Range Curve: NbSplits, SplitValue(0 .. NbSplits + 1), Splitting.
void gb_gcv_knot_split_case(std::istream& in){
 GbReader r{in};const int range=r.integer();
 GB_TRY(GeomConvert_BSplineCurveKnotSplitting s(gb_gcv_read_bspline(r),range);const int n=s.NbSplits();gb_int(n);
  for(int i=0;i<=n+1;++i){GB_TRY(gb_int(s.SplitValue(i)));}
  NCollection_Array1<int> v(1,n);s.Splitting(v);for(int i=1;i<=n;++i)gb_int(v(i)));
 gb_end();
}
// gcv_knot_split_surface URange VRange Surface: NbUSplits, NbVSplits, the split values
// (0 .. n + 1) of each direction, then Splitting.
void gb_gcv_knot_split_surface_case(std::istream& in){
 GbReader r{in};const int ur=r.integer(),vr=r.integer();
 GB_TRY(GeomConvert_BSplineSurfaceKnotSplitting s(gb_gcv_read_bspline_surface(r),ur,vr);const int nu=s.NbUSplits();gb_int(nu);
  int nv=0;GB_TRY(nv=s.NbVSplits();gb_int(nv));
  for(int i=0;i<=nu+1;++i){GB_TRY(gb_int(s.USplitValue(i)));}
  for(int i=0;i<=nv+1;++i){GB_TRY(gb_int(s.VSplitValue(i)));}
  NCollection_Array1<int> u(1,nu),v(1,nv);s.Splitting(u,v);for(int i=1;i<=nu;++i)gb_int(u(i));for(int i=1;i<=nv;++i)gb_int(v(i)));
 gb_end();
}
// gcv_bezier Mode U1 U2 Tol Curve: the whole curve (mode 0) or [U1, U2]; NbArcs, Knots,
// Arc(0 .. NbArcs + 1), then Arcs.
void gb_gcv_bezier_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),tol=r.real();
 GB_TRY(occ::handle<Geom_BSplineCurve> c=gb_gcv_read_bspline(r);
  std::unique_ptr<GeomConvert_BSplineCurveToBezierCurve> s(mode==0?new GeomConvert_BSplineCurveToBezierCurve(c):new GeomConvert_BSplineCurveToBezierCurve(c,u1,u2,tol));
  const int n=s->NbArcs();gb_int(n);NCollection_Array1<double> k(1,n+1);s->Knots(k);for(int i=1;i<=n+1;++i)gb_real(k(i));
  for(int i=0;i<=n+1;++i){GB_TRY(gb_gcv_bezier(s->Arc(i)));}
  NCollection_Array1<occ::handle<Geom_BezierCurve>> arcs(1,n);s->Arcs(arcs);for(int i=1;i<=n;++i)gb_gcv_bezier(arcs(i)));
 gb_end();
}
// gcv_bezier_surface Mode U1 U2 V1 V2 Tol Surface: the whole surface (mode 0) or the
// range; the patch counts, both knot sequences, Patch(0 .. n + 1, 1) and
// Patch(1, 0 .. m + 1), then Patches.
void gb_gcv_bezier_surface_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real(),tol=r.real();
 GB_TRY(occ::handle<Geom_BSplineSurface> s=gb_gcv_read_bspline_surface(r);
  std::unique_ptr<GeomConvert_BSplineSurfaceToBezierSurface> b(mode==0?new GeomConvert_BSplineSurfaceToBezierSurface(s):new GeomConvert_BSplineSurfaceToBezierSurface(s,u1,u2,v1,v2,tol));
  const int nu=b->NbUPatches(),nv=b->NbVPatches();gb_int(nu);gb_int(nv);
  NCollection_Array1<double> uk(1,nu+1),vk(1,nv+1);b->UKnots(uk);b->VKnots(vk);
  for(int i=1;i<=nu+1;++i)gb_real(uk(i));for(int i=1;i<=nv+1;++i)gb_real(vk(i));
  for(int i=0;i<=nu+1;++i){GB_TRY(gb_gcv_bezier_surface(b->Patch(i,1)));}
  for(int j=0;j<=nv+1;++j){GB_TRY(gb_gcv_bezier_surface(b->Patch(1,j)));}
  NCollection_Array2<occ::handle<Geom_BezierSurface>> p(1,nu,1,nv);b->Patches(p);
  for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)gb_gcv_bezier_surface(p(i,j)));
 gb_end();
}
// gcv_comp Parameterisation WithFirst N (Tol After WithRatio MinM Curve)...: the first
// curve through the constructor (WithFirst) or Add, the others through Add (each result
// printed), the B-spline, then Clear.
void gb_gcv_comp_case(std::istream& in){
 GbReader r{in};const Convert_ParameterisationType p=Convert_ParameterisationType(r.integer());const bool withFirst=r.flag();const int n=r.integer();
 GB_TRY(
  std::unique_ptr<GeomConvert_CompCurveToBSplineCurve> comp;
  for(int i=0;i<n;++i){
   const double tol=r.real();const bool after=r.flag(),ratio=r.flag();const int minM=r.integer();
   occ::handle<Geom_BoundedCurve> c=occ::down_cast<Geom_BoundedCurve>(gt_curve(r));
   if(i==0&&withFirst){comp.reset(new GeomConvert_CompCurveToBSplineCurve(c,p));continue;}
   if(!comp)comp.reset(new GeomConvert_CompCurveToBSplineCurve(p));
   GB_TRY(gb_flag(comp->Add(c,tol,after,ratio,minM)));
  }
  gb_gcv_bspline(comp->BSplineCurve());comp->Clear();gb_flag(comp->BSplineCurve().IsNull()));
 gb_end();
}
// The members of a CompBezierSurfacesToBSplineSurface, then the (degree-raised) patches.
void gb_gcv_comp_bezier_result(const GeomConvert_CompBezierSurfacesToBSplineSurface& c){
 gb_flag(c.IsDone());gb_int(c.NbUKnots());gb_int(c.NbUPoles());gb_int(c.NbVKnots());gb_int(c.NbVPoles());
 gb_int(c.UDegree());gb_int(c.VDegree());
 const NCollection_Array2<gp_Pnt>& p=c.Poles()->Array2();
 for(int i=p.LowerRow();i<=p.UpperRow();++i)for(int j=p.LowerCol();j<=p.UpperCol();++j)gb_pnt(p(i,j));
 for(double k:c.UKnots()->Array1())gb_real(k);for(double k:c.VKnots()->Array1())gb_real(k);
 for(int m:c.UMultiplicities()->Array1())gb_int(m);for(int m:c.VMultiplicities()->Array1())gb_int(m);
}
// gcv_comp_bezier Mode Tol RemoveKnots UCont VCont NU NV (NU+1 UKnots) (NV+1 VKnots)
// Surface...: the constructor of the patches (0), with a tolerance (1) or with knots
// and continuities (2); the result members and the patches afterwards.
void gb_gcv_comp_bezier_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double tol=r.real();const bool rk=r.flag();
 const GeomAbs_Shape uc=GeomAbs_Shape(r.integer()),vc=GeomAbs_Shape(r.integer());const int nu=r.integer(),nv=r.integer();
 NCollection_Array1<double> uk(1,nu+1),vk(1,nv+1);for(int i=1;i<=nu+1;++i)uk(i)=r.real();for(int i=1;i<=nv+1;++i)vk(i)=r.real();
 NCollection_Array2<occ::handle<Geom_BezierSurface>> b(1,nu,1,nv);
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)b(i,j)=occ::down_cast<Geom_BezierSurface>(gt_surface(r));
 GB_TRY(
  if(mode==0){GeomConvert_CompBezierSurfacesToBSplineSurface c(b);gb_gcv_comp_bezier_result(c);}
  else if(mode==1){GeomConvert_CompBezierSurfacesToBSplineSurface c(b,tol,rk);gb_gcv_comp_bezier_result(c);}
  else{GeomConvert_CompBezierSurfacesToBSplineSurface c(b,uk,vk,uc,vc,tol);gb_gcv_comp_bezier_result(c);});
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)gb_gcv_bezier_surface(b(i,j));
 gb_end();
}
// gcv_concat Kind Closed ClosedTol AngTol NTol Tol... NCurves Curve...: ConcatG1 (kind
// 0), ConcatC1 (1) or ConcatC1 with the angular tolerance (2); then the closed flag,
// the indices, the concatenated curves and the input array.
void gb_gcv_concat_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();bool closed=r.flag();const double ctol=r.real(),ang=r.real();
 const int nt=r.integer();NCollection_Array1<double> tols(0,nt-1);for(int i=0;i<nt;++i)tols(i)=r.real();
 const int nc=r.integer();NCollection_Array1<occ::handle<Geom_BSplineCurve>> curves(0,nc-1);for(int i=0;i<nc;++i)curves(i)=gb_gcv_read_bspline(r);
 occ::handle<NCollection_HArray1<int>> idx;occ::handle<NCollection_HArray1<occ::handle<Geom_BSplineCurve>>> res;
 GB_TRY(if(kind==0)GeomConvert::ConcatG1(curves,tols,res,closed,ctol);else if(kind==1)GeomConvert::ConcatC1(curves,tols,idx,res,closed,ctol);
  else GeomConvert::ConcatC1(curves,tols,idx,res,closed,ctol,ang));
 gb_flag(closed);
 if(idx.IsNull())gb_int(-1);else{gb_int(idx->Lower());gb_int(idx->Upper());for(int i=idx->Lower();i<=idx->Upper();++i)gb_int(idx->Value(i));}
 if(res.IsNull())gb_int(-1);else{gb_int(res->Lower());gb_int(res->Upper());for(int i=res->Lower();i<=res->Upper();++i)gb_gcv_bspline(res->Value(i));}
 for(int i=0;i<nc;++i)gb_gcv_bspline(curves(i));
 gb_end();
}
// gcv_c0c1 Mode Tol Ang Curve: C0BSplineToC1BSplineCurve (mode 0), then
// C0BSplineToArrayOfC1BSplineCurve with Tol (1) or with Ang and Tol (2).
void gb_gcv_c0c1_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double tol=r.real(),ang=r.real();
 GB_TRY(occ::handle<Geom_BSplineCurve> c=gb_gcv_read_bspline(r);
  if(mode==0){GeomConvert::C0BSplineToC1BSplineCurve(c,tol,ang);gb_gcv_bspline(c);}
  else{occ::handle<NCollection_HArray1<occ::handle<Geom_BSplineCurve>>> t;
   GB_TRY(if(mode==1)GeomConvert::C0BSplineToArrayOfC1BSplineCurve(c,t,tol);else GeomConvert::C0BSplineToArrayOfC1BSplineCurve(c,t,ang,tol));
   if(t.IsNull())gb_int(-1);else{gb_int(t->Lower());gb_int(t->Upper());for(int i=t->Lower();i<=t->Upper();++i)gb_gcv_bspline(t->Value(i));}});
 gb_end();
}
// gcv_approx_curve Adaptor Ranged U1 U2 Tol Order MaxSeg MaxDeg Curve: the curve or
// adaptor constructor; IsDone, HasResult, MaxError and the curve.
void gb_gcv_approx_curve_case(std::istream& in){
 GbReader r{in};const bool viaAdaptor=r.flag(),ranged=r.flag();const double u1=r.real(),u2=r.real(),tol=r.real();
 const GeomAbs_Shape order=GeomAbs_Shape(r.integer());const int seg=r.integer(),deg=r.integer();
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);std::unique_ptr<GeomConvert_ApproxCurve> a;
  if(viaAdaptor){occ::handle<GeomAdaptor_Curve> h=ranged?new GeomAdaptor_Curve(c,u1,u2):new GeomAdaptor_Curve(c);a.reset(new GeomConvert_ApproxCurve(occ::handle<Adaptor3d_Curve>(h),tol,order,seg,deg));}
  else a.reset(new GeomConvert_ApproxCurve(c,tol,order,seg,deg));
  gb_flag(a->IsDone());gb_flag(a->HasResult());gb_real(a->MaxError());gb_gcv_bspline(a->Curve()));
 gb_end();
}
// gcv_approx_surface Adaptor Tol UCont VCont MaxDegU MaxDegV MaxSeg Precis Surface: the
// surface or adaptor constructor; IsDone, HasResult, MaxError and the surface.
void gb_gcv_approx_surface_case(std::istream& in){
 GbReader r{in};const bool viaAdaptor=r.flag();const double tol=r.real();
 const GeomAbs_Shape uc=GeomAbs_Shape(r.integer()),vc=GeomAbs_Shape(r.integer());
 const int du=r.integer(),dv=r.integer(),seg=r.integer(),precis=r.integer();
 GB_TRY(occ::handle<Geom_Surface> s=gt_surface(r);std::unique_ptr<GeomConvert_ApproxSurface> a;
  if(viaAdaptor){occ::handle<GeomAdaptor_Surface> h=new GeomAdaptor_Surface(s);a.reset(new GeomConvert_ApproxSurface(occ::handle<Adaptor3d_Surface>(h),tol,uc,vc,du,dv,seg,precis));}
  else a.reset(new GeomConvert_ApproxSurface(s,tol,uc,vc,du,dv,seg,precis));
  gb_flag(a->IsDone());gb_flag(a->HasResult());gb_real(a->MaxError());gb_gcv_bspline_surface(a->Surface()));
 gb_end();
}
