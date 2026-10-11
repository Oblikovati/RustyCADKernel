// AIS_Plane native state and exact presentation/selection packets at OCCT 3d097a0.
#include <AIS_Plane.hxx>
#include <Geom_Axis2Placement.hxx>
#include <Select3D_SensitiveTriangulation.hxx>
class AisPlaneProbe : public AIS_Plane {
public:
  using AIS_Plane::AIS_Plane;
  using PrsMgr_PresentableObject::Compute;
  void ProbeCompute(const occ::handle<Graphic3d_Structure>& prs,int mode) {Compute(nullptr,prs,mode);}
};
static occ::handle<Geom_Plane> ais_plane_geometry(int kind) {
  gp_Ax3 frame(gp_Pnt(1,2,3),gp_Dir(0,0,1),gp_Dir(1,0,0));
  if(kind==1)frame.YReverse();
  if(kind==2)frame=gp_Ax3(gp_Pnt(-2,4,6),gp_Dir(0,.6,.8),gp_Dir(1,0,0));
  return new Geom_Plane(frame);
}
static occ::handle<Geom_Axis2Placement> ais_plane_axis() {
  return new Geom_Axis2Placement(gp_Ax2(gp_Pnt(2,3,4),gp_Dir(0,.6,.8),gp_Dir(1,0,0)));
}
static occ::handle<AIS_Plane> ais_plane_construct(int ctor,int kind) {
  auto pl=ais_plane_geometry(kind);gp_Pnt c(2,3,4),a(-1,0,2),b(99,99,99);
  if(ctor==0)return new AisPlaneProbe(pl,false);
  if(ctor==1)return new AisPlaneProbe(pl,c,true);
  if(ctor==2)return new AisPlaneProbe(pl,c,a,b,true);
  return new AisPlaneProbe(pl,false);
}
static void ais_plane_snapshot(const occ::handle<AIS_Plane>& obj) {
  ga_int(obj->Signature());ga_int(obj->Type());ga_int(obj->TypeOfPlane());
  ga_flag(obj->IsXYZPlane());ga_flag(obj->CurrentMode());ga_pnt(obj->Center());
  ga_flag(obj->Axis2Placement().IsNull());ga_int(obj->TypeOfSensitivity());
  if(!obj->Component().IsNull()) {ga_flag(true);ga_pnt(obj->Component()->Location());double a,b,c,d;obj->Component()->Coefficients(a,b,c,d);ga_real(a);ga_real(b);ga_real(c);ga_real(d);}
  else ga_flag(false);
  occ::handle<Geom_Plane> component;gp_Pnt c(9,8,7),a(9,8,7),b(9,8,7);
  ga_flag(obj->PlaneAttributes(component,c,a,b));ga_flag(component.IsNull());ga_pnt(c);ga_pnt(a);ga_pnt(b);
  for(int mode:{-1,0,1,2,99})ga_flag(obj->AcceptDisplayMode(mode));
}
static void ais_plane_appearance(const occ::handle<AIS_Plane>& obj) {
  ga_flag(obj->HasOwnSize());ga_flag(obj->HasColor());ga_flag(obj->HasMinimumSize());ga_flag(obj->IsInfinite());
  double x,y;ga_flag(obj->Size(x,y));ga_real(x);ga_real(y);
  auto drawer=obj->Attributes();auto pa=drawer->PlaneAspect();auto da=drawer->DatumAspect();
  ga_flag(drawer->HasOwnPlaneAspect());ga_flag(drawer->HasOwnDatumAspect());
  ga_real(pa->IsoDistance());ga_real(pa->EdgesAspect()->Aspect()->Width());ais_line_color(pa->EdgesAspect()->Aspect()->Color());
  for(auto part:{Prs3d_DatumParts_XAxis,Prs3d_DatumParts_YAxis,Prs3d_DatumParts_ZAxis}) {ga_real(da->AxisLength(part));ais_line_color(da->LineAspect(part)->Aspect()->Color());}
  ais_line_color(drawer->Color());ais_line_color(drawer->ShadingAspect()->Color());ga_real(drawer->ShadingAspect()->Transparency());
}
static void ais_plane_selection(const occ::handle<AIS_Plane>& obj,int mode) {
  occ::handle<SelectMgr_Selection> sel=new SelectMgr_Selection(mode);
  auto base=static_cast<SelectMgr_SelectableObject*>(obj.get());base->ComputeSelection(sel,mode);base->ComputeSelection(sel,mode);
  ga_int(sel->Entities().Size());auto s=occ::down_cast<Select3D_SensitiveTriangulation>(sel->Entities().First()->BaseSensitive());
  ga_int(s->OwnerId()->Priority());ga_flag(s->OwnerId()->Selectable()==obj);ga_int(s->NbSubElements());
  auto poly=s->Triangulation();ga_int(poly->NbNodes());ga_int(poly->NbTriangles());
  for(int i=1;i<=poly->NbNodes();++i)ga_pnt(poly->Node(i));
  for(int i=1;i<=poly->NbTriangles();++i){int a,b,c;poly->Triangle(i).Get(a,b,c);ga_int(a);ga_int(b);ga_int(c);}
}
static void ais_plane_operation(const occ::handle<AIS_Plane>& obj,const occ::handle<Graphic3d_Structure>& prs,V3DisplayTrace& trace,int op) {
  switch(op) {
  case 0:case 1:case 2:case 3: {
    trace.events.clear();static_cast<AisPlaneProbe*>(obj.get())->ProbeCompute(prs,op==3?99:op);
    v3_display_trace(trace);ga_flag(prs->IsInfinite());break; }
  case 4:obj->SetSize(8.);break;
  case 5:obj->SetSize(4.,6.);break;
  case 6:obj->UnsetSize();break;
  case 7:obj->SetColor(Quantity_NOC_RED);break;
  case 8:obj->UnsetColor();break;
  case 9:obj->SetMinimumSize(2.);break;
  case 10:obj->SetMinimumSize(0.);break;
  case 11:obj->UnsetMinimumSize();break;
  case 12:obj->SetCenter(gp_Pnt(7,8,9));break;
  case 13:obj->SetComponent(ais_plane_geometry(2));break;
  case 14:obj->SetAxis2Placement(ais_plane_axis(),AIS_TOPL_YZPlane);break;
  case 15:obj->SetPlaneAttributes(ais_plane_geometry(1),gp_Pnt(2,3,4),gp_Pnt(2,1,0),gp_Pnt(100,100,100));break;
  case 16:obj->SetCurrentMode(!obj->CurrentMode());break;
  case 17:obj->SetTypeOfSensitivity(Select3D_TOS_INTERIOR);break;
  case 18:obj->SetTypeOfSensitivity(Select3D_TOS_BOUNDARY);break;
  case 19:ais_plane_selection(obj,-1);break;
  case 20:ais_plane_selection(obj,0);break;
  case 21:ais_plane_selection(obj,99);break;
  case 22:obj->SetInfiniteState(true);break;
  case 23:obj->SetContext(nullptr);break;
  }
}
void v3_ais_plane(std::istream& input) {
  GaReader r{input};int ctor=r.integer(),kind=r.integer(),linked=r.integer(),count=r.integer();
  V3DisplayTrace trace;occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<Graphic3d_Structure> prs=new Graphic3d_Structure(viewer->StructureManager());
  GA_TRY(auto obj=ais_plane_construct(ctor,kind);
    if(!linked){obj->Attributes()->SetPlaneAspect(new Prs3d_PlaneAspect());obj->Attributes()->SetDatumAspect(new Prs3d_DatumAspect());}
    if(linked){occ::handle<Prs3d_Drawer> d=new Prs3d_Drawer();d->SetupOwnDefaults();d->PlaneAspect()->SetPlaneLength(11,13);d->PlaneAspect()->SetIsoDistance(7);d->DatumAspect()->SetAxisLength(2,3,5);obj->Attributes()->Link(d);}
    if(ctor>=3)obj->SetAxis2Placement(ais_plane_axis(),AIS_TypeOfPlane(ctor-3));
    ais_plane_snapshot(obj);
    // Avoid undefined native reads of constructor-uninitialized myHasOwnSize.
    obj->SetSize(4.,6.);ais_plane_appearance(obj);
    for(int i=0;i<count;++i){int op=r.integer();GA_TRY(ais_plane_operation(obj,prs,trace,op))ais_plane_snapshot(obj);ais_plane_appearance(obj);})
  ga_end();
}
