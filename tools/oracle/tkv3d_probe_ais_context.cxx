// Real AIS context construction and retained style graph; no reimplemented native bodies.
#include <AIS_InteractiveContext.hxx>
#include <Prs3d_PlaneAspect.hxx>
#include <Prs3d_DatumAspect.hxx>
static void v3_context_color(const Quantity_Color& color) {
  ga_real(color.Red());ga_real(color.Green());ga_real(color.Blue());
}
static void v3_context_line(const occ::handle<Prs3d_LineAspect>& line) {
  ga_flag(!line.IsNull());if(line.IsNull())return;
  v3_context_color(line->Aspect()->Color());ga_real(line->Aspect()->Width());ga_int(line->Aspect()->Type());
}
static void v3_context_plane(const occ::handle<Prs3d_PlaneAspect>& plane) {
  ga_flag(!plane.IsNull());if(plane.IsNull())return;
  ga_real(plane->PlaneXLength());ga_real(plane->PlaneYLength());
  v3_context_line(plane->EdgesAspect());v3_context_line(plane->IsoAspect());v3_context_line(plane->ArrowAspect());
}
static void v3_context_datum(const occ::handle<Prs3d_DatumAspect>& datum) {
  ga_flag(!datum.IsNull());if(datum.IsNull())return;
  for(int i=0;i<Prs3d_DatumParts_None;++i)v3_context_line(datum->LineAspect(Prs3d_DatumParts(i)));
  for(auto axis:{Prs3d_DatumParts_XAxis,Prs3d_DatumParts_YAxis,Prs3d_DatumParts_ZAxis})ga_real(datum->AxisLength(axis));
}
static void v3_context_style(const occ::handle<Prs3d_Drawer>& style,const occ::handle<Prs3d_Drawer>& base) {
  ga_int(style->DisplayMode());ga_int(style->ZLayer());ga_int(style->Method());v3_context_color(style->Color());
  ga_flag(style->Link()==base);ga_flag(style->IsAutoTriangulation());
  v3_context_line(style->LineAspect());v3_context_line(style->WireAspect());
  v3_context_line(style->FreeBoundaryAspect());v3_context_line(style->UnFreeBoundaryAspect());
  v3_context_line(style->HiddenLineAspect());v3_context_plane(style->PlaneAspect());v3_context_datum(style->DatumAspect());
  auto point=style->PointAspect();ga_flag(!point.IsNull());
  if(!point.IsNull()){v3_context_color(point->Aspect()->Color());ga_int(point->Aspect()->Type());ga_real(point->Aspect()->Scale());}
  auto shade=style->ShadingAspect();ga_flag(!shade.IsNull());
  if(!shade.IsNull()){v3_context_color(shade->Color(Aspect_TOFM_FRONT_SIDE));ga_int(shade->Material(Aspect_TOFM_FRONT_SIDE).Name());}
  ga_flag(!style->PlaneAspect().IsNull()&&style->PlaneAspect()->EdgesAspect()->Aspect()==base->PlaneAspect()->EdgesAspect()->Aspect());
  ga_flag(!style->LineAspect().IsNull()&&style->LineAspect()->Aspect()==base->LineAspect()->Aspect());
}
static int v3_context_style_id(const occ::handle<Prs3d_Drawer>& style,const std::vector<occ::handle<Prs3d_Drawer>>& styles) {
  for(int i=0;i<int(styles.size());++i)if(style==styles[i])return i;return -1;
}
static void v3_context_state(const occ::handle<AIS_InteractiveContext>& ctx,const std::vector<occ::handle<Prs3d_Drawer>>& styles) {
  ga_flag(ctx->ToHilightSelected());ga_flag(ctx->AutomaticHilight());ga_flag(ctx->GetAutoActivateSelection());ga_int(ctx->PickingStrategy());
  ga_int(v3_context_style_id(ctx->DefaultDrawer(),styles));
  for(int i=0;i<6;++i)ga_int(v3_context_style_id(ctx->HighlightStyle(Prs3d_TypeOfHighlight(i)),styles));
  ga_int(v3_context_style_id(ctx->HighlightStyle(),styles));ga_int(v3_context_style_id(ctx->SelectionStyle(),styles));
}
static void v3_context_op(const occ::handle<AIS_InteractiveContext>& ctx,const std::vector<occ::handle<Prs3d_Drawer>>& styles,int op,int slot,int value) {
  switch(op){
    case 0:ctx->SetDefaultDrawer(styles[value]);break;
    case 1:ctx->SetHighlightStyle(Prs3d_TypeOfHighlight(slot),styles[value]);break;
    case 2:ctx->SetHighlightStyle(styles[value]);break;case 3:ctx->SetSelectionStyle(styles[value]);break;
    case 4:ctx->SetToHilightSelected(value!=0);break;case 5:ctx->SetAutomaticHilight(value!=0);break;
    case 6:ctx->SetAutoActivateSelection(value!=0);break;case 7:ctx->SetPickingStrategy(SelectMgr_PickingStrategy(value));break;
    case 8:ctx->UpdateCurrentViewer();break;
  }
}
void v3_ais_context(std::istream& input) {
  GaReader r{input};occ::handle<V3d_Viewer> viewer=new V3d_Viewer(new SvDriver());
  occ::handle<AIS_InteractiveContext> ctx=new AIS_InteractiveContext(viewer);
  std::vector<occ::handle<Prs3d_Drawer>> styles={nullptr};
  for(int i=0;i<6;++i)styles.push_back(ctx->HighlightStyle(Prs3d_TypeOfHighlight(i)));
  styles.push_back(new Prs3d_Drawer());
  ga_flag(ctx->CurrentViewer()==viewer);ga_flag(!ctx->MainPrsMgr().IsNull());ga_flag(!ctx->SelectionManager().IsNull());
  ga_flag(ctx->MainSelector()==ctx->SelectionManager()->Selector());ga_flag(ctx->Selection()->IsEmpty());ga_int(ctx->MainSelector()->PixelTolerance());
  for(int i=1;i<=6;++i)v3_context_style(styles[i],styles[1]);
  v3_context_state(ctx,styles);
  for(int count=r.integer();count>0;--count){int op=r.integer(),slot=r.integer(),value=r.integer();v3_context_op(ctx,styles,op,slot,value);v3_context_state(ctx,styles);}
  ga_end();
}
