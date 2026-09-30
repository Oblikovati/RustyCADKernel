#include <BRepMesh_SelectorOfDataStructureOfDelaun.hxx>
static occ::handle<BRepMesh_DataStructureOfDelaun> selector_mesh() {
 occ::handle<NCollection_IncAllocator> arena=new NCollection_IncAllocator;
 occ::handle<BRepMesh_DataStructureOfDelaun> mesh=new BRepMesh_DataStructureOfDelaun(arena,1);
 mesh->Data()->SetCellSize(0.5);
 for(int i=0;i<7;++i)mesh->AddNode(BRepMesh_Vertex(i,0,BRepMesh_Free),true);
 for(auto p:{std::pair<int,int>(1,2),{2,3},{3,1},{3,4},{4,1},{3,6},{6,7},{7,3}})mesh->AddLink(BRepMesh_Edge(p.first,p.second,BRepMesh_Free));
 mesh->AddElement(delaun_make_triangle(1,2,3,true,true,true,0));
 mesh->AddElement(delaun_make_triangle(3,4,5,true,true,true,0));
 mesh->AddElement(delaun_make_triangle(6,7,8,true,true,true,0));
 return mesh;
}
static void selector_select(BRepMesh_SelectorOfDataStructureOfDelaun& selector,int op,int index) {
 switch(op) {
 case 0:selector.NeighboursOf(BRepMesh_Vertex(index-1,0,BRepMesh_Free));break;
 case 1:selector.NeighboursOfNode(index);break;
 case 2:selector.NeighboursOf(BRepMesh_Edge(2,index,BRepMesh_Free));break;
 case 3:selector.NeighboursOfLink(index);break;
 case 4:selector.NeighboursOf(delaun_make_triangle(1,2,index,true,true,true,0));break;
 case 5:selector.NeighboursOfElement(index);break;
 case 6:selector.NeighboursByEdgeOf(delaun_make_triangle(1,2,index,true,true,true,0));break;
 default:selector.AddNeighbours();selector.NeighboursOf(selector);break;
 }
}
static void selector_snapshot(const BRepMesh_SelectorOfDataStructureOfDelaun& selector) {
 delaun_map(selector.Nodes());delaun_map(selector.Links());delaun_map(selector.Elements());delaun_map(selector.FrontierLinks());
}
void mesh_selector(std::istream& input) {
 int op,index,mode;input>>op>>index>>mode;const auto mesh=selector_mesh();
 occ::handle<BRepMesh_SelectorOfDataStructureOfDelaun> selector=new BRepMesh_SelectorOfDataStructureOfDelaun(mesh);
 ga_flag(selector->IsInstance(STANDARD_TYPE(BRepMesh_SelectorOfDataStructureOfDelaun)));ga_flag(selector->IsKind(STANDARD_TYPE(Standard_Transient)));
 if(mode&1)selector->NeighboursOfNode(1);
 if(mode&2)mesh->RemoveElement(1);
 if(mode&4)selector->Initialize(mesh);
 try{selector_select(*selector,op,index);std::cout<<"OK ";}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 selector_snapshot(*selector);
 BRepMesh_SelectorOfDataStructureOfDelaun empty;selector->NeighboursOf(empty);selector->AddNeighbours();selector_snapshot(*selector);
 selector->Initialize(nullptr);selector_snapshot(*selector);ga_end();
}
void mesh_selector_invalid(std::istream& input) {
 int op;input>>op;BRepMesh_SelectorOfDataStructureOfDelaun selector;
 try{selector_select(selector,op,1);std::cout<<"OK ";}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();
}
