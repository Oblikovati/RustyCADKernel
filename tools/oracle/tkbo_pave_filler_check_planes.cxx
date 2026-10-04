#include <BOPAlgo_PaveFiller.hxx>
#include <BOPDS_DS.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <TopoDS_Face.hxx>
#include <gp_Pln.hxx>
#include <iostream>

class Probe final : public BOPAlgo_PaveFiller {
public:
  void Initialize() { Init(Message_ProgressRange()); }
  bool Check(int first, int second) const { return CheckPlanes(first, second); }
  BOPDS_DS& DS() { return *myDS; }
};

void Assign(BOPDS_FaceInfo& info, int inside, int on) {
  info.ChangeVerticesIn().Clear();
  info.ChangeVerticesOn().Clear();
  for (int index = 0; index < 3; ++index) {
    if (inside & (1 << index)) info.ChangeVerticesIn().Add(index + 2);
    if (on & (1 << index)) info.ChangeVerticesOn().Add(index + 2);
  }
}

void Cases(Probe& probe, int first, int second) {
  const int masks[][4] = {{0,0,0,0}, {1,0,1,0}, {3,0,3,0}, {1,0,1,1},
                         {1,1,1,0}, {1,2,0,3}, {1,0,2,4}, {0,3,1,2}};
  int index = 0;
  for (const auto& mask : masks) {
    Assign(probe.DS().ChangeFaceInfo(first), mask[0], mask[1]);
    Assign(probe.DS().ChangeFaceInfo(second), mask[2], mask[3]);
    std::cout << "case " << index++ << ' ' << probe.Check(first, second) << '\n';
  }
}

int main() {
  const auto first = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,0), gp_Dir(0,0,1)), -1,1,-1,1).Face();
  const auto second = BRepBuilderAPI_MakeFace(gp_Pln(gp_Pnt(0,0,2), gp_Dir(0,0,1)), -1,1,-1,1).Face();
  Probe probe;
  probe.AddArgument(first); probe.AddArgument(second); probe.Initialize();
  const int firstIndex = probe.DS().Index(first), secondIndex = probe.DS().Index(second);
  std::cout << "lazy " << probe.DS().HasFaceInfo(firstIndex) << ' ' << probe.DS().HasFaceInfo(secondIndex) << '\n';
  std::cout << "initial " << probe.Check(firstIndex, secondIndex) << '\n';
  std::cout << "ready " << probe.DS().HasFaceInfo(firstIndex) << ' ' << probe.DS().HasFaceInfo(secondIndex) << '\n';
  std::cout << "same " << probe.Check(firstIndex, firstIndex) << '\n';
  Cases(probe, firstIndex, secondIndex);
}
