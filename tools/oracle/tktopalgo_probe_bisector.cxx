// Bisector probe operations; the Rust twin is crates/tktopalgo/examples/tp_probe/bisector.rs.
//
// A 2D curve is read as tkgeomalgo_probe_geom2dint.cxx reads it (ga_curve2d; the Rust twin
// reuses crates/tkgeomalgo/examples/ga_probe/curve2d.rs):
// 0 line P D | 1 circle P XD sense R | 2 ellipse P XD sense A B | 3 hyperbola P XD sense A B |
// 4 parabola P XD sense F | 5 bezier rational n (x y w)*n | 6 bspline rational periodic degree
// nk (k m)*nk np (x y w)*np | 7 trimmed u1 u2 sense adjust <curve> | 8 offset d notcheck <curve>.
// The native Bisector_BisecAna prints "Not yet implemented" on std::cout for unsupported curve
// types; the port does not print, so every bisector computation runs with std::cout muted.
#include <Bisector.hxx>
#include <Bisector_Bisec.hxx>
#include <Bisector_BisecAna.hxx>
#include <Bisector_BisecCC.hxx>
#include <Bisector_BisecPC.hxx>
#include <Bisector_Curve.hxx>
#include <Bisector_FunctionH.hxx>
#include <Bisector_FunctionInter.hxx>
#include <Bisector_Inter.hxx>
#include <Bisector_PointOnBis.hxx>
#include <Bisector_PolyBis.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_CartesianPoint.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <IntRes2d_Domain.hxx>
#include <IntRes2d_IntersectionPoint.hxx>
#include <IntRes2d_IntersectionSegment.hxx>
#include <Standard_ProgramError.hxx>
#include <GeomAbs_JoinType.hxx>
// Runs `body` with std::cout sent to a sink (the native "Not yet implemented" messages).
template<class F> void tp_muted(F body){
 std::ostringstream sink;std::streambuf* old=std::cout.rdbuf(sink.rdbuf());
 try{body();}catch(...){std::cout.rdbuf(old);throw;}
 std::cout.rdbuf(old);}
gp_Dir2d tp_dir2d(GaReader& r){double x=r.real();return gp_Dir2d(x,r.real());}
gp_Ax2d tp_ax2d(GaReader& r){gp_Pnt2d p=r.pnt2d();return gp_Ax2d(p,tp_dir2d(r));}
// The curve reader of tkgeomalgo_probe_geom2dint.cxx (ga_curve2d), same format.
occ::handle<Geom2d_Curve> tp_curve2d(GaReader& r){
 int kind=r.integer();
 if(kind==0){gp_Pnt2d p=r.pnt2d();return new Geom2d_Line(p,tp_dir2d(r));}
 if(kind>=1&&kind<=4){gp_Ax2d a=tp_ax2d(r);bool sense=r.flag();double x=r.real();
  if(kind==1)return new Geom2d_Circle(a,x,sense);
  if(kind==4)return new Geom2d_Parabola(a,x,sense);
  double y=r.real();if(kind==2)return new Geom2d_Ellipse(a,x,y,sense);return new Geom2d_Hyperbola(a,x,y,sense);}
 if(kind==5){bool rational=r.flag();int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);
  for(int i=1;i<=n;++i){p(i)=r.pnt2d();w(i)=r.real();}
  if(rational)return new Geom2d_BezierCurve(p,w);return new Geom2d_BezierCurve(p);}
 if(kind==6){bool rational=r.flag(),periodic=r.flag();int degree=r.integer(),nk=r.integer();
  NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);for(int i=1;i<=nk;++i){k(i)=r.real();m(i)=r.integer();}
  int np=r.integer();NCollection_Array1<gp_Pnt2d> p(1,np);NCollection_Array1<double> w(1,np);
  for(int i=1;i<=np;++i){p(i)=r.pnt2d();w(i)=r.real();}
  if(rational)return new Geom2d_BSplineCurve(p,w,k,m,degree,periodic);return new Geom2d_BSplineCurve(p,k,m,degree,periodic);}
 if(kind==7){double u1=r.real(),u2=r.real();bool sense=r.flag(),adjust=r.flag();occ::handle<Geom2d_Curve> c=tp_curve2d(r);return new Geom2d_TrimmedCurve(c,u1,u2,sense,adjust);}
 if(kind==8){double d=r.real();bool nocheck=r.flag();occ::handle<Geom2d_Curve> c=tp_curve2d(r);return new Geom2d_OffsetCurve(c,d,nocheck);}
 throw Standard_ProgramError("unknown curve kind");}
