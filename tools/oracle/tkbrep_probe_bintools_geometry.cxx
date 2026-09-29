// BinTools oracle, part 2: every Geom, Geom2d curve and Geom surface class through the
// BinTools curve, 2D curve and surface sets: the written bytes, the classes read back,
// their evaluation and the bytes written again. The Rust twin is
// crates/tkbrep/examples/br_probe/bintools_geometry.rs.
#include <BinTools_Curve2dSet.hxx>
#include <BinTools_CurveSet.hxx>
#include <BinTools_SurfaceSet.hxx>
#include <Geom_BSplineCurve.hxx>
#include <Geom_BSplineSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_Ellipse.hxx>
#include <Geom_Hyperbola.hxx>
#include <Geom_OffsetCurve.hxx>
#include <Geom_Parabola.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <Geom2d_BSplineCurve.hxx>
#include <Geom2d_BezierCurve.hxx>
#include <Geom2d_Ellipse.hxx>
#include <Geom2d_Hyperbola.hxx>
#include <Geom2d_OffsetCurve.hxx>
#include <Geom2d_Parabola.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <NCollection_Array2.hxx>
struct BtGeometry{
 std::vector<occ::handle<Geom_Curve>> curves;std::vector<occ::handle<Geom_Surface>> surfaces;
 std::vector<occ::handle<Geom2d_Curve>> pcurves;};
gp_Ax2 bt_ax2(BrReader& r){gp_Pnt p=br_point(r);gp_Dir n=br_direction(r);return gp_Ax2(p,n,br_direction(r));}
gp_Ax3 bt_ax3(BrReader& r){gp_Pnt p=br_point(r);gp_Dir n=br_direction(r);gp_Ax3 a(p,n,br_direction(r));if(r.flag())a.YReverse();return a;}
gp_Ax22d bt_ax22d(BrReader& r){gp_Pnt2d p=br_point2d(r);double x=r.real();gp_Dir2d d(x,r.real());return gp_Ax22d(p,d,r.flag());}
// `rational n (P [w])...`: the poles and, when rational, the weights.
template<class P,class F> void bt_poles(BrReader& r,bool rational,int n,NCollection_Array1<P>& poles,NCollection_Array1<double>& w,F point){
 for(int i=1;i<=n;++i){poles(i)=point(r);w(i)=rational?r.real():1.0;}}
void bt_knots(BrReader& r,NCollection_Array1<double>& k,NCollection_Array1<int>& m){
 for(int i=k.Lower();i<=k.Upper();++i){k(i)=r.real();m(i)=r.integer();}}
occ::handle<Geom_Curve> bt_spline(BrReader& r,int kind){
 bool rational=r.flag();
 if(kind==5){int n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<double> w(1,n);bt_poles(r,rational,n,p,w,br_point);
  return rational?new Geom_BezierCurve(p,w):new Geom_BezierCurve(p);}
 bool periodic=r.flag();int degree=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<double> w(1,n);bt_poles(r,rational,n,p,w,br_point);
 int nk=r.integer();NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);bt_knots(r,k,m);
 return rational?new Geom_BSplineCurve(p,w,k,m,degree,periodic):new Geom_BSplineCurve(p,k,m,degree,periodic);}
occ::handle<Geom_Curve> bt_curve(BtGeometry& g,BrReader& r,int kind){
 switch(kind){
  case 0:{gp_Pnt p=br_point(r);return new Geom_Line(p,br_direction(r));}
  case 1:{gp_Ax2 a=bt_ax2(r);return new Geom_Circle(a,r.real());}
  case 2:{gp_Ax2 a=bt_ax2(r);double r1=r.real();return new Geom_Ellipse(a,r1,r.real());}
  case 3:{gp_Ax2 a=bt_ax2(r);return new Geom_Parabola(a,r.real());}
  case 4:{gp_Ax2 a=bt_ax2(r);double r1=r.real();return new Geom_Hyperbola(a,r1,r.real());}
  case 5:case 6:return bt_spline(r,kind);
  case 7:{int i=r.integer();double u1=r.real();return new Geom_TrimmedCurve(br_at(g.curves,i),u1,r.real());}
  default:{int i=r.integer();double d=r.real();return new Geom_OffsetCurve(br_at(g.curves,i),d,br_direction(r));}}}
