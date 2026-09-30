// IntCurveSurface probe operations over the records, the curve tool, the polygon, the
// polyhedron and the header utilities (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_curve_surface.rs. The geometry readers are
// shared with tkgeomalgo_probe_intcurvesurface_inter.cxx.
#include <IntCurveSurface_Intersection.hxx>
#include <IntCurveSurface_IntersectionPoint.hxx>
#include <IntCurveSurface_IntersectionSegment.hxx>
#include <IntCurveSurface_TheHCurveTool.hxx>
#include <IntCurveSurface_ThePolygonOfHInter.hxx>
#include <IntCurveSurface_ThePolygonToolOfHInter.hxx>
#include <IntCurveSurface_ThePolyhedronOfHInter.hxx>
#include <IntCurveSurface_ThePolyhedronToolOfHInter.hxx>
#include <IntCurveSurface_PolygonUtils.pxx>
#include <IntCurveSurface_PolyhedronUtils.pxx>
#include <Geom_BezierCurve.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_TrimmedCurve.hxx>
// Weights `flag [w...]` of n poles, or none.
bool ga_ics_weights(GaReader& r,int n,NCollection_Array1<double>& w){
 if(!r.flag())return false;for(int i=1;i<=n;++i)w(i)=r.real();return true;}
// Knots and multiplicities `nk knots... mults...`.
void ga_ics_knots(GaReader& r,NCollection_Array1<double>& k,NCollection_Array1<int>& m){
 for(int i=1;i<=k.Length();++i)k(i)=r.real();for(int i=1;i<=m.Length();++i)m(i)=r.integer();}
