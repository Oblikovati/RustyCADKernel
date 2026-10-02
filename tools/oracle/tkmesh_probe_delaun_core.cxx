// Pinned public Delaun implementation plus private polygon members accessed by explicit instantiation.
// This does not alter OCCT headers or translation units: member-pointer access bypass only.
#include <algorithm>
#include <vector>
#include <BRepMesh_Delaun.hxx>
template<class Tag,typename Tag::type Member>struct DelaunAccess{friend typename Tag::type delaun_member(Tag){return Member;}};
struct DelaunWindingTag{using type=bool(BRepMesh_Delaun::*)(const int&,const IMeshData::VectorOfInteger&)const;friend type delaun_member(DelaunWindingTag);};
struct DelaunAreaTag{using type=double(BRepMesh_Delaun::*)(const IMeshData::SequenceOfInteger&,int,int)const;friend type delaun_member(DelaunAreaTag);};
struct DelaunIntersectionTag{using type=BRepMesh_GeomTool::IntFlag(BRepMesh_Delaun::*)(const BRepMesh_Edge&,const BRepMesh_Edge&,bool,bool,gp_Pnt2d&)const;friend type delaun_member(DelaunIntersectionTag);};
struct DelaunMeshPolygonTag{using type=void(BRepMesh_Delaun::*)(IMeshData::SequenceOfInteger&,IMeshData::SequenceOfBndB2d&,Handle(IMeshData::MapOfInteger));friend type delaun_member(DelaunMeshPolygonTag);};
struct DelaunSuperMeshTag{using type=void(BRepMesh_Delaun::*)(const Bnd_Box2d&);friend type delaun_member(DelaunSuperMeshTag);};
template struct DelaunAccess<DelaunWindingTag,&BRepMesh_Delaun::isVertexInsidePolygon>;
template struct DelaunAccess<DelaunAreaTag,&BRepMesh_Delaun::polyArea>;
template struct DelaunAccess<DelaunIntersectionTag,&BRepMesh_Delaun::intSegSeg>;
template struct DelaunAccess<DelaunMeshPolygonTag,&BRepMesh_Delaun::meshPolygon>;
template struct DelaunAccess<DelaunSuperMeshTag,&BRepMesh_Delaun::superMesh>;
static auto delaun_core_mesh(int kind,int flags){
 occ::handle<NCollection_IncAllocator> arena=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun> mesh=new BRepMesh_DataStructureOfDelaun(arena,10);
 gp_XY points[]={gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2)};
 if(kind==1)std::swap(points[1],points[3]);else if(kind==2)points[1]=gp_XY(1e-10,0);
 else if(kind==3){points[2]=gp_XY(4,0);points[3]=gp_XY(6,0);}else if(kind==4){for(auto&p:points)p+=gp_XY(1e9,-1e9);}else if(kind==5)points[2]=gp_XY(.5,.5);
 for(auto&p:points)mesh->AddNode(BRepMesh_Vertex(p,0,BRepMesh_Free),true);
 int i=0;for(auto pair:{std::pair<int,int>{1,2},{2,3},{3,4},{4,1},{1,3}}){auto freedom=(flags&(1<<i))==0?BRepMesh_Free:(i%2==0?BRepMesh_Frontier:BRepMesh_Fixed);mesh->AddLink(BRepMesh_Edge(pair.first,pair.second,freedom));++i;}
 mesh->AddElement(BRepMesh_Triangle({1,2,5},{true,true,false},BRepMesh_Free));mesh->AddElement(BRepMesh_Triangle({5,3,4},{true,true,true},BRepMesh_Free));return mesh;
}
static void delaun_core_map(const IMeshData::MapOfInteger& map){ga_int(map.Extent());for(IMeshData::MapOfInteger::Iterator it(map);it.More();it.Next())ga_int(it.Key());}
static void delaun_core_snapshot(const BRepMesh_Delaun& delaun){
 delaun_core_map(*delaun.Frontier());delaun_core_map(*delaun.InternalEdges());delaun_core_map(*delaun.FreeEdges());
 delaun_core_map(delaun.Result()->ElementsOfDomain());delaun_core_map(delaun.Result()->LinksOfDomain());
 for(int i=1;i<=4;++i){auto&v=delaun.GetVertex(i);ga_real(v.Coord().X());ga_real(v.Coord().Y());ga_int(v.Movability());}
 for(int i=1;i<=5;++i){auto&e=delaun.GetEdge(i);ga_int(e.FirstNode());ga_int(e.LastNode());ga_int(e.Movability());}
 for(int i=1;i<=2;++i){auto&t=delaun.GetTriangle(i);for(auto e:t.myEdges)ga_int(e);for(auto o:t.myOrientations)ga_flag(o);ga_int(t.Movability());}
}
static void delaun_core_contains(const BRepMesh_Delaun&delaun,int triangle,int kind){
 gp_XY offset=kind==4?gp_XY(1e9,-1e9):gp_XY();
 for(auto p:{gp_XY(1,1),gp_XY(1,0),gp_XY(0,0),gp_XY(-1,1),gp_XY(1,1e-10),gp_XY(1,.3)})for(double tolerance:{-1.,0.,1e-18,.1}){
 int edge=99;GA_TRY(ga_flag(delaun.Contains(triangle,BRepMesh_Vertex(p+offset,0,BRepMesh_Free),tolerance,edge)));ga_int(edge);}
}
#define DC_TRY(expr) try{expr;std::cout<<"OK ";}catch(const Standard_Failure&e){std::cout<<e.ExceptionType()<<' ';}
void mesh_delaun_core(std::istream&input){
 std::array<int,6>v;for(auto& value:v)input>>value;
 DC_TRY(auto mesh=delaun_core_mesh(v[0],v[1]);BRepMesh_Delaun delaun(mesh,v[2],v[3],v[4]!=0);
 ga_flag(delaun.Result()==mesh);ga_flag(delaun.Circles().IsEmpty());ga_flag(delaun.UseEdge(-1));delaun_core_snapshot(delaun);
 for(int triangle=1;triangle<=2;++triangle)delaun_core_contains(delaun,triangle,v[0]);
 IMeshData::VectorOfInteger auxiliary(3);for(int i=1;i<=4;++i)if(v[5]&(1<<(i-1)))auxiliary.Append(i);
 delaun.SetAuxVertices(auxiliary);auxiliary.Clear();DC_TRY(delaun.RemoveAuxElements());delaun_core_snapshot(delaun);
 DC_TRY(delaun.InitCirclesTool(v[2],v[3]));ga_flag(delaun.Circles().IsEmpty()));ga_end();
}
static void delaun_polygon_area(const BRepMesh_Delaun&delaun,int reverse){
 IMeshData::SequenceOfInteger polygon;for(int i=1;i<=4;++i)polygon.Append(reverse==0?i:-(5-i));
 for(auto bounds:{std::pair<int,int>{1,4},{1,3},{2,4},{4,1},{5,6},{0,2},{1,5}}){GA_TRY(ga_real((delaun.*delaun_member(DelaunAreaTag{}))(polygon,bounds.first,bounds.second)));}
}
void mesh_delaun_polygon(std::istream&input){
 std::array<int,5>v;for(auto&value:v)input>>value;
 DC_TRY(auto mesh=delaun_core_mesh(v[0],0);gp_XY center=v[1]==0?gp_XY(1,1):(v[1]==1?gp_XY(0,0):gp_XY(-1,1));
 int index=mesh->AddNode(BRepMesh_Vertex(center,0,BRepMesh_Free),true);BRepMesh_Delaun delaun(mesh,-1,-1,false);
 IMeshData::VectorOfInteger vertices(5);for(int i=0;i<v[2];++i)vertices.Append(i%4+1);
 ga_flag((delaun.*delaun_member(DelaunWindingTag{}))(index,vertices));delaun_polygon_area(delaun,v[3]);
 for(int a=1;a<=5;++a)for(int b=1;b<=5;++b){gp_Pnt2d point(7,8);ga_int((delaun.*delaun_member(DelaunIntersectionTag{}))(delaun.GetEdge(a),delaun.GetEdge(b),v[4]&1,v[4]&2,point));ga_real(point.X());ga_real(point.Y());});ga_end();
}
static void delaun_polygon_mesh_snapshot(const BRepMesh_Delaun&delaun){
 auto mesh=delaun.Result();std::vector<int>links,triangles;for(IMeshData::IteratorOfMapOfInteger it(mesh->LinksOfDomain());it.More();it.Next())links.push_back(it.Key());
 for(IMeshData::IteratorOfMapOfInteger it(mesh->ElementsOfDomain());it.More();it.Next())triangles.push_back(it.Key());std::sort(links.begin(),links.end());std::sort(triangles.begin(),triangles.end());
 ga_int((int)links.size());for(auto index:links)ga_int(index);ga_int((int)triangles.size());for(auto index:triangles)ga_int(index);
 for(auto index:links){auto&e=delaun.GetEdge(index);ga_int(index);ga_int(e.FirstNode());ga_int(e.LastNode());ga_int(e.Movability());}
 for(auto index:triangles){auto&t=delaun.GetTriangle(index);ga_int(index);for(auto e:t.myEdges)ga_int(e);for(auto o:t.myOrientations)ga_flag(o);ga_int(t.Movability());}
}
void mesh_delaun_polygon_mesh(std::istream&input){
 int kind,reverse,frontier;input>>kind>>reverse>>frontier;occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun>mesh=new BRepMesh_DataStructureOfDelaun(arena,8);
 gp_XY points[]={gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2)};if(kind==1){points[2]=gp_XY(1,1);}
 for(auto&p:points)mesh->AddNode(BRepMesh_Vertex(p,0,BRepMesh_Free),true);
 for(int i=0;i<4;++i)mesh->AddLink(BRepMesh_Edge(i+1,(i+1)%4+1,frontier?BRepMesh_Frontier:BRepMesh_Free));
 BRepMesh_Delaun delaun(mesh,-1,-1,false);IMeshData::SequenceOfInteger polygon;IMeshData::SequenceOfBndB2d boxes;
 for(int i=0;i<4;++i){int edge=reverse?-(4-i):i+1;polygon.Append(edge);auto&e=delaun.GetEdge(std::abs(edge));Bnd_B2d box;box.Add(delaun.GetVertex(edge>0?e.FirstNode():e.LastNode()).Coord());box.Add(delaun.GetVertex(edge>0?e.LastNode():e.FirstNode()).Coord());box.Enlarge(Precision::PConfusion());boxes.Append(box);}
 std::cout<<"OK ";(delaun.*delaun_member(DelaunMeshPolygonTag{}))(polygon,boxes,Handle(IMeshData::MapOfInteger)());delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_super_mesh(std::istream&input){
 double minX,minY,maxX,maxY;input>>minX>>minY>>maxX>>maxY;occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun>mesh=new BRepMesh_DataStructureOfDelaun(arena,3);BRepMesh_Delaun delaun(mesh,-1,-1,false);Bnd_Box2d bounds;
 bounds.Add(gp_Pnt2d(minX,minY));bounds.Add(gp_Pnt2d(maxX,maxY));(delaun.*delaun_member(DelaunSuperMeshTag{}))(bounds);std::cout<<"OK ";
 for(int i=1;i<=mesh->NbNodes();++i){auto&node=delaun.GetVertex(i);ga_int(i);ga_real(node.Coord().X());ga_real(node.Coord().Y());}
 for(int i=1;i<=mesh->NbLinks();++i){auto&edge=delaun.GetEdge(i);ga_int(i);ga_int(edge.FirstNode());ga_int(edge.LastNode());ga_int(edge.Movability());}ga_end();
}
#undef DC_TRY
void mesh_delaun_null(std::istream&input){
 int op;input>>op;occ::handle<BRepMesh_DataStructureOfDelaun> mesh;BRepMesh_Delaun delaun(mesh,1,1,false);
 volatile int observed=0;GA_TRY(switch(op){case 0:{BRepMesh_Delaun filled(mesh,1,1,true);break;}case 1:delaun.InitCirclesTool(1,1);break;
 case 2:observed=delaun.GetVertex(1).Movability();break;case 3:observed=delaun.GetEdge(1).Movability();break;case 4:observed=delaun.GetTriangle(1).Movability();break;
 case 5:(void)delaun.Frontier();break;case 6:(void)delaun.InternalEdges();break;case 7:(void)delaun.FreeEdges();break;
 default:{int edge=99;observed=delaun.Contains(1,BRepMesh_Vertex(),0.,edge);break;}}std::cout<<"OK ");ga_end();
}