occ::handle<Geom_Surface> bt_spline_surface(BrReader& r,int kind){
 bool rational=r.flag();bool uperiodic=false,vperiodic=false;int udeg=0,vdeg=0;
 if(kind==18){uperiodic=r.flag();vperiodic=r.flag();udeg=r.integer();vdeg=r.integer();}
 int nu=r.integer(),nv=r.integer();NCollection_Array2<gp_Pnt> p(1,nu,1,nv);NCollection_Array2<double> w(1,nu,1,nv);
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j){p(i,j)=br_point(r);w(i,j)=rational?r.real():1.0;}
 if(kind==17)return rational?new Geom_BezierSurface(p,w):new Geom_BezierSurface(p);
 int nuk=r.integer();NCollection_Array1<double> uk(1,nuk);NCollection_Array1<int> um(1,nuk);bt_knots(r,uk,um);
 int nvk=r.integer();NCollection_Array1<double> vk(1,nvk);NCollection_Array1<int> vm(1,nvk);bt_knots(r,vk,vm);
 if(rational)return new Geom_BSplineSurface(p,w,uk,vk,um,vm,udeg,vdeg,uperiodic,vperiodic);
 return new Geom_BSplineSurface(p,uk,vk,um,vm,udeg,vdeg,uperiodic,vperiodic);}
occ::handle<Geom_Surface> bt_surface(BtGeometry& g,BrReader& r,int kind){
 switch(kind){
  case 10:return new Geom_Plane(bt_ax3(r));
  case 11:{gp_Ax3 a=bt_ax3(r);return new Geom_CylindricalSurface(a,r.real());}
  case 12:{gp_Ax3 a=bt_ax3(r);double rad=r.real();return new Geom_ConicalSurface(a,r.real(),rad);}
  case 13:{gp_Ax3 a=bt_ax3(r);return new Geom_SphericalSurface(a,r.real());}
  case 14:{gp_Ax3 a=bt_ax3(r);double r1=r.real();return new Geom_ToroidalSurface(a,r1,r.real());}
  case 15:{int i=r.integer();return new Geom_SurfaceOfLinearExtrusion(br_at(g.curves,i),br_direction(r));}
  case 16:{int i=r.integer();gp_Pnt p=br_point(r);return new Geom_SurfaceOfRevolution(br_at(g.curves,i),gp_Ax1(p,br_direction(r)));}
  case 17:case 18:return bt_spline_surface(r,kind);
  case 19:{int i=r.integer();double u1=r.real(),u2=r.real(),v1=r.real();return new Geom_RectangularTrimmedSurface(br_at(g.surfaces,i),u1,u2,v1,r.real());}
  default:{int i=r.integer();return new Geom_OffsetSurface(br_at(g.surfaces,i),r.real());}}}
occ::handle<Geom2d_Curve> bt_spline2d(BrReader& r,int kind){
 bool rational=r.flag();
 if(kind==35){int n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);bt_poles(r,rational,n,p,w,br_point2d);
  return rational?new Geom2d_BezierCurve(p,w):new Geom2d_BezierCurve(p);}
 bool periodic=r.flag();int degree=r.integer(),n=r.integer();NCollection_Array1<gp_Pnt2d> p(1,n);NCollection_Array1<double> w(1,n);bt_poles(r,rational,n,p,w,br_point2d);
 int nk=r.integer();NCollection_Array1<double> k(1,nk);NCollection_Array1<int> m(1,nk);bt_knots(r,k,m);
 return rational?new Geom2d_BSplineCurve(p,w,k,m,degree,periodic):new Geom2d_BSplineCurve(p,k,m,degree,periodic);}
occ::handle<Geom2d_Curve> bt_curve2d(BtGeometry& g,BrReader& r,int kind){
 switch(kind){
  case 30:{gp_Pnt2d p=br_point2d(r);double x=r.real();return new Geom2d_Line(p,gp_Dir2d(x,r.real()));}
  case 31:{gp_Ax22d a=bt_ax22d(r);return new Geom2d_Circle(a,r.real());}
  case 32:{gp_Ax22d a=bt_ax22d(r);double r1=r.real();return new Geom2d_Ellipse(a,r1,r.real());}
  case 33:{gp_Ax22d a=bt_ax22d(r);return new Geom2d_Parabola(a,r.real());}
  case 34:{gp_Ax22d a=bt_ax22d(r);double r1=r.real();return new Geom2d_Hyperbola(a,r1,r.real());}
  case 35:case 36:return bt_spline2d(r,kind);
  case 37:{int i=r.integer();double u1=r.real();return new Geom2d_TrimmedCurve(br_at(g.pcurves,i),u1,r.real());}
  default:{int i=r.integer();return new Geom2d_OffsetCurve(br_at(g.pcurves,i),r.real());}}}
