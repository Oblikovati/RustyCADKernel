// Martensite appearance oracle uses real displayed presentations, mutable maps and native aspect slots.
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"
#define main interaction_probe_unused_main
#include "martensite_view_cube_interaction.cxx"
#undef main
#pragma GCC diagnostic pop
#include <PrsMgr_Presentation.hxx>
#include <limits>
struct AppearanceTrace {int synchronized=0,arrays=0,vertices=0,indices=0;};
class AppearanceGroup:public SvGroup {
  AppearanceTrace& trace;
public:
  AppearanceGroup(const occ::handle<Graphic3d_Structure>& structure,AppearanceTrace& value):SvGroup(structure),trace(value){}
  void SynchronizeAspects() override {++trace.synchronized;}
  void AddPrimitiveArray(Graphic3d_TypeOfPrimitiveArray,const occ::handle<Graphic3d_IndexBuffer>& indices,const occ::handle<Graphic3d_Buffer>& vertices,const occ::handle<Graphic3d_BoundBuffer>&,bool) override {
    ++trace.arrays;trace.vertices+=vertices->NbElements;trace.indices+=indices.IsNull()?0:indices->NbElements;
  }
};
class AppearanceStructure:public SvCStructure {
  AppearanceTrace& trace;
public:
  AppearanceStructure(const occ::handle<Graphic3d_StructureManager>& manager,AppearanceTrace& value):SvCStructure(manager),trace(value){}
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& structure) override {
    auto group=occ::handle<Graphic3d_Group>(new AppearanceGroup(structure,trace));myGroups.Append(group);return group;
  }
};
class AppearanceDriver:public SvDriver {
  AppearanceTrace& trace;
public:
  AppearanceDriver(AppearanceTrace& value):trace(value){}
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& manager) override {return new AppearanceStructure(manager,trace);}
};
class AppearanceCube:public WidgetCube {
public:
  using AIS_ViewCube::setDefaultAttributes;
  using AIS_ViewCube::setDefaultHighlightAttributes;
  auto& sideLabels() {return myBoxSideLabels;}
  auto& axesLabels() {return myAxesLabels;}
  void replaceEdge(const occ::handle<Prs3d_ShadingAspect>& style) {myBoxEdgeAspect=style;}
  void replaceCorner(const occ::handle<Prs3d_ShadingAspect>& style) {myBoxCornerAspect=style;}
};
struct AppearanceScene {
  AppearanceTrace trace;occ::handle<V3d_Viewer> viewer;occ::handle<V3d_View> view;occ::handle<WidgetContext> context;occ::handle<AppearanceCube> cube;
  AppearanceScene() {
    viewer=new V3d_Viewer(new AppearanceDriver(trace));view=viewer->CreateView();auto window=occ::handle<Aspect_NeutralWindow>(new Aspect_NeutralWindow());window->SetSize(640,480);
    view->SetWindow(window);view->SetImmediateUpdate(false);context=new WidgetContext(viewer);context->activate(view);cube=new AppearanceCube();context->Display(cube,false);
    trace.synchronized=0;trace.arrays=trace.vertices=trace.indices=0;
  }
};
static void colorChannels(const Quantity_Color& value) {std::cout<<' '<<value.Red()<<' '<<value.Green()<<' '<<value.Blue();}
static void asciiChannels(const TCollection_AsciiString& value) {
  std::cout<<' '<<value.Length()<<' ';for(int i=1;i<=value.Length();++i)std::cout<<std::hex<<std::setw(2)<<std::setfill('0')<<int(static_cast<unsigned char>(value.Value(i)));std::cout<<std::dec<<std::setfill(' ');
  if(value.IsEmpty())std::cout<<'-';
}
static void appearanceState(const char* kind,int id,AppearanceScene& scene) {
  auto cube=scene.cube;auto prs=scene.context->MainPrsMgr()->Presentation(cube,0);auto selection=cube->Selection(0);
  std::cout<<kind<<' '<<id<<' '<<cube->Size()<<' '<<cube->BoxFacetExtension()<<' '<<cube->AxesPadding()<<' '<<cube->BoxEdgeGap()<<' '<<cube->BoxEdgeMinSize()<<' '<<cube->BoxCornerMinSize()<<' '<<cube->RoundRadius()<<' '<<cube->AxesRadius()<<' '<<cube->AxesConeRadius()<<' '<<cube->AxesSphereRadius()<<' '<<cube->FontHeight();
  std::cout<<' '<<cube->ToDrawAxes()<<' '<<cube->ToDrawEdges()<<' '<<cube->ToDrawVertices()<<' '<<cube->IsYup()<<' '<<prs->MustBeUpdated()<<' '<<scene.trace.synchronized<<' '<<selection->Entities().Size()<<' '<<int(selection->UpdateStatus());
  std::cout<<' '<<cube->HasColor()<<' '<<cube->HasMaterial()<<' '<<cube->Transparency()<<' '<<cube->BoxTransparency();
  colorChannels(cube->BoxColor());colorChannels(cube->BoxEdgeStyle()->Color());colorChannels(cube->BoxCornerStyle()->Color());colorChannels(cube->InnerColor());colorChannels(cube->TextColor());
  asciiChannels(cube->Font());for(int side=0;side<26;++side)asciiChannels(cube->BoxSideLabel(V3d_TypeOfOrientation(side)));
  for(int part=0;part<Prs3d_DatumParts_NB;++part)asciiChannels(cube->AxisLabel(Prs3d_DatumParts(part)));
  std::cout<<' '<<scene.trace.arrays<<' '<<scene.trace.vertices<<' '<<scene.trace.indices<<'\n';
}
static void mutateDimensions(const occ::handle<AppearanceCube>& cube,int id,double delta) {
  switch(id) {
    case 0:cube->SetSize(70+delta,false);break;case 1:cube->SetSize(90+delta,true);break;
    case 2:cube->SetBoxFacetExtension(10.5+delta);break;case 3:cube->SetAxesPadding(7+delta);break;
    case 4:cube->SetBoxEdgeGap(delta);break;case 5:cube->SetBoxEdgeMinSize(2+delta);break;
    case 6:cube->SetBoxCornerMinSize(2+delta);break;case 7:cube->SetRoundRadius(delta);break;
    case 8:cube->SetAxesRadius(1+delta);break;case 9:cube->SetAxesConeRadius(3+delta);break;
    case 10:cube->SetAxesSphereRadius(4+delta);break;case 11:cube->SetFontHeight(11.2+delta);break;
  }
}
static void mutateLabels(const occ::handle<AppearanceCube>& cube,int id) {
  switch(id) {
    case 12:cube->SetDrawAxes(false);break;case 13:cube->SetDrawEdges(false);break;
    case 14:cube->SetDrawVertices(false);break;case 15:cube->SetYup(true,true);break;
    case 16:cube->SetYup(true,false);break;case 17:cube->SetBoxSideLabel(V3d_Zpos,"roof");break;
    case 18:cube->SetAxesLabels("u","v","w");break;case 19:cube->SetFont("mono-test");break;
  }
}
static void mutateStyle(const occ::handle<AppearanceCube>& cube,int id) {
  Quantity_Color red(0.2,0.4,0.6,Quantity_TOC_RGB);
  switch(id) {
    case 20:cube->SetBoxColor(red);break;case 21:cube->SetBoxTransparency(0.3);break;
    case 22:cube->SetInnerColor(red);break;case 23:cube->SetTextColor(red);break;
    case 24:cube->SetColor(red);break;case 25:cube->UnsetColor();break;
    case 26:cube->SetTransparency(0.3);break;case 27:cube->UnsetTransparency();break;
    case 28:cube->SetMaterial(Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold));break;
    case 29:cube->UnsetMaterial();break;case 30:cube->UnsetAttributes();break;
    case 31:cube->UnsetHilightAttributes();break;case 32:cube->ResetStyles();break;
    case 33:cube->setDefaultAttributes();break;case 34:cube->setDefaultHighlightAttributes();break;
  }
}
static void prepareCustomAppearance(const occ::handle<AppearanceCube>& cube) {
  cube->SetYup(true,false);cube->SetBoxSideLabel(V3d_Zpos,"custom-top");cube->SetAxesLabels("custom-x","custom-y","custom-z");
  cube->SetAxesRadius(2.4);cube->SetAxesConeRadius(4.2);cube->SetAxesSphereRadius(5.3);cube->SetSize(100,false);
  cube->SetBoxFacetExtension(-2);cube->SetAxesPadding(0);cube->SetBoxEdgeMinSize(5);cube->SetBoxCornerMinSize(6);cube->SetBoxEdgeGap(2);cube->SetRoundRadius(0.3);
  cube->SetDrawAxes(false);cube->SetDrawEdges(false);cube->SetDrawVertices(false);cube->SetFont("custom-font");cube->SetFontHeight(23);
  cube->SetBoxColor(Quantity_Color(0.7,0.2,0.1,Quantity_TOC_RGB));cube->SetBoxTransparency(0.45);cube->SetInnerColor(Quantity_Color(0.1,0.8,0.2,Quantity_TOC_RGB));cube->SetTextColor(Quantity_Color(0.8,0.1,0.4,Quantity_TOC_RGB));
}
static void applyAppearanceMutation(const occ::handle<AppearanceCube>& cube,int id,double delta) {
  if(id<12)mutateDimensions(cube,id,delta);else if(id<20)mutateLabels(cube,id);else mutateStyle(cube,id);
}
static void mutationCase(int id,int pass) {
  AppearanceScene scene;double delta=pass==0?0:pass==1?1e-8:0.2;
  if(id>=12 && pass==2)prepareCustomAppearance(scene.cube);
  if(id>=12 && pass==1)applyAppearanceMutation(scene.cube,id,delta);
  scene.context->MainPrsMgr()->Presentation(scene.cube,0)->SetUpdateStatus(false);scene.trace.synchronized=0;
  applyAppearanceMutation(scene.cube,id,delta);
  appearanceState("mutation",id*3+pass,scene);
}
static void selectionLifecycle() {
  AppearanceScene scene;auto original=scene.cube->Selection(0);auto first=original->Entities().First()->BaseSensitive();
  scene.cube->SetDrawEdges(false);scene.cube->SetDrawVertices(false);appearanceState("lifecycle",0,scene);
  scene.context->RecomputePrsOnly(scene.cube,false,false);appearanceState("lifecycle",1,scene);
  std::cout<<"cache 0 "<<(scene.cube->Selection(0)==original)<<' '<<(scene.cube->Selection(0)->Entities().First()->BaseSensitive()==first)<<'\n';
  scene.context->Redisplay(scene.cube,false);appearanceState("lifecycle",2,scene);
  std::cout<<"cache 1 "<<(scene.cube->Selection(0)==original)<<' '<<(scene.cube->Selection(0)->Entities().First()->BaseSensitive()==first)<<'\n';
}
static void arbitraryLabels() {
  AppearanceScene scene;scene.cube->sideLabels().UnBind(V3d_Yneg);scene.cube->sideLabels().Bind(V3d_XposYpos,"edge-key");scene.cube->axesLabels().Bind(Prs3d_DatumParts_Origin,"origin-key");scene.cube->axesLabels().UnBind(Prs3d_DatumParts_XAxis);
  appearanceState("maps",0,scene);scene.cube->SetYup(true,true);appearanceState("maps",1,scene);scene.cube->SetYup(false,true);appearanceState("maps",2,scene);
}

