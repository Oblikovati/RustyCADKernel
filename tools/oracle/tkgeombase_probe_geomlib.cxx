// TKGeomBase oracle: the static members of GeomLib (tkgeombase_probe.cxx), with the
// GeomTools builders and the printers of tkgeombase_probe_geomconvert.cxx. Output
// references start from the GB_GCV_UNSET sentinel, so an untouched output shows. The
// Rust twin is crates/tkgeombase/examples/gb_probe/geom_lib/statics.rs.
#include <Adaptor3d_CurveOnSurface.hxx>
#include <GeomLib.hxx>
#include <NCollection_LinearVector.hxx>
#include <gp_GTrsf2d.hxx>
occ::handle<Adaptor3d_CurveOnSurface> gb_gl_curve_on_surface(const occ::handle<Geom2d_Curve>& c,const occ::handle<Geom_Surface>& s){
 return new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(c),new GeomAdaptor_Surface(s));
}
// gl_to3d Ax2 Curve2d: To3d.
void gb_gl_to3d_case(std::istream& in){
 GbReader r{in};const gp_Ax2 a=r.ax2();GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);gb_gcv_curve(GeomLib::To3d(a,c)));gb_end();
}
// gl_gtransform A11 A12 A21 A22 T1 T2 Curve2d: GTransform by the general transformation
// of vectorial part A and translation T.
void gb_gl_gtransform_case(std::istream& in){
 GbReader r{in};gp_GTrsf2d g;
 for(int i=1;i<=2;++i)for(int j=1;j<=2;++j)g.SetValue(i,j,r.real());const double t1=r.real(),t2=r.real();g.SetValue(1,3,t1);g.SetValue(2,3,t2);
 GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);gb_gcv_curve2d(GeomLib::GTransform(c,g)));gb_end();
}
// gl_same_range Tol First Last RequestedFirst RequestedLast Curve2d: SameRange.
void gb_gl_same_range_case(std::istream& in){
 GbReader r{in};const double tol=r.real(),f=r.real(),l=r.real(),rf=r.real(),rl=r.real();
 GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);occ::handle<Geom2d_Curve> nc;GB_TRY(GeomLib::SameRange(tol,c,f,l,rf,rl,nc));gb_gcv_curve2d(nc));gb_end();
}
// gl_build_curve3d Tol First Last Continuity MaxDegree MaxSegment Curve2d Surface:
// BuildCurve3d on the adapted pcurve and surface; the curve and both deviations.
void gb_gl_build_curve3d_case(std::istream& in){
 GbReader r{in};const double tol=r.real(),f=r.real(),l=r.real();const GeomAbs_Shape cont=GeomAbs_Shape(r.integer());const int deg=r.integer(),seg=r.integer();
 GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);occ::handle<Geom_Surface> s=gt_surface(r);occ::handle<Adaptor3d_CurveOnSurface> cos=gb_gl_curve_on_surface(c,s);
  occ::handle<Geom_Curve> nc;double mx=GB_GCV_UNSET,av=GB_GCV_UNSET;GB_TRY(GeomLib::BuildCurve3d(tol,*cos,f,l,nc,mx,av,cont,deg,seg));
  gb_gcv_curve(nc);gb_real(mx);gb_real(av));
 gb_end();
}
// gl_adjust P1 P2 T1 T2 Curve: AdjustExtremity of the bounded curve.
void gb_gl_adjust_case(std::istream& in){
 GbReader r{in};const gp_Pnt p1=r.pnt(),p2=r.pnt();const gp_Vec t1=r.vec(),t2=r.vec();
 GB_TRY(occ::handle<Geom_BoundedCurve> c=occ::down_cast<Geom_BoundedCurve>(gt_curve(r));GB_TRY(GeomLib::AdjustExtremity(c,p1,p2,t1,t2));gb_gcv_curve(c));gb_end();
}
// gl_extend_curve Continuity After Point Curve: ExtendCurveToPoint.
void gb_gl_extend_curve_case(std::istream& in){
 GbReader r{in};const int cont=r.integer();const bool after=r.flag();const gp_Pnt p=r.pnt();
 GB_TRY(occ::handle<Geom_BoundedCurve> c=occ::down_cast<Geom_BoundedCurve>(gt_curve(r));GB_TRY(GeomLib::ExtendCurveToPoint(c,p,cont,after));gb_gcv_curve(c));gb_end();
}
// gl_extend_surf Length Continuity InU After Surface: ExtendSurfByLength.
void gb_gl_extend_surf_case(std::istream& in){
 GbReader r{in};const double len=r.real();const int cont=r.integer();const bool inU=r.flag(),after=r.flag();
 GB_TRY(occ::handle<Geom_BoundedSurface> s=occ::down_cast<Geom_BoundedSurface>(gt_surface(r));GB_TRY(GeomLib::ExtendSurfByLength(s,len,cont,inU,after));gb_gcv_surface(s));gb_end();
}
// gl_inertia Tol N Point...: Inertia (barycentre, directions, gaps) and AxeOfInertia.
void gb_gl_inertia_case(std::istream& in){
 GbReader r{in};const double tol=r.real();const int n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);for(int i=1;i<=n;++i)p(i)=r.pnt();
 gp_Pnt b;gp_Dir x,y;double gx=GB_GCV_UNSET,gy=GB_GCV_UNSET,gz=GB_GCV_UNSET;
 GB_TRY(GeomLib::Inertia(p,b,x,y,gx,gy,gz));gb_pnt(b);gb_dir(x);gb_dir(y);gb_real(gx);gb_real(gy);gb_real(gz);
 gp_Ax2 a;bool singular=false;GB_TRY(GeomLib::AxeOfInertia(p,a,singular,tol));gb_ax2(a);gb_flag(singular);gb_end();
}
void gb_gl_reals(const NCollection_Array1<double>& a){gb_int(a.Lower());gb_int(a.Upper());for(double v:a)gb_real(v);}
NCollection_Array1<double> gb_gl_read_reals(GbReader& r){
 const int lower=r.integer(),n=r.integer();if(n==0)return NCollection_Array1<double>();
 NCollection_Array1<double> a(lower,lower+n-1);for(int i=lower;i<lower+n;++i)a(i)=r.real();return a;
}
// gl_arrays Mode Count Array [Array Confusion Adjust]: RemovePointsFromArray (mode 0),
// DensifyArray1OfReal (1) or FuseIntervals (2); an array is Lower N values.
void gb_gl_arrays_case(std::istream& in){
 GbReader r{in};const int mode=r.integer(),count=r.integer();const NCollection_Array1<double> a=gb_gl_read_reals(r);
 if(mode==2){const NCollection_Array1<double> b=gb_gl_read_reals(r);const double conf=r.real();const bool adjust=r.flag();
  NCollection_LinearVector<double> f;f.Append(-1.);GB_TRY(GeomLib::FuseIntervals(a,b,f,conf,adjust));
  gb_int(int(f.Size()));for(double v:f)gb_real(v);gb_end();return;}
 NCollection_Array1<double> out;
 GB_TRY(if(mode==0)GeomLib::RemovePointsFromArray(count,a,out);else GeomLib::DensifyArray1OfReal(count,a,out);gb_gl_reals(out));gb_end();
}
// gl_eval_max Mode Tol Array Curve Curve: EvalMaxParametricDistance (mode 0) or
// EvalMaxDistanceAlongParameter (1) of the first curve from the second.
void gb_gl_eval_max_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double tol=r.real();const NCollection_Array1<double> p=gb_gl_read_reals(r);
 GB_TRY(occ::handle<Geom_Curve> a=gt_curve(r);occ::handle<Geom_Curve> b=gt_curve(r);GeomAdaptor_Curve ca(a),cb(b);double d=GB_GCV_UNSET;
  GB_TRY(if(mode==0)GeomLib::EvalMaxParametricDistance(ca,cb,tol,p,d);else GeomLib::EvalMaxDistanceAlongParameter(ca,cb,tol,p,d));gb_real(d));
 gb_end();
}
// gl_cancel_denom UDirection VDirection Surface: CancelDenominatorDerivative.
void gb_gl_cancel_denom_case(std::istream& in){
 GbReader r{in};const bool ud=r.flag(),vd=r.flag();
 GB_TRY(occ::handle<Geom_BSplineSurface> s=gb_gcv_read_bspline_surface(r);GB_TRY(GeomLib::CancelDenominatorDerivative(s,ud,vd));gb_gcv_bspline_surface(s));gb_end();
}
// gl_norm_estim U V Tol Surface: NormEstim status and normal.
void gb_gl_norm_estim_case(std::istream& in){
 GbReader r{in};const double u=r.real(),v=r.real(),tol=r.real();
 GB_TRY(occ::handle<Geom_Surface> s=gt_surface(r);gp_Dir n;gb_int(GeomLib::NormEstim(s,gp_Pnt2d(u,v),tol,n));gb_dir(n));gb_end();
}
// gl_is_closed Mode Param1 Param2 Tol Surface: IsClosed (mode 0), IsBSplUClosed (1),
// IsBSplVClosed (2), IsBzUClosed (3) or IsBzVClosed (4).
void gb_gl_is_closed_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double p1=r.real(),p2=r.real(),tol=r.real();
 GB_TRY(occ::handle<Geom_Surface> s=gt_surface(r);
  if(mode==0){bool u=false,v=false;GeomLib::IsClosed(s,tol,u,v);gb_flag(u);gb_flag(v);}
  else if(mode<=2){occ::handle<Geom_BSplineSurface> b=occ::down_cast<Geom_BSplineSurface>(s);gb_flag(mode==1?GeomLib::IsBSplUClosed(b,p1,p2,tol):GeomLib::IsBSplVClosed(b,p1,p2,tol));}
  else{occ::handle<Geom_BezierSurface> b=occ::down_cast<Geom_BezierSurface>(s);gb_flag(mode==3?GeomLib::IsBzUClosed(b,p1,p2,tol):GeomLib::IsBzVClosed(b,p1,p2,tol));});
 gb_end();
}
// gl_iso_line First Last Tol Explicit IsU Param IsForward Curve2d Surface: isIsoLine of
// the pcurve adapted on [First, Last], then buildC3dOnIsoLine with the detected (or,
// when Explicit, the given) isoline.
void gb_gl_iso_line_case(std::istream& in){
 GbReader r{in};const double f=r.real(),l=r.real(),tol=r.real();const bool expl=r.flag();bool isU=r.flag();double par=r.real();bool fwd=r.flag();
 GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);occ::handle<Geom_Surface> s=gt_surface(r);
  occ::handle<Adaptor2d_Curve2d> c2d=new Geom2dAdaptor_Curve(c,f,l);occ::handle<Adaptor3d_Surface> sa=new GeomAdaptor_Surface(s);
  bool dU=false,dF=false;double dP=GB_GCV_UNSET;const bool iso=GeomLib::isIsoLine(c2d,dU,dP,dF);gb_flag(iso);gb_flag(dU);gb_real(dP);gb_flag(dF);
  if(!expl){isU=dU;par=dP;fwd=dF;}
  if(iso||expl){GB_TRY(gb_gcv_curve(GeomLib::buildC3dOnIsoLine(c2d,sa,f,l,tol,isU,par,fwd)));});
 gb_end();
}
