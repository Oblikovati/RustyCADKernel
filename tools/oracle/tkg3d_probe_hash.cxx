// TKG3d oracle: the GeomHash hashers (tkg3d_probe.cxx). Hashes print with an 'h' prefix
// so the NaN-tolerant comparator never reads them as doubles.
#include <GeomHash_CurveHasher.hxx>
#include <GeomHash_SurfaceHasher.hxx>
#include <GeomHash_AxisPlacement.pxx>
#include <GeomHash_VectorHasher.pxx>
#include <GeomHash_Polygon2DHasher.hxx>
#include <GeomHash_Polygon3DHasher.hxx>
#include <GeomHash_PolygonOnTriHasher.hxx>
#include <GeomHash_TriangulationHasher.hxx>
#include <Poly_Polygon2D.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_PolygonOnTriangulation.hxx>
#include <Poly_Triangulation.hxx>
void g3_hash_value(size_t v){std::cout<<'h'<<v<<' ';}
// The hasher tolerances: a negative CompTolerance selects the default constructor.
struct G3HashTolerances{bool defaults;double comp,tol;};
G3HashTolerances g3_hash_tolerances(G3Reader& r){double c=r.real(),t=r.real();return {c<0.,c,t};}
template<class Hasher> Hasher g3_hasher(const G3HashTolerances& t){return t.defaults?Hasher():Hasher(t.comp,t.tol);}
// Prints every hash, then equality for every ordered pair (items may repeat a handle);
// each case ends its line.
template<class Hasher,class Key> void g3_hash_all(const Hasher& h,const std::vector<Key>& keys){
 for(const Key& k:keys)g3_hash_value(h(k));
 for(const Key& a:keys)for(const Key& b:keys)g3_flag(h(a,b));
}
// Item grammar of the handle hashers: 0 <object> | 1 (null handle) | 2 index (the same handle
// as an earlier item). `build` reads one object and returns null after reporting a failure.
template<class T,class Build> bool g3_hash_items(G3Reader& r,std::vector<occ::handle<T>>& items,Build build){
 int n=r.integer();
 for(int i=0;i<n;++i){int mode=r.integer();occ::handle<T> h;
  if(mode==0){h=build(r);if(h.IsNull())return false;}
  if(mode==2)h=items[r.integer()];
  items.push_back(h);}
 return true;
}
// g3_hash_curves comp hash n item*n: CurveHasher over curve specs of g3_curve.
void g3_hash_curves_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);std::vector<occ::handle<Geom_Curve>> items;
 if(!g3_hash_items(r,items,[](G3Reader& rr){return g3_built_curve(rr);}))return;
 g3_hash_all(g3_hasher<GeomHash_CurveHasher>(t),items);std::cout<<'\n';
}
// g3_hash_surfaces comp hash n item*n: SurfaceHasher over surface specs of g3_surface.
void g3_hash_surfaces_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);std::vector<occ::handle<Geom_Surface>> items;
 if(!g3_hash_items(r,items,[](G3Reader& rr){return g3_built_surface(rr);}))return;
 g3_hash_all(g3_hasher<GeomHash_SurfaceHasher>(t),items);std::cout<<'\n';
}
// g3_hash_primitives comp hash P1 P2 D1 D2 V1 V2 F1 F2 (F = P N X): the point, direction,
// vector and gp_Ax2 hashers on raw values, including huge and non-finite coordinates.
void g3_hash_primitives_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);
 std::vector<gp_Pnt> p{r.pnt(),r.pnt()};std::vector<gp_Dir> d{r.dir(),r.dir()};std::vector<gp_Vec> v{r.vec(),r.vec()};
 std::vector<gp_Ax2> f;
 for(int i=0;i<2;++i){try{f.push_back(r.ax2());}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}}
 std::cout<<"p ";g3_hash_all(g3_hasher<GeomHash_PointHasher>(t),p);
 std::cout<<"d ";g3_hash_all(g3_hasher<GeomHash_DirectionHasher>(t),d);
 std::cout<<"v ";g3_hash_all(g3_hasher<GeomHash_VectorHasher>(t),v);
 std::cout<<"f ";g3_hash_all(g3_hasher<GeomHash_AxisPlacement>(t),f);std::cout<<'\n';
}
// Mesh objects: poly2d defl n (x y)*n | poly3d defl params n (x y z)*n [u*n] |
// ontri defl params n node*n [u*n] | tri defl uv nn nt (x y z)*nn [(u v)*nn] (a b c)*nt.
occ::handle<Poly_Polygon2D> g3_poly2d(G3Reader& r){
 double f=r.real();int n=r.integer();occ::handle<Poly_Polygon2D> p=new Poly_Polygon2D(n);
 for(int i=1;i<=n;++i){double x=r.real();p->ChangeNodes().SetValue(i,gp_Pnt2d(x,r.real()));}
 p->Deflection(f);return p;
}
occ::handle<Poly_Polygon3D> g3_poly3d(G3Reader& r){
 double f=r.real();bool u=r.flag();int n=r.integer();occ::handle<Poly_Polygon3D> p=new Poly_Polygon3D(n,u);
 for(int i=1;i<=n;++i)p->ChangeNodes().SetValue(i,r.pnt());
 if(u)for(int i=1;i<=n;++i)p->ChangeParameters().SetValue(i,r.real());
 p->Deflection(f);return p;
}
occ::handle<Poly_PolygonOnTriangulation> g3_ontri(G3Reader& r){
 double f=r.real();bool u=r.flag();int n=r.integer();
 occ::handle<Poly_PolygonOnTriangulation> p=new Poly_PolygonOnTriangulation(n,u);
 for(int i=1;i<=n;++i)p->SetNode(i,r.integer());
 if(u)for(int i=1;i<=n;++i)p->SetParameter(i,r.real());
 p->Deflection(f);return p;
}
occ::handle<Poly_Triangulation> g3_tri(G3Reader& r){
 double f=r.real();bool uv=r.flag();int nn=r.integer(),nt=r.integer();
 occ::handle<Poly_Triangulation> t=new Poly_Triangulation(nn,nt,uv);
 for(int i=1;i<=nn;++i)t->SetNode(i,r.pnt());
 if(uv)for(int i=1;i<=nn;++i){double x=r.real();t->SetUVNode(i,gp_Pnt2d(x,r.real()));}
 for(int i=1;i<=nt;++i){int a=r.integer(),b=r.integer();t->SetTriangle(i,Poly_Triangle(a,b,r.integer()));}
 t->Deflection(f);return t;
}
void g3_hash_poly2d_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);std::vector<occ::handle<Poly_Polygon2D>> items;
 g3_hash_items(r,items,g3_poly2d);g3_hash_all(g3_hasher<GeomHash_Polygon2DHasher>(t),items);std::cout<<'\n';
}
void g3_hash_poly3d_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);std::vector<occ::handle<Poly_Polygon3D>> items;
 g3_hash_items(r,items,g3_poly3d);g3_hash_all(g3_hasher<GeomHash_Polygon3DHasher>(t),items);std::cout<<'\n';
}
void g3_hash_triangulation_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);std::vector<occ::handle<Poly_Triangulation>> items;
 g3_hash_items(r,items,g3_tri);g3_hash_all(g3_hasher<GeomHash_TriangulationHasher>(t),items);std::cout<<'\n';
}
// g3_hash_ontri comp hash n (id item)*n: PolygonOnTriHasher keys pair a TriRepId with a polygon.
void g3_hash_ontri_case(std::istream& in){
 G3Reader r{in};G3HashTolerances t=g3_hash_tolerances(r);int n=r.integer();std::vector<PolygonOnTriHashKey> keys;
 for(int i=0;i<n;++i){uint32_t id=uint32_t(r.real());int mode=r.integer();occ::handle<Poly_PolygonOnTriangulation> p;
  if(mode==0)p=g3_ontri(r);
  if(mode==2)p=keys[r.integer()].Poly;
  keys.push_back(PolygonOnTriHashKey{p,id});}
 g3_hash_all(g3_hasher<GeomHash_PolygonOnTriHasher>(t),keys);std::cout<<'\n';
}
