// Martensite checked topology faults at OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#define MARTENSITE_TRIANGULATION_EMBEDDED
#include "martensite_sensitive_triangulation_matching.cxx"
#undef MARTENSITE_TRIANGULATION_EMBEDDED
class FaultTriangulation : public Select3D_SensitiveTriangulation {
public:
  FaultTriangulation(const occ::handle<Poly_Triangulation>& mesh,
                    const occ::handle<NCollection_HArray1<int>>& edges, int kind)
    : Select3D_SensitiveTriangulation({},mesh,TriLocation(kind>=6?1:0),edges,gp_Pnt(),kind!=0) {}
  void Map(int value) {
    // Inject after valid construction to avoid undefined native topology setup.
    myBVHPrimIndexes->SetValue(0,value);
  }
  void Detection(int value) {
    // A raw detected index proves checked lookup failure independently of traversal.
    myDetectedIdx=value;
  }
  int Detected() const {
    // Observe state without performing the very lookup that may be faulty.
    return myDetectedIdx;
  }
  int TreeSize() {
    // Called only when clean, so this observation cannot rebuild failed state.
    return myContent.GetBVH()->Length();
  }
};
struct FaultOutputs {
  Poly_Triangle triangle{77,78,79};
  gp_Pnt nodes[3]={gp_Pnt(77,78,79),gp_Pnt(80,81,82),gp_Pnt(83,84,85)};
  SelectBasics_PickResult pick;
};
static int FaultOperation(FaultTriangulation& probe,int operation,FaultOutputs& outputs) {
  try {
    auto volume=Volume(0,false);
    if(operation==0)probe.Box(0);
    if(operation==1)probe.Box(-1);
    if(operation==2)probe.LastDetectedTriangle(outputs.triangle);
    if(operation==3)probe.LastDetectedTriangle(outputs.triangle,outputs.nodes);
    if(operation==4)probe.Matches(volume,outputs.pick);
    if(operation==5)probe.BVH();
    return 0;
  } catch(const Standard_OutOfRange&) {return 1;}
    catch(const Standard_Failure&) {return 2;}
}
static void FaultMutation(FaultTriangulation& probe,const occ::handle<Poly_Triangulation>& mesh,
                          const occ::handle<NCollection_HArray1<int>>& edges,int kind,bool repair) {
  const int invalid=repair?1:99;
  if(kind==0)edges->SetValue(1,invalid);
  if(kind==1)probe.Map(repair?0:99);
  if(kind==2)probe.Detection(repair?0:99);
  if(kind<3)return;
  Poly_Triangle triangle(1,2,3);
  if(!repair)triangle.Set((kind-3)%3+1,99);
  mesh->SetTriangle(1,triangle);
}
static void FaultColumns(FaultTriangulation& probe,int failure,const FaultOutputs& outputs) {
  std::cout<<' '<<failure<<' '<<probe.ToBuildBVH()<<' '<<probe.Detected();
  std::cout<<' '<<outputs.pick.IsValid()<<' '<<(outputs.pick.IsValid()?outputs.pick.Depth():0);
  std::cout<<' '<<(probe.ToBuildBVH()?-1:probe.TreeSize());
  for(int slot=1;slot<=3;++slot)std::cout<<' '<<outputs.triangle.Value(slot);
  for(const auto& point:outputs.nodes)PointColumns(point);
}
static void FaultScenario(int kind,int operation) {
  auto mesh=TriMesh(1);auto edges=TriEdges(1);FaultTriangulation probe(mesh,edges,kind);
  probe.Detection(0);FaultMutation(probe,mesh,edges,kind,false);FaultOutputs failed;
  const int failure=FaultOperation(probe,operation,failed);std::cout<<kind<<' '<<operation;
  FaultColumns(probe,failure,failed);FaultMutation(probe,mesh,edges,kind,true);FaultOutputs repaired;
  const int repairedFailure=FaultOperation(probe,operation,repaired);FaultColumns(probe,repairedFailure,repaired);
  std::cout<<' '<<probe.LastDetectedTriangleIndex()<<'\n';
}
int main() {
  // Only checked access paths of valid constructed geometry are mutated; undefined nulls are excluded.
  std::cout<<std::setprecision(17);
  for(int kind=0;kind<9;++kind)for(int operation=0;operation<6;++operation)FaultScenario(kind,operation);
}
