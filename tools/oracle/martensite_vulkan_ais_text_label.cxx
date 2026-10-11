// Native AIS annotations use viewer fonts, persistence and ordered flipping.
#include <AIS_TextLabel.hxx>
static occ::handle<AIS_TextLabel> make_text_label(const std::string& mode,double offset) {
  auto label=occ::handle<AIS_TextLabel>(new AIS_TextLabel());
  label->SetText(TCollection_ExtendedString("AV"));label->SetFont("DejaVu Sans");label->SetHeight(16.0);
  label->SetPosition(gp_Pnt(offset,0,0));
  label->SetHJustification(Graphic3d_HTA_CENTER);label->SetVJustification(Graphic3d_VTA_CENTER);
  if(mode=="label-zoom") {label->SetZoomable(true);label->SetHeight(2.0);}
  if(mode=="label-plane" || mode=="label-flip") {
    label->SetOrientation3D(gp_Ax2(label->Position(),gp_Dir(0,0,1)));
    label->SetFlipping(mode=="label-flip");
  }
  if(mode=="label-angle") label->SetAngle(0.37);
  return label;
}
struct MartensiteOracleLabelScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_TextLabel> label;
  MartensiteOracleLabelScene(const occ::handle<V3d_Viewer>& viewer,const occ::handle<V3d_View>& view,const std::string& mode,double offset) {
    context=new AIS_InteractiveContext(viewer);label=make_text_label(mode,offset);
    context->Display(label,false);
    if(mode=="label-update") {
      label->SetText(TCollection_ExtendedString("VA"));label->SetPosition(gp_Pnt(1,-1,0));
      context->Redisplay(label,false);
    }
    if(mode=="label-flip") view->Camera()->SetUp(gp_Dir(0,-1,0));
  }
};
