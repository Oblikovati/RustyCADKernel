// IntPatch polyhedron and polygon probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_patch_poly.rs: IntPatch_Polyhedron with its tool,
// IntPatch_PolyhedronBVH, IntPatch_BVHTraversal, IntPatch_InterferencePolyhedron,
// IntPatch_PolyLine, IntPatch_PolyArc and IntPatch_PrmPrmIntersection_T3Bits. Needs the
// readers of tkgeomalgo_probe_intcurvesurface.cxx, tkgeomalgo_probe_intf.cxx
// (ga_interference_print), tkgeomalgo_probe_intpatch.cxx and _intpatch_lines.cxx and
// _intpatch_tools.cxx (ga_ip_arc).
#include <IntPatch_BVHTraversal.hxx>
#include <IntPatch_InterferencePolyhedron.hxx>
#include <IntPatch_PolyArc.hxx>
#include <IntPatch_PolyLine.hxx>
#include <IntPatch_Polyhedron.hxx>
#include <IntPatch_PolyhedronBVH.hxx>
#include <IntPatch_PolyhedronTool.hxx>
#include <IntPatch_PrmPrmIntersection_T3Bits.hxx>
#include <Bnd_Box2d.hxx>
// A polyhedron `surface auto` (the automatic subdivision) or `surface 0 nbu nbv`.
IntPatch_Polyhedron* ga_ip_polyhedron(GaReader& r){
 occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);if(r.flag())return new IntPatch_Polyhedron(s);
 int nbu=r.integer();return new IntPatch_Polyhedron(s,nbu,r.integer());}
void ga_ip_box3(const Bnd_Box& b){ga_flag(b.IsVoid());GA_TRY(double a,c,d,e,f,g;b.Get(a,c,d,e,f,g);ga_real(a);ga_real(c);ga_real(d);ga_real(e);ga_real(f);ga_real(g));}
void ga_ip_polyh_report(const IntPatch_Polyhedron& p,int nconnex){
 typedef IntPatch_PolyhedronTool T;int nu,nv;p.Size(nu,nv);ga_int(nu);ga_int(nv);ga_int(p.NbTriangles());ga_int(p.NbPoints());
 ga_real(p.DeflectionOverEstimation());ga_real(T::DeflectionOverEstimation(p));ga_ip_box3(p.Bounding());ga_ip_box3(T::Bounding(p));
 ga_pnt(p.Point(0));
 for(int i=1;i<=p.NbPoints();++i){double u,v;ga_pnt(p.Point(i,u,v));ga_real(u);ga_real(v);p.Parameters(i,u,v);ga_real(u);ga_real(v);
  gp_Pnt q;p.Point(i,q);ga_pnt(q);ga_pnt(T::Point(p,i));}
 const occ::handle<NCollection_HArray1<Bnd_Box>>& boxes=T::ComponentsBounding(p);ga_int(boxes->Length());
 for(int t=1;t<=p.NbTriangles();++t){int a,b,c;p.Triangle(t,a,b,c);ga_int(a);ga_int(b);ga_int(c);T::Triangle(p,t,a,b,c);ga_int(a+b+c);
  ga_ip_box3(boxes->Value(t));gp_XYZ n(7,7,7);double d=12345.;p.PlaneEquation(t,n,d);ga_xyz(n);ga_real(d);
  gp_Pnt g((p.Point(a).XYZ()+p.Point(b).XYZ()+p.Point(c).XYZ())/3.);ga_flag(p.Contain(t,g));ga_flag(p.Contain(t,gp_Pnt(1e3,1e3,1e3)));}
 for(int t=0;t<=std::min(nconnex,p.NbTriangles());++t){int a=1,b=2,c=3;if(t>0)p.Triangle(t,a,b,c);int v[4]={a,b,c,0};
  for(int i=0;i<3;++i)for(int j=0;j<4;++j){if(i==j)continue;GA_TRY(int tc=-5,op=-5;int res=p.TriConnex(t,v[i],v[j],tc,op);ga_int(res);ga_int(tc);ga_int(op);
   int tc2=-5,op2=-5;ga_int(T::TriConnex(p,t,v[i],v[j],tc2,op2)));}}
}
// `polyhedron nconnex deflection`: every member of the polyhedron and its tool (TriConnex on
// the first `nconnex` triangles and on the unknown triangle 0), then a new deflection with
// the boxes filled again.
void ga_ip_polyh_case(std::istream& input){
 GaReader r{input};
 GA_TRY(std::unique_ptr<IntPatch_Polyhedron> p(ga_ip_polyhedron(r));int nconnex=r.integer();double defl=r.real();
  ga_ip_polyh_report(*p,nconnex);p->Point(gp_Pnt(1,2,3),1,1,0.5,0.5);ga_pnt(p->Point(1));
  p->DeflectionOverEstimation(defl);ga_real(p->DeflectionOverEstimation());ga_ip_box3(p->Bounding());p->FillBounding();
  for(int t=1;t<=p->NbTriangles();++t)ga_ip_box3(p->ComponentsBounding()->Value(t)));
 ga_end();
}
void ga_ip_bvh_report(const IntPatch_PolyhedronBVH& s){
 ga_flag(s.IsInitialized());ga_int(s.Size());
 for(int i=-1;i<=s.Size();++i){ga_int(s.OriginalIndex(i));BVH_Box<double,3> b=s.Box(i);ga_flag(b.IsValid());
  if(b.IsValid()){ga_real(b.CornerMin().x());ga_real(b.CornerMin().y());ga_real(b.CornerMin().z());
   ga_real(b.CornerMax().x());ga_real(b.CornerMax().y());ga_real(b.CornerMax().z());}
  for(int a=0;a<=3;++a)ga_real(s.Center(i,a));}
}
void ga_ip_pairs(const IntPatch_BVHTraversal& t){
 ga_int(int(t.Pairs().Size()));for(const auto& p:t.Pairs()){ga_int(p.First);ga_int(p.Second);}}