// A curve `kind ...`: 0 line p d, 1 circle p d r, 2 ellipse p d R r, 3 parabola p d f,
// 4 hyperbola p d R r, 5 Bezier `n poles weights`, 6 B-spline `deg n poles weights nk
// knots mults periodic`, 7 offset `offset d basis`, 8 trimmed `u1 u2 basis`.
occ::handle<Geom_Curve> ga_ics_geom_curve(GaReader& r){
 int kind=r.integer();
 if(kind<=4){gp_Pnt p=r.pnt();gp_Dir d=r.dir();if(kind==0)return new Geom_Line(p,d);
  double a=r.real();gp_Ax2 ax(p,d);
  if(kind==1)return new Geom_Circle(ax,a);if(kind==3)return new Geom_Parabola(ax,a);
  double b=r.real();if(kind==2)return new Geom_Ellipse(ax,a,b);return new Geom_Hyperbola(ax,a,b);}
 if(kind==5){int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;++i)poles(i)=r.pnt();
  NCollection_Array1<double> w(1,n);if(ga_ics_weights(r,n,w))return new Geom_BezierCurve(poles,w);return new Geom_BezierCurve(poles);}
 if(kind==6){int deg=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;++i)poles(i)=r.pnt();
  NCollection_Array1<double> w(1,n);bool rational=ga_ics_weights(r,n,w);int nk=r.integer();
  NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);ga_ics_knots(r,k,m);bool periodic=r.flag();
  if(rational)return new Geom_BSplineCurve(poles,w,k,m,deg,periodic);return new Geom_BSplineCurve(poles,k,m,deg,periodic);}
 if(kind==7){double off=r.real();gp_Dir d=r.dir();occ::handle<Geom_Curve> basis=ga_ics_geom_curve(r);return new Geom_OffsetCurve(basis,off,d);}
 double u1=r.real(),u2=r.real();occ::handle<Geom_Curve> basis=ga_ics_geom_curve(r);return new Geom_TrimmedCurve(basis,u1,u2);
}
// A curve adaptor `curve 0` (natural range) or `curve 1 first last`.
occ::handle<Adaptor3d_Curve> ga_ics_curve(GaReader& r){
 occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);if(!r.flag())return new GeomAdaptor_Curve(c);
 double a=r.real(),b=r.real();return new GeomAdaptor_Curve(c,a,b);
}
// A surface `kind ...`: 0 plane p d, 1 cylinder p d r, 2 cone p d ang r, 3 sphere p d r,
// 4 torus p d R r, 5 Bezier `nu nv poles`, 6 B-spline `udeg vdeg nu nv poles nuk uknots
// umults nvk vknots vmults`, 7 revolution `p d curve`, 8 extrusion `d curve`, 9 offset
// `offset surface`, 10 trimmed `u1 u2 v1 v2 surface`.
occ::handle<Geom_Surface> ga_ics_geom_surface(GaReader& r){
 int kind=r.integer();
 if(kind<=4){gp_Pnt p=r.pnt();gp_Dir d=r.dir();gp_Ax3 ax(p,d);if(kind==0)return new Geom_Plane(ax);double a=r.real();
  if(kind==1)return new Geom_CylindricalSurface(ax,a);if(kind==3)return new Geom_SphericalSurface(ax,a);
  double b=r.real();if(kind==2)return new Geom_ConicalSurface(ax,a,b);return new Geom_ToroidalSurface(ax,a,b);}
 if(kind==5||kind==6){int ud=0,vd=0;if(kind==6){ud=r.integer();vd=r.integer();}int nu=r.integer(),nv=r.integer();
  NCollection_Array2<gp_Pnt> poles(1,nu,1,nv);for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j)poles(i,j)=r.pnt();
  if(kind==5)return new Geom_BezierSurface(poles);
  int nuk=r.integer();NCollection_Array1<double> uk(1,nuk);NCollection_Array1<int> um(1,nuk);ga_ics_knots(r,uk,um);
  int nvk=r.integer();NCollection_Array1<double> vk(1,nvk);NCollection_Array1<int> vm(1,nvk);ga_ics_knots(r,vk,vm);
  return new Geom_BSplineSurface(poles,uk,vk,um,vm,ud,vd);}
 if(kind==7){gp_Pnt p=r.pnt();gp_Dir d=r.dir();occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);return new Geom_SurfaceOfRevolution(c,gp_Ax1(p,d));}
 if(kind==8){gp_Dir d=r.dir();occ::handle<Geom_Curve> c=ga_ics_geom_curve(r);return new Geom_SurfaceOfLinearExtrusion(c,d);}
 if(kind==9){double off=r.real();occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);return new Geom_OffsetSurface(s,off);}
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);
 return new Geom_RectangularTrimmedSurface(s,u1,u2,v1,v2);
}
// A surface adaptor `surface 0` (natural bounds) or `surface 1 u1 u2 v1 v2`.
occ::handle<Adaptor3d_Surface> ga_ics_surface(GaReader& r){
 occ::handle<Geom_Surface> s=ga_ics_geom_surface(r);if(!r.flag())return new GeomAdaptor_Surface(s);
 double u1=r.real(),u2=r.real(),v1=r.real(),v2=r.real();return new GeomAdaptor_Surface(s,u1,u2,v1,v2);
}
// Parameters `n values...` from index 1 (at least one slot).
NCollection_Array1<double> ga_ics_params(GaReader& r){
 int n=r.integer();if(n<=0)return NCollection_Array1<double>();NCollection_Array1<double> a(1,n);
 for(int i=1;i<=n;++i)a(i)=r.real();return a;}
void ga_ics_ipoint(const IntCurveSurface_IntersectionPoint& p){
 ga_pnt(p.Pnt());ga_real(p.U());ga_real(p.V());ga_real(p.W());ga_int(p.Transition());}
IntCurveSurface_IntersectionPoint ga_ics_read_ipoint(GaReader& r){
 gp_Pnt p=r.pnt();double u=r.real(),v=r.real(),w=r.real();
 return IntCurveSurface_IntersectionPoint(p,u,v,w,IntCurveSurface_TransitionOnCurve(r.integer()));}
