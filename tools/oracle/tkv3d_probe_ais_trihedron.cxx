// Native protected trihedron cache and ownership transitions at pinned OCCT 3d097a0.
#include <AIS_Trihedron.hxx>
#include <AIS_TrihedronOwner.hxx>
class V3TrihedronProbe : public AIS_Trihedron {
public:
  explicit V3TrihedronProbe(const occ::handle<Geom_Axis2Placement>& frame) : AIS_Trihedron(frame) {}
  using AIS_Trihedron::arrayOfPrimitives;
  using AIS_Trihedron::Compute;
  using AIS_Trihedron::ComputeSelection;
};
static occ::handle<Geom_Axis2Placement> trihedron_frame(int kind) {
  if(kind == 0) return new Geom_Axis2Placement(gp_Ax2(gp_Pnt(1,2,3),gp_Dir(0,0,1),gp_Dir(1,0,0)));
  return new Geom_Axis2Placement(gp_Ax2(gp_Pnt(-2,4,6),gp_Dir(1,2,3),gp_Dir(2,-1,0)));
}
static void trihedron_snapshot(const occ::handle<V3TrihedronProbe>& object) {
  ga_int(object->Signature());ga_int(object->Type());ga_flag(object->IsAutoHilight());
  ga_int(object->DatumDisplayMode());ga_flag(object->HasOwnSize());ga_real(object->Size());
  ga_flag(object->HasColor());ga_flag(object->HasTextColor());ga_flag(object->HasArrowColor());
  ga_flag(object->ToDrawArrows());ais_line_color(object->TextColor());ais_line_color(object->ArrowColor());
  ga_flag(object->Attributes()->HasOwnDatumAspect());
  for(int index=0;index<Prs3d_DatumParts_NB;++index) {
    auto part=static_cast<Prs3d_DatumParts>(index);
    auto datum=object->Attributes()->DatumAspect();
    ga_int(object->SelectionPriority(part));ga_int(object->Label(part).Length());
    ga_flag(datum->DrawDatumPart(part));ga_real(datum->AxisLength(part));
    if(index<Prs3d_DatumParts_None) {
      ais_line_color(datum->ShadingAspect(part)->Color());
      if(index!=0)ais_line_color(datum->LineAspect(part)->Aspect()->Color());
    }
  }
}
static void trihedron_cached_arrays(const occ::handle<V3TrihedronProbe>& object) {
  for(int index=0;index<Prs3d_DatumParts_NB;++index) {
    const auto& array=object->arrayOfPrimitives(static_cast<Prs3d_DatumParts>(index));
    ga_flag(array.IsNull());if(!array.IsNull())sv_array_state(array);
  }
}
static void trihedron_selected(const occ::handle<V3TrihedronProbe>& object,int mode) {
  occ::handle<SelectMgr_Selection> selection=new SelectMgr_Selection(mode);
  object->ComputeSelection(selection,mode);ga_int(selection->Entities().Size());
  occ::handle<SelectMgr_EntityOwner> first;
  for(const auto& item:selection->Entities()) {
    auto sensitive=item->BaseSensitive();auto owner=sensitive->OwnerId();
    if(first.IsNull())first=owner;
    ga_int(owner->Priority());ga_flag(owner==first);ga_flag(owner->Selectable()==object);
    auto partOwner=occ::down_cast<AIS_TrihedronOwner>(owner);
    ga_int(partOwner.IsNull()?-1:static_cast<int>(partOwner->DatumPart()));
    auto segment=occ::down_cast<Select3D_SensitiveSegment>(sensitive);
    auto point=occ::down_cast<Select3D_SensitivePoint>(sensitive);
    ga_flag(!segment.IsNull());ga_flag(!point.IsNull());
    if(!segment.IsNull()){ga_pnt(segment->StartPoint());ga_pnt(segment->EndPoint());}
    if(!point.IsNull())ga_pnt(point->Point());
  }
}
static void trihedron_mutation(const occ::handle<V3TrihedronProbe>& object,int op,double size) {
  switch(op) {
    case 0: object->SetSize(size);break;
    case 1: object->UnsetSize();break;
    case 2: object->SetColor(Quantity_NOC_RED);break;
    case 3: object->UnsetColor();break;
    case 4: object->SetTextColor(Quantity_NOC_BLUE);break;
    case 5: object->SetTextColor(Prs3d_DatumParts_XAxis,Quantity_NOC_GREEN);break;
    case 6: object->SetArrowColor(Quantity_NOC_BLUE);break;
    case 7: object->SetArrowColor(Prs3d_DatumParts_XAxis,Quantity_NOC_GREEN);break;
    case 8: object->SetOriginColor(Quantity_NOC_RED);break;
    case 9: object->SetXAxisColor(Quantity_NOC_RED);object->SetYAxisColor(Quantity_NOC_GREEN);object->SetAxisColor(Quantity_NOC_BLUE);break;
    case 10: object->SetDatumDisplayMode(Prs3d_DM_Shaded);break;
    case 11: object->SetDatumDisplayMode(Prs3d_DM_WireFrame);break;
    case 12: object->SetDrawArrows(false);break;
    case 13: object->SetDrawArrows(true);break;
    case 14: object->SetComponent(trihedron_frame(1));break;
    case 15: object->SetLabel(Prs3d_DatumParts_XAxis,"U");object->SetSelectionPriority(Prs3d_DatumParts_XAxis,11);break;
  }
}
void v3_ais_trihedron(std::istream& input) {
  GaReader reader{input};int kind=reader.integer(),count=reader.integer();double size=reader.real();
  V3DisplayTrace trace;
  occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new V3DisplayDriver(trace));
  occ::handle<PrsMgr_PresentationManager> manager=new PrsMgr_PresentationManager(viewer->StructureManager());
  occ::handle<Prs3d_Presentation> prs=new Prs3d_Presentation(viewer->StructureManager());
  GA_TRY(
    occ::handle<V3TrihedronProbe> object=new V3TrihedronProbe(trihedron_frame(kind));
    occ::handle<Prs3d_Drawer> linked=new Prs3d_Drawer();linked->SetupOwnDefaults();
    linked->DatumAspect()->SetAxisLength(2,3,4);object->Attributes()->SetLink(linked);
    trihedron_snapshot(object);
    for(int index=0;index<count;++index) {
      int op=reader.integer();
      GA_TRY(if(op==16){prs->Clear();object->Compute(manager,prs,0);trihedron_cached_arrays(object);}
        else if(op>=20 && op<=24)trihedron_selected(object,op-20);
        else trihedron_mutation(object,op,size);)
      trihedron_snapshot(object);
    }
    ga_flag(prs->IsInfinite());ga_int(prs->Groups().Size());
  )
  ga_end();
}
