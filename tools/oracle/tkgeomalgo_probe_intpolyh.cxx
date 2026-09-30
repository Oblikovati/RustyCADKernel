// IntPolyh probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_polyh.rs. The Dump members write with printf:
// ga_ph_printed swaps the C `stdout` for a memory stream while they run (std::cout keeps
// the original stream) and prints their text as one token. Needs the surface readers of
// tkgeomalgo_probe_intcurvesurface.cxx.
#include <IntPolyh_Couple.hxx>
#include <IntPolyh_Edge.hxx>
#include <IntPolyh_Intersection.hxx>
#include <IntPolyh_MaillageAffinage.hxx>
#include <IntPolyh_Point.hxx>
#include <IntPolyh_SectionLine.hxx>
#include <IntPolyh_StartPoint.hxx>
#include <IntPolyh_Tools.hxx>
#include <IntPolyh_Triangle.hxx>
#include <cstdio>
template<class F> void ga_ph_printed(F body){
 std::cout.flush();fflush(stdout);char* buf=nullptr;size_t len=0;FILE* old=stdout;stdout=open_memstream(&buf,&len);
 std::string error;try{body();}catch(const Standard_Failure& f){error=f.ExceptionType();}
 fflush(stdout);fclose(stdout);stdout=old;ga_text(std::string(buf,len));free(buf);if(!error.empty())std::cout<<error<<' ';}
IntPolyh_Point ga_ph_read_point(GaReader& r){double x=r.real(),y=r.real(),z=r.real(),u=r.real();return IntPolyh_Point(x,y,z,u,r.real());}
void ga_ph_point(const IntPolyh_Point& p){ga_real(p.X());ga_real(p.Y());ga_real(p.Z());ga_real(p.U());ga_real(p.V());ga_int(p.PartOfCommon());ga_flag(p.Degenerated());}
void ga_ph_sp(const IntPolyh_StartPoint& s){
 ga_real(s.X());ga_real(s.Y());ga_real(s.Z());ga_real(s.U1());ga_real(s.V1());ga_real(s.U2());ga_real(s.V2());
 ga_int(s.T1());ga_int(s.E1());ga_real(s.Lambda1());ga_int(s.T2());ga_int(s.E2());ga_real(s.Lambda2());ga_real(s.GetAngle());ga_int(s.ChainList());}
IntPolyh_StartPoint ga_ph_read_sp(GaReader& r){
 double x=r.real(),y=r.real(),z=r.real(),u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();int t1=r.integer(),e1=r.integer();double l1=r.real();
 int t2=r.integer(),e2=r.integer();double l2=r.real();int cl=r.integer();return IntPolyh_StartPoint(x,y,z,u1,v1,u2,v2,t1,e1,l1,t2,e2,l2,cl);}
void ga_ph_triangle(const IntPolyh_Triangle& t){
 ga_int(t.FirstPoint());ga_int(t.SecondPoint());ga_int(t.ThirdPoint());
 for(int k=0;k<=4;++k){ga_int(t.GetEdgeNumber(k));ga_int(t.GetEdgeOrientation(k));}
 ga_real(t.Deflection());ga_flag(t.IsIntersectionPossible());ga_flag(t.HasIntersection());ga_flag(t.IsDegenerated());}
void ga_ph_edge(const IntPolyh_Edge& e){ga_int(e.FirstPoint());ga_int(e.SecondPoint());ga_int(e.FirstTriangle());ga_int(e.SecondTriangle());}
void ga_ph_box(const Bnd_Box& b){ga_flag(b.IsVoid());if(b.IsVoid())return;double a[6];b.Get(a[0],a[1],a[2],a[3],a[4],a[5]);for(double x:a)ga_real(x);}
// The items of an array, and the one just past them when the allocated storage holds
// it (NbItems below GetN).
template<class A,class P> void ga_ph_items(const A& a,P print){
 ga_int(a.NbItems());ga_int(a.GetN());int n=a.NbItems()+(a.NbItems()<a.GetN()?1:0);for(int i=0;i<n;++i)print(a[i]);}
