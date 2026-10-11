// Plane captures use the defined native constructor path; retain its actual 80% transparency.
#include <AIS_Plane.hxx>
#include <Geom_Axis2Placement.hxx>
struct MartensiteCapturePlane : AIS_Plane {
  using AIS_Plane::AIS_Plane;
  using PrsMgr_PresentableObject::Compute;
};
struct MartensiteOracleAisPlaneScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<MartensiteCapturePlane> object;
  occ::handle<Graphic3d_Structure> shaded;
  MartensiteOracleAisPlaneScene(const occ::handle<V3d_Viewer>& viewer,const std::string& mode,double offset) {
    context=new AIS_InteractiveContext(viewer);
    gp_Ax3 frame;frame.SetLocation(gp_Pnt(offset,0,0));auto pl=occ::handle<Geom_Plane>(new Geom_Plane(frame));
    if(mode=="aisplane-manual")object=new MartensiteCapturePlane(pl,gp_Pnt(offset,0,0),gp_Pnt(offset-3,-2,0),gp_Pnt(99,99,99),true);
    else object=new MartensiteCapturePlane(pl,gp_Pnt(offset,0,0),mode=="aisplane-current");
    object->SetContext(context);object->SetSize(12,10);configure(mode,offset);
    if(mode.find("shaded")!=std::string::npos){shaded=new Graphic3d_Structure(viewer->StructureManager());object->Compute(nullptr,shaded,1);shaded->Display();}
    else {context->Display(object,false);transition(mode);}
  }
  void configure(const std::string& mode,double offset) {
    auto shading=object->Attributes()->ShadingAspect();shading->Aspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
    shading->Aspect()->SetPolygonOffsets(Aspect_POM_Off,0,0);
    if(mode=="aisplane-shaded-alpha")shading->SetTransparency(0.8);
    if(mode=="aisplane-shaded-opaque")shading->SetTransparency(0);
    if(mode=="aisplane-minimum")object->SetMinimumSize(30);
    AIS_TypeOfPlane kind=AIS_TOPL_Unknown;
    if(mode=="aisplane-xy"||mode=="aisplane-shaded-xyz"||mode=="aisplane-sized")kind=AIS_TOPL_XYPlane;
    if(mode=="aisplane-xz")kind=AIS_TOPL_XZPlane;
    if(mode=="aisplane-yz")kind=AIS_TOPL_YZPlane;
    if(kind!=AIS_TOPL_Unknown)object->SetAxis2Placement(new Geom_Axis2Placement(gp_Ax2(gp_Pnt(offset-3,-3,0),gp_Dir(0,.6,.8),gp_Dir(1,0,0))),kind);
  }
  void transition(const std::string& mode) {
    if(mode=="aisplane-erased")context->Erase(object,false);
    if(mode=="aisplane-redisplayed"){context->Erase(object,false);context->Display(object,false);}
    if(mode=="aisplane-selected")context->SetSelected(object,false);
    if(mode=="aisplane-sized")object->SetSize(6,8);
    if(mode=="aisplane-unset-color"){object->SetColor(Quantity_NOC_RED);object->UnsetColor();context->Redisplay(object,false);}
  }
  void detect(const occ::handle<V3d_View>& view) {
    const gp_Pnt c=object->Center();int x,y;view->Convert(c.X()+6,c.Y(),c.Z(),x,y);context->MoveTo(x,y,view,false);
    if(context->DetectedInteractive()!=object)throw std::runtime_error("expected detected AIS_Plane boundary");
  }
};
