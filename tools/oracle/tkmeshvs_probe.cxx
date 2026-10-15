// Exact numeric oracle for TKMeshVS at pinned OCCT 3d097a0. Each input line is
// `operation numbers...`; tools/verify_tkmeshvs_numeric.py compares the output with
// crates/tkmeshvs/examples/numeric_probe.rs token for token (doubles as bits). Every request
// prints exactly one line and never depends on an earlier request.
#include <map>
#include <memory>
#include <string>
#include <vector>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
#include <Standard_Failure.hxx>
#include <gp_Dir.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt.hxx>
#include <gp_Pnt2d.hxx>
#include <gp_XY.hxx>
#include <gp_XYZ.hxx>
#include <gp_Ax1.hxx>
#include <MeshVS_Buffer.hxx>
#include <MeshVS_CommonSensitiveEntity.hxx>
#include <MeshVS_DataSource3D.hxx>
#include <MeshVS_DeformedDataSource.hxx>
#include <MeshVS_Drawer.hxx>
#include <MeshVS_DrawerAttribute.hxx>
#include <MeshVS_DummySensitiveEntity.hxx>
#include <MeshVS_ElementalColorPrsBuilder.hxx>
#include <MeshVS_Mesh.hxx>
#include <MeshVS_MeshEntityOwner.hxx>
#include <MeshVS_MeshOwner.hxx>
#include <MeshVS_MeshPrsBuilder.hxx>
#include <MeshVS_NodalColorPrsBuilder.hxx>
#include <MeshVS_SensitiveFace.hxx>
#include <MeshVS_SensitiveMesh.hxx>
#include <MeshVS_SensitivePolyhedron.hxx>
#include <MeshVS_SensitiveQuad.hxx>
#include <MeshVS_SensitiveSegment.hxx>
#include <MeshVS_TextPrsBuilder.hxx>
#include <MeshVS_VectorPrsBuilder.hxx>
#include <MeshVS_TwoColors.hxx>
#include <MeshVS_SelectionModeFlags.hxx>
#include <PrsMgr_PresentationManager.hxx>
#include <Prs3d_Drawer.hxx>
#include <Prs3d_ShadingAspect.hxx>
#include <Select3D_SensitivePoint.hxx>
#include <SelectMgr_Selection.hxx>
#include <SelectMgr_EntityOwner.hxx>
#include <SelectMgr_SelectingVolumeManager.hxx>
#include <TColStd_HPackedMapOfInteger.hxx>
#include <V3d_Viewer.hxx>
#include <Graphic3d_Texture2D.hxx>
#include "tkgeomalgo_probe_common.cxx"
#include "tkservice_probe_null_driver.cxx"
#include "tkv3d_probe_sensitive_display_driver.cxx"
#include "tkmeshvs_probe_source.cxx"
#include "tkmeshvs_probe_print.cxx"
#include "tkmeshvs_probe_build.cxx"
#include "tkmeshvs_probe_select.cxx"
const std::map<std::string,void(*)(std::istream&)> probe_cases={
 // Data sources (tkmeshvs_probe_source.cxx)
 {"mv_ds",mv_ds},
 {"mv_topology",mv_topology},
 {"mv_deformed",mv_deformed},
 // Presentation builders and the mesh (tkmeshvs_probe_build.cxx)
 {"mv_build",mv_build},
 {"mv_compute",mv_compute},
 {"mv_mesh_api",mv_mesh_api},
 // Selection, owners and sensitive entities (tkmeshvs_probe_select.cxx)
 {"mv_select",mv_select},
 {"mv_highlight",mv_highlight},
 {"mv_sensitive",mv_sensitive},
};
int main() {
 std::string line;
 while(std::getline(std::cin,line)) {
  std::istringstream input(line);std::string op;input>>op;
  if(op.empty())continue;
  auto found=probe_cases.find(op);
  if(found==probe_cases.end()) {std::cerr<<"Unknown probe operation: "<<op<<std::endl;return 2;}
  found->second(input);
 }
}
