// Retained nested references use actual native subfields with original owners kept alive.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"
#define MARTENSITE_APPEARANCE_SOURCE_ONLY
#include "martensite_view_cube_appearance.cxx"
#undef MARTENSITE_APPEARANCE_SOURCE_ONLY
#pragma GCC diagnostic pop
static void colorAliasRow(int step,const Quantity_Color& old,const Quantity_Color& current,const Quantity_Color& comparison) {
  std::cout<<"color "<<step<<' '<<(&old==&current)<<' '<<(&old==&comparison);colorChannels(old);colorChannels(current);std::cout<<'\n';
}
static void retainedColorFields() {
  AppearanceScene scene;auto cube=scene.cube;auto shade=cube->BoxSideStyle();auto fill=shade->Aspect();
  const auto& first=cube->BoxColor();const auto& second=cube->BoxColor();
  colorAliasRow(0,first,second,fill->InteriorColor());
  cube->SetBoxColor(Quantity_Color(0.2,0.4,0.6,Quantity_TOC_RGB));colorAliasRow(1,first,cube->BoxColor(),fill->InteriorColor());
  Graphic3d_MaterialAspect physical(Graphic3d_NameOfMaterial_Gold);cube->SetMaterial(physical);
  colorAliasRow(2,first,cube->BoxColor(),fill->InteriorColor());
  const auto& materialColor=cube->BoxColor();Graphic3d_MaterialAspect aspect(Graphic3d_NameOfMaterial_UserDefined);aspect.SetMaterialType(Graphic3d_MATERIAL_ASPECT);cube->SetMaterial(aspect);
  colorAliasRow(3,materialColor,cube->BoxColor(),fill->FrontMaterial().Color());
  cube->SetBoxColor(Quantity_Color(0.7,0.2,0.1,Quantity_TOC_RGB));colorAliasRow(4,materialColor,cube->BoxColor(),fill->FrontMaterial().Color());
  const auto& interior=cube->BoxColor();cube->SetMaterial(physical);colorAliasRow(7,interior,cube->BoxColor(),fill->InteriorColor());
  const auto& back=cube->InnerColor();cube->SetInnerColor(Quantity_Color(0.1,0.8,0.2,Quantity_TOC_RGB));colorAliasRow(5,back,cube->InnerColor(),fill->BackMaterial().Color());
  auto text=cube->Attributes()->TextAspect()->Aspect();const auto& textColor=cube->TextColor();cube->SetTextColor(Quantity_Color(0.8,0.1,0.4,Quantity_TOC_RGB));colorAliasRow(6,textColor,cube->TextColor(),text->Color());
}
static void retainedFontPayload() {
  AppearanceScene scene;auto cube=scene.cube;auto fontOwner=cube->Attributes()->TextAspect()->Aspect()->TextFont();
  const auto& left=cube->Font();const auto& right=cube->Font();
  std::cout<<"font 0 "<<(&left==&right)<<' '<<fontOwner->GetRefCount();asciiChannels(left);asciiChannels(cube->Font());std::cout<<'\n';
  cube->SetFont("new-font");std::cout<<"font 1 "<<(&left==&cube->Font())<<' '<<fontOwner->GetRefCount();asciiChannels(left);asciiChannels(cube->Font());std::cout<<'\n';
  fontOwner->SetValue(1,'Q');std::cout<<"font 2 "<<(&left==&right)<<' '<<fontOwner->GetRefCount();asciiChannels(left);asciiChannels(cube->Font());std::cout<<'\n';
  cube->SetFont("");const auto& empty=cube->Font();std::cout<<"font 3 "<<(&left==&empty)<<' '<<fontOwner->GetRefCount();asciiChannels(left);asciiChannels(empty);std::cout<<'\n';
  cube->SetFont("restored");std::cout<<"font 4 "<<(&empty==&cube->Font())<<' '<<fontOwner->GetRefCount();asciiChannels(empty);asciiChannels(cube->Font());std::cout<<'\n';
}
static void shadeAliasRow(int step,const occ::handle<Prs3d_ShadingAspect>& alias,const occ::handle<Prs3d_ShadingAspect>& current,const occ::handle<Prs3d_ShadingAspect>& original,const occ::handle<Prs3d_ShadingAspect>& next) {
  std::cout<<"shade "<<step<<' '<<(&alias==&current)<<' '<<(alias==original)<<' '<<(alias==next)<<' '<<(current==alias)<<' '<<original->GetRefCount()<<' '<<next->GetRefCount()<<'\n';
}
static void retainedShadingSlots() {
  AppearanceScene scene;auto cube=scene.cube;auto drawer=cube->Attributes();auto original=cube->BoxSideStyle();auto next=occ::handle<Prs3d_ShadingAspect>(new Prs3d_ShadingAspect());
  const auto& side=cube->BoxSideStyle();shadeAliasRow(0,side,cube->BoxSideStyle(),original,next);
  drawer->SetShadingAspect(next);shadeAliasRow(1,side,cube->BoxSideStyle(),original,next);
  auto replacement=occ::handle<Prs3d_Drawer>(new Prs3d_Drawer());replacement->SetShadingAspect(original);cube->SetAttributes(replacement);
  shadeAliasRow(2,side,cube->BoxSideStyle(),original,next);
  auto edgeOriginal=cube->BoxEdgeStyle();const auto& edge=cube->BoxEdgeStyle();shadeAliasRow(3,edge,cube->BoxEdgeStyle(),edgeOriginal,next);
  cube->replaceEdge(next);shadeAliasRow(4,edge,cube->BoxEdgeStyle(),edgeOriginal,next);
  auto cornerOriginal=cube->BoxCornerStyle();const auto& corner=cube->BoxCornerStyle();shadeAliasRow(5,corner,cube->BoxCornerStyle(),cornerOriginal,next);
  cube->replaceCorner(next);shadeAliasRow(6,corner,cube->BoxCornerStyle(),cornerOriginal,next);
}
#include "martensite_view_cube_appearance_lifetimes.cxx"
int main() {std::cout<<std::setprecision(17);retainedColorFields();retainedFontPayload();retainedShadingSlots();inheritedShadingSlots();retainedResetOwners();resetVirtualDispatch();}