static void boundaryCases() {
  int id=0;for(double radius:{-0.1,0.6,0.0,0.5,std::numeric_limits<double>::quiet_NaN()}) {
    AppearanceScene scene;try {scene.cube->SetRoundRadius(radius);std::cout<<"boundary "<<id<<" none ";}
    catch(const Standard_Failure& failure) {std::cout<<"boundary "<<id<<' '<<failure.ExceptionType()<<' ';}
    std::cout<<scene.cube->RoundRadius()<<' '<<scene.context->MainPrsMgr()->Presentation(scene.cube,0)->MustBeUpdated()<<std::endl;++id;
  }
  for(double transparency:{-0.1,1.1,1.0000000001,std::numeric_limits<double>::quiet_NaN()}) {
    AppearanceScene scene;try {scene.cube->SetBoxTransparency(transparency);std::cout<<"boundary "<<id<<" none ";}
    catch(const Standard_Failure& failure) {std::cout<<"boundary "<<id<<' '<<failure.ExceptionType()<<' ';}
    std::cout<<scene.cube->BoxTransparency()<<' '<<scene.trace.synchronized<<std::endl;++id;
  }
}
static void allProtectedKeys() {
  AppearanceScene scene;scene.cube->sideLabels().Clear();scene.cube->axesLabels().Clear();
  for(int key=0;key<26;++key)scene.cube->sideLabels().Bind(V3d_TypeOfOrientation(key),TCollection_AsciiString(key));
  for(int key=0;key<Prs3d_DatumParts_NB;++key)scene.cube->axesLabels().Bind(Prs3d_DatumParts(key),TCollection_AsciiString(key));
  appearanceState("allkeys",0,scene);
  std::cout<<"map-order sides";for(auto it=scene.cube->sideLabels().begin();it!=scene.cube->sideLabels().end();++it)std::cout<<' '<<int(it.Key());std::cout<<std::endl;
  std::cout<<"map-order axes";for(auto it=scene.cube->axesLabels().begin();it!=scene.cube->axesLabels().end();++it)std::cout<<' '<<int(it.Key());std::cout<<std::endl;
}

#ifndef MARTENSITE_APPEARANCE_SOURCE_ONLY
int main() {
  std::cout<<std::setprecision(17);for(int id=0;id<35;++id)for(int pass=0;pass<3;++pass)mutationCase(id,pass);
  selectionLifecycle();arbitraryLabels();boundaryCases();allProtectedKeys();
}

#endif