// `p u v w tr p2 u2 v2 w2 tr2`: the point records, the segment records and their dumps.
void ga_ics_records_case(std::istream& input){
 GaReader r{input};IntCurveSurface_IntersectionPoint a=ga_ics_read_ipoint(r),b=ga_ics_read_ipoint(r),d;
 ga_ics_ipoint(d);ga_ics_ipoint(a);gp_Pnt p;double u,v,w;IntCurveSurface_TransitionOnCurve tr;
 a.Values(p,u,v,w,tr);ga_pnt(p);ga_real(u);ga_real(v);ga_real(w);ga_int(tr);
 d.SetValues(b.Pnt(),b.U(),b.V(),b.W(),b.Transition());ga_ics_ipoint(d);ga_captured([&]{a.Dump();});
 IntCurveSurface_IntersectionSegment s0,s(a,b);IntCurveSurface_IntersectionPoint f,l;
 ga_ics_ipoint(s0.FirstPoint());ga_ics_ipoint(s0.SecondPoint());s.Values(f,l);ga_ics_ipoint(f);ga_ics_ipoint(l);
 s.FirstPoint(f);s.SecondPoint(l);ga_ics_ipoint(f);ga_ics_ipoint(l);s0.SetValues(b,a);ga_ics_ipoint(s0.FirstPoint());
 ga_ics_ipoint(s0.SecondPoint());ga_captured([&]{s.Dump();});ga_end();
}
// The protected members of IntCurveSurface_Intersection, made public for the probe.
class GaIcsIntersection : public IntCurveSurface_Intersection{
public:
 using IntCurveSurface_Intersection::Append;using IntCurveSurface_Intersection::ResetFields;
 using IntCurveSurface_Intersection::SetValues;
 void SetFlags(bool d,bool p){done=d;myIsParallel=p;}
};
void ga_ics_intersection_print(const IntCurveSurface_Intersection& x){
 ga_flag(x.IsDone());ga_flag(x.IsParallel());
 GA_TRY(int n=x.NbPoints();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(ga_ics_ipoint(x.Point(i))));
 GA_TRY(int n=x.NbSegments();ga_int(n);for(int i=0;i<=n+1;++i)GA_TRY(const IntCurveSurface_IntersectionSegment& s=x.Segment(i);
  ga_ics_ipoint(s.FirstPoint());ga_ics_ipoint(s.SecondPoint())));
 ga_captured([&]{x.Dump();});
}
// `count (target command args)...`: commands on two intersections: 0 append a point,
// 1 append a segment, 2 set the done and parallel flags, 3 ResetFields, 4 SetValues from
// the other one, 5 Append the other one with a range, 6 print.
void ga_ics_intersection_case(std::istream& input){
 GaReader r{input};GaIcsIntersection x[2];int count=r.integer();
 for(int k=0;k<count;++k){int t=r.integer(),c=r.integer();GaIcsIntersection& me=x[t];const GaIcsIntersection& other=x[1-t];
  switch(c){
   case 0:me.Append(ga_ics_read_ipoint(r));break;
   case 1:{IntCurveSurface_IntersectionPoint a=ga_ics_read_ipoint(r);me.Append(IntCurveSurface_IntersectionSegment(a,ga_ics_read_ipoint(r)));break;}
   case 2:{bool d=r.flag();me.SetFlags(d,r.flag());break;}
   case 3:me.ResetFields();break;
   case 4:me.SetValues(other);break;
   case 5:{double a=r.real();me.Append(other,a,r.real());break;}
   default:ga_ics_intersection_print(me);
  }}
 ga_end();
}
void ga_ics_lin(const gp_Lin& l){ga_pnt(l.Location());ga_dir(l.Direction());}
void ga_ics_ax2(const gp_Ax2& a){ga_pnt(a.Location());ga_dir(a.Direction());ga_dir(a.XDirection());}
// `curve u u0 u1 defl nbmin`: every member of IntCurveSurface_TheHCurveTool.
void ga_ics_hcurve_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);double u=r.real(),u0=r.real(),u1=r.real(),defl=r.real();
 int nbmin=r.integer();using T=IntCurveSurface_TheHCurveTool;
 GA_TRY(ga_real(T::FirstParameter(c)));GA_TRY(ga_real(T::LastParameter(c)));GA_TRY(ga_int(T::Continuity(c)));
 for(GeomAbs_Shape s:{GeomAbs_C0,GeomAbs_C1,GeomAbs_C2,GeomAbs_C3,GeomAbs_CN})
  GA_TRY(int n=T::NbIntervals(c,s);ga_int(n);NCollection_Array1<double> t(1,n+1);T::Intervals(c,t,s);for(double x:t)ga_real(x));
 GA_TRY(ga_flag(T::IsClosed(c)));GA_TRY(ga_flag(T::IsPeriodic(c)));GA_TRY(ga_real(T::Period(c)));
 GA_TRY(ga_pnt(T::Value(c,u)));GA_TRY(gp_Pnt p;T::D0(c,u,p);ga_pnt(p));GA_TRY(gp_Pnt p;gp_Vec v;T::D1(c,u,p,v);ga_pnt(p);ga_xyz(v.XYZ()));
 GA_TRY(gp_Pnt p;gp_Vec v1,v2;T::D2(c,u,p,v1,v2);ga_pnt(p);ga_xyz(v1.XYZ());ga_xyz(v2.XYZ()));
 GA_TRY(gp_Pnt p;gp_Vec v1,v2,v3;T::D3(c,u,p,v1,v2,v3);ga_pnt(p);ga_xyz(v1.XYZ());ga_xyz(v2.XYZ());ga_xyz(v3.XYZ()));
 for(int n=1;n<=3;++n)GA_TRY(ga_xyz(T::DN(c,u,n).XYZ()));
 GA_TRY(ga_real(T::Resolution(c,1e-7)));GA_TRY(ga_int(T::GetType(c)));
 GA_TRY(ga_ics_lin(T::Line(c)));GA_TRY(gp_Circ x=T::Circle(c);ga_ics_ax2(x.Position());ga_real(x.Radius()));
 GA_TRY(gp_Elips x=T::Ellipse(c);ga_ics_ax2(x.Position());ga_real(x.MajorRadius());ga_real(x.MinorRadius()));
 GA_TRY(gp_Hypr x=T::Hyperbola(c);ga_ics_ax2(x.Position());ga_real(x.MajorRadius());ga_real(x.MinorRadius()));
 GA_TRY(gp_Parab x=T::Parabola(c);ga_ics_ax2(x.Position());ga_real(x.Focal()));
 GA_TRY(ga_int(T::Bezier(c)->NbPoles()));GA_TRY(occ::handle<Geom_BSplineCurve> b=T::BSpline(c);ga_int(b->NbPoles());ga_int(b->NbKnots()));
 GA_TRY(ga_int(T::NbSamples(c,u0,u1)));
 GA_TRY(occ::handle<NCollection_HArray1<double>> pars=T::SamplePars(c,u0,u1,defl,nbmin);ga_int(pars->Length());for(double x:pars->Array1())ga_real(x));
 ga_end();
}
void ga_box3d_print(const Bnd_Box& box);
// The text of a polygon tool Dump with the call number replaced by `#`, since the native
// counter depends on the earlier requests of the process.
std::string ga_ics_polygon_dump(const IntCurveSurface_ThePolygonOfHInter& p){
 std::ostringstream console;std::streambuf* old=std::cout.rdbuf(console.rdbuf());
 IntCurveSurface_ThePolygonToolOfHInter::Dump(p);std::cout.rdbuf(old);
 std::istringstream lines(console.str());std::string line,out;
 while(std::getline(lines,line)){std::istringstream words(line);std::string w;int k=0;std::string rebuilt;
  while(words>>w){if(k==1&&(rebuilt=="pol2d"||rebuilt=="pnt"))w="#";rebuilt+=(k?" ":"")+w;++k;}out+=rebuilt+"\n";}
 return out;
}
// A polygon `curve mode ...`: mode 0 `nb`, 1 `u1 u2 nb`, 2 `params`.
IntCurveSurface_ThePolygonOfHInter ga_ics_polygon(GaReader& r,const occ::handle<Adaptor3d_Curve>& c){
 int mode=r.integer();if(mode==0)return IntCurveSurface_ThePolygonOfHInter(c,r.integer());
 if(mode==1){double a=r.real(),b=r.real();return IntCurveSurface_ThePolygonOfHInter(c,a,b,r.integer());}
 NCollection_Array1<double> pars=ga_ics_params(r);return IntCurveSurface_ThePolygonOfHInter(c,pars);
}
// `curve polygon m (index param)... deflection`: every polygon and tool member.
void ga_ics_polygon_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);
 GA_TRY(IntCurveSurface_ThePolygonOfHInter p=ga_ics_polygon(r,c);using T=IntCurveSurface_ThePolygonToolOfHInter;
  ga_box3d_print(p.Bounding());ga_real(p.DeflectionOverEstimation());ga_flag(p.Closed());int n=p.NbSegments();ga_int(n);
  ga_real(p.InfParameter());ga_real(p.SupParameter());
  for(int i=-1;i<=n+2;++i){GA_TRY(ga_pnt(p.BeginOfSeg(i)));GA_TRY(ga_pnt(p.EndOfSeg(i)));}
  int m=r.integer();for(int k=0;k<m;++k){int i=r.integer();double t=r.real();GA_TRY(ga_real(p.ApproxParamOnCurve(i,t)));}
  ga_box3d_print(T::Bounding(p));ga_real(T::DeflectionOverEstimation(p));ga_flag(T::Closed(p));ga_int(T::NbSegments(p));
  for(int i=1;i<=n;++i){ga_pnt(T::BeginOfSeg(p,i));ga_pnt(T::EndOfSeg(p,i));}
  ga_text(ga_ics_polygon_dump(p));p.Dump();p.Closed(true);ga_flag(p.Closed());ga_flag(T::Closed(p));p.Closed(false);ga_flag(p.Closed());
  p.SetDeflectionOverEstimation(r.real());ga_real(p.DeflectionOverEstimation());ga_box3d_print(p.Bounding()));
 ga_end();
}
// A polyhedron `surface mode ...`: mode 0 `nbdu nbdv u1 v1 u2 v2`, 1 `upars vpars`.
IntCurveSurface_ThePolyhedronOfHInter ga_ics_polyhedron(GaReader& r,const occ::handle<Adaptor3d_Surface>& s){
 if(r.integer()==0){int nu=r.integer(),nv=r.integer();double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();
  return IntCurveSurface_ThePolyhedronOfHInter(s,nu,nv,u1,v1,u2,v2);}
 NCollection_Array1<double> up=ga_ics_params(r),vp=ga_ics_params(r);return IntCurveSurface_ThePolyhedronOfHInter(s,up,vp);
}
// The points, parameters and boundary pairs of a polyhedron.
void ga_ics_polyhedron_points(const IntCurveSurface_ThePolyhedronOfHInter& p){
 int nu,nv;p.Size(nu,nv);int n=p.NbPoints();
 for(int i=1;i<=n;++i){double u,v;ga_pnt(p.Point(i,u,v));ga_real(u);ga_real(v);gp_Pnt q;p.Point(i,q);ga_pnt(q);ga_pnt(p.Point(i));
  p.Parameters(i,u,v);ga_real(u);ga_real(v);
  for(int j:{i+1,i-1,i+nv+1,i-nv-1,i+2})if(j>=1&&j<=n)ga_flag(p.IsOnBound(i,j));}
}
// The triangles of a polyhedron: points, boxes, plane equations, containment, TriConnex.
void ga_ics_polyhedron_triangles(const IntCurveSurface_ThePolyhedronOfHInter& p,const gp_Pnt& probe){
 using T=IntCurveSurface_ThePolyhedronToolOfHInter;int nt=p.NbTriangles();
 for(int t=1;t<=nt;++t){int a,b,c;p.Triangle(t,a,b,c);ga_int(a);ga_int(b);ga_int(c);ga_box3d_print(p.ComponentsBounding()->Value(t));
  gp_XYZ nrm;double d;p.PlaneEquation(t,nrm,d);ga_xyz(nrm);ga_real(d);
  gp_Pnt g((p.Point(a).XYZ()+p.Point(b).XYZ()+p.Point(c).XYZ())/3.);ga_flag(p.Contain(t,g));ga_flag(p.Contain(t,probe));}
 for(int t=0;t<=nt;++t){int a,b,c;p.Triangle(t<1?1:t,a,b,c);
  for(int pivot:{a,b,c})for(int pedge:{0,a,b,c})if(pedge!=pivot){int tc=-9,o=-9;ga_int(p.TriConnex(t,pivot,pedge,tc,o));ga_int(tc);ga_int(o);
   int tc2=-9,o2=-9;ga_int(T::TriConnex(p,t,pivot,pedge,tc2,o2));ga_int(tc2);ga_int(o2);}}
}
// `surface polyhedron detail probe flags(4) flec`: every polyhedron and tool member
// (detail 0 prints the summary only).
void ga_ics_polyhedron_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);
 GA_TRY(IntCurveSurface_ThePolyhedronOfHInter p=ga_ics_polyhedron(r,s);using T=IntCurveSurface_ThePolyhedronToolOfHInter;
  bool detail=r.flag();gp_Pnt probe=r.pnt();int nu,nv;p.Size(nu,nv);ga_int(nu);ga_int(nv);ga_int(p.NbTriangles());ga_int(p.NbPoints());
  ga_real(p.DeflectionOverEstimation());ga_real(p.GetBorderDeflection());ga_box3d_print(p.Bounding());
  ga_int(T::NbTriangles(p));ga_real(T::DeflectionOverEstimation(p));ga_real(T::GetBorderDeflection(p));ga_box3d_print(T::Bounding(p));
  ga_int(p.ComponentsBounding()->Length());ga_int(T::ComponentsBounding(p)->Length());ga_pnt(T::Point(p,1));
  int a,b,c;T::Triangle(p,1,a,b,c);ga_int(a);ga_int(b);ga_int(c);ga_flag(T::IsOnBound(p,1,2));
  if(detail){ga_ics_polyhedron_points(p);ga_ics_polyhedron_triangles(p,probe);}
  bool f[4];for(bool& x:f)x=r.flag();p.UMinSingularity(f[0]);p.UMaxSingularity(f[1]);p.VMinSingularity(f[2]);p.VMaxSingularity(f[3]);
  ga_flag(p.HasUMinSingularity());ga_flag(p.HasUMaxSingularity());ga_flag(p.HasVMinSingularity());ga_flag(p.HasVMaxSingularity());
  p.Point(gp_Pnt(9,9,9),1,1,9.,9.);ga_pnt(p.Point(1));p.Dump();T::Dump(p);
  p.DeflectionOverEstimation(r.real());ga_real(p.DeflectionOverEstimation());ga_box3d_print(p.Bounding());p.FillBounding();
  ga_box3d_print(p.ComponentsBounding()->Value(1));ga_box3d_print(p.ComponentsBounding()->Value(p.NbTriangles()));p.Destroy());
 ga_end();
}
// `op args`: the IntCurveSurface_PolyhedronUtils and _PolygonUtils functions on data:
// 0 PlaneEquation p1 p2 p3, 1 Contain p1 p2 p3 p, 2 ComputeDeflectionWithCenter p1 p2 p3 c,
// 3 Triangle index nbdv, 4 TriConnex t pivot pedge nbdu nbdv, 5 NbTriangles/NbPoints nbdu nbdv,
// 6 SetDeflectionOverEstimation flec, 7 ApproxParamOnCurve index param binf bsup n params.
void ga_ics_utils_case(std::istream& input){
 GaReader r{input};namespace PU=IntCurveSurface_PolyhedronUtils;int op=r.integer();
 if(op<=2){gp_Pnt p1=r.pnt(),p2=r.pnt(),p3=r.pnt();
  if(op==0){gp_XYZ n(-1,-2,-3);double d=-4;PU::PlaneEquation(p1,p2,p3,n,d);ga_xyz(n);ga_real(d);}
  else if(op==1)ga_flag(PU::Contain(p1,p2,p3,r.pnt()));else ga_real(PU::ComputeDeflectionWithCenter(p1,p2,p3,r.pnt()));}
 else if(op==3){int i=r.integer(),nv=r.integer();int a=-9,b=-9,c=-9;PU::Triangle(i,a,b,c,nv);ga_int(a);ga_int(b);ga_int(c);}
 else if(op==4){int t=r.integer(),pv=r.integer(),pe=r.integer(),nu=r.integer(),nv=r.integer();int tc=-9,o=-9;
  ga_int(PU::TriConnex(t,pv,pe,tc,o,nu,nv));ga_int(tc);ga_int(o);}
 else if(op==5){int nu=r.integer(),nv=r.integer();ga_int(PU::NbTriangles(nu,nv));ga_int(PU::NbPoints(nu,nv));}
 else if(op==6){double d=-1;Bnd_Box b;b.Add(gp_Pnt(0,0,0));PU::SetDeflectionOverEstimation(r.real(),d,b);ga_real(d);ga_box3d_print(b);}
 else{int i=r.integer();double t=r.real(),bi=r.real(),bs=r.real();int n=r.integer();int np=r.integer();occ::handle<NCollection_HArray1<double>> h;
  if(np>0){h=new NCollection_HArray1<double>(1,np);for(int k=1;k<=np;++k)h->SetValue(k,r.real());}
  GA_TRY(ga_real(IntCurveSurface_PolygonUtils::ApproxParamOnCurve(i,t,bi,bs,n,h)));}
 ga_end();
}
