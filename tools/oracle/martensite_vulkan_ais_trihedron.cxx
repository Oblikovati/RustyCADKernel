// Native AIS_Trihedron uses the same datum frame, mutations and selected parts as Martensite.
#include <AIS_Trihedron.hxx>
#include <AIS_TrihedronOwner.hxx>
#include <Geom_Axis2Placement.hxx>
struct MartensiteOracleTrihedronScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_Trihedron> object;
  MartensiteOracleTrihedronScene(const occ::handle<V3d_Viewer>& viewer,double offset,const std::string& mode) {
    context=new AIS_InteractiveContext(viewer);
    gp_Ax2 frame;frame.SetLocation(gp_Pnt(-3,-3,0));
    object=new AIS_Trihedron(new Geom_Axis2Placement(frame));object->SetSize(6);
    if(mode.find("shaded")!=std::string::npos)object->SetDatumDisplayMode(Prs3d_DM_Shaded);
    gp_Trsf transform;transform.SetTranslation(gp_Vec(offset,0,0));object->SetLocalTransformation(transform);
    context->Display(object,false);
  }
  void select_part(Prs3d_DatumParts part) {
    NCollection_Sequence<occ::handle<SelectMgr_EntityOwner>> owners;
    owners.Append(new AIS_TrihedronOwner(object,part,object->SelectionPriority(part)));
    object->HilightSelected(context->MainPrsMgr(),owners);
  }
  void transition(const std::string& mode) {
    if(mode=="tri-wire-noarrows" || mode=="tri-shaded-noarrows")object->SetDrawArrows(false);
    if(mode=="tri-labels") {object->SetLabel(Prs3d_DatumParts_XAxis,"U");object->SetLabel(Prs3d_DatumParts_YAxis,"V");object->SetLabel(Prs3d_DatumParts_ZAxis,"W");}
    if(mode=="tri-color")object->SetColor(Quantity_NOC_RED);
    if(mode=="tri-arrow-color")object->SetArrowColor(Quantity_NOC_GREEN);
    if(mode=="tri-text-color")object->SetTextColor(Quantity_NOC_CYAN);
    if(mode=="tri-size")object->SetSize(4);
    if(mode=="tri-moved") {gp_Ax2 frame;frame.SetLocation(gp_Pnt(-2,1,0));object->SetComponent(new Geom_Axis2Placement(frame));}
    if(mode=="tri-selected") {context->SetSelected(object,false);return;}
    if(mode=="tri-axis" || mode=="tri-shaded-axis") {select_part(Prs3d_DatumParts_XAxis);return;}
    if(mode=="tri-origin" || mode=="tri-shaded-origin") {select_part(Prs3d_DatumParts_Origin);return;}
    if(mode=="tri-plane") {select_part(Prs3d_DatumParts_XOYAxis);return;}
    context->Redisplay(object,false);
  }
};