// One geometry spec; `-` or the construction failure, and a null entry on failure.
void bt_read_geometry(BtGeometry& g,BrReader& r){
 int kind=r.integer();
 try{
  if(kind<10)g.curves.push_back(bt_curve(g,r,kind));
  else if(kind<30)g.surfaces.push_back(bt_surface(g,r,kind));
  else g.pcurves.push_back(bt_curve2d(g,r,kind));
  br_word("-");
 }catch(const Standard_Failure& e){br_word(e.ExceptionType());
  if(kind<10)g.curves.push_back(nullptr);else if(kind<30)g.surfaces.push_back(nullptr);else g.pcurves.push_back(nullptr);}}
BtGeometry bt_geometry(BrReader& r){BtGeometry g;int n=r.integer();for(int i=0;i<n;++i)bt_read_geometry(g,r);return g;}
// The class and D0 of the set members 1..n read back.
void bt_eval(const occ::handle<Geom_Curve>& c){br_word(c->DynamicType()->Name());gp_Pnt p=c->Value(0.3);br_real(p.X());br_real(p.Y());br_real(p.Z());}
void bt_eval(const occ::handle<Geom2d_Curve>& c){br_word(c->DynamicType()->Name());gp_Pnt2d p=c->Value(0.3);br_real(p.X());br_real(p.Y());}
void bt_eval(const occ::handle<Geom_Surface>& s){br_word(s->DynamicType()->Name());gp_Pnt p=s->Value(0.3,0.2);br_real(p.X());br_real(p.Y());br_real(p.Z());}
template<class Set> std::string bt_set_bytes(const Set& set){std::ostringstream o;BT_TRY(set.Write(o));return o.str();}
// Writes a set, reads it back into a new set, evaluates its members and writes it again.
template<class Set,class Get> void bt_set_round_trip(const Set& set,int n,Get get){
 std::string bytes=bt_set_bytes(set);bt_bytes(bytes);
 Set back;std::istringstream is(bytes);BT_TRY(back.Read(is));br_int(int(is.rdstate()));
 for(int i=1;i<=n;++i)BT_TRY(bt_eval(get(back,i)));
 bt_bytes(bt_set_bytes(back));}
// `br_bin_geometry n spec...`: the three sets of the geometry list, each added in list
// order (a failed construction adds nothing).
void br_bin_geometry_case(std::istream& in){
 BrReader r{in};BtGeometry g=bt_geometry(r);
 BinTools_CurveSet curves;int nc=0;
 for(auto& c:g.curves)if(!c.IsNull()){br_int(curves.Add(c));++nc;}
 bt_set_round_trip(curves,nc,[](const BinTools_CurveSet& s,int i){return s.Curve(i);});
 BinTools_Curve2dSet pcurves;int np=0;
 for(auto& c:g.pcurves)if(!c.IsNull()){br_int(pcurves.Add(c));++np;}
 bt_set_round_trip(pcurves,np,[](const BinTools_Curve2dSet& s,int i){return s.Curve2d(i);});
 BinTools_SurfaceSet surfaces;int ns=0;
 for(auto& s:g.surfaces)if(!s.IsNull()){br_int(surfaces.Add(s));++ns;}
 bt_set_round_trip(surfaces,ns,[](const BinTools_SurfaceSet& s,int i){return s.Surface(i);});
 for(auto& c:g.curves)if(!c.IsNull())br_int(curves.Index(c));
 br_int(curves.Index(nullptr));BT_TRY(br_flag(curves.Curve(0).IsNull()));BT_TRY(curves.Curve(nc+1));
 BT_TRY(pcurves.Curve2d(0));BT_TRY(surfaces.Surface(ns+1));
 br_end();}