void tp_vec(const gp_Vec2d& v){ga_xy(v.XY());}
GeomAbs_JoinType tp_join(int k){return k==0?GeomAbs_Arc:(k==1?GeomAbs_Tangent:GeomAbs_Intersection);}
// The sampled span of a trimmed bisector: an infinite bound is replaced 10 away from the other.
void tp_span(double& a,double& b){if(b>1e10)b=a+10.;if(b<-1e10)b=a-10.;if(a<-1e10)a=b-10.;if(a>1e10)a=b+10.;}
double tp_sample(double a,double b,int k){return a+(b-a)*(k*0.25);}
// The description of a bisector curve: its intervals and extensions.
void tp_bisector_curve(const occ::handle<Bisector_Curve>& bc){
 ga_int(bc->NbIntervals());
 for(int i=1;i<=bc->NbIntervals();++i){GA_TRY(ga_real(bc->IntervalFirst(i));ga_real(bc->IntervalLast(i)))}
 ga_flag(bc->IsExtendAtStart());ga_flag(bc->IsExtendAtEnd());}
// Five samples of a curve on [a, b]: the point, the parameter of the point on the bisector
// curve `bc` (when not null) and the first derivative.
void tp_samples(const occ::handle<Geom2d_Curve>& c,const occ::handle<Bisector_Curve>& bc,double a,double b){
 tp_span(a,b);
 for(int k=0;k<=4;++k){double u=tp_sample(a,b,k);
  GA_TRY(gp_Pnt2d p=c->Value(u);ga_pnt2d(p);if(!bc.IsNull()){GA_TRY(ga_real(bc->Parameter(p)))})
  GA_TRY(gp_Pnt2d p;gp_Vec2d v;c->D1(u,p,v);ga_pnt2d(p);tp_vec(v))}}
// A trimmed bisector (Bisector_Bisec::Value): its basis type, range, intervals and samples.
void tp_trimmed(const occ::handle<Geom2d_TrimmedCurve>& t){
 if(t.IsNull()){ga_int(0);return;}
 ga_int(1);occ::handle<Geom2d_Curve> basis=t->BasisCurve();ga_text(basis->DynamicType()->Name());
 ga_real(t->FirstParameter());ga_real(t->LastParameter());
 occ::handle<Bisector_Curve> bc=occ::down_cast<Bisector_Curve>(basis);
 tp_bisector_curve(bc);
 occ::handle<Bisector_BisecAna> ana=occ::down_cast<Bisector_BisecAna>(basis);
 if(!ana.IsNull())ga_text(ana->Geom2dCurve()->DynamicType()->Name());
 tp_samples(t,bc,t->FirstParameter(),t->LastParameter());}
// The point of passage and the two vectors: read, or taken at the common point of the
// elements as MAT2d_Tool2d takes them (the end tangent of the first, the reversed start
// tangent of the second).
struct TpPassage{gp_Pnt2d p;gp_Vec2d v1,v2;};
TpPassage tp_read_passage(GaReader& r){TpPassage s;s.p=r.pnt2d();gp_Pnt2d a=r.pnt2d();gp_Pnt2d b=r.pnt2d();
 s.v1=gp_Vec2d(a.XY());s.v2=gp_Vec2d(b.XY());return s;}
TpPassage tp_curves_passage(GaReader& r,bool automatic,const occ::handle<Geom2d_Curve>& c1,const occ::handle<Geom2d_Curve>& c2){
 if(!automatic)return tp_read_passage(r);
 TpPassage s;s.p=c2->Value(c2->FirstParameter());s.v1=c1->DN(c1->LastParameter(),1);s.v2=c2->DN(c2->FirstParameter(),1).Reversed();return s;}
TpPassage tp_curve_passage(GaReader& r,bool automatic,const occ::handle<Geom2d_Curve>& c,bool atEnd){
 if(!automatic)return tp_read_passage(r);
 TpPassage s;double u=atEnd?c->LastParameter():c->FirstParameter();s.p=c->Value(u);s.v1=c->DN(u,1);s.v2=s.v1.Reversed();return s;}