// `polyhedron1 polyhedron2`: the BVH sets of both polyhedra before and after the build, the
// pairs of both, the self pairs of the first, and an empty and a cleared set.
void ga_ip_bvh_case(std::istream& input){
 GaReader r{input};
 GA_TRY(std::unique_ptr<IntPatch_Polyhedron> p1(ga_ip_polyhedron(r));std::unique_ptr<IntPatch_Polyhedron> p2(ga_ip_polyhedron(r));
  IntPatch_PolyhedronBVH s1(*p1),s2(*p2),e;ga_ip_bvh_report(s1);ga_ip_bvh_report(e);
  IntPatch_BVHTraversal t;ga_int(t.Perform(s1,s2,false));ga_ip_pairs(t);ga_ip_bvh_report(s1);ga_ip_bvh_report(s2);
  ga_int(t.Perform(s1,s1,true));ga_ip_pairs(t);ga_int(t.Perform(s1,s1,false));ga_ip_pairs(t);
  ga_int(t.Perform(s1,e,false));ga_ip_pairs(t);t.Clear();ga_ip_pairs(t);
  s2.Clear();ga_ip_bvh_report(s2);ga_int(t.Perform(s1,s2,false));s2.Init(*p1);ga_int(t.Perform(s1,s2,true));ga_ip_pairs(t);
  BVH_Vec3d a(0,0,0),b(1,1,1),c(1,1,1),d(2,2,2),f(1.5,1.5,1.5);double m=0;
  ga_flag(t.RejectNode(a,b,c,d,m));ga_flag(t.RejectNode(a,b,f,d,m));ga_flag(t.RejectNode(c,d,a,b,m)));
 ga_end();
}
// `mode polyhedron1 [polyhedron2]`: the interference of two polyhedra (mode 0 the
// constructor, 1 Perform after a self Perform) or the self interference of one (2 the
// constructor, 3 Perform).
void ga_ip_interf_case(std::istream& input){
 GaReader r{input};int mode=r.integer();
 GA_TRY(std::unique_ptr<IntPatch_Polyhedron> p1(ga_ip_polyhedron(r));
  if(mode<=1){std::unique_ptr<IntPatch_Polyhedron> p2(ga_ip_polyhedron(r));
   if(mode==0){IntPatch_InterferencePolyhedron i(*p1,*p2);ga_interference_print(i);}
   else{IntPatch_InterferencePolyhedron i;ga_interference_print(i);i.Perform(*p1);ga_interference_print(i);i.Perform(*p1,*p2);ga_interference_print(i);}}
  else if(mode==2){IntPatch_InterferencePolyhedron i(*p1);ga_interference_print(i);}
  else{IntPatch_InterferencePolyhedron i;i.Perform(*p1);ga_interference_print(i);});
 ga_end();
}
void ga_ip_box2(const Bnd_Box2d& b){ga_flag(b.IsVoid());GA_TRY(double a,c,d,e;b.Get(a,c,d,e);ga_real(a);ga_real(c);ga_real(d);ga_real(e));ga_real(b.GetGap());}
void ga_ip_polygo(const IntPatch_Polygo& p){
 ga_int(p.NbPoints());for(int i=1;i<=p.NbPoints();++i)ga_pnt2d(p.Point(i));ga_real(p.Error());ga_real(p.DeflectionOverEstimation());
 ga_int(p.NbSegments());for(int i=1;i<=p.NbSegments();++i){gp_Pnt2d a,b;p.Segment(i,a,b);ga_pnt2d(a);ga_pnt2d(b);}
 ga_flag(p.Closed());ga_ip_box2(p.Bounding());}
