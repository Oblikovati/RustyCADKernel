// TKGeomBase oracle: the GeomConvert canonical recognizers (CurveToAnaCurve,
// SurfToAnaSurf), the least-squares distance functions and GeomConvert_Units
// (tkgeombase_probe.cxx), with the printers of tkgeombase_probe_geomconvert.cxx. The
// Rust twin is crates/tkgeombase/examples/gb_probe/geom_convert/ana.rs.
#include <GeomConvert_CurveToAnaCurve.hxx>
#include <GeomConvert_FuncConeLSDist.hxx>
#include <GeomConvert_FuncCylinderLSDist.hxx>
#include <GeomConvert_FuncSphereLSDist.hxx>
#include <GeomConvert_SurfToAnaSurf.hxx>
#include <GeomConvert_Units.hxx>
#include <math_Vector.hxx>
// The in-out outputs start from these sentinels, so an output a member leaves
// untouched shows.
constexpr double GB_GCV_UNSET=-7.25;
// gcv_ana_curve Mode ConvType Target Tol F L Curve: mode 0 converts with
// CurveToAnaCurve (type and target set, then Init resets the gap); modes 1 .. 4 call
// ComputeCurve, ComputeLine, ComputeCircle and ComputeEllipse; the result, range and gap.
void gb_gcv_ana_curve_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const GeomConvert_ConvType ct=GeomConvert_ConvType(r.integer());
 const GeomAbs_CurveType target=GeomAbs_CurveType(r.integer());const double tol=r.real(),f=r.real(),l=r.real();
 GB_TRY(occ::handle<Geom_Curve> c=gt_curve(r);double cf=GB_GCV_UNSET,cl=GB_GCV_UNSET,gap=GB_GCV_UNSET;occ::handle<Geom_Curve> res;
  if(mode==0){GeomConvert_CurveToAnaCurve conv(c);conv.SetConvType(ct);conv.SetTarget(target);gb_int(conv.GetConvType());gb_int(conv.GetTarget());
   GB_TRY(gb_flag(conv.ConvertToAnalytical(tol,res,f,l,cf,cl)));gb_real(conv.Gap());conv.Init(c);gap=conv.Gap();}
  else if(mode==1){GB_TRY(res=GeomConvert_CurveToAnaCurve::ComputeCurve(c,tol,f,l,cf,cl,gap,ct,target));}
  else if(mode==2){GB_TRY(res=GeomConvert_CurveToAnaCurve::ComputeLine(c,tol,f,l,cf,cl,gap));}
  else if(mode==3){GB_TRY(res=GeomConvert_CurveToAnaCurve::ComputeCircle(c,tol,f,l,cf,cl,gap));}
  else{GB_TRY(res=GeomConvert_CurveToAnaCurve::ComputeEllipse(c,tol,f,l,cf,cl,gap));}
  gb_gcv_curve(res);gb_real(cf);gb_real(cl);gb_real(gap));
 gb_end();
}
// gcv_ana_empty Tol: a converter without a curve (null result, untouched range).
void gb_gcv_ana_empty_case(std::istream& in){
 GbReader r{in};const double tol=r.real();GeomConvert_CurveToAnaCurve conv;occ::handle<Geom_Curve> res;double cf=GB_GCV_UNSET,cl=GB_GCV_UNSET,gap=GB_GCV_UNSET;
 gb_real(conv.Gap());gb_int(conv.GetConvType());gb_int(conv.GetTarget());
 GB_TRY(gb_flag(conv.ConvertToAnalytical(tol,res,0.,1.,cf,cl)));gb_gcv_curve(res);gb_real(cf);gb_real(cl);
 GB_TRY(res=GeomConvert_CurveToAnaCurve::ComputeCurve(res,tol,0.,1.,cf,cl,gap));gb_gcv_curve(res);gb_real(cf);gb_real(cl);gb_real(gap);
 GB_TRY(res=GeomConvert_CurveToAnaCurve::ComputeLine(res,tol,0.5,2.,cf,cl,gap));gb_gcv_curve(res);gb_real(cf);gb_real(cl);gb_real(gap);gb_end();
}
// gcv_is_linear Tol N Point...: IsLinear and its deviation.
void gb_gcv_is_linear_case(std::istream& in){
 GbReader r{in};const double tol=r.real();const int n=r.integer();NCollection_Array1<gp_Pnt> p(1,std::max(n,1));
 for(int i=1;i<=n;++i)p(i)=r.pnt();double dev=GB_GCV_UNSET;
 if(n==0){NCollection_Array1<gp_Pnt> e;GB_TRY(gb_flag(GeomConvert_CurveToAnaCurve::IsLinear(e,tol,dev)));}
 else{GB_TRY(gb_flag(GeomConvert_CurveToAnaCurve::IsLinear(p,tol,dev)));}
 gb_real(dev);gb_end();
}
// gcv_get_line P1 P2: GetLine and the parameters of both points.
void gb_gcv_get_line_case(std::istream& in){
 GbReader r{in};const gp_Pnt p1=r.pnt(),p2=r.pnt();double cf=GB_GCV_UNSET,cl=GB_GCV_UNSET;
 GB_TRY(gb_lin(GeomConvert_CurveToAnaCurve::GetLine(p1,p2,cf,cl)));gb_real(cf);gb_real(cl);gb_end();
}
// gcv_get_circle P0 P1 P2: GetCircle and the circle (a default circle when untouched).
void gb_gcv_get_circle_case(std::istream& in){
 GbReader r{in};const gp_Pnt p0=r.pnt(),p1=r.pnt(),p2=r.pnt();gp_Circ c;
 GB_TRY(gb_flag(GeomConvert_CurveToAnaCurve::GetCircle(c,p0,p1,p2)));gb_circ(c);gb_end();
}
// gcv_ana_surf Mode ConvType Target Tol U1 U2 V1 V2 Surface: SurfToAnaSurf with the
// type and target set (skipped when ConvType is negative), converted on the surface
// bounds (mode 0) or on the range (mode 1); the result, the gap and IsCanonical of both.
void gb_gcv_ana_surf_case(std::istream& in){
 GbReader r{in};const int mode=r.integer(),ct=r.integer();const GeomAbs_SurfaceType target=GeomAbs_SurfaceType(r.integer());
 const double tol=r.real(),u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();
 GB_TRY(occ::handle<Geom_Surface> s=gt_surface(r);GeomConvert_SurfToAnaSurf conv(s);gb_real(conv.Gap());
  if(ct>=0){conv.SetConvType(GeomConvert_ConvType(ct));conv.SetTarget(target);}
  occ::handle<Geom_Surface> res;GB_TRY(res=mode==0?conv.ConvertToAnalytical(tol):conv.ConvertToAnalytical(tol,u1,u2,v1,v2));
  gb_gcv_surface(res);gb_real(conv.Gap());gb_flag(GeomConvert_SurfToAnaSurf::IsCanonical(s));gb_flag(GeomConvert_SurfToAnaSurf::IsCanonical(res)));
 gb_end();
}
// gcv_ana_surf_empty Tol: a converter without a surface (the bounded conversion
// raises from its adaptor; the unbounded one would dereference null natively).
void gb_gcv_ana_surf_empty_case(std::istream& in){
 GbReader r{in};const double tol=r.real();GeomConvert_SurfToAnaSurf conv;gb_real(conv.Gap());
 GB_TRY(gb_gcv_surface(conv.ConvertToAnalytical(tol,0.,1.,0.,1.)));
 gb_flag(GeomConvert_SurfToAnaSurf::IsCanonical(occ::handle<Geom_Surface>()));gb_end();
}
// gcv_same_surf Tol Surface Surface: IsSame.
void gb_gcv_same_surf_case(std::istream& in){
 GbReader r{in};const double tol=r.real();
 GB_TRY(occ::handle<Geom_Surface> a=gt_surface(r);occ::handle<Geom_Surface> b=gt_surface(r);gb_flag(GeomConvert_SurfToAnaSurf::IsSame(a,b,tol)));gb_end();
}
void gb_gcv_vector(const math_Vector& v){for(int i=v.Lower();i<=v.Upper();++i)gb_real(v(i));}
// gcv_ls_dist Kind Setters Dir N Point... NX X...: the cone (0), cylinder (1) or sphere
// (2) function, default-constructed then set (Setters) or constructed from the points;
// NbVariables, Value, and for the gradient functions Gradient and Values, each followed
// by the gradient vector.
void gb_gcv_ls_dist_case(std::istream& in){
 GbReader r{in};const int kind=r.integer();const bool setters=r.flag();const gp_Dir d=r.dir();const int n=r.integer();
 occ::handle<NCollection_HArray1<gp_XYZ>> pts=new NCollection_HArray1<gp_XYZ>(1,n);for(int i=1;i<=n;++i)pts->SetValue(i,r.xyz());
 const int nx=r.integer();math_Vector x(1,nx);for(int i=1;i<=nx;++i)x(i)=r.real();
 math_Vector g(1,4,9.5);double f=GB_GCV_UNSET;
 if(kind==0){GeomConvert_FuncConeLSDist fn(pts,d);if(setters){GeomConvert_FuncConeLSDist e;e.SetPoints(pts);e.SetDir(d);fn=e;}
  gb_int(fn.NbVariables());GB_TRY(gb_flag(fn.Value(x,f)));gb_real(f);}
 else if(kind==1){GeomConvert_FuncCylinderLSDist fn(pts,d);if(setters){GeomConvert_FuncCylinderLSDist e;e.SetPoints(pts);e.SetDir(d);fn=e;}
  gb_int(fn.NbVariables());GB_TRY(gb_flag(fn.Value(x,f)));gb_real(f);
  GB_TRY(gb_flag(fn.Gradient(x,g)));gb_gcv_vector(g);GB_TRY(gb_flag(fn.Values(x,f,g)));gb_real(f);gb_gcv_vector(g);}
 else{GeomConvert_FuncSphereLSDist fn(pts);if(setters){GeomConvert_FuncSphereLSDist e;e.SetPoints(pts);fn=e;}
  gb_int(fn.NbVariables());GB_TRY(gb_flag(fn.Value(x,f)));gb_real(f);
  GB_TRY(gb_flag(fn.Gradient(x,g)));gb_gcv_vector(g);GB_TRY(gb_flag(fn.Values(x,f,g)));gb_real(f);gb_gcv_vector(g);}
 gb_end();
}
// gcv_units Mode LengthFactor FactorRadianDegree Curve2d Surface: RadianToDegree (0),
// DegreeToRadian (1) or MirrorPCurve (2) of the pcurve.
void gb_gcv_units_case(std::istream& in){
 GbReader r{in};const int mode=r.integer();const double lf=r.real(),fr=r.real();
 GB_TRY(occ::handle<Geom2d_Curve> c=gt_curve2d(r);occ::handle<Geom_Surface> s=gt_surface(r);
  if(mode==0)gb_gcv_curve2d(GeomConvert_Units::RadianToDegree(c,s,lf,fr));
  else if(mode==1)gb_gcv_curve2d(GeomConvert_Units::DegreeToRadian(c,s,lf,fr));
  else gb_gcv_curve2d(GeomConvert_Units::MirrorPCurve(c)));
 gb_end();
}
