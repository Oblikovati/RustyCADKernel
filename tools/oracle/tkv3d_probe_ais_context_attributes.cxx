// AIS_InteractiveContext attribute, display mode, activation, filter, sub-intensity and default
// drawer members on a scene of a cylinder AIS_Shape, an edge AIS_Shape and an AIS_Point,
// the twin of crates/tkv3d/examples/v3_probe/ais_context_attributes.rs. A request is
// `v3_ais_ctx_attr Count Op Object Arg...`.
#include <AIS_InteractiveContext.hxx>
#include <AIS_Point.hxx>
#include <AIS_Shape.hxx>
#include <AIS_TypeFilter.hxx>
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_CartesianPoint.hxx>
#include <Graphic3d_TransformPers.hxx>
#include <Prs3d_DatumAspect.hxx>
#include <Prs3d_PlaneAspect.hxx>
#include <SelectMgr_AndOrFilter.hxx>
#include <gp_Cylinder.hxx>

// Expose the protected InitAttributes.
class V3ProbeAttrContext : public AIS_InteractiveContext {
public:
  V3ProbeAttrContext(const occ::handle<V3d_Viewer>& v):AIS_InteractiveContext(v){}
  void initAttributes(){InitAttributes();}
};
static void v3_ctx_attr_state(const occ::handle<AIS_InteractiveContext>& ctx,const std::vector<occ::handle<AIS_InteractiveObject>>& objects){
  for(const occ::handle<AIS_InteractiveObject>& o:objects){
    ga_int(ctx->DisplayStatus(o));ga_flag(o->HasDisplayMode());ga_int(o->DisplayMode());ga_flag(ctx->HasColor(o));
    Quantity_Color c;ctx->Color(o,c);ga_real(c.Red());ga_real(c.Green());ga_real(c.Blue());ga_real(ctx->Width(o));ga_real(o->Transparency());
    ga_flag(ctx->HasPolygonOffsets(o));int m=0;float f=0,u=0;ctx->PolygonOffsets(o,m,f,u);ga_int(m);ga_real(f);ga_real(u);
    ga_int(ctx->GetZLayer(o));ga_int(ctx->DisplayPriority(o));NCollection_List<int> modes;ctx->ActivatedModes(o,modes);ga_int(modes.Extent());for(int v:modes)ga_int(v);
    ga_flag(ctx->IsSelected(o));ga_flag(o->HasMaterial());ga_int(o->CurrentFacingModel());
    occ::handle<NCollection_Shared<NCollection_IndexedMap<occ::handle<SelectMgr_EntityOwner>>>> owners;ctx->EntityOwners(owners,o,-1);ga_int(owners.IsNull()?-1:owners->Extent());
    TCollection_ExtendedString status;ctx->Status(o,status);ga_int(status.Length());
  }
  ga_int(ctx->Filters().Extent());ga_int(ctx->FilterType());ga_int(ctx->PixelTolerance());ga_real(ctx->TrihedronSize());
  double x,y;ga_flag(ctx->PlaneSize(x,y));ga_real(x);ga_real(y);ga_flag(ctx->DrawHiddenLine());ga_int(ctx->DisplayMode());
  ga_flag(ctx->HasSelectedShape());ga_int(ctx->NbSelected());
}
static void v3_ctx_attr_op(const occ::handle<AIS_InteractiveContext>& ctx,const occ::handle<AIS_InteractiveObject>& o,int op,int arg){
  switch(op){
    case 0:ctx->SetColor(o,Quantity_NOC_RED,true);break;
    case 1:ctx->UnsetColor(o,true);break;
    case 2:ctx->SetWidth(o,3.,true);break;
    case 3:ctx->UnsetWidth(o,true);break;
    case 4:ctx->SetMaterial(o,Graphic3d_MaterialAspect(Graphic3d_NameOfMaterial_Gold),true);break;
    case 5:ctx->UnsetMaterial(o,true);break;
    case 6:ctx->SetTransparency(o,arg/10.,true);break;
    case 7:ctx->UnsetTransparency(o,true);break;
    case 8:ctx->SetPolygonOffsets(o,1,1.5f,2.f,true);break;
    case 9:ctx->SetDisplayMode(o,arg,true);break;
    case 10:ctx->UnsetDisplayMode(o,true);break;
    case 11:ctx->SetDisplayMode(arg,true);break;
    case 12:ctx->SetDisplayPriority(o,Graphic3d_DisplayPriority(arg));break;
    case 13:ctx->SetZLayer(o,arg);break;
    case 14:ctx->Activate(o,arg);break;
    case 15:ctx->Deactivate(o);break;
    case 16:ctx->Deactivate(o,arg);break;
    case 17:ctx->Activate(arg);break;
    case 18:ctx->Deactivate();break;
    case 19:ctx->Deactivate(arg);break;
    case 20:ctx->SetSelectionModeActive(o,arg,true,AIS_SelectionModesConcurrency_Single);break;
    case 21:ctx->SetSelectionModeActive(o,arg,true,AIS_SelectionModesConcurrency_Multiple);break;
    case 22:ctx->AddFilter(new AIS_TypeFilter(AIS_KindOfInteractive(arg)));break;
    case 23:ctx->RemoveFilters();break;
    case 24:ctx->SetFilterType(SelectMgr_FilterType(arg));break;
    case 25:ctx->SetPixelTolerance(arg);break;
    case 26:ctx->SetTrihedronSize(arg,true);break;
    case 27:ctx->SetPlaneSize(arg,arg+1.,true);break;
    case 28:ctx->SetPlaneSize(arg,true);break;
    case 29:ctx->EnableDrawHiddenLine();break;
    case 30:ctx->DisableDrawHiddenLine();break;
    case 31:ctx->SubIntensityOn(o,true);break;
    case 32:ctx->SubIntensityOff(o,true);break;
    case 33:ctx->SetSelected(o,true);break;
    case 34:occ::down_cast<V3ProbeAttrContext>(ctx)->initAttributes();break;
    case 35:{occ::handle<Prs3d_Drawer> d=new Prs3d_Drawer();d->SetupOwnDefaults();d->SetColor(Quantity_NOC_BLUE1);ctx->SetLocalAttributes(o,d,true);break;}
    case 36:ctx->UnsetLocalAttributes(o,true);break;
    case 37:ctx->SetAngleAndDeviation(o,0.3,true);break;
    case 38:ctx->SetSelectionSensitivity(o,0,arg);break;
    case 39:ctx->SetCurrentFacingModel(o,Aspect_TypeOfFacingModel(arg));break;
    case 40:ctx->Erase(o,true);break;
    case 41:ctx->Display(o,true);break;
    case 42:ctx->RebuildSelectionStructs();break;
    case 43:ctx->SetSubIntensityColor(Quantity_NOC_GREEN);break;
    case 44:ctx->SetTransformPersistence(o,new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp_Pnt(1,2,3)));break;
    case 45:{occ::handle<Prs3d_LineAspect> l=new Prs3d_LineAspect(Quantity_NOC_ORANGE,Aspect_TOL_DOT,2.);ctx->SetHiddenLineAspect(l);break;}
    case 46:ctx->ClearSelected(true);break;
    case 47:ctx->Redisplay(AIS_KindOfInteractive(arg),0,true);break;
  }
}
void v3_ais_ctx_attr(std::istream& input){
  GaReader r{input};const int count=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<AIS_InteractiveContext> ctx=new V3ProbeAttrContext(viewer);
  const gp_Ax3 ax(gp_Pnt(0,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0));
  std::vector<occ::handle<AIS_InteractiveObject>> objects={
    new AIS_Shape(BRepBuilderAPI_MakeFace(gp_Cylinder(ax,2.),0.,M_PI,0.,3.).Face()),
    new AIS_Shape(BRepBuilderAPI_MakeEdge(gp_Pnt(0,0,0),gp_Pnt(2,1,0)).Edge()),
    new AIS_Point(new Geom_CartesianPoint(1,2,3))};
  for(const auto& o:objects)ctx->Display(o,false);
  v3_ctx_attr_state(ctx,objects);
  for(int i=0;i<count;++i){const int op=r.integer(),obj=r.integer(),arg=r.integer();
    GA_TRY(v3_ctx_attr_op(ctx,objects[obj%3],op,arg))v3_ctx_attr_state(ctx,objects);}
  ga_int(int(trace.events.size()));ga_end();
}
