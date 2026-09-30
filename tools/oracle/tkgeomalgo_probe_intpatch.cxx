// IntPatch record probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_patch.rs. The records (IntPatch_Point, the walking
// line of IntPatch_TheIWalking, the path points and segments of IntPatch_TheSOnBounds) are
// edited by small command scripts and printed member by member. Every request builds the
// same fixture of arcs and vertices, printed by their rank in it. Needs the readers of
// tkgeomalgo_probe_intwalk.cxx and tkgeomalgo_probe_iwalk.cxx (ga_iw_pnt2s,
// ga_iw_line_report).
#include <Adaptor3d_HVertex.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2d_Line.hxx>
#include <IntPatch_Point.hxx>
#include <IntPatch_TheIWLineOfTheIWalking.hxx>
#include <IntPatch_ThePathPointOfTheSOnBounds.hxx>
#include <IntPatch_TheSegmentOfTheSOnBounds.hxx>
#include <IntSurf_PathPoint.hxx>
#include <IntSurf_Transition.hxx>
#include <NCollection_IncAllocator.hxx>
#include <cstdio>
#include <unistd.h>
// The arcs `k = 1..3` (the vertical lines u = k over v in [0, 10]) and the vertices `k = 1..3`
// (at (k, 0.5), tolerance 1e-7 k) of every request; rank 0 is the null handle.
struct GaIpFixture{
 std::vector<occ::handle<Adaptor2d_Curve2d>> arcs;std::vector<occ::handle<Adaptor3d_HVertex>> vtx;
 GaIpFixture(){arcs.push_back(nullptr);vtx.push_back(nullptr);
  for(int k=1;k<=3;++k){arcs.push_back(new Geom2dAdaptor_Curve(new Geom2d_Line(gp_Pnt2d(k,0),gp_Dir2d(0,1)),0.,10.));
   vtx.push_back(new Adaptor3d_HVertex(gp_Pnt2d(k,0.5),TopAbs_FORWARD,1.e-7*k));}}
 const occ::handle<Adaptor2d_Curve2d>& arc(int k)const{return arcs[(k<0||k>3)?0:k];}
 const occ::handle<Adaptor3d_HVertex>& vertex(int k)const{return vtx[(k<0||k>3)?0:k];}
 int arc_rank(const occ::handle<Adaptor2d_Curve2d>& a)const{for(int k=0;k<=3;++k)if(a==arcs[k])return k;return -1;}
 int vertex_rank(const occ::handle<Adaptor3d_HVertex>& v)const{for(int k=0;k<=3;++k)if(v==vtx[k])return k;return -1;}
};
// The transition of code `c`: 0 undecided, 1 IN, 2 OUT, 3 tangent IN, 4 tangent OUT, 5 TOUCH
// inside, 6 TOUCH outside opposite, 7 TOUCH unknown.
IntSurf_Transition ga_ip_transition(int c){
 switch(c){case 1:return IntSurf_Transition(false,IntSurf_In);case 2:return IntSurf_Transition(false,IntSurf_Out);
  case 3:return IntSurf_Transition(true,IntSurf_In);case 4:return IntSurf_Transition(true,IntSurf_Out);
  case 5:return IntSurf_Transition(true,IntSurf_Inside,false);case 6:return IntSurf_Transition(true,IntSurf_Outside,true);
  case 7:return IntSurf_Transition(false,IntSurf_Unknown,false);default:return IntSurf_Transition();}
}
void ga_ip_trans(const IntSurf_Transition& t){
 ga_int(t.TransitionType());GA_TRY(ga_flag(t.IsTangent()));GA_TRY(ga_int(t.Situation()));GA_TRY(ga_flag(t.IsOpposite()));}
