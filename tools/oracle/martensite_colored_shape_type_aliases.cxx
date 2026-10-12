// Genuine AIS_ColoredShape public parent and protected native specialization aliases; OCCT 3d097a0.
#include <AIS_ColoredShape.hxx>
#include <BRep_Builder.hxx>
#include <iostream>
class ColoredAliasProbe:public AIS_ColoredShape {
public:
  using AIS_ColoredShape::AIS_ColoredShape;
  using CompoundGroups=AIS_ColoredShape::DataMapOfDrawerCompd;
};
int main() {
  TopoDS_Compound shape;BRep_Builder().MakeCompound(shape);
  const occ::handle<ColoredAliasProbe> object=new ColoredAliasProbe(shape);
  {
    const occ::handle<AIS_ColoredShape::base_type> parent(object.get());
    std::cout<<"parent-alias "<<(parent.get()==object.get())<<' '<<object->GetRefCount()<<' '<<parent->Shape().IsEqual(shape)<<'\n';
  }
  const occ::handle<AIS_ColoredDrawer> drawer=new AIS_ColoredDrawer(object->Attributes());
  ColoredAliasProbe::CompoundGroups groups;groups.Add(drawer,shape);
  std::cout<<"group-alias "<<groups.Extent()<<' '<<(groups.FindKey(1)==drawer)<<' '<<groups.FindFromIndex(1).IsEqual(shape)<<' '<<drawer->GetRefCount()<<'\n';
  groups.Clear(false);
  std::cout<<"aliases-finished "<<object->GetRefCount()<<' '<<drawer->GetRefCount()<<' '<<shape.TShape()->GetRefCount()<<'\n';
  return 0;
}
