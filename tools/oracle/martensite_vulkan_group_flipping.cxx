// OpenGl_Flipper command pairs at OCCT 3d097a0, including text and cap filtering.
static void configure_flipping_view(const occ::handle<Graphic3d_Group>& group,
    const occ::handle<V3d_View>& view, const std::string& mode) {
  view->Camera()->SetUp(gp_Dir(0,mode=="flip-front" || mode=="flip-back"?1:-1,0));
  if(mode=="flip-back") view->Camera()->SetEye(gp_Pnt(0,0,-20));
  view->ChangeRenderingParams().FrustumCullingState=Graphic3d_RenderingParams::FrustumCulling_Off;
  if(mode=="flip-pers") group->SetTransformPersistence(new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp_Pnt(2,1,0)));
}
static void add_flipped_label(const occ::handle<Graphic3d_Group>& group) {
  occ::handle<Graphic3d_AspectText3d> style=new Graphic3d_AspectText3d(Quantity_Color(Quantity_NOC_WHITE),"DejaVu Sans",1,0,Aspect_TOST_NORMAL,Aspect_TODT_NORMAL);
  occ::handle<Graphic3d_Text> text=new Graphic3d_Text(16);
  text->SetText("AV"); text->SetOrientation(gp_Ax2(gp_Pnt(-5,0,0),gp_Dir(0,0,1),gp_Dir(1,0,0)));
  group->SetPrimitivesAspect(style); group->AddText(text);
}
static void add_flipped_cap(const occ::handle<Graphic3d_Group>& group, const occ::handle<V3d_View>& view) {
  cap_cube(group,0); gp_Trsf local; local.SetTranslation(gp_Vec(3,0,1)); group->SetTransformation(local);
  occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane;
  planes->Append(cap_plane(0,0,-1,0)); view->SetClipPlanes(planes);
}
static void add_flipped_geometry(const occ::handle<Graphic3d_Group>& group,
    const occ::handle<V3d_View>& view, const std::string& mode) {
  if(mode=="flip-text") { add_flipped_label(group); return; }
  if(mode=="flip-cap") { add_flipped_cap(group,view); return; }
  if(mode=="flip-lit") {
    view->ChangeRenderingParams().ShadingModel=Graphic3d_TypeOfShadingModel_Phong;
    group->Aspects()->SetShadingModel(Graphic3d_TypeOfShadingModel_PhongFacet);
    group->SynchronizeAspects();
    occ::handle<Graphic3d_CLight> light=new Graphic3d_CLight(Graphic3d_TypeOfLightSource_Directional);
    light->SetDirection(gp_Dir(0,0,-1)); view->SetLightOn(light);
  }
  group->AddPrimitiveArray(triangle_vertices());
}
static void group_flipping_scene(const occ::handle<Graphic3d_Structure>& structure,
    const occ::handle<V3d_View>& view, const std::string& mode) {
  auto group=structure->Groups().First(); group->Clear(false);
  group->SetGroupPrimitivesAspect(unlit_aspects()); configure_flipping_view(group,view,mode);
  const gp_Pnt origin=mode=="flip-offset" || mode=="flip-cap"?gp_Pnt(2,1,0):gp_Pnt(0,0,0);
  const gp_Ax2 plane(origin,gp_Dir(0,0,1),gp_Dir(1,0,0));
  group->SetFlippingOptions(true,plane);
  if(mode=="flip-nested") group->SetFlippingOptions(true,plane);
  add_flipped_geometry(group,view,mode);
  if(mode=="flip-nested") group->SetFlippingOptions(false,plane);
  group->SetFlippingOptions(false,plane); structure->CalculateBoundBox();
}