// `kind onfirst defle points`: the polygon of a walking (kind 0) or restriction (1) line of
// the points on one surface, with the default (defle < 0) or the given initial deflection.
void ga_ip_polyline_case(std::istream& input){
 GaReader r{input};int kind=r.integer();bool onfirst=r.flag();double defle=r.real();occ::handle<IntSurf_LineOn2S> c=ga_ip_read_line2s(r);
 IntPatch_PolyLine p=defle<0?IntPatch_PolyLine():IntPatch_PolyLine(defle);ga_real(p.Error());
 GA_TRY(if(kind==0){occ::handle<IntPatch_WLine> l=new IntPatch_WLine(c,false);p.SetWLine(onfirst,l);}
  else{occ::handle<IntPatch_RLine> l=new IntPatch_RLine(false);l->Add(c);p.SetRLine(onfirst,l);}
  ga_ip_polygo(p);GA_TRY(p.Point(0));GA_TRY(p.Point(p.NbPoints()+1));p.ResetError();ga_real(p.Error()));
 ga_end();
}
// `arc nbsample pfirst plast void [xmin ymin xmax ymax] ox oy`: the polygon of the arc
// compared with the box, then moved by the offset.
void ga_ip_polyarc_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Adaptor2d_Curve2d> a=ga_ip_arc(r);int nb=r.integer();double pf=r.real(),pl=r.real();Bnd_Box2d b;
  if(!r.flag()){double x0=r.real(),y0=r.real(),x1=r.real(),y1=r.real();b.Update(x0,y0,x1,y1);}double ox=r.real(),oy=r.real();
  IntPatch_PolyArc p(a,nb,pf,pl,b);ga_ip_polygo(p);for(int i=1;i<=p.NbPoints();++i)ga_real(p.Parameter(i));
  GA_TRY(p.Parameter(0));GA_TRY(p.Point(p.NbPoints()+1));p.SetOffset(ox,oy);ga_ip_polygo(p);p.SetOffset(0,0);ga_ip_polygo(p));
 ga_end();
}
// `size n (command t)*`: T3Bits edits on two sets (1 add to the first, 2 add to the
// second, 3 raz on the first, 4 val on the first, 5 And from the index t, 6 ResetAnd).
void ga_ip_t3bits_case(std::istream& input){
 GaReader r{input};int size=r.integer();IntPatch_PrmPrmIntersection_T3Bits a(size),b(size);int n=r.integer();
 for(int i=0;i<n;++i){int c=r.integer(),t=r.integer();
  if(c==1)a.Add(t);else if(c==2)b.Add(t);else if(c==3)a.Raz(t);else if(c==4)ga_int(a.Val(t));
  else if(c==5){int k=t;ga_int(a.And(b,k));ga_int(k);}else a.ResetAnd();}
 ga_end();
}
