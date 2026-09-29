// IntCurve probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_curve.rs. A conic is read as eight numbers
// `kind x y dx dy sense r1 r2` (kind 0 line, 1 circle, 2 ellipse, 3 parabola, 4 hyperbola,
// on the frame (x, y, dx, dy, sense)); a domain as eight `kind p1 t1 p2 t2 closed z zp`
// (kind 0 infinite, 1 bounded, 2 first bound only, 3 last bound only, the points on the
// conic at the parameters, closed by SetEquivalentParameters(z, zp)).
#include <ElCLib.hxx>
#include <IntCurve_IConicTool.hxx>
#include <IntCurve_IntConicConic.hxx>
#include <IntCurve_IntImpConicParConic.hxx>
#include <IntCurve_MyImpParToolOfIntImpConicParConic.hxx>
#include <IntCurve_PConic.hxx>
#include <IntCurve_PConicTool.hxx>
#include <IntCurve_ProjectOnPConicTool.hxx>
#include <gp_Ax22d.hxx>
#include <gp_Circ2d.hxx>
#include <gp_Elips2d.hxx>
#include <gp_Hypr2d.hxx>
#include <gp_Lin2d.hxx>
#include <gp_Parab2d.hxx>
struct GaConic{int kind;gp_Pnt2d p;gp_Dir2d d;bool sense;double r1,r2;
 gp_Ax22d ax()const{return gp_Ax22d(p,d,sense);}
 gp_Lin2d lin()const{return gp_Lin2d(p,d);}
 gp_Circ2d circ()const{return gp_Circ2d(ax(),r1);}
 gp_Elips2d elips()const{return gp_Elips2d(ax(),r1,r2);}
 gp_Parab2d parab()const{return gp_Parab2d(ax(),r1);}
 gp_Hypr2d hypr()const{return gp_Hypr2d(ax(),r1,r2);}
 gp_Pnt2d value(double u)const{
  switch(kind){case 0:return ElCLib::Value(u,lin());case 1:return ElCLib::Value(u,circ());
   case 2:return ElCLib::Value(u,elips());case 3:return ElCLib::Value(u,parab());default:return ElCLib::Value(u,hypr());}
 }
};
GaConic ga_read_conic(GaReader& r){
 GaConic c;c.kind=r.integer();c.p=r.pnt2d();double dx=r.real(),dy=r.real();
 c.d=gp_Dir2d(dx,dy);c.sense=r.flag();c.r1=r.real();c.r2=r.real();return c;
}
struct GaDomainSpec{int kind;double p1,t1,p2,t2;bool closed;double z,zp;};
GaDomainSpec ga_read_domain(GaReader& r){
 GaDomainSpec s;s.kind=r.integer();s.p1=r.real();s.t1=r.real();s.p2=r.real();s.t2=r.real();
 s.closed=r.flag();s.z=r.real();s.zp=r.real();return s;
}
// The domain of a spec on a curve evaluated by `value`; SetEquivalentParameters may raise.
template<class F>IntRes2d_Domain ga_domain(const GaDomainSpec& s,F value){
 IntRes2d_Domain d;
 if(s.kind==1)d.SetValues(value(s.p1),s.p1,s.t1,value(s.p2),s.p2,s.t2);
 else if(s.kind==2)d.SetValues(value(s.p1),s.p1,s.t1,true);
 else if(s.kind==3)d.SetValues(value(s.p2),s.p2,s.t2,false);
 if(s.closed)d.SetEquivalentParameters(s.z,s.zp);
 return d;
}
IntRes2d_Domain ga_conic_domain(const GaDomainSpec& s,const GaConic& c){
 return ga_domain(s,[&](double u){return c.value(u);});
}
IntCurve_IConicTool ga_itool(const GaConic& c){
 switch(c.kind){case 0:return IntCurve_IConicTool(c.lin());case 1:return IntCurve_IConicTool(c.circ());
  case 2:return IntCurve_IConicTool(c.elips());case 3:return IntCurve_IConicTool(c.parab());
  case 4:return IntCurve_IConicTool(c.hypr());default:return IntCurve_IConicTool();}
}
IntCurve_PConic ga_pconic(const GaConic& c){
 switch(c.kind){case 0:return IntCurve_PConic(c.lin());case 1:return IntCurve_PConic(c.circ());
  case 2:return IntCurve_PConic(c.elips());case 3:return IntCurve_PConic(c.parab());default:return IntCurve_PConic(c.hypr());}
}
// The Perform overload of the (ordered) pair of conic kinds.
void ga_perform_pair(IntCurve_IntConicConic& i,const GaConic& a,const IntRes2d_Domain& da,
                     const GaConic& b,const IntRes2d_Domain& db,double tc,double t){
 switch(a.kind*10+b.kind){
  case 0:i.Perform(a.lin(),da,b.lin(),db,tc,t);break;case 1:i.Perform(a.lin(),da,b.circ(),db,tc,t);break;
  case 2:i.Perform(a.lin(),da,b.elips(),db,tc,t);break;case 3:i.Perform(a.lin(),da,b.parab(),db,tc,t);break;
  case 4:i.Perform(a.lin(),da,b.hypr(),db,tc,t);break;case 11:i.Perform(a.circ(),da,b.circ(),db,tc,t);break;
  case 12:i.Perform(a.circ(),da,b.elips(),db,tc,t);break;case 13:i.Perform(a.circ(),da,b.parab(),db,tc,t);break;
  case 14:i.Perform(a.circ(),da,b.hypr(),db,tc,t);break;case 22:i.Perform(a.elips(),da,b.elips(),db,tc,t);break;
  case 23:i.Perform(a.elips(),da,b.parab(),db,tc,t);break;case 24:i.Perform(a.elips(),da,b.hypr(),db,tc,t);break;
  case 33:i.Perform(a.parab(),da,b.parab(),db,tc,t);break;case 34:i.Perform(a.parab(),da,b.hypr(),db,tc,t);break;
  default:i.Perform(a.hypr(),da,b.hypr(),db,tc,t);break;
 }
}
// `conicA domainA conicB domainB tolconf tol reversed`: IntCurve_IntConicConic.
void ga_intcurve_conics_case(std::istream& input){
 GaReader r{input};GaConic a=ga_read_conic(r);GaDomainSpec sa=ga_read_domain(r);
 GaConic b=ga_read_conic(r);GaDomainSpec sb=ga_read_domain(r);double tc=r.real(),t=r.real();bool rev=r.flag();
 GA_TRY(IntRes2d_Domain da=ga_conic_domain(sa,a);IntRes2d_Domain db=ga_conic_domain(sb,b);
  IntCurve_IntConicConic inter;inter.SetReversedParameters(rev);ga_perform_pair(inter,a,da,b,db,tc,t);ga_inter(inter));
 ga_end();
}
// `conicA domainA conicB domainB tolconf tol reversed accuracy epsx`: IntImpConicParConic with
// the first conic implicit and the second parametric.
void ga_intcurve_imppar_case(std::istream& input){
 GaReader r{input};GaConic a=ga_read_conic(r);GaDomainSpec sa=ga_read_domain(r);
 GaConic b=ga_read_conic(r);GaDomainSpec sb=ga_read_domain(r);double tc=r.real(),t=r.real();bool rev=r.flag();
 int accuracy=r.integer();double epsx=r.real();
 GA_TRY(IntRes2d_Domain da=ga_conic_domain(sa,a);IntRes2d_Domain db=ga_conic_domain(sb,b);
  IntCurve_PConic pc=ga_pconic(b);pc.SetAccuracy(accuracy);pc.SetEpsX(epsx);
  IntCurve_IntImpConicParConic inter;inter.SetReversedParameters(rev);inter.Perform(ga_itool(a),da,pc,db,tc,t);
  ga_inter(inter));
 ga_end();
}
// Discards what the native code writes to std::cout (the "Erreur sur le type de la courbe"
// line of a default IntCurve_IConicTool) while it lives; the port does not print.
struct GaMute{std::ostringstream sink;std::streambuf* old;GaMute():old(std::cout.rdbuf(sink.rdbuf())){}~GaMute(){std::cout.rdbuf(old);}};
// `conic u x y`: every member of IntCurve_IConicTool (kind 9: the default tool), the D1/D2
// outputs first set to sentinels.
void ga_intcurve_itool_case(std::istream& input){
 GaReader r{input};GaConic c=ga_read_conic(r);double u=r.real();gp_Pnt2d p=r.pnt2d();
 IntCurve_IConicTool tool=ga_itool(c);IntCurve_IConicTool copy(tool);
 gp_Pnt2d v,q1(-7.,-8.),q2(-7.,-8.);gp_Vec2d t1(-9.,-10.),t2(-9.,-10.),n(-11.,-12.),g;double dist;
 {GaMute mute;v=copy.Value(u);tool.D1(u,q1,t1);tool.D2(u,q2,t2,n);dist=tool.Distance(p);g=tool.GradDistance(p);}
 ga_pnt2d(v);ga_pnt2d(q1);ga_v2(t1);ga_pnt2d(q2);ga_v2(t2);ga_v2(n);
 ga_real(dist);ga_v2(g);GA_TRY(ga_real(tool.FindParameter(p)));ga_end();
}
// `conic u x y low high accuracy epsx`: IntCurve_PConic, IntCurve_PConicTool (sentinel
// outputs) and both IntCurve_ProjectOnPConicTool::FindParameter.
void ga_intcurve_ptool_case(std::istream& input){
 GaReader r{input};GaConic c=ga_read_conic(r);double u=r.real();gp_Pnt2d p=r.pnt2d();
 double low=r.real(),high=r.real();int accuracy=r.integer();double epsx=r.real();
 IntCurve_PConic pc=ga_pconic(c);IntCurve_PConic copy(pc);
 ga_int(copy.TypeCurve());ga_real(copy.Param1());ga_real(copy.Param2());ga_int(copy.Accuracy());ga_real(copy.EpsX());
 ga_pnt2d(copy.Axis2().Location());ga_xy(copy.Axis2().XDirection().XY());ga_xy(copy.Axis2().YDirection().XY());
 pc.SetAccuracy(accuracy);pc.SetEpsX(epsx);
 ga_int(IntCurve_PConicTool::NbSamples(pc));ga_int(IntCurve_PConicTool::NbSamples(pc,low,high));
 ga_real(IntCurve_PConicTool::EpsX(pc));ga_pnt2d(IntCurve_PConicTool::Value(pc,u));
 gp_Pnt2d q(-7.,-8.);gp_Vec2d t(-9.,-10.),n(-11.,-12.);
 IntCurve_PConicTool::D1(pc,u,q,t);ga_pnt2d(q);ga_v2(t);q.SetCoord(-7.,-8.);t.SetCoord(-9.,-10.);
 IntCurve_PConicTool::D2(pc,u,q,t,n);ga_pnt2d(q);ga_v2(t);ga_v2(n);
 GA_TRY(ga_real(IntCurve_ProjectOnPConicTool::FindParameter(pc,p,1e-7)));
 GA_TRY(ga_real(IntCurve_ProjectOnPConicTool::FindParameter(pc,p,low,high,1e-7)));ga_end();
}
// `conicA conicB u`: IntCurve_MyImpParToolOfIntImpConicParConic at `u`.
void ga_intcurve_function_case(std::istream& input){
 GaReader r{input};GaConic a=ga_read_conic(r),b=ga_read_conic(r);double u=r.real();
 IntCurve_PConic pc=ga_pconic(b);IntCurve_MyImpParToolOfIntImpConicParConic f(ga_itool(a),pc);
 double v=-1.,d=-2.;ga_flag(f.Value(u,v));ga_real(v);ga_flag(f.Derivative(u,d));ga_real(d);
 v=-1.;d=-2.;ga_flag(f.Values(u,v,d));ga_real(v);ga_real(d);ga_end();
}
// `kind p1 t1 p2 t2 closed z zp a b ha hb`: Interval and PeriodicInterval on a domain and
// an interval (ha/hb: bounds of the Interval(a, ha, b, hb)).
void ga_intcurve_interval_case(std::istream& input){
 GaReader r{input};GaDomainSpec s=ga_read_domain(r);double a=r.real(),b=r.real();bool ha=r.flag(),hb=r.flag();
 auto pr=[](PeriodicInterval& i){ga_flag(i.IsNull());ga_real(i.Binf);ga_real(i.Bsup);ga_real(i.Length());};
 auto ir=[](Interval& i){ga_flag(i.IsNull);ga_flag(i.HasFirstBound);ga_flag(i.HasLastBound);ga_real(i.Binf);ga_real(i.Bsup);ga_real(i.Length());};
 GA_TRY(IntRes2d_Domain d=ga_domain(s,[](double u){return gp_Pnt2d(u,0.);});
  Interval id(d);ir(id);Interval ab(a,b);ir(ab);Interval hab(a,ha,b,hb);ir(hab);
  Interval w=hab.IntersectionWithBounded(ab);ir(w);Interval w2=id.IntersectionWithBounded(ab);ir(w2);Interval nul;ir(nul);
  PeriodicInterval pd(d);pr(pd);PeriodicInterval pab(a,b);pr(pab);PeriodicInterval c=pab;c.Complement();pr(c);
  PeriodicInterval n2=pd;n2.Normalize();pr(n2);PeriodicInterval sv;sv.SetValues(b,a);pr(sv);
  PeriodicInterval x=pab;PeriodicInterval f=n2.FirstIntersection(x);pr(f);pr(x);
  PeriodicInterval g=n2.SecondIntersection(x);pr(g);PeriodicInterval sn=pab;sn.SetNull();pr(sn));
 ga_end();
}