// `auto c1 c2 sense join tol oncurve [p v1 v2]`: Bisector_Bisec::Perform of two curves.
void tp_bisec_curves_case(std::istream& input){
 GaReader r{input};
 GA_TRY(bool automatic=r.flag();occ::handle<Geom2d_Curve> c1=tp_curve2d(r);occ::handle<Geom2d_Curve> c2=tp_curve2d(r);
  double sense=r.real();int join=r.integer();double tol=r.real();bool oncurve=r.flag();TpPassage s=tp_curves_passage(r,automatic,c1,c2);
  Bisector_Bisec b;tp_muted([&]{b.Perform(c1,c2,s.p,s.v1,s.v2,sense,tp_join(join),tol,oncurve);});tp_trimmed(b.Value()))
 ga_end();}
// `which auto c x y sense tol oncurve [p v1 v2]`: Bisector_Bisec::Perform of a curve and a point
// (which 0) or of a point and a curve (which 1).
void tp_bisec_curve_point_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int which=r.integer();bool automatic=r.flag();occ::handle<Geom2d_Curve> c=tp_curve2d(r);
  occ::handle<Geom2d_Point> pnt=new Geom2d_CartesianPoint(r.pnt2d());double sense=r.real(),tol=r.real();bool oncurve=r.flag();
  TpPassage s=tp_curve_passage(r,automatic,c,which==0);Bisector_Bisec b;
  tp_muted([&]{if(which==0)b.Perform(c,pnt,s.p,s.v1,s.v2,sense,tol,oncurve);else b.Perform(pnt,c,s.p,s.v1,s.v2,sense,tol,oncurve);});
  tp_trimmed(b.Value()))
 ga_end();}
// `x1 y1 x2 y2 p v1 v2 sense tol oncurve`: Bisector_Bisec::Perform of two points.
void tp_bisec_points_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Point> p1=new Geom2d_CartesianPoint(r.pnt2d());occ::handle<Geom2d_Point> p2=new Geom2d_CartesianPoint(r.pnt2d());
  TpPassage s=tp_read_passage(r);double sense=r.real(),tol=r.real();bool oncurve=r.flag();
  Bisector_Bisec b;tp_muted([&]{b.Perform(p1,p2,s.p,s.v1,s.v2,sense,tol,oncurve);});tp_trimmed(b.Value()))
 ga_end();}
// The members of an analytic bisector after a Perform.
void tp_ana_members(const occ::handle<Bisector_BisecAna>& a){
 GA_TRY(occ::handle<Geom2d_Curve> g=a->Geom2dCurve();ga_text(g->DynamicType()->Name()))
 GA_TRY(ga_real(a->ParameterOfStartPoint());ga_real(a->ParameterOfEndPoint()))
 GA_TRY(ga_real(a->FirstParameter());ga_real(a->LastParameter()))
 tp_bisector_curve(a);GA_TRY(ga_real(a->IntervalFirst(2)))
 GA_TRY(ga_flag(a->IsClosed()));GA_TRY(ga_flag(a->IsPeriodic()));GA_TRY(ga_int(a->Continuity()));GA_TRY(ga_flag(a->IsCN(2)))
 GA_TRY(ga_real(a->ReversedParameter(0.5)))
 tp_samples(a,a,a->FirstParameter(),a->LastParameter());
 GA_TRY(occ::handle<Bisector_BisecAna> c=occ::down_cast<Bisector_BisecAna>(a->Copy());ga_real(c->FirstParameter());ga_real(c->LastParameter()))
 ga_captured([&]{a->Dump(0,2);});
 GA_TRY(a->Reverse();ga_real(a->FirstParameter());ga_real(a->LastParameter()))}
