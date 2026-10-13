#pragma once
// Actual viewer-layer geometry retains both structures until all native dump callbacks finish.
#include "martensite_oit_fixture.hxx"
#include <vector>
static std::vector<occ::handle<Graphic3d_Structure>> oit_native_layers(
  const occ::handle<V3d_Viewer>& viewer,const std::string& scene) {
  Graphic3d_ZLayerSettings settings;settings.SetClearDepth(scene=="layer-clear");
  Graphic3d_ZLayerId layer=0;if(!viewer->AddZLayer(layer,settings))throw Standard_Failure("received no layer; expected actual AddZLayer success");
  std::vector<occ::handle<Graphic3d_Structure>> structures;
  for(int rank=0;rank<2;++rank) {
    const occ::handle<Graphic3d_Structure> structure=new Graphic3d_Structure(viewer->StructureManager());
    structure->SetZLayer(rank==0?Graphic3d_ZLayerId_Default:layer);const auto group=structure->NewGroup();
    group->SetGroupPrimitivesAspect(oit_native_aspect(rank));group->AddPrimitiveArray(oit_native_quad(rank));
    structure->Display();structures.push_back(structure);
  }
  return structures;
}
static void oit_native_full_capture(const occ::handle<V3d_View>& view,const std::string& prefix) {
  for(auto kind:{Graphic3d_BT_RGBA,Graphic3d_BT_Depth}) {
    Image_PixMap pixels;
    if(!view->ToPixMap(pixels,64,64,kind,false,Graphic3d_ZLayerId_BotOSD,false))
      throw Standard_Failure("received failed OIT layered capture; expected64square buffers");
    std::ofstream output(prefix+(kind==Graphic3d_BT_RGBA?".rgba":".depth"),std::ios::binary);
    for(int row=0;row<64;++row)output.write(reinterpret_cast<const char*>(pixels.Row(row)),256);
  }
}
