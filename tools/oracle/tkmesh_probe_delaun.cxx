#include <BRepMesh_DataStructureOfDelaun.hxx>
static BRepMesh_Vertex delaun_vertex(int index,int freedom) {
 return BRepMesh_Vertex(index,index%2,BRepMesh_DegreeOfFreedom(freedom));
}
static BRepMesh_Triangle delaun_make_triangle(int a,int b,int c,bool x,bool y,bool z,int freedom) {
 int edges[3]={a,b,c};bool orientations[3]={x,y,z};
 return BRepMesh_Triangle(edges,orientations,BRepMesh_DegreeOfFreedom(freedom));
}
static BRepMesh_Triangle delaun_triangle(int freedom) {
 return delaun_make_triangle(1,2,3,true,false,true,BRepMesh_DegreeOfFreedom(freedom));
}
static void delaun_map(const IMeshData::MapOfInteger& indices) {
 ga_int(indices.Extent());
 for(IMeshData::MapOfInteger::Iterator it(indices);it.More();it.Next())ga_int(it.Key());
}
static void delaun_snapshot(BRepMesh_DataStructureOfDelaun& mesh) {
 ga_int(mesh.NbNodes());ga_int(mesh.NbLinks());ga_int(mesh.NbElements());
 delaun_map(mesh.LinksOfDomain());delaun_map(mesh.ElementsOfDomain());
 for(int i=1;i<=mesh.NbNodes();++i) {
  const auto node=mesh.GetNode(i);report_vertex(node);
  ga_int(mesh.IndexOf(node));const auto& links=mesh.LinksConnectedTo(i);ga_int(links.Extent());
  for(IMeshData::ListOfInteger::Iterator it(links);it.More();it.Next())ga_int(it.Value());
 }
 for(int i=1;i<=mesh.NbLinks();++i) {
  const auto& edge=mesh.GetLink(i);ga_int(edge.FirstNode());ga_int(edge.LastNode());ga_int(edge.Movability());ga_int(mesh.IndexOf(edge));
  const auto& pair=mesh.ElementsConnectedTo(i);ga_int(pair.Extent());ga_int(pair.Index(1));ga_int(pair.Index(2));
 }
 for(int i=1;i<=mesh.NbElements();++i) {
  const auto& element=mesh.GetElement(i);int edges[3];bool orientation[3];element.Edges(edges,orientation);
  for(int edge:edges)ga_int(edge);for(bool flag:orientation)ga_flag(flag);ga_int(element.Movability());
  try {int nodes[3];mesh.ElementNodes(element,nodes);std::cout<<"OK ";for(int node:nodes)ga_int(node);}
  catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 }
 const auto& queue=mesh.Data()->GetListOfDelNodes();ga_int(queue.Extent());
 for(IMeshData::ListOfInteger::Iterator it(queue);it.More();it.Next())ga_int(it.Value());
}
template<class F> static void delaun_index(F operation) {
 try {const int index=operation();std::cout<<"OK ";ga_int(index);}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
}
static void delaun_alter(BRepMesh_DataStructureOfDelaun& mesh,int op) {
 switch(op) {
 case 0:delaun_index([&](){return mesh.AddLink(BRepMesh_Edge(2,1,BRepMesh_Free));});break;
 case 1:ga_flag(mesh.SubstituteNode(1,delaun_vertex(2,0)));break;
 case 2:ga_flag(mesh.SubstituteNode(1,delaun_vertex(9,0)));break;
 case 3:mesh.RemoveNode(1,true);break;
 case 4:mesh.RemoveLink(6,false);break;
 case 5:mesh.RemoveLink(6,true);break;
 case 6:ga_flag(mesh.SubstituteLink(6,BRepMesh_Edge(2,5,BRepMesh_Free)));break;
 case 7:ga_flag(mesh.SubstituteLink(1,BRepMesh_Edge(4,2,BRepMesh_Free)));break;
 case 8:mesh.RemoveElement(1);break;
 case 9:ga_flag(mesh.SubstituteElement(1,delaun_make_triangle(4,5,6,false,false,false,BRepMesh_Fixed)));break;
 case 10:mesh.ClearDomain();break;
 case 11:mesh.RemoveLink(6,true);mesh.ClearDeleted();break;
 case 12:mesh.RemoveLink(6,true);mesh.RemoveNode(5,true);mesh.ClearDeleted();break;
 case 13:mesh.RemoveElement(1);mesh.RemoveLink(1,true);mesh.ClearDeleted();break;
 case 14:mesh.RemoveLink(6,true);delaun_index([&](){return mesh.AddLink(BRepMesh_Edge(2,5,BRepMesh_Free));});break;
 case 15:delaun_index([&](){return mesh.AddElement(delaun_triangle(0));});break;
 case 16:mesh.RemoveElement(1);ga_flag(mesh.SubstituteElement(1,delaun_triangle(5)));break;
 case 17:delaun_index([&](){return mesh.AddLink(BRepMesh_Edge(1,99,BRepMesh_Free));});break;
 case 18:ga_flag(mesh.SubstituteLink(6,BRepMesh_Edge(2,1,BRepMesh_Free)));break;
 case 20:mesh.RemoveElement(1);mesh.RemoveLink(1,true);mesh.AddElement(delaun_make_triangle(2,4,6,true,true,true,0));mesh.AddElement(delaun_make_triangle(2,5,6,true,true,true,0));mesh.RemoveElement(4);mesh.ClearDeleted();break;
 default:mesh.RemoveLink(6,true);ga_flag(mesh.SubstituteLink(6,BRepMesh_Edge(1,4,BRepMesh_Free)));break;
 }
}
void mesh_delaun(std::istream& input) {
 int reserved,nodes,links,elements,op;input>>reserved>>nodes>>links>>elements>>op;
 occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun> mesh=new BRepMesh_DataStructureOfDelaun(allocator,reserved);
 ga_flag(mesh->Allocator()==allocator);ga_flag(mesh->IsInstance(STANDARD_TYPE(BRepMesh_DataStructureOfDelaun)));ga_flag(mesh->IsKind(STANDARD_TYPE(Standard_Transient)));
 mesh->Data()->SetCellSize(0.5);
 for(int i=1;i<=5;++i){std::cout<<"OK ";ga_int(mesh->AddNode(delaun_vertex(i,nodes),true));}
 for(auto endpoints:{std::pair<int,int>(1,2),{2,3},{3,1},{3,4},{4,1},{5,5}}){std::cout<<"OK ";ga_int(mesh->AddLink(BRepMesh_Edge(endpoints.first,endpoints.second,BRepMesh_DegreeOfFreedom(links))));}
 std::cout<<"OK ";ga_int(mesh->AddElement(delaun_triangle(elements)));
 std::cout<<"OK ";ga_int(mesh->AddElement(delaun_make_triangle(3,4,5,false,true,false,BRepMesh_DegreeOfFreedom(elements))));
 delaun_snapshot(*mesh);
 try {delaun_alter(*mesh,op);std::cout<<"OK ";}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 delaun_snapshot(*mesh);ga_end();
}
void mesh_delaun_invalid(std::istream& input) {
 int op,index;input>>op>>index;occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun> mesh=new BRepMesh_DataStructureOfDelaun(allocator,1);
 mesh->AddNode(delaun_vertex(1,0),true);
 if(op==3) {
  for(int i=2;i<=3;++i)mesh->AddNode(delaun_vertex(i,0),true);
  for(auto p:{std::pair<int,int>(1,1),{1,2},{2,3},{3,1}})mesh->AddLink(BRepMesh_Edge(p.first,p.second,BRepMesh_Free));
 } else for(int i=0;i<4;++i)mesh->AddLink(BRepMesh_Edge(1,1,BRepMesh_Deleted));
 mesh->AddElement(delaun_make_triangle(2,3,4,true,true,true,BRepMesh_Free));
 try {
  volatile int observed=0;
  if(op==0)observed=mesh->GetNode(index).Movability();
  else if(op==1)observed=mesh->GetElement(index).Movability();
  else if(op==2)mesh->SubstituteElement(1,delaun_triangle(0));
  else {mesh->RemoveLink(1,true);mesh->ClearDeleted();}
  (void)observed;std::cout<<"OK ";
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();
}
#include <unistd.h>
#include <fstream>
void mesh_delaun_dump(std::istream& input) {
 int mode;input>>mode;occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun> mesh=new BRepMesh_DataStructureOfDelaun(allocator,1);
 for(int i=1;i<=3;++i)mesh->AddNode(delaun_vertex(i,0),true);
 if(mode>0)mesh->AddLink(BRepMesh_Edge(1,mode==1?2:1,BRepMesh_Free));
 const std::string file="/tmp/rustycad-delaun-dump-"+std::to_string(getpid())+".brep";
 mesh->Dump(file.c_str());std::ifstream stream(file,std::ios::binary);char byte;
 while(stream.get(byte))ga_int(static_cast<unsigned char>(byte));
 stream.close();std::remove(file.c_str());ga_end();
}