// `which ...`: Bisector_BisecAna::Perform (which 0 curves as tp_bisec_curves, 1 curve-point,
// 2 point-curve as tp_bisec_curve_point without `which`, 3 points as tp_bisec_points) and its
// members.
void tp_bisec_ana_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int which=r.integer();occ::handle<Bisector_BisecAna> a=new Bisector_BisecAna();
  if(which==0){bool automatic=r.flag();occ::handle<Geom2d_Curve> c1=tp_curve2d(r);occ::handle<Geom2d_Curve> c2=tp_curve2d(r);
   double sense=r.real();int join=r.integer();double tol=r.real();bool oncurve=r.flag();TpPassage s=tp_curves_passage(r,automatic,c1,c2);
   tp_muted([&]{a->Perform(c1,c2,s.p,s.v1,s.v2,sense,tp_join(join),tol,oncurve);});}
  else if(which==3){occ::handle<Geom2d_Point> p1=new Geom2d_CartesianPoint(r.pnt2d());occ::handle<Geom2d_Point> p2=new Geom2d_CartesianPoint(r.pnt2d());
   TpPassage s=tp_read_passage(r);double sense=r.real(),tol=r.real();bool oncurve=r.flag();
   tp_muted([&]{a->Perform(p1,p2,s.p,s.v1,s.v2,sense,tol,oncurve);});}
  else{bool automatic=r.flag();occ::handle<Geom2d_Curve> c=tp_curve2d(r);occ::handle<Geom2d_Point> pnt=new Geom2d_CartesianPoint(r.pnt2d());
   double sense=r.real(),tol=r.real();bool oncurve=r.flag();TpPassage s=tp_curve_passage(r,automatic,c,which==1);
   tp_muted([&]{if(which==1)a->Perform(c,pnt,s.p,s.v1,s.v2,sense,tol,oncurve);else a->Perform(pnt,c,s.p,s.v1,s.v2,sense,tol,oncurve);});}
  tp_ana_members(a))
 ga_end();}
// The polygon of a curve-curve bisector.
void tp_polygon(const Bisector_PolyBis& poly){
 ga_int(poly.Length());
 for(int i=1;i<=poly.Length();++i){const Bisector_PointOnBis& p=poly.Value(i);
  ga_real(p.ParamOnC1());ga_real(p.ParamOnC2());ga_real(p.ParamOnBis());ga_real(p.Distance());ga_pnt2d(p.Point());}}
// The values of a curve-curve bisector at `u`: ValueAndDist and ValueByInt.
void tp_cc_values(const occ::handle<Bisector_BisecCC>& b,double u){
 GA_TRY(double u1=0,u2=0,d=0;gp_Pnt2d p=b->ValueAndDist(u,u1,u2,d);ga_pnt2d(p);ga_real(u1);ga_real(u2);ga_real(d))
 GA_TRY(double u1=0,u2=0,d=0;gp_Pnt2d p=b->ValueByInt(u,u1,u2,d);ga_pnt2d(p);ga_real(u1);ga_real(u2);ga_real(d))
 GA_TRY(gp_Pnt2d p;gp_Vec2d v1,v2;b->D2(u,p,v1,v2);ga_pnt2d(p);tp_vec(v1);tp_vec(v2))
 GA_TRY(gp_Pnt2d p;gp_Vec2d v1,v2,v3;b->D3(u,p,v1,v2,v3);tp_vec(v3))
 GA_TRY(tp_vec(b->DN(u,1)));GA_TRY(tp_vec(b->DN(u,4)))}
// `c1 c2 side1 side2 ox oy distmax`: Bisector_BisecCC and its members.
void tp_bisec_cc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c1=tp_curve2d(r);occ::handle<Geom2d_Curve> c2=tp_curve2d(r);double s1=r.real(),s2=r.real();
  gp_Pnt2d o=r.pnt2d();double dmax=r.real();occ::handle<Bisector_BisecCC> b=new Bisector_BisecCC(c1,c2,s1,s2,o,dmax);
  ga_flag(b->IsEmpty());tp_polygon(b->Polygon());tp_bisector_curve(b);
  if(b->NbIntervals()>0){double a=b->FirstParameter(),e=b->LastParameter();ga_real(a);ga_real(e);
   GA_TRY(ga_flag(b->IsClosed()));ga_flag(b->IsPeriodic());ga_int(b->Continuity());ga_int(b->IntervalContinuity());GA_TRY(ga_flag(b->IsCN(1)))
   GA_TRY(ga_real(b->ReversedParameter(a)));ga_real(b->LinkBisCurve(a));ga_real(b->LinkCurveBis(a));
   tp_samples(b,b,a,e);double lo=a,hi=e;tp_span(lo,hi);for(int k=0;k<=4;++k)tp_cc_values(b,tp_sample(lo,hi,k));
   tp_cc_values(b,a-1.);tp_cc_values(b,e+1.);
   GA_TRY(occ::handle<Bisector_BisecCC> g=b->ChangeGuide();tp_polygon(g->Polygon());ga_real(g->FirstParameter());ga_real(g->LastParameter()))
   GA_TRY(occ::handle<Bisector_BisecCC> c=occ::down_cast<Bisector_BisecCC>(b->Copy());ga_real(c->FirstParameter());tp_polygon(c->Polygon()))
   ga_captured([&]{b->Dump(0,1);});}
  GA_TRY(b->Reverse()))
 ga_end();}
