// Pinned public Delaun implementation plus private polygon members accessed by explicit instantiation.
// This does not alter OCCT headers or translation units: member-pointer access bypass only.
#include <algorithm>
#include <array>
#include <tuple>
#include <vector>
#include <BRepMesh_Delaun.hxx>
template<class Tag,typename Tag::type Member>struct DelaunAccess{friend typename Tag::type delaun_member(Tag){return Member;}};
struct DelaunWindingTag{using type=bool(BRepMesh_Delaun::*)(const int&,const IMeshData::VectorOfInteger&)const;friend type delaun_member(DelaunWindingTag);};
struct DelaunAreaTag{using type=double(BRepMesh_Delaun::*)(const IMeshData::SequenceOfInteger&,int,int)const;friend type delaun_member(DelaunAreaTag);};
struct DelaunIntersectionTag{using type=BRepMesh_GeomTool::IntFlag(BRepMesh_Delaun::*)(const BRepMesh_Edge&,const BRepMesh_Edge&,bool,bool,gp_Pnt2d&)const;friend type delaun_member(DelaunIntersectionTag);};
struct DelaunMeshPolygonTag{using type=void(BRepMesh_Delaun::*)(IMeshData::SequenceOfInteger&,IMeshData::SequenceOfBndB2d&,Handle(IMeshData::MapOfInteger));friend type delaun_member(DelaunMeshPolygonTag);};
struct DelaunSuperMeshTag{using type=void(BRepMesh_Delaun::*)(const Bnd_Box2d&);friend type delaun_member(DelaunSuperMeshTag);};
struct DelaunFrontierReachabilityTag{using type=bool(BRepMesh_Delaun::*)(const int,const int);friend type delaun_member(DelaunFrontierReachabilityTag);};
struct DelaunCleanupTag{using type=void(BRepMesh_Delaun::*)();friend type delaun_member(DelaunCleanupTag);};
struct DelaunCheckIntersectionTag{using type=bool(BRepMesh_Delaun::*)(const BRepMesh_Edge&,const IMeshData::SequenceOfInteger&,const IMeshData::SequenceOfBndB2d&,bool,bool,bool,Bnd_B2d&)const;friend type delaun_member(DelaunCheckIntersectionTag);};
struct DelaunNextPolygonLinkTag{using type=int(BRepMesh_Delaun::*)(const int&,const int&,const BRepMesh_Vertex&,const gp_Vec2d&,const IMeshData::SequenceOfBndB2d&,const IMeshData::SequenceOfInteger&,const Handle(IMeshData::MapOfInteger)&,const bool&,IMeshData::MapOfInteger&,IMeshData::MapOfInteger&,int&,gp_Vec2d&,Bnd_B2d&);friend type delaun_member(DelaunNextPolygonLinkTag);};
struct DelaunMeshLeftPolygonTag{using type=bool(BRepMesh_Delaun::*)(const int,const bool,Handle(IMeshData::MapOfInteger));friend type delaun_member(DelaunMeshLeftPolygonTag);};
struct DelaunCleanupPolygonTag{using type=void(BRepMesh_Delaun::*)(const IMeshData::SequenceOfInteger&,const IMeshData::SequenceOfBndB2d&);friend type delaun_member(DelaunCleanupPolygonTag);};
struct DelaunInsertInternalEdgesTag{using type=void(BRepMesh_Delaun::*)();friend type delaun_member(DelaunInsertInternalEdgesTag);};
struct DelaunCreateTrianglesTag{using type=void(BRepMesh_Delaun::*)(const int,IMeshData::MapOfIntegerInteger&);friend type delaun_member(DelaunCreateTrianglesTag);};
template struct DelaunAccess<DelaunWindingTag,&BRepMesh_Delaun::isVertexInsidePolygon>;
template struct DelaunAccess<DelaunAreaTag,&BRepMesh_Delaun::polyArea>;
template struct DelaunAccess<DelaunIntersectionTag,&BRepMesh_Delaun::intSegSeg>;
template struct DelaunAccess<DelaunMeshPolygonTag,&BRepMesh_Delaun::meshPolygon>;
template struct DelaunAccess<DelaunSuperMeshTag,&BRepMesh_Delaun::superMesh>;
template struct DelaunAccess<DelaunFrontierReachabilityTag,&BRepMesh_Delaun::isBoundToFrontier>;
template struct DelaunAccess<DelaunCleanupTag,&BRepMesh_Delaun::cleanupMesh>;
template struct DelaunAccess<DelaunCheckIntersectionTag,&BRepMesh_Delaun::checkIntersection>;
template struct DelaunAccess<DelaunNextPolygonLinkTag,&BRepMesh_Delaun::findNextPolygonLink>;
template struct DelaunAccess<DelaunMeshLeftPolygonTag,&BRepMesh_Delaun::meshLeftPolygonOf>;
template struct DelaunAccess<DelaunCleanupPolygonTag,&BRepMesh_Delaun::cleanupPolygon>;
template struct DelaunAccess<DelaunInsertInternalEdgesTag,&BRepMesh_Delaun::insertInternalEdges>;
template struct DelaunAccess<DelaunCreateTrianglesTag,&BRepMesh_Delaun::createTriangles>;
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
static auto delaun_center_polygon_mesh(){
 occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;auto mesh=new BRepMesh_DataStructureOfDelaun(arena,12);
 for(auto point:{gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2),gp_XY(1,1)})mesh->AddNode(BRepMesh_Vertex(point,0,BRepMesh_Free),true);
 for(auto edge:{std::tuple<int,int,BRepMesh_DegreeOfFreedom>{1,2,BRepMesh_Frontier},{2,3,BRepMesh_Frontier},{3,4,BRepMesh_Frontier},{4,1,BRepMesh_Frontier},{1,5,BRepMesh_Free},{2,5,BRepMesh_Free},{3,5,BRepMesh_Free},{4,5,BRepMesh_Free}})mesh->AddLink(BRepMesh_Edge(std::get<0>(edge),std::get<1>(edge),std::get<2>(edge)));
 int triangleEdges[][3]={{1,6,5},{2,7,6},{3,8,7},{4,5,8}};bool orientations[]={true,true,false};
 for(auto&edges:triangleEdges)mesh->AddElement(BRepMesh_Triangle(edges,orientations,BRepMesh_Free));return mesh;
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
 for(int node=1;node<=4;++node)for(int link=1;link<=5;++link)GA_TRY(ga_flag((delaun.*delaun_member(DelaunFrontierReachabilityTag{}))(node,link)));
 for(int triangle=1;triangle<=2;++triangle)delaun_core_contains(delaun,triangle,v[0]);
 IMeshData::VectorOfInteger auxiliary(3);for(int i=1;i<=4;++i)if(v[5]&(1<<(i-1)))auxiliary.Append(i);
 delaun.SetAuxVertices(auxiliary);auxiliary.Clear();DC_TRY(delaun.RemoveAuxElements());delaun_core_snapshot(delaun);
 DC_TRY(delaun.InitCirclesTool(v[2],v[3]));ga_flag(delaun.Circles().IsEmpty()));ga_end();
}
void mesh_delaun_use_edge(std::istream& input){
 int index;input>>index;occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun>mesh=new BRepMesh_DataStructureOfDelaun(arena,0);
 BRepMesh_Delaun delaun(mesh,-1,-1,false);ga_flag(delaun.UseEdge(index));ga_end();
}
void mesh_delaun_sort(std::istream& input){
 int count;input>>count;occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun>mesh=new BRepMesh_DataStructureOfDelaun(arena,count);
 std::vector<int> indices;for(int i=0;i<count;++i){double x,y;input>>x>>y;indices.push_back(mesh->AddNode(BRepMesh_Vertex(gp_XY(x,y),0,BRepMesh_Free),true));}
 auto compare=[&](int left,int right){const gp_XY&a=mesh->GetNode(left).Coord();const gp_XY&b=mesh->GetNode(right).Coord();return a.X()+a.Y()<b.X()+b.Y();};
 std::make_heap(indices.begin(),indices.end(),compare);std::sort_heap(indices.begin(),indices.end(),compare);for(int index:indices)ga_int(index);ga_end();
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
void mesh_delaun_cleanup(std::istream&input){
 int kind,flags;input>>kind>>flags;auto mesh=delaun_core_mesh(kind,flags);BRepMesh_Delaun delaun(mesh,-1,-1,false);
 (delaun.*delaun_member(DelaunCleanupTag{}))();std::cout<<"OK ";delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_intersections(std::istream&input){
 int kind,flags;input>>kind>>flags;auto mesh=delaun_core_mesh(kind,flags);BRepMesh_Delaun delaun(mesh,-1,-1,false);
 IMeshData::SequenceOfInteger polygon;IMeshData::SequenceOfBndB2d boxes;
 for(int i=1;i<=5;++i){polygon.Append(i);auto&e=delaun.GetEdge(i);Bnd_B2d box;box.Add(delaun.GetVertex(e.FirstNode()).Coord());box.Add(delaun.GetVertex(e.LastNode()).Coord());box.Enlarge(Precision::PConfusion());boxes.Append(box);}
 std::cout<<"OK ";for(int link=1;link<=5;++link)for(int endpoint=0;endpoint<=1;++endpoint)for(int onEdge=0;onEdge<=1;++onEdge)for(int skip=0;skip<=1;++skip){Bnd_B2d bounds;GA_TRY(ga_flag((delaun.*delaun_member(DelaunCheckIntersectionTag{}))(delaun.GetEdge(link),polygon,boxes,endpoint!=0,onEdge!=0,skip!=0,bounds)));gp_XY minimum=bounds.CornerMin(),maximum=bounds.CornerMax();ga_real(minimum.X());ga_real(minimum.Y());ga_real(maximum.X());ga_real(maximum.Y());}ga_end();
}
static void delaun_seed_map(IMeshData::MapOfInteger&map,int mask){for(int i=1;i<=5;++i)if(mask&(1<<(i-1)))map.Add(i);}
static void delaun_report_map(const IMeshData::MapOfInteger&map){std::vector<int>indices;for(IMeshData::MapOfInteger::Iterator it(map);it.More();it.Next())indices.push_back(it.Key());std::sort(indices.begin(),indices.end());ga_int((int)indices.size());for(int index:indices)ga_int(index);}
void mesh_delaun_next_link(std::istream&input){
 int kind,flags;input>>kind>>flags;auto mesh=delaun_core_mesh(kind,flags);BRepMesh_Delaun delaun(mesh,-1,-1,false);IMeshData::SequenceOfInteger polygon;IMeshData::SequenceOfBndB2d boxes;
 for(int i=1;i<=5;++i){polygon.Append(i);auto&e=delaun.GetEdge(i);Bnd_B2d box;box.Add(delaun.GetVertex(e.FirstNode()).Coord());box.Add(delaun.GetVertex(e.LastNode()).Coord());box.Enlarge(Precision::PConfusion());boxes.Append(box);}
 std::cout<<"OK ";for(int pivot=1;pivot<=4;++pivot)for(int first=1;first<=4;++first)for(int mode=0;mode<3;++mode)for(bool skipLeprous:{false,true})for(auto seeds:{std::array<int,3>{0,0,0},{1,0,0},{0,1,0},{0,0,1},{3,5,2},{31,31,31}}){IMeshData::MapOfInteger skipped,leprous,dead;delaun_seed_map(skipped,seeds[0]);delaun_seed_map(leprous,seeds[1]);delaun_seed_map(dead,seeds[2]);int nextPivot=-77;gp_Vec2d nextDir(81.,82.);Bnd_B2d bounds;gp_Vec2d reference(mode==0?gp_XY(1.,0.):(mode==1?gp_XY(0.,1.):gp_XY(-1.,0.)));Handle(IMeshData::MapOfInteger) skippedHandle=new IMeshData::MapOfInteger(skipped);int next=(delaun.*delaun_member(DelaunNextPolygonLinkTag{}))(first,pivot,delaun.GetVertex(pivot),reference,boxes,polygon,skippedHandle,skipLeprous,leprous,dead,nextPivot,nextDir,bounds);ga_int(next);ga_int(nextPivot);ga_real(nextDir.X());ga_real(nextDir.Y());gp_XY minimum=bounds.CornerMin(),maximum=bounds.CornerMax();ga_real(minimum.X());ga_real(minimum.Y());ga_real(maximum.X());ga_real(maximum.Y());delaun_report_map(skipped);delaun_report_map(leprous);delaun_report_map(dead);}
 ga_end();
}
void mesh_delaun_left_polygon(std::istream&input){
 int kind,start,forward;input>>kind>>start>>forward;
 auto mesh=kind==3?delaun_core_mesh(0,0):(kind==4?delaun_center_polygon_mesh():occ::handle<BRepMesh_DataStructureOfDelaun>());
 if(kind!=3&&kind!=4){occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;mesh=new BRepMesh_DataStructureOfDelaun(arena,8);gp_XY points[]={gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2)};if(kind==2)points[2]=gp_XY(1,1);for(auto&p:points)mesh->AddNode(BRepMesh_Vertex(p,0,BRepMesh_Free),true);auto freedom=kind==1?BRepMesh_Frontier:BRepMesh_Free;for(auto pair:{std::pair<int,int>{1,2},{2,3},{3,4},{4,1}})mesh->AddLink(BRepMesh_Edge(pair.first,pair.second,freedom));}
 BRepMesh_Delaun delaun(mesh,-1,-1,false);std::cout<<"OK ";GA_TRY(ga_flag((delaun.*delaun_member(DelaunMeshLeftPolygonTag{}))(start,forward!=0,Handle(IMeshData::MapOfInteger)())));delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_cleanup_victim(std::istream&){
 auto mesh=delaun_center_polygon_mesh();BRepMesh_Delaun delaun(mesh,-1,-1,false);IMeshData::SequenceOfInteger polygon;IMeshData::SequenceOfBndB2d boxes;
 for(int index=1;index<=4;++index){polygon.Append(index);auto&edge=delaun.GetEdge(index);Bnd_B2d bounds;bounds.Add(delaun.GetVertex(edge.FirstNode()).Coord());bounds.Add(delaun.GetVertex(edge.LastNode()).Coord());bounds.Enlarge(Precision::PConfusion());boxes.Append(bounds);}
 std::cout<<"OK ";(delaun.*delaun_member(DelaunCleanupPolygonTag{}))(polygon,boxes);delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_internal_constraint(std::istream&){
 occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;auto mesh=new BRepMesh_DataStructureOfDelaun(arena,6);gp_XY points[]={gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2)};
 for(auto&point:points)mesh->AddNode(BRepMesh_Vertex(point,0,BRepMesh_Free),true);mesh->AddLink(BRepMesh_Edge(1,2,BRepMesh_Free));mesh->AddLink(BRepMesh_Edge(2,3,BRepMesh_Free));mesh->AddLink(BRepMesh_Edge(3,4,BRepMesh_Free));mesh->AddLink(BRepMesh_Edge(4,1,BRepMesh_Free));mesh->AddLink(BRepMesh_Edge(1,3,BRepMesh_Fixed));mesh->AddElement(BRepMesh_Triangle({1,2,5},{true,true,false},BRepMesh_Free));
 BRepMesh_Delaun delaun(mesh,-1,-1,false);(delaun.*delaun_member(DelaunInsertInternalEdgesTag{}))();std::cout<<"OK ";delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_process_constraints(std::istream&){
 occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;auto mesh=new BRepMesh_DataStructureOfDelaun(arena,8);gp_XY points[]={gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2)};
 for(auto&point:points)mesh->AddNode(BRepMesh_Vertex(point,0,BRepMesh_Free),true);for(auto pair:{std::pair<int,int>{1,2},{2,3},{3,4},{4,1}})mesh->AddLink(BRepMesh_Edge(pair.first,pair.second,BRepMesh_Frontier));
 BRepMesh_Delaun delaun(mesh,-1,-1,false);delaun.ProcessConstraints();std::cout<<"OK ";delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_create_triangles(std::istream&){
 occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;auto mesh=new BRepMesh_DataStructureOfDelaun(arena,8);gp_XY points[]={gp_XY(0,0),gp_XY(2,0),gp_XY(2,2),gp_XY(0,2)};
 for(auto&point:points)mesh->AddNode(BRepMesh_Vertex(point,0,BRepMesh_Free),true);for(auto pair:{std::pair<int,int>{1,2},{2,3},{3,4},{4,1}})mesh->AddLink(BRepMesh_Edge(pair.first,pair.second,BRepMesh_Free));int center=mesh->AddNode(BRepMesh_Vertex(gp_XY(1,1),0,BRepMesh_Free),true);
 BRepMesh_Delaun delaun(mesh,-1,-1,false);IMeshData::MapOfIntegerInteger polygon;for(int edge=1;edge<=4;++edge)polygon.Bind(edge,true);(delaun.*delaun_member(DelaunCreateTrianglesTag{}))(center,polygon);std::cout<<"OK ";delaun_polygon_mesh_snapshot(delaun);ga_end();
}
void mesh_delaun_insert_vertex(std::istream&input){
 int mode;input>>mode;auto mesh=delaun_core_mesh(0,mode==3?1:0);std::vector<gp_XY>points;
 if(mode==2)points={gp_XY(1,.5),gp_XY(.5,1.5)};else if(mode==3)points={gp_XY(1,0)};else if(mode==4)points={gp_XY(1,1)};else if(mode==5)points={gp_XY(3,3)};else points={gp_XY(1,.5)};
 IMeshData::VectorOfInteger vertices((int)points.size());for(auto&point:points)vertices.Append(mesh->AddNode(BRepMesh_Vertex(point,0,BRepMesh_Free),true));
 BRepMesh_Delaun delaun(mesh,-1,-1,false);delaun.InitCirclesTool(8,8);delaun.AddVertices(vertices,Message_ProgressRange());std::cout<<"OK ";delaun_polygon_mesh_snapshot(delaun);ga_end();
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
void mesh_delaun_super_fan(std::istream&input){
 double x,y;input>>x>>y;occ::handle<NCollection_IncAllocator>arena=new NCollection_IncAllocator;occ::handle<BRepMesh_DataStructureOfDelaun>mesh=new BRepMesh_DataStructureOfDelaun(arena,4);
 mesh->AddNode(BRepMesh_Vertex(x,y,BRepMesh_Free),true);BRepMesh_Delaun delaun(mesh,-1,-1,false);Bnd_Box2d bounds;bounds.Add(gp_Pnt2d(0,0));bounds.Add(gp_Pnt2d(2,2));
 (delaun.*delaun_member(DelaunSuperMeshTag{}))(bounds);IMeshData::MapOfIntegerInteger edges;for(int i=1;i<=3;++i)edges.Bind(i,1);
 std::cout<<"OK ";(delaun.*delaun_member(DelaunCreateTrianglesTag{}))(1,edges);delaun_polygon_mesh_snapshot(delaun);ga_end();
}
#undef DC_TRY
void mesh_delaun_null(std::istream&input){
 int op;input>>op;occ::handle<BRepMesh_DataStructureOfDelaun> mesh;BRepMesh_Delaun delaun(mesh,1,1,false);
 volatile int observed=0;GA_TRY(switch(op){case 0:{BRepMesh_Delaun filled(mesh,1,1,true);break;}case 1:delaun.InitCirclesTool(1,1);break;
 case 2:observed=delaun.GetVertex(1).Movability();break;case 3:observed=delaun.GetEdge(1).Movability();break;case 4:observed=delaun.GetTriangle(1).Movability();break;
 case 5:(void)delaun.Frontier();break;case 6:(void)delaun.InternalEdges();break;case 7:(void)delaun.FreeEdges();break;
 default:{int edge=99;observed=delaun.Contains(1,BRepMesh_Vertex(),0.,edge);break;}}std::cout<<"OK ");ga_end();
}
