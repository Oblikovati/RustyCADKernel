// ShapeExtend_ComplexCurve probe operations; the Rust twin is
// crates/tkshhealing/examples/sh_probe/complex_curve.rs.
//
// No OCCT class derives from the abstract ShapeExtend_ComplexCurve, so the probe defines
// one on both sides: segment i of n covers the global range [(i - 1) / n, i / n], mapped
// linearly onto the segment's own range, and the scale factor is n times that range.
//
// `sh_complex program n (curve)... preci nu (u)... trsf ...`: the segments are curves of
// the shape program's curve table; trsf 0 is none, 1 `dx dy dz` a translation, 2
// `x y z dx dy dz angle` a rotation. Prints the constant members, CheckConnectivity(preci)
// and IsClosed around it, the evaluators at each u (with DN of order 0 to 4), Period(),
// then Transform(trsf) and the points at each u again.
#include <ShapeExtend_ComplexCurve.hxx>
#include <gp_Ax1.hxx>
#include <gp_Trsf.hxx>
#include <cmath>
class ShProbeComplexCurve : public ShapeExtend_ComplexCurve{
public:
 explicit ShProbeComplexCurve(const std::vector<occ::handle<Geom_Curve>>& c):curves(c){}
 int NbCurves() const override{return int(curves.size());}
 const occ::handle<Geom_Curve>& Curve(const int i) const override{return curves.at(i-1);}
 int LocateParameter(const double U,double& UOut) const override{
  int n=NbCurves();double s=U*n;int i=int(std::floor(s))+1;if(i<1)i=1;if(i>n)i=n;
  double f=Curve(i)->FirstParameter(),l=Curve(i)->LastParameter();UOut=f+(s-(i-1))*(l-f);return i;}
 double LocalToGlobal(const int i,const double u) const override{
  double f=Curve(i)->FirstParameter(),l=Curve(i)->LastParameter();return ((i-1)+(u-f)/(l-f))/NbCurves();}
 double GetScaleFactor(const int i) const override{
  return NbCurves()*(Curve(i)->LastParameter()-Curve(i)->FirstParameter());}
 void Reverse() override{}
 occ::handle<Geom_Geometry> Copy() const override{return new ShProbeComplexCurve(curves);}
private:
 std::vector<occ::handle<Geom_Curve>> curves;};
void sh_cc_vec(const gp_Vec& v){ga_xyz(v.XYZ());}
void sh_cc_eval(const occ::handle<ShProbeComplexCurve>& c,double u){
 GA_TRY(ga_pnt(c->Value(u)))
 GA_TRY(Geom_Curve::ResD1 d=c->EvalD1(u);ga_pnt(d.Point);sh_cc_vec(d.D1))
 GA_TRY(Geom_Curve::ResD2 d=c->EvalD2(u);ga_pnt(d.Point);sh_cc_vec(d.D1);sh_cc_vec(d.D2))
 GA_TRY(Geom_Curve::ResD3 d=c->EvalD3(u);ga_pnt(d.Point);sh_cc_vec(d.D1);sh_cc_vec(d.D2);sh_cc_vec(d.D3))
 for(int n=0;n<=4;++n)GA_TRY(sh_cc_vec(c->EvalDN(u,n)))
 GA_TRY(double local;ga_int(c->LocateParameter(u,local));ga_real(local))}
gp_Trsf sh_cc_trsf(GaReader& r){
 gp_Trsf T;int kind=r.integer();
 if(kind==1){double x=r.real(),y=r.real();T.SetTranslation(gp_Vec(x,y,r.real()));}
 else if(kind==2){gp_Pnt p=r.pnt();gp_Dir d=r.dir();T.SetRotation(gp_Ax1(p,d),r.real());}
 return T;}
void sh_complex_case(std::istream& input){
 GaReader r{input};TpShapes t=tp_shapes(r);std::vector<occ::handle<Geom_Curve>> curves(r.integer());
 for(occ::handle<Geom_Curve>& c:curves)c=t.curves.at(r.integer());
 double preci=r.real();std::vector<double> us=r.reals();gp_Trsf T=sh_cc_trsf(r);
 occ::handle<ShProbeComplexCurve> c=new ShProbeComplexCurve(curves);
 ga_int(c->NbCurves());ga_real(c->FirstParameter());ga_real(c->LastParameter());ga_flag(c->IsPeriodic());
 ga_int(c->Continuity());for(int n=-1;n<=2;++n)ga_flag(c->IsCN(n));ga_real(c->ReversedParameter(0.3));
 ga_flag(c->IsClosed());GA_TRY(ga_flag(c->CheckConnectivity(preci)))ga_flag(c->IsClosed());
 for(int i=1;i<=c->NbCurves();++i){ga_real(c->GetScaleFactor(i));ga_real(c->LocalToGlobal(i,c->Curve(i)->FirstParameter()));}
 for(double u:us)sh_cc_eval(c,u);
 GA_TRY(ga_real(c->Period()))
 GA_TRY(c->Transform(T))
 for(double u:us)GA_TRY(ga_pnt(c->Value(u)))
 ga_flag(c->IsKind(STANDARD_TYPE(ShapeExtend_ComplexCurve)));ga_flag(c->IsKind(STANDARD_TYPE(Geom_Curve)));
 ga_end();}