// The members of a curve-point bisector.
void tp_pc_members(const occ::handle<Bisector_BisecPC>& b){
 ga_flag(b->IsEmpty());tp_bisector_curve(b);
 if(b->NbIntervals()==0)return;
 double a=b->FirstParameter(),e=b->LastParameter();ga_real(a);ga_real(e);
 GA_TRY(ga_flag(b->IsClosed()));ga_flag(b->IsPeriodic());ga_int(b->Continuity());ga_int(b->IntervalContinuity());GA_TRY(ga_flag(b->IsCN(1)))
 GA_TRY(ga_real(b->ReversedParameter(a)));ga_real(b->LinkBisCurve(a));ga_real(b->LinkCurveBis(a));
 tp_samples(b,b,a,e);double lo=a,hi=e;tp_span(lo,hi);
 for(int k=0;k<=4;++k){double u=tp_sample(lo,hi,k);GA_TRY(ga_real(b->Distance(u)))
  GA_TRY(gp_Pnt2d p;gp_Vec2d v1,v2;b->D2(u,p,v1,v2);ga_pnt2d(p);tp_vec(v1);tp_vec(v2))
  GA_TRY(gp_Pnt2d p;gp_Vec2d v1,v2,v3;b->D3(u,p,v1,v2,v3);ga_pnt2d(p);tp_vec(v3))
  GA_TRY(tp_vec(b->DN(u,2)));GA_TRY(tp_vec(b->DN(u,4)))}
 for(double u:{a-1.,e+1.}){GA_TRY(gp_Pnt2d p;gp_Vec2d v;b->D1(u,p,v);ga_pnt2d(p);tp_vec(v))}
 GA_TRY(occ::handle<Bisector_BisecPC> c=occ::down_cast<Bisector_BisecPC>(b->Copy());ga_real(c->FirstParameter());ga_real(c->LastParameter()))
 ga_captured([&]{b->Dump(0,1);});
 GA_TRY(b->Reverse())}
// `mode c x y side distmax|umin umax`: Bisector_BisecPC(Cu, P, Side, DistMax) (mode 0) or
// Bisector_BisecPC(Cu, P, Side, UMin, UMax) (mode 1) and its members.
void tp_bisec_pc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int mode=r.integer();occ::handle<Geom2d_Curve> c=tp_curve2d(r);gp_Pnt2d p=r.pnt2d();double side=r.real();
  occ::handle<Bisector_BisecPC> b;if(mode==0){double dmax=r.real();b=new Bisector_BisecPC(c,p,side,dmax);}
  else{double u1=r.real(),u2=r.real();b=new Bisector_BisecPC(c,p,side,u1,u2);}
  tp_pc_members(b))
 ga_end();}
// One bisector of an intersection: `kind` 0 curves, 1 curve-point, 2 point-curve, then the
// arguments of tp_bisec_curves or tp_bisec_curve_point (without `which`), then the domain:
// the trimmed range capped 100 away from its start, with the tolerance `tol`.
struct TpBisecDomain{Bisector_Bisec b;IntRes2d_Domain d;};
TpBisecDomain tp_read_bisec(GaReader& r){
 TpBisecDomain s;int kind=r.integer();bool automatic=r.flag();
 if(kind==0){occ::handle<Geom2d_Curve> c1=tp_curve2d(r);occ::handle<Geom2d_Curve> c2=tp_curve2d(r);double sense=r.real();int join=r.integer();
  double tol=r.real();bool oncurve=r.flag();TpPassage p=tp_curves_passage(r,automatic,c1,c2);
  tp_muted([&]{s.b.Perform(c1,c2,p.p,p.v1,p.v2,sense,tp_join(join),tol,oncurve);});}
 else{occ::handle<Geom2d_Curve> c=tp_curve2d(r);occ::handle<Geom2d_Point> pnt=new Geom2d_CartesianPoint(r.pnt2d());double sense=r.real(),tol=r.real();
  bool oncurve=r.flag();TpPassage p=tp_curve_passage(r,automatic,c,kind==1);
  tp_muted([&]{if(kind==1)s.b.Perform(c,pnt,p.p,p.v1,p.v2,sense,tol,oncurve);else s.b.Perform(pnt,c,p.p,p.v1,p.v2,sense,tol,oncurve);});}
 double tol=r.real();occ::handle<Geom2d_TrimmedCurve> t=s.b.Value();double a=t->FirstParameter(),e=t->LastParameter();
 if(e>a+100.)e=a+100.;s.d.SetValues(t->Value(a),a,tol,t->Value(e),e,tol);return s;}