// One command of a point script (see ga_ip_point_case).
void ga_ip_point_command(GaReader& r,const GaIpFixture& fx,IntPatch_Point& p,int c){
 switch(c){
  case 1:{gp_Pnt x=r.pnt();double tol=r.real();p.SetValue(x,tol,r.flag());break;}
  case 2:p.SetValue(r.pnt());break;
  case 3:{gp_Pnt x=r.pnt();double a=r.real(),b=r.real(),e=r.real(),d=r.real();
   IntSurf_PntOn2S q;q.SetValue(x,a,b,e,d);p.SetValue(q);break;}
  case 4:p.SetTolerance(r.real());break;
  case 5:{double a=r.real(),b=r.real(),e=r.real();p.SetParameters(a,b,e,r.real());break;}
  case 6:p.SetParameter(r.real());break;
  case 7:{bool first=r.flag();p.SetVertex(first,fx.vertex(r.integer()));break;}
  case 8:{bool first=r.flag();int a=r.integer();double prm=r.real();IntSurf_Transition tl=ga_ip_transition(r.integer());
   p.SetArc(first,fx.arc(a),prm,tl,ga_ip_transition(r.integer()));break;}
  case 9:p.SetMultiple(r.flag());break;
  default:p.ReverseTransition();break;
 }
}
// A point built by `n (command args)*`.
IntPatch_Point ga_ip_read_point(GaReader& r,const GaIpFixture& fx){
 IntPatch_Point p;int n=r.integer();for(int i=0;i<n;++i)ga_ip_point_command(r,fx,p,r.integer());return p;}
void ga_ip_point(const GaIpFixture& fx,const IntPatch_Point& p){
 ga_pnt(p.Value());ga_real(p.ParameterOnLine());ga_real(p.Tolerance());ga_flag(p.IsTangencyPoint());ga_flag(p.IsMultiple());
 double u,v;p.ParametersOnS1(u,v);ga_real(u);ga_real(v);p.ParametersOnS2(u,v);ga_real(u);ga_real(v);
 ga_iw_pnt2s(p.PntOn2S());double a,b,c,d;p.Parameters(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);
 ga_flag(p.IsOnDomS1());ga_flag(p.IsVertexOnS1());GA_TRY(ga_int(fx.vertex_rank(p.VertexOnS1())));
 GA_TRY(ga_int(fx.arc_rank(p.ArcOnS1())));GA_TRY(ga_ip_trans(p.TransitionLineArc1()));GA_TRY(ga_ip_trans(p.TransitionOnS1()));
 GA_TRY(ga_real(p.ParameterOnArc1()));
 ga_flag(p.IsOnDomS2());ga_flag(p.IsVertexOnS2());GA_TRY(ga_int(fx.vertex_rank(p.VertexOnS2())));
 GA_TRY(ga_int(fx.arc_rank(p.ArcOnS2())));GA_TRY(ga_ip_trans(p.TransitionLineArc2()));GA_TRY(ga_ip_trans(p.TransitionOnS2()));
 GA_TRY(ga_real(p.ParameterOnArc2()));
}
// Runs `body` with the process standard output captured (the IntPatch Dump members mix
// printf and std::cout), then prints the text and the raised exception type, if any.
template<class F> void ga_ip_captured(F body){
 std::cout.flush();fflush(stdout);int saved=dup(1);FILE* tmp=tmpfile();dup2(fileno(tmp),1);std::string error;
 try{body();}catch(const Standard_Failure& failure){error=failure.ExceptionType();}
 std::cout.flush();fflush(stdout);dup2(saved,1);close(saved);
 long n=ftell(tmp);rewind(tmp);std::string text(n>0?n:0,'\0');if(n>0&&fread(&text[0],1,n,tmp)!=size_t(n))text.clear();fclose(tmp);
 ga_text(text);if(!error.empty())std::cout<<error<<' ';}