void ga_ph_side(IntPolyh_MaillageAffinage& m,int s){
 ga_ph_items(m.GetArrayOfPoints(s),ga_ph_point);ga_ph_items(m.GetArrayOfEdges(s),ga_ph_edge);
 ga_ph_items(m.GetArrayOfTriangles(s),ga_ph_triangle);
 ga_ph_box(m.GetBox(s));ga_real(m.GetMinDeflection(s));ga_real(m.GetMaxDeflection(s));}
void ga_ph_couples(IntPolyh_MaillageAffinage& m){
 ga_int(m.GetCouples().Extent());for(const IntPolyh_Couple& c:m.GetCouples()){ga_int(c.FirstValue());ga_int(c.SecondValue());ga_flag(c.IsAnalyzed());ga_real(c.Angle());}}
void ga_ph_lines(const IntPolyh_ArrayOfSectionLines& l,const IntPolyh_ArrayOfTangentZones& z){
 ga_int(l.NbItems());for(int i=0;i<l.NbItems();++i){const IntPolyh_SectionLine& s=l[i];ga_int(s.NbStartPoints());for(int k=0;k<s.NbStartPoints();++k)ga_ph_sp(s[k]);}
 ga_int(z.NbItems());for(int i=0;i<z.NbItems();++i)ga_ph_sp(z[i]);}