// `bisec1 bisec2 tolconf tol comun`: Bisector_Inter of two bisectors on their domains.
void tp_bisec_inter_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpBisecDomain b1=tp_read_bisec(r);TpBisecDomain b2=tp_read_bisec(r);double tc=r.real(),t=r.real();bool comun=r.flag();
  Bisector_Inter i;tp_muted([&]{i.Perform(b1.b,b1.d,b2.b,b2.d,tc,t,comun);});ga_flag(i.IsDone());
  if(i.IsDone()){ga_int(i.NbPoints());for(int k=1;k<=i.NbPoints();++k){const IntRes2d_IntersectionPoint& p=i.Point(k);
    ga_pnt2d(p.Value());ga_real(p.ParamOnFirst());ga_real(p.ParamOnSecond());}
   ga_int(i.NbSegments());for(int k=1;k<=i.NbSegments();++k){const IntRes2d_IntersectionSegment& s=i.Segment(k);
    ga_flag(s.HasFirstPoint());ga_flag(s.HasLastPoint());if(s.HasFirstPoint())ga_pnt2d(s.FirstPoint().Value());}})
 ga_end();}
// `c x y tx ty u`: Bisector_FunctionH at `u`.
void tp_bisec_function_h_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=tp_curve2d(r);gp_Pnt2d p=r.pnt2d();gp_Pnt2d t=r.pnt2d();double u=r.real();
  Bisector_FunctionH h(c,p,gp_Vec2d(t.XY()));double f=0,d=0;ga_flag(h.Value(u,f));ga_real(f);ga_flag(h.Derivative(u,d));ga_real(d);
  f=0;d=0;ga_flag(h.Values(u,f,d));ga_real(f);ga_real(d))
 ga_end();}
// `c x1 y1 x2 y2 side u`: Bisector_FunctionInter of `c` and its bisectors with two points.
void tp_bisec_function_inter_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=tp_curve2d(r);gp_Pnt2d p1=r.pnt2d(),p2=r.pnt2d();double side=r.real(),u=r.real();
  occ::handle<Bisector_BisecPC> b1=new Bisector_BisecPC(c,p1,side,c->FirstParameter(),c->LastParameter());
  occ::handle<Bisector_BisecPC> b2=new Bisector_BisecPC(c,p2,side,c->FirstParameter(),c->LastParameter());
  Bisector_FunctionInter fi(c,b1,b2);double f=0,d=0;ga_flag(fi.Value(u,f));ga_real(f);ga_flag(fi.Values(u,f,d));ga_real(f);ga_real(d);
  Bisector_FunctionInter fe;fe.Perform(c,b2,b1);d=0;ga_flag(fe.Derivative(u,d));ga_real(d))
 ga_end();}
// `n (u1 u2 ub d x y)*n u`: Bisector_PolyBis and Bisector_PointOnBis.
void tp_bisec_poly_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int n=r.integer();Bisector_PolyBis poly;ga_flag(poly.IsEmpty());
  for(int i=0;i<n;++i){double u1=r.real(),u2=r.real(),ub=r.real(),d=r.real();gp_Pnt2d p=r.pnt2d();poly.Append(Bisector_PointOnBis(u1,u2,ub,d,p));}
  double u=r.real();ga_int(poly.Length());ga_flag(poly.IsEmpty());ga_real(poly.First().ParamOnBis());ga_real(poly.Last().ParamOnBis());
  if(n>=2)ga_int(poly.Interval(u));gp_Trsf2d t;t.SetRotation(gp_Pnt2d(1.,2.),0.3);poly.Transform(t);tp_polygon(poly);
  Bisector_PointOnBis q;ga_flag(q.IsInfinite());q.IsInfinite(true);q.ParamOnC1(u);q.ParamOnC2(2.*u);q.ParamOnBis(3.*u);q.Distance(4.);
  q.Point(gp_Pnt2d(u,u));ga_flag(q.IsInfinite());ga_real(q.ParamOnC2());ga_captured([&]{q.Dump();}))
 ga_end();}
// `c sign`: Bisector::IsConvex.
void tp_bisec_is_convex_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=tp_curve2d(r);double s=r.real();ga_flag(Bisector::IsConvex(c,s)))
 ga_end();}
