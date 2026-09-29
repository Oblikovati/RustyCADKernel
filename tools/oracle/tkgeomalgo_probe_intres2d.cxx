// IntRes2d probe operations and the printers of 2D intersection results shared by the
// IntImpParGen, IntCurve, Geom2dInt and Geom2dAPI probes (tkgeomalgo_probe.cxx); the Rust
// twin is crates/tkgeomalgo/examples/ga_probe/int_res2d.rs.
#include <IntRes2d_Domain.hxx>
#include <IntRes2d_Intersection.hxx>
#include <IntRes2d_IntersectionPoint.hxx>
#include <IntRes2d_IntersectionSegment.hxx>
#include <IntRes2d_Transition.hxx>
// A transition: position, type, then the tangency (not for Undecided) and, for Touch, the
// situation and the relative orientation.
void ga_trans(const IntRes2d_Transition& t){
 ga_int(t.PositionOnCurve());ga_int(t.TransitionType());
 if(t.TransitionType()!=IntRes2d_Undecided)ga_flag(t.IsTangent());
 if(t.TransitionType()==IntRes2d_Touch){ga_int(t.Situation());ga_flag(t.IsOpposite());}
}
void ga_ipoint(const IntRes2d_IntersectionPoint& p){
 ga_pnt2d(p.Value());ga_real(p.ParamOnFirst());ga_real(p.ParamOnSecond());
 ga_trans(p.TransitionOfFirst());ga_trans(p.TransitionOfSecond());
}
void ga_iseg(const IntRes2d_IntersectionSegment& s){
 ga_flag(s.IsOpposite());ga_flag(s.HasFirstPoint());if(s.HasFirstPoint())ga_ipoint(s.FirstPoint());
 ga_flag(s.HasLastPoint());if(s.HasLastPoint())ga_ipoint(s.LastPoint());
}
// A whole result: done, then the points and the segments.
void ga_inter(const IntRes2d_Intersection& r){
 ga_flag(r.IsDone());if(!r.IsDone())return;
 ga_int(r.NbPoints());for(int i=1;i<=r.NbPoints();i++)ga_ipoint(r.Point(i));
 ga_int(r.NbSegments());for(int i=1;i<=r.NbSegments();i++)ga_iseg(r.Segment(i));
}
// `u1 u2 x y pos1 type1 tangent1 situ1 oppos1 pos2 ...`: a point with its transitions (type 2
// is Touch, 3 Undecided).
IntRes2d_Transition ga_read_trans(GaReader& r){
 int pos=r.integer(),type=r.integer();bool tangent=r.flag();int situ=r.integer();bool oppos=r.flag();
 if(type==IntRes2d_Touch)return IntRes2d_Transition(tangent,IntRes2d_Position(pos),IntRes2d_Situation(situ),oppos);
 if(type==IntRes2d_Undecided)return IntRes2d_Transition(IntRes2d_Position(pos));
 return IntRes2d_Transition(tangent,IntRes2d_Position(pos),IntRes2d_TypeTrans(type));
}
IntRes2d_IntersectionPoint ga_read_ipoint(GaReader& r){
 double u1=r.real(),u2=r.real();gp_Pnt2d p=r.pnt2d();
 IntRes2d_Transition t1=ga_read_trans(r),t2=ga_read_trans(r);
 return IntRes2d_IntersectionPoint(p,u1,u2,t1,t2,false);
}
// Exposes the protected members of IntRes2d_Intersection, as the algorithms deriving from it do.
struct GaIntersection:public IntRes2d_Intersection{
 void insert(const IntRes2d_IntersectionPoint& p){Insert(p);}
 void add_point(const IntRes2d_IntersectionPoint& p){Append(p);}
 void add_segment(const IntRes2d_IntersectionSegment& s){Append(s);}
 void merge(const GaIntersection& o,double f1,double l1,double f2,double l2){Append(o,f1,l1,f2,l2);}
 void set_done(){done=true;}
};
// `n point...`: each point inserted, then the resulting list.
void ga_intres2d_insert_case(std::istream& input){
 GaReader r{input};GaIntersection inter;int n=r.integer();
 GA_TRY(for(int i=0;i<n;i++)inter.insert(ga_read_ipoint(r));inter.set_done();ga_inter(inter));ga_end();
}
// `f1 l1 f2 l2 np point... ns (oppos point point)...` twice: two composite parts appended in
// turn to an empty result with the bounds of the composite curves.
void ga_read_part(GaReader& r,GaIntersection& part){
 int np=r.integer();for(int i=0;i<np;i++)part.add_point(ga_read_ipoint(r));
 int ns=r.integer();
 for(int i=0;i<ns;i++){bool oppos=r.flag();IntRes2d_IntersectionPoint a=ga_read_ipoint(r),b=ga_read_ipoint(r);
  part.add_segment(IntRes2d_IntersectionSegment(a,b,oppos,false));}
 part.set_done();
}
void ga_intres2d_append_case(std::istream& input){
 GaReader r{input};double f1=r.real(),l1=r.real(),f2=r.real(),l2=r.real();
 GaIntersection first,second,merged;ga_read_part(r,first);ga_read_part(r,second);
 GA_TRY(merged.merge(first,f1,l1,f2,l2);merged.merge(second,f1,l1,f2,l2);ga_inter(merged));ga_end();
}
// `kind p1 t1 p2 t2 closed z zp point`: the domain accessors, each raising or not.
void ga_intres2d_domain_case(std::istream& input){
 GaReader r{input};int kind=r.integer();double p1=r.real(),t1=r.real(),p2=r.real(),t2=r.real();
 bool closed=r.flag();double z=r.real(),zp=r.real();gp_Pnt2d a(p1,-p1),b(p2,-p2);
 IntRes2d_Domain d;
 if(kind==1)d.SetValues(a,p1,t1,b,p2,t2);else if(kind==2)d.SetValues(a,p1,t1,true);else if(kind==3)d.SetValues(b,p2,t2,false);
 GA_TRY(if(closed)d.SetEquivalentParameters(z,zp));
 ga_flag(d.HasFirstPoint());ga_flag(d.HasLastPoint());ga_flag(d.IsClosed());
 GA_TRY(ga_real(d.FirstParameter()));GA_TRY(ga_pnt2d(d.FirstPoint()));GA_TRY(ga_real(d.FirstTolerance()));
 GA_TRY(ga_real(d.LastParameter()));GA_TRY(ga_pnt2d(d.LastPoint()));GA_TRY(ga_real(d.LastTolerance()));
 double e1=-1.,e2=-1.;d.EquivalentParameters(e1,e2);ga_real(e1);ga_real(e2);ga_end();
}