// `mode s1 s2 [...]`: IntPolyh_Intersection with the default sampling (0), sample counts
// `nsu1 nsv1 nsu2 nsv2` (1) or the parameters `u1 v1 u2 v2` (2), then every result and
// the reads past them that stay in the allocated storage.
void ga_ph_inter_case(std::istream& input){
 GaReader r{input};int mode=r.integer();occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r),s2=ga_ics_surface(r);
 std::unique_ptr<IntPolyh_Intersection> x;
 GA_TRY(if(mode==0)x.reset(new IntPolyh_Intersection(s1,s2));
  else if(mode==1){int a=r.integer(),b=r.integer(),c=r.integer(),d=r.integer();x.reset(new IntPolyh_Intersection(s1,a,b,s2,c,d));}
  else{NCollection_Array1<double> u1=ga_ics_params(r),v1=ga_ics_params(r),u2=ga_ics_params(r),v2=ga_ics_params(r);x.reset(new IntPolyh_Intersection(s1,u1,v1,s2,u2,v2));});
 if(x){ga_flag(x->IsDone());ga_flag(x->IsParallel());int n=x->NbSectionLines();ga_int(n);
  for(int i=1;i<=n;++i){int np=x->NbPointsInLine(i);ga_int(np);
   for(int k=1;k<=np+1;++k)GA_TRY(double a[8];x->GetLinePoint(i,k,a[0],a[1],a[2],a[3],a[4],a[5],a[6],a[7]);for(double v:a)ga_real(v));
   GA_TRY(double a[8];x->GetLinePoint(i,0,a[0],a[1],a[2],a[3],a[4],a[5],a[6],a[7]));
   GA_TRY(double a[8];x->GetLinePoint(i,np+2,a[0],a[1],a[2],a[3],a[4],a[5],a[6],a[7]));}
  ga_int(x->NbPointsInLine(n+1));int nz=x->NbTangentZones();ga_int(nz);
  for(int i=1;i<=nz+1;++i){double a[7];x->GetTangentZonePoint(i,1,a[0],a[1],a[2],a[3],a[4],a[5],a[6]);for(double v:a)ga_real(v);}
  ga_int(x->NbPointsInTangentZone(1));}
 ga_end();
}
// `s1 nsu1 nsv1 s2 nsu2 nsv2 steps...`: IntPolyh_MaillageAffinage driven step by step
// (each step `op [args]`), then its whole state: 0 FillArrayOfPnt(id), 1 FillArrayOfPnt(id,
// fwd), 2 FillArrayOfPnt(id, sampling, defl or none), 3 FillArrayOfPnt(id, fwd,
// sampling, defl or none), 4 CommonBox, 5 FillArrayOfTriangles(id), 6
// FillArrayOfEdges(id), 7 TrianglesDeflectionsRefinementBSB, 8 CommonPartRefinement, 9
// LocalSurfaceRefinement(id), 10 ComputeDeflections(id), 11 TriangleCompare, 12
// StartPointsChain, 13 SetEnlargeZone(flag), 14 the advanced FillArrayOfPnt of both
// surfaces (with the normals of the sampling), 15 the whole ComputeIntersection.
void ga_ph_maillage_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s1=ga_ics_surface(r);int a=r.integer(),b=r.integer();
 occ::handle<Adaptor3d_Surface> s2=ga_ics_surface(r);int c=r.integer(),d=r.integer();
 IntPolyh_MaillageAffinage m(s1,a,b,s2,c,d,0);IntPolyh_ArrayOfSectionLines lines;IntPolyh_ArrayOfTangentZones zones;lines.Init(1000);zones.Init(10000);
 int nsteps=r.integer();
 for(int k=0;k<nsteps;++k){int op=r.integer();
  switch(op){
   case 0:{int id=r.integer();GA_TRY(m.FillArrayOfPnt(id));break;}
   case 1:{int id=r.integer();bool f=r.flag();GA_TRY(m.FillArrayOfPnt(id,f));break;}
   case 2:{int id=r.integer();NCollection_Array1<double> u=ga_ics_params(r),v=ga_ics_params(r);bool has=r.flag();double t=r.real();
    GA_TRY(m.FillArrayOfPnt(id,u,v,has?&t:nullptr));break;}
   case 3:{int id=r.integer();bool f=r.flag();NCollection_Array1<double> u=ga_ics_params(r),v=ga_ics_params(r);bool has=r.flag();double t=r.real();
    GA_TRY(m.FillArrayOfPnt(id,f,u,v,has?&t:nullptr));break;}
   case 4:GA_TRY(m.CommonBox());break;
   case 5:{int id=r.integer();GA_TRY(m.FillArrayOfTriangles(id));break;}
   case 6:{int id=r.integer();GA_TRY(m.FillArrayOfEdges(id));break;}
   case 7:GA_TRY(m.TrianglesDeflectionsRefinementBSB());break;
   case 8:GA_TRY(m.CommonPartRefinement());break;
   case 9:{int id=r.integer();GA_TRY(m.LocalSurfaceRefinement(id));break;}
   case 10:{int id=r.integer();GA_TRY(m.ComputeDeflections(id));break;}
   case 11:GA_TRY(ga_int(m.TriangleCompare()));break;
   case 12:GA_TRY(ga_int(m.StartPointsChain(lines,zones)));break;
   case 13:m.SetEnlargeZone(r.flag());break;
   case 14:{bool f1=r.flag(),f2=r.flag();GA_TRY(NCollection_Array1<double> u1,v1,u2,v2;m.MakeSampling(1,u1,v1);m.MakeSampling(2,u2,v2);
    IntPolyh_ArrayOfPointNormal p1,p2;IntPolyh_Tools::FillArrayOfPointNormal(s1,u1,v1,p1);IntPolyh_Tools::FillArrayOfPointNormal(s2,u2,v2,p2);
    m.FillArrayOfPnt(1,f1,p1,u1,v1,IntPolyh_Tools::ComputeDeflection(s1,u1,v1));m.FillArrayOfPnt(2,f2,p2,u2,v2,IntPolyh_Tools::ComputeDeflection(s2,u2,v2)));break;}
   default:GA_TRY(m.CommonBox();m.FillArrayOfTriangles(1);m.FillArrayOfTriangles(2);m.FillArrayOfEdges(1);m.FillArrayOfEdges(2);
    m.TrianglesDeflectionsRefinementBSB();ga_int(m.TriangleCompare()));
  }}
 ga_flag(m.GetEnlargeZone());ga_ph_side(m,1);ga_ph_side(m,2);ga_ph_couples(m);ga_ph_lines(lines,zones);
 GA_TRY(double z[6];m.CommonBox(m.GetBox(1),m.GetBox(2),z[0],z[1],z[2],z[3],z[4],z[5]);for(double v:z)ga_real(v));
 ga_end();
}
// `angle p1 p2 p3 q1 q2 q3` (x y z u v each): TriContact and the angle it leaves.
void ga_ph_tricontact_case(std::istream& input){
 GaReader r{input};double angle=r.real();IntPolyh_Point p[6];for(IntPolyh_Point& x:p)x=ga_ph_read_point(r);
 occ::handle<Adaptor3d_Surface> s;IntPolyh_MaillageAffinage m(s,s,0);
 ga_int(m.TriContact(p[0],p[1],p[2],p[3],p[4],p[5],angle));ga_real(angle);ga_end();
}
// `id edge tri1(e1 e2 e3 o1 o2 o3) tri2(...) pt1 pt2 pt3 pe1 pe2 sp1 sp2`: TriangleEdgeContact
// with the sides, edge and unit normal StartingPointsResearch would pass.
void ga_ph_edgecontact_case(std::istream& input){
 GaReader r{input};int id=r.integer(),ei=r.integer();IntPolyh_Triangle t[2];
 for(IntPolyh_Triangle& x:t){int e[3],o[3];for(int& v:e)v=r.integer();for(int& v:o)v=r.integer();x.SetFirstEdge(e[0],o[0]);x.SetSecondEdge(e[1],o[1]);x.SetThirdEdge(e[2],o[2]);}
 IntPolyh_Point p[5];for(IntPolyh_Point& x:p)x=ga_ph_read_point(r);IntPolyh_StartPoint sp1=ga_ph_read_sp(r),sp2=ga_ph_read_sp(r);
 IntPolyh_Point c12=p[1]-p[0],c23=p[2]-p[1],c31=p[0]-p[2],edge=p[4]-p[3],nn;nn.Cross(c12,c23);IntPolyh_Point n=nn.Divide(sqrt(nn.SquareModulus()));
 occ::handle<Adaptor3d_Surface> s;IntPolyh_MaillageAffinage m(s,s,0);
 GA_TRY(ga_int(m.TriangleEdgeContact(id,ei,t[0],t[1],p[0],p[1],p[2],c12,c23,c31,p[3],p[4],edge,n,sp1,sp2)));
 ga_ph_sp(sp1);ga_ph_sp(sp2);ga_end();
}
// `p1 p2 r surface i`: every IntPolyh_Point member and its dumps.
void ga_ph_point_case(std::istream& input){
 GaReader r{input};IntPolyh_Point a=ga_ph_read_point(r),b=ga_ph_read_point(r);double rr=r.real();occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);int i=r.integer();
 IntPolyh_Point d;ga_ph_point(d);ga_ph_point(a+b);ga_ph_point(a-b);ga_ph_point(a/rr);ga_ph_point(a*rr);ga_ph_point(a.Add(b));ga_ph_point(a.Sub(b));
 ga_ph_point(a.Divide(rr));ga_ph_point(a.Multiplication(rr));ga_real(a.SquareModulus());ga_real(a.SquareDistance(b));ga_real(a.Dot(b));
 IntPolyh_Point c=a;c.Cross(a,b);ga_ph_point(c);c.Set(1,2,3,4,5,i);ga_ph_point(c);c.Set(6,7,8,9,10);ga_ph_point(c);
 c.SetX(rr);c.SetY(-rr);c.SetZ(2*rr);c.SetU(3*rr);c.SetV(4*rr);c.SetPartOfCommon(i);c.SetDegenerated(true);ga_ph_point(c);
 GA_TRY(IntPolyh_Point m=c;m.Middle(s,a,b);ga_ph_point(m));
 ga_ph_printed([&]{a.Dump();});ga_ph_printed([&]{b.Dump(i);});ga_end();
}
// `surface nsu nsv num multiple crit box(6)`: the IntPolyh_Triangle members on the grid
// mesh of the surface (FillArrayOfPnt, FillArrayOfTriangles, FillArrayOfEdges of a
// MaillageAffinage), then MiddleRefinement or MultipleMiddleRefinement of the triangle
// `num` on copies of the arrays, and the setters on a free triangle.
void ga_ph_triangle_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);int nsu=r.integer(),nsv=r.integer(),num=r.integer(),multiple=r.integer();
 double crit=r.real();Bnd_Box bb;{double x0=r.real(),y0=r.real(),z0=r.real(),x1=r.real(),y1=r.real(),z1=r.real();bb.Update(x0,y0,z0,x1,y1,z1);}
 IntPolyh_MaillageAffinage m(s,nsu,nsv,s,nsu,nsv,0);
 GA_TRY(m.FillArrayOfPnt(1);m.FillArrayOfTriangles(1);m.FillArrayOfEdges(1);
  IntPolyh_ArrayOfPoints pts(m.GetArrayOfPoints(1));IntPolyh_ArrayOfTriangles tris(m.GetArrayOfTriangles(1));IntPolyh_ArrayOfEdges edges(m.GetArrayOfEdges(1));
  for(int k=0;k<tris.NbItems();++k){GA_TRY(ga_real(tris[k].ComputeDeflection(s,pts)));ga_ph_triangle(tris[k]);ga_ph_box(tris[k].BoundingBox(pts));
   for(int e=0;e<=4;++e)ga_int(tris[k].GetNextTriangle(k,e,edges));}
  ga_ph_printed([&]{tris[0].Dump(0);});
  IntPolyh_Triangle l=tris[0];l.LinkEdges2Triangle(edges,0,1,2);ga_ph_triangle(l);l.LinkEdges2Triangle(edges,-1,1,2);ga_ph_triangle(l);
  IntPolyh_Triangle t(0,1,2);t.SetEdge(0,5);t.SetEdge(2,7);t.SetEdgeOrientation(4,1);t.SetEdgeOrientation(3,-1);t.SetDeflection(0.5);
  t.SetFirstEdge(3,1);t.SetSecondEdge(4,-1);t.SetThirdEdge(6,1);t.SetFirstPoint(9);t.SetSecondPoint(8);t.SetThirdPoint(7);
  t.SetIntersectionPossible(false);t.SetIntersection(true);t.SetDegenerated(true);ga_ph_triangle(t);
  GA_TRY(if(multiple)tris[num].MultipleMiddleRefinement(crit,bb,num,s,pts,tris,edges);else tris[num].MiddleRefinement(num,s,pts,tris,edges));
  ga_ph_items(pts,ga_ph_point);ga_ph_items(tris,ga_ph_triangle);ga_ph_items(edges,ga_ph_edge));
 ga_end();
}
// `sp1 sp2 t(p1 p2 p3)`: every IntPolyh_StartPoint member, CheckSameSP both ways,
// GetEdgePoints and the dumps.
void ga_ph_startpoint_case(std::istream& input){
 GaReader r{input};IntPolyh_StartPoint a=ga_ph_read_sp(r),b=ga_ph_read_sp(r);int p1=r.integer(),p2=r.integer();IntPolyh_Triangle t(p1,p2,r.integer());
 IntPolyh_StartPoint d;ga_ph_sp(d);ga_ph_sp(a);ga_int(a.CheckSameSP(b));ga_int(b.CheckSameSP(a));
 int f=-9,s=-9,l=-9;ga_int(a.GetEdgePoints(t,f,s,l));ga_int(f);ga_int(s);ga_int(l);
 IntPolyh_StartPoint c=b;c.SetXYZ(1,2,3);c.SetUV1(4,5);c.SetUV2(6,7);c.SetEdge1(8);c.SetLambda1(0.25);c.SetEdge2(9);c.SetLambda2(0.75);
 c.SetCoupleValue(10,11);c.SetAngle(0.5);c.SetChainList(12);ga_ph_sp(c);
 ga_ph_printed([&]{a.Dump();});ga_ph_printed([&]{b.Dump(7);});ga_end();
}
// `n (sp)* ops...`: IntPolyh_SectionLine: 0 Init, 1 IncrementNbStartPoints, 2 Prepend(sp i),
// 3 ChangeValue(index) = sp i, 4 copy from a copy, 5 read Value(index), 6 Dump, 7 the
// constructor with a size.
void ga_ph_sectionline_case(std::istream& input){
 GaReader r{input};int n=r.integer();std::vector<IntPolyh_StartPoint> sps;for(int i=0;i<n;++i)sps.push_back(ga_ph_read_sp(r));
 IntPolyh_SectionLine line;int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  switch(op){
   case 0:line.Init(r.integer());break;
   case 1:line.IncrementNbStartPoints();break;
   case 2:line.Prepend(sps.at(r.integer()));break;
   case 3:{int idx=r.integer(),i=r.integer();GA_TRY(line[idx]=sps.at(i));break;}
   case 4:{IntPolyh_SectionLine c(line);IntPolyh_SectionLine e;e=c;line.Copy(e);break;}
   case 5:{int idx=r.integer();GA_TRY(ga_ph_sp(line.Value(idx)));break;}
   case 6:ga_ph_printed([&]{line.Dump();});break;
   default:{IntPolyh_SectionLine c(r.integer());line=c;}
  }
  ga_int(line.GetN());ga_int(line.NbStartPoints());}
 ga_end();
}
// `n increment ops...`: IntPolyh_ArrayOfPoints: 0 Init(n), 1 IncrementNbItems, 2
// SetNbItems(n), 3 set point i, 4 assign from a copy with another increment, 5 Dump; then
// the counts and the stored points.
void ga_ph_array_case(std::istream& input){
 GaReader r{input};int n=r.integer(),inc=r.integer();IntPolyh_ArrayOfPoints a(n,inc);int nops=r.integer();
 for(int k=0;k<nops;++k){int op=r.integer();
  switch(op){
   case 0:a.Init(r.integer());break;
   case 1:a.IncrementNbItems();break;
   case 2:a.SetNbItems(r.integer());break;
   case 3:{int i=r.integer();a[i]=ga_ph_read_point(r);break;}
   case 4:{IntPolyh_ArrayOfPoints b(3,256);b=a;IntPolyh_ArrayOfPoints c(a);a=b;ga_int(c.GetN());b.IncrementNbItems();ga_int(b.GetN());break;}
   default:ga_ph_printed([&]{a.Dump();});
  }
  ga_int(a.NbItems());ga_int(a.GetN());}
 for(int i=0;i<a.NbItems();++i)ga_ph_point(a.Value(i));ga_end();
}
// `surface nsu nsv`: the IntPolyh_Tools members.
void ga_ph_tools_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);int nsu=r.integer(),nsv=r.integer();
 GA_TRY(bool u,v;IntPolyh_Tools::IsEnlargePossible(s,u,v);ga_flag(u);ga_flag(v));
 for(int e=0;e<2;++e){NCollection_Array1<double> up,vp;
  GA_TRY(IntPolyh_Tools::MakeSampling(s,nsu,nsv,e!=0,up,vp);for(double x:up)ga_real(x);for(double x:vp)ga_real(x));
  GA_TRY(ga_real(IntPolyh_Tools::ComputeDeflection(s,up,vp)));
  IntPolyh_ArrayOfPointNormal pn;
  GA_TRY(IntPolyh_Tools::FillArrayOfPointNormal(s,up,vp,pn);ga_int(pn.NbItems());
   for(int i=0;i<pn.NbItems();++i){ga_pnt(pn[i].Point);ga_xyz(pn[i].Normal.XYZ());});}
 ga_end();
}
// `a b angle i`: IntPolyh_Couple and IntPolyh_Edge members and dumps.
void ga_ph_couple_case(std::istream& input){
 GaReader r{input};int a=r.integer(),b=r.integer();double angle=r.real();int i=r.integer();
 IntPolyh_Couple d,c(a,b,angle),e(b,a),f(a,a+1);ga_int(d.FirstValue());ga_int(d.SecondValue());ga_flag(d.IsAnalyzed());ga_real(d.Angle());
 ga_flag(c.IsEqual(e));ga_flag(c==f);ga_flag(std::hash<IntPolyh_Couple>()(c)==std::hash<IntPolyh_Couple>()(e));
 c.SetAnalyzed(true);c.SetAngle(-angle);c.SetCoupleValue(b,a);ga_int(c.FirstValue());ga_int(c.SecondValue());ga_flag(c.IsAnalyzed());ga_real(c.Angle());
 ga_ph_printed([&]{c.Dump(i);});
 IntPolyh_Edge de,ed(a,b,i,-i);ga_ph_edge(de);ga_ph_edge(ed);ed.SetFirstPoint(i);ed.SetSecondPoint(a);ed.SetFirstTriangle(b);ed.SetSecondTriangle(-1);ga_ph_edge(ed);
 ga_ph_printed([&]{ed.Dump(i);});ga_end();
}
