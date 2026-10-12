// Real group virtual setters receive the original shading Aspect() field, including replacement inside the override.
#define main colored_compute_fixture_main
#include "martensite_colored_compute.cxx"
#undef main
struct ColoredAspectArgumentState {
  V3DisplayTrace trace;
  occ::handle<Prs3d_ShadingAspect> shading;
  occ::handle<Graphic3d_AspectFillArea3d> previous,incoming;
  bool armed=true;
};
class ColoredAspectArgumentGroup:public V3DisplayGroup
{
  ColoredAspectArgumentState& state;
public:
  ColoredAspectArgumentGroup(const occ::handle<Graphic3d_Structure>& prs,ColoredAspectArgumentState& input)
    :V3DisplayGroup(prs,input.trace),state(input){}
  void SetPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& aspect) override
  {
    if(state.armed)
    {
      state.armed=false;
      std::cout<<"aspect-argument-before "<<((const void*)&aspect==(const void*)&state.shading->Aspect())<<' '<<aspect->GetRefCount()<<' '<<state.previous->GetRefCount()<<' '<<state.incoming->GetRefCount()<<'\n';
      state.shading->SetAspect(state.incoming);
      std::cout<<"aspect-argument-after "<<((const void*)&aspect==(const void*)&state.shading->Aspect())<<' '<<(aspect==state.incoming)<<' '<<aspect->GetRefCount()<<' '<<state.previous->GetRefCount()<<' '<<state.incoming->GetRefCount()<<'\n';
    }
    V3DisplayGroup::SetPrimitivesAspect(aspect);
  }
};
class ColoredAspectArgumentStructure:public SvCStructure
{
  ColoredAspectArgumentState& state;
public:
  ColoredAspectArgumentStructure(const occ::handle<Graphic3d_StructureManager>& manager,ColoredAspectArgumentState& input):SvCStructure(manager),state(input){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& prs) override
  {
    auto group=new ColoredAspectArgumentGroup(prs,state);
    myGroups.Append(group);
    return group;
  }
};
class ColoredAspectArgumentDriver:public SvDriver
{
  ColoredAspectArgumentState& state;
public:
  ColoredAspectArgumentDriver(ColoredAspectArgumentState& input):state(input){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override
  {
    return new ColoredAspectArgumentStructure(manager,state);
  }
};
int main()
{
  ColoredExpansionInputs input;const TopoDS_Shape shape=compute_input(input,4);
  const occ::handle<ColoredEmissionProbe> object=new ColoredEmissionProbe(shape);
  configure_compute(object,shape,input,4);
  ColoredAspectArgumentState state;state.shading=object->Attributes()->ShadingAspect();
  state.previous=state.shading->Aspect();state.incoming=new Graphic3d_AspectFillArea3d;
  const occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new ColoredAspectArgumentDriver(state));
  const occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  object->Compute(occ::handle<PrsMgr_PresentationManager>(),prs,1);
  std::cout<<"aspect-argument-finished "<<prs->Groups().Size()<<' '<<state.previous->GetRefCount()<<' '<<state.incoming->GetRefCount()<<'\n';
  return 0;
}
