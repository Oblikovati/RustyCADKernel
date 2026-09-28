// Poly triangulation, package functions, text I/O and Poly_Connect oracle at pinned OCCT 3d097a0.
#include <Poly.hxx>
#include <Poly_Connect.hxx>
#include <Poly_Polygon2D.hxx>
#include <Poly_Polygon3D.hxx>
#include <Poly_Triangulation.hxx>
#include <NCollection_List.hxx>
#include <Bnd_Box.hxx>
namespace poly_probe {
void failure(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';}
int integer(std::istream& in){double v;in>>v;return int(v);}
double real(std::istream& in){double v;in>>v;return v;}
occ::handle<Poly_Triangulation> mesh(std::istream& in){
 bool single=integer(in)!=0;int n=integer(in),t=integer(in);
 occ::handle<Poly_Triangulation> m=new Poly_Triangulation();m->SetDoublePrecision(!single);
 m->ResizeNodes(n,false);m->ResizeTriangles(t,false);
 for(int i=1;i<=n;++i){double x=real(in),y=real(in),z=real(in);m->SetNode(i,gp_Pnt(x,y,z));}
 for(int i=1;i<=t;++i){int a=integer(in),b=integer(in),c=integer(in);m->SetTriangle(i,Poly_Triangle(a,b,c));}
 return m;
}
void box(const Bnd_Box& b){
 if(b.IsVoid()){std::cout<<"void ";return;}
 double v[6];b.Get(v[0],v[1],v[2],v[3],v[4],v[5]);for(double x:v)std::cout<<bits(x)<<' ';
}
void fvec(const NCollection_Vec3<float>& v){std::cout<<bits(v.x())<<' '<<bits(v.y())<<' '<<bits(v.z())<<' ';}
gp_Trsf transform(std::istream& in){
 double tx=real(in),ty=real(in),tz=real(in),angle=real(in);int form=integer(in);gp_Trsf t;
 if(form==1){t.SetTranslation(gp_Vec(tx,ty,tz));}
 else if(form==2){t.SetRotation(gp_Ax1(gp_Pnt(tx,ty,tz),gp_Dir(0,0,1)),angle);}
 else if(form==3){t.SetScale(gp_Pnt(tx,ty,tz),angle);}
 else if(form==4){t.SetMirror(gp_Pnt(tx,ty,tz));}
 return t;
}
void normals(const occ::handle<Poly_Triangulation>& m){
 try{m->ComputeNormals();NCollection_Vec3<float> v;for(int i=1;i<=m->NbNodes();++i){m->Normal(i,v);fvec(v);}
  xyz(m->Normal(1).XYZ());}catch(const Standard_Failure& f){failure(f);}
}
void ranges(const occ::handle<Poly_Triangulation>& m,const gp_Trsf& t,bool accurate){
 Bnd_Box b;std::cout<<m->MinMax(b,t,accurate)<<' ';box(b);
 m->UpdateCachedMinMax();std::cout<<m->HasCachedMinMax()<<' ';box(m->CachedMinMax());
 Bnd_Box c;std::cout<<m->MinMax(c,t,accurate)<<' ';box(c);
 Bnd_Box d;std::cout<<m->MinMax(d,t,false)<<' ';box(d);
 m->SetCachedMinMax(Bnd_Box());std::cout<<m->HasCachedMinMax()<<' ';
}
void rays(const occ::handle<Poly_Triangulation>& m,std::istream& in){
 double o[3],d[3];for(double& v:o)v=real(in);for(double& v:d)v=real(in);
 try{gp_Ax1 axis(gp_Pnt(o[0],o[1],o[2]),gp_Dir(d[0],d[1],d[2]));
  for(int closest=1;closest>=0;--closest){Poly_Triangle hit(-1,-1,-1);double dist=91;
   bool ok=Poly::Intersect(m,axis,closest!=0,hit,dist);int a,b,c;hit.Get(a,b,c);
   std::cout<<ok<<' '<<a<<' '<<b<<' '<<c<<' '<<bits(dist)<<' ';}
 }catch(const Standard_Failure& f){failure(f);}
}
void catenated(const occ::handle<Poly_Triangulation>& m){
 NCollection_List<occ::handle<Poly_Triangulation>> list;list.Append(m);list.Append(occ::handle<Poly_Triangulation>());list.Append(m->Copy());
 try{occ::handle<Poly_Triangulation> r=Poly::Catenate(list);
  std::cout<<r->NbNodes()<<' '<<r->NbTriangles()<<' '<<r->HasUVNodes()<<' '<<r->HasNormals()<<' ';
  int a,b,c;r->Triangle(r->NbTriangles()).Get(a,b,c);std::cout<<a<<' '<<b<<' '<<c<<' ';xyz(r->Node(r->NbNodes()).XYZ());
 }catch(const Standard_Failure& f){failure(f);}
}
void mesh_case(std::istream& in){
 occ::handle<Poly_Triangulation> m=mesh(in);gp_Trsf t=transform(in);bool accurate=integer(in)!=0;
 std::cout<<m->NbNodes()<<' '<<m->NbTriangles()<<' '<<m->HasGeometry()<<' '<<m->IsDoublePrecision()<<' ';
 for(int i=1;i<=m->NbNodes();++i)xyz(m->Node(i).XYZ());
 ranges(m,t,accurate);rays(m,in);catenated(m);normals(m);
 std::ostringstream json;m->DumpJson(json);std::cout<<json.str()<<' ';
 occ::handle<NCollection_HArray1<float>> flat=m->MapNormalArray();std::cout<<(flat.IsNull()?0:flat->Length())<<'\n';
}
void point_on_triangle_case(std::istream& in){
 double v[8];for(double& x:v)x=real(in);gp_XY uv(91,92);
 double d=Poly::PointOnTriangle(gp_XY(v[0],v[1]),gp_XY(v[2],v[3]),gp_XY(v[4],v[5]),gp_XY(v[6],v[7]),uv);
 std::cout<<bits(d)<<' ';xy(uv);std::cout<<'\n';
}
void tri_line_case(std::istream& in){
 double v[15];for(double& x:v)x=real(in);
 try{double p=91;int r=Poly::IntersectTriLine(gp_XYZ(v[0],v[1],v[2]),gp_Dir(v[3],v[4],v[5]),gp_XYZ(v[6],v[7],v[8]),gp_XYZ(v[9],v[10],v[11]),gp_XYZ(v[12],v[13],v[14]),p);
  std::cout<<r<<' '<<bits(p)<<' ';}catch(const Standard_Failure& f){failure(f);}
 std::cout<<'\n';
}
void properties_case(std::istream& in){
 int n=integer(in);NCollection_Sequence<gp_Pnt2d> points;
 for(int i=0;i<n;++i){double x=real(in),y=real(in);points.Append(gp_Pnt2d(x,y));}
 double area=91,perimeter=92;bool ok=Poly::PolygonProperties(points,area,perimeter);
 std::cout<<ok<<' '<<bits(area)<<' '<<bits(perimeter)<<'\n';
}
std::string flat(const std::string& text){std::string r=text;for(char& c:r)if(c=='\n')c='|';return r;}
template <class T> void round_trip(const occ::handle<T>& shape,occ::handle<T> (*read)(Standard_IStream&)){
 std::ostringstream compact,dump;Poly::Write(shape,compact,true);Poly::Dump(shape,dump);
 std::cout<<flat(compact.str())<<' '<<flat(dump.str())<<' ';
 std::istringstream again(compact.str()+compact.str());
 try{occ::handle<T> first=read(again),second=read(again);std::ostringstream text;Poly::Write(first,text);Poly::Write(second,text);
  std::cout<<flat(text.str())<<' ';}catch(const Standard_Failure& f){failure(f);}
 std::ostringstream json;shape->DumpJson(json);std::cout<<json.str()<<' ';
}
void io_case(std::istream& in){
 occ::handle<Poly_Triangulation> m=mesh(in);int flags=integer(in);double deflection=real(in),scale=real(in);
 m->Deflection(deflection);if(flags&1){m->AddUVNodes();for(int i=1;i<=m->NbNodes();++i)m->SetUVNode(i,gp_Pnt2d(m->Node(i).X()*scale,m->Node(i).Y()/scale));}
 round_trip(m,&Poly::ReadTriangulation);
 NCollection_Array1<gp_Pnt> nodes(1,m->NbNodes());NCollection_Array1<double> parameters(1,m->NbNodes());
 NCollection_Array1<gp_Pnt2d> uv(1,m->NbNodes());
 for(int i=1;i<=m->NbNodes();++i){nodes(i)=m->Node(i);parameters(i)=i*scale;uv(i)=gp_Pnt2d(nodes(i).Y(),nodes(i).Z());}
 occ::handle<Poly_Polygon3D> p3=(flags&2)?new Poly_Polygon3D(nodes,parameters):new Poly_Polygon3D(nodes);p3->Deflection(deflection*2);
 round_trip(p3,&Poly::ReadPolygon3D);
 occ::handle<Poly_Polygon2D> p2=new Poly_Polygon2D(uv);p2->Deflection(deflection/3);
 round_trip(p2,&Poly::ReadPolygon2D);
 std::istringstream wrong("Poly_Polygon2D 1 0 0 0");std::cout<<Poly::ReadTriangulation(wrong).IsNull()<<'\n';
}
void connect_case(std::istream& in){
 occ::handle<Poly_Triangulation> m=mesh(in);
 try{Poly_Connect c(m);
  for(int n=1;n<=m->NbNodes();++n)std::cout<<c.Triangle(n)<<' ';
  for(int t=1;t<=m->NbTriangles();++t){int a,b,d,e,f,g;c.Triangles(t,a,b,d);c.Nodes(t,e,f,g);std::cout<<a<<' '<<b<<' '<<d<<' '<<e<<' '<<f<<' '<<g<<' ';}
  for(int n=1;n<=m->NbNodes();++n){std::cout<<'[';for(c.Initialize(n);c.More();c.Next())std::cout<<c.Value()<<' ';std::cout<<"] ";}
 }catch(const Standard_Failure& f){failure(f);}
 std::cout<<'\n';
}
} // namespace poly_probe