// `dump point-script`: a point and its printed members; its Dump when `dump` (only for
// points without arcs or vertices, whose addresses would be printed).
void ga_ip_point_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;bool dump=r.flag();
 GA_TRY(IntPatch_Point p=ga_ip_read_point(r,fx);ga_ip_point(fx,p);IntPatch_Point q(p);q.ReverseTransition();ga_ip_point(fx,q);
  if(dump)ga_ip_captured([&]{p.Dump();}));
 ga_end();
}
// `n (command args)*` on a walking line of IntPatch_TheIWalking: 1 point, 2 status first,
// 3 status first with a path point, 4 status first-last, 5 status last, 6 status last with
// a path point, 7 passing index, 8 tangent vector, 9/10 tangency at the ends, 11 reverse,
// 12 cut; then the line and its out-of-range reads.
void ga_ip_iwline_case(std::istream& input){
 GaReader r{input};occ::handle<IntPatch_TheIWLineOfTheIWalking> l=new IntPatch_TheIWLineOfTheIWalking(new NCollection_IncAllocator());
 int n=r.integer();
 for(int i=0;i<n;++i){int c=r.integer();GA_TRY(
  if(c==1){gp_Pnt x=r.pnt();double a=r.real(),b=r.real(),e=r.real(),d=r.real();IntSurf_PntOn2S q;q.SetValue(x,a,b,e,d);l->AddPoint(q);}
  else if(c==2){bool cl=r.flag();l->AddStatusFirst(cl,r.flag());}
  else if(c==3){bool cl=r.flag(),hf=r.flag();int k=r.integer();gp_Pnt x=r.pnt();double u=r.real();l->AddStatusFirst(cl,hf,k,IntSurf_PathPoint(x,u,r.real()));}
  else if(c==4){bool cl=r.flag(),hf=r.flag();l->AddStatusFirstLast(cl,hf,r.flag());}
  else if(c==5)l->AddStatusLast(r.flag());
  else if(c==6){bool hl=r.flag();int k=r.integer();gp_Pnt x=r.pnt();double u=r.real();l->AddStatusLast(hl,k,IntSurf_PathPoint(x,u,r.real()));}
  else if(c==7)l->AddIndexPassing(r.integer());
  else if(c==8){gp_XYZ v=r.xyz();l->SetTangentVector(gp_Vec(v),r.integer());}
  else if(c==9)l->SetTangencyAtBegining(r.flag());
  else if(c==10)l->SetTangencyAtEnd(r.flag());
  else if(c==11)l->Reverse();
  else l->Cut(r.integer()));}
 ga_iw_line_report(l);ga_int(l->Line()->NbPoints());
 GA_TRY(l->Value(0));GA_TRY(l->Value(l->NbPoints()+1));GA_TRY(int a,b;l->PassingPoint(0,a,b));
 GA_TRY(int a,b;l->PassingPoint(l->NbPassingPoint()+1,a,b));
 ga_end();
}
void ga_ip_path_point(const GaIpFixture& fx,const IntPatch_ThePathPointOfTheSOnBounds& p){
 ga_pnt(p.Value());ga_real(p.Tolerance());ga_flag(p.IsNew());ga_int(fx.arc_rank(p.Arc()));ga_real(p.Parameter());
 GA_TRY(ga_int(fx.vertex_rank(p.Vertex())));}
// `x y z tol vertex arc prm x2 y2 z2 tol2 arc2 prm2 segarc first last`: the path points of
// both constructors and setters, and a segment with the limit points `first`/`last` (0 none,
// 1 the vertex point, 2 the new point).
void ga_ip_pathpoint_case(std::istream& input){
 GaReader r{input};GaIpFixture fx;gp_Pnt x=r.pnt();double tol=r.real();int v=r.integer(),a=r.integer();double prm=r.real();
 gp_Pnt x2=r.pnt();double tol2=r.real();int a2=r.integer();double prm2=r.real();int sa=r.integer(),f=r.integer(),l=r.integer();
 IntPatch_ThePathPointOfTheSOnBounds d,p1(x,tol,fx.vertex(v),fx.arc(a),prm),p2(x2,tol2,fx.arc(a2),prm2);
 ga_ip_path_point(fx,d);ga_ip_path_point(fx,p1);ga_ip_path_point(fx,p2);
 IntPatch_ThePathPointOfTheSOnBounds q=p1;q.SetValue(x2,tol2,fx.arc(a2),prm2);ga_ip_path_point(fx,q);
 q.SetValue(x,tol,fx.vertex(v),fx.arc(a),prm);ga_ip_path_point(fx,q);
 IntPatch_TheSegmentOfTheSOnBounds s;ga_int(fx.arc_rank(s.Curve()));ga_flag(s.HasFirstPoint());ga_flag(s.HasLastPoint());
 GA_TRY(ga_ip_path_point(fx,s.FirstPoint()));GA_TRY(ga_ip_path_point(fx,s.LastPoint()));
 s.SetValue(fx.arc(sa));if(f)s.SetLimitPoint(f==1?p1:p2,true);if(l)s.SetLimitPoint(l==1?p1:p2,false);
 ga_int(fx.arc_rank(s.Curve()));ga_flag(s.HasFirstPoint());ga_flag(s.HasLastPoint());
 GA_TRY(ga_ip_path_point(fx,s.FirstPoint()));GA_TRY(ga_ip_path_point(fx,s.LastPoint()));
 s.SetValue(fx.arc(sa));ga_flag(s.HasFirstPoint());ga_flag(s.HasLastPoint());GA_TRY(ga_ip_path_point(fx,s.FirstPoint()));
 ga_end();
}
