// Local Martensite prerequisite: native concrete/source Texture2D descriptors share one ancestry.
#include <Graphic3d_Texture2D.hxx>
#include <Graphic3d_Texture2Dplane.hxx>
#include <Graphic3d_TextureMap.hxx>
#include <iostream>
int main() {
  occ::handle<Graphic3d_Texture2D> plain=new Graphic3d_Texture2D("");
  occ::handle<Graphic3d_Texture2Dplane> plane=new Graphic3d_Texture2Dplane("");
  std::cout<<plain->DynamicType()->Name()<<' '<<plain->DynamicType()->Parent()->Name()<<' '<<plane->DynamicType()->Name()<<' '<<plane->DynamicType()->Parent()->Name();
  std::cout<<' '<<plain->IsKind(STANDARD_TYPE(Graphic3d_TextureMap))<<' '<<plane->IsKind(STANDARD_TYPE(Graphic3d_Texture2D))<<'\n';
  return 0;
}
