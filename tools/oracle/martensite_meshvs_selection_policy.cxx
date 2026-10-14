// SPDX-License-Identifier: GPL-2.0-only
// Values are observed from pinned OCCT declarations, independently of Martensite.
#include <MeshVS_BuilderPriority.hxx>
#include <MeshVS_MeshSelectionMethod.hxx>
#include <MeshVS_SelectionModeFlags.hxx>
#include <initializer_list>
#include <iostream>
struct NamedPolicy { const char* name; int value; };
static void emit(std::initializer_list<NamedPolicy> values)
{
  for (const NamedPolicy& item : values)
    std::cout << item.name << ' ' << item.value << '\n';
}
static void priorities()
{
  emit({{"MeshVS_BP_Mesh", MeshVS_BP_Mesh}, {"MeshVS_BP_NodalColor", MeshVS_BP_NodalColor},
        {"MeshVS_BP_ElemColor", MeshVS_BP_ElemColor}, {"MeshVS_BP_Text", MeshVS_BP_Text},
        {"MeshVS_BP_Vector", MeshVS_BP_Vector}, {"MeshVS_BP_User", MeshVS_BP_User},
        {"MeshVS_BP_Default", MeshVS_BP_Default}});
}
static void selection_masks()
{
  emit({{"MeshVS_SMF_Mesh", MeshVS_SMF_Mesh}, {"MeshVS_SMF_Node", MeshVS_SMF_Node},
        {"MeshVS_SMF_0D", MeshVS_SMF_0D}, {"MeshVS_SMF_Link", MeshVS_SMF_Link},
        {"MeshVS_SMF_Face", MeshVS_SMF_Face}, {"MeshVS_SMF_Volume", MeshVS_SMF_Volume},
        {"MeshVS_SMF_Element", MeshVS_SMF_Element}, {"MeshVS_SMF_All", MeshVS_SMF_All},
        {"MeshVS_SMF_Group", MeshVS_SMF_Group}});
}
int main()
{
  priorities();
  selection_masks();
  emit({{"MeshVS_MSM_PRECISE", MeshVS_MSM_PRECISE}, {"MeshVS_MSM_NODES", MeshVS_MSM_NODES},
        {"MeshVS_MSM_BOX", MeshVS_MSM_BOX}, {"MeshVS_MSM_unnamed3", int(MeshVS_MeshSelectionMethod(3))}});
  emit({{"sizeof_priority", sizeof(MeshVS_BuilderPriority)},
        {"sizeof_selection_mask", sizeof(MeshVS_SelectionModeFlags)},
        {"sizeof_selection_method", sizeof(MeshVS_MeshSelectionMethod)}});
  for (int value = 0; value < 512; ++value)
    std::cout << "mask " << value << ' ' << int(MeshVS_SelectionModeFlags(value)) << '\n';
}
