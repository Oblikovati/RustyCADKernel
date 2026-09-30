#include <IntTools.hxx>
void bo_algorithms_roots(const std::vector<double>& v){
 NCollection_Sequence<IntTools_Root> roots;for(int i=0;i<int(v[0]);++i){int seed=int(v[1]);IntTools_Root root(((i*7+seed)%11-5)*v[2],(i+seed)%5-1);root.SetLayerHeight(i);root.SetStateBefore(TopAbs_OUT);root.SetStateAfter(TopAbs_UNKNOWN);root.SetInterval(-1,1,(i%3-1)*v[2],(i%5-2)*v[2]);roots.Append(root);}
 IntTools::RemoveIdenticalRoots(roots,v[3]);bo_int(roots.Length());for(const auto& root:roots)bo_root_state(root);
 IntTools::SortRoots(roots,v[3]);bo_int(roots.Length());for(const auto& root:roots)bo_root_state(root);
 IntTools::FindRootStates(roots,v[4]);bo_int(roots.Length());for(const auto& root:roots)bo_root_state(root);
}
void bo_algorithms_geometry_case(const std::vector<double>& v){
 auto source=ee_geometry_fixture(int(v[0]),0.5);TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(source,0,1);BRep_Builder builder;builder.Degenerated(edge,v[6]!=0);BRepAdaptor_Curve curve(edge);
 try{bo_real(IntTools::Length(edge));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 double radius=17;try{bo_int(IntTools::GetRadius(curve,v[1],v[2],radius));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_real(radius);
 double parameter=19;try{bo_int(IntTools::Parameter(gp_Pnt(v[7],v[8],v[9]),source,parameter));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}bo_real(parameter);
 NCollection_Array1<double> arguments(1,3);arguments.Init(23);try{bo_int(IntTools::PrepareArgs(curve,v[2],v[1],int(v[3]),v[4],arguments));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 bo_int(arguments.Lower());bo_int(arguments.Upper());for(double parameter:arguments)bo_real(parameter);
}

void bo_algorithms_geometry(const std::vector<double>& v){try{bo_algorithms_geometry_case(v);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}}
