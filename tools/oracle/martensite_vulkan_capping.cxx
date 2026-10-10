// Paired native closed-solid fixtures for retained_capping, pinned OCCT 3d097a0.
static void cap_cube(const occ::handle<Graphic3d_Group>& group, double offset) {
  occ::handle<Graphic3d_ArrayOfTriangles> array = new Graphic3d_ArrayOfTriangles(8,36);
  for (const auto& p : std::array<std::array<double,3>,8>{{{-6,-6,-6},{6,-6,-6},{6,6,-6},{-6,6,-6},
       {-6,-6,6},{6,-6,6},{6,6,6},{-6,6,6}}}) array->AddVertex(p[0]+offset,p[1],p[2]);
  for (int i : {1,3,2,1,4,3,5,6,7,5,7,8,1,2,6,1,6,5,4,8,7,4,7,3,1,5,8,1,8,4,2,3,7,2,7,6}) array->AddEdge(i);
  group->AddPrimitiveArray(array); group->SetClosed(true);
}
static occ::handle<Graphic3d_ClipPlane> cap_plane(double x,double y,double z,double w) {
  occ::handle<Graphic3d_ClipPlane> plane = new Graphic3d_ClipPlane(NCollection_Vec4<double>(x,y,z,w));
  plane->SetCapping(true); plane->CappingAspect()->SetShadingModel(Graphic3d_TypeOfShadingModel_Unlit);
  plane->CappingAspect()->SetPolygonOffsets(Aspect_POM_Off,0,0);
  plane->SetCappingColor(Quantity_Color(0,1,0,Quantity_TOC_RGB)); return plane;
}
static void capping_scene(const occ::handle<Graphic3d_Structure>& structure,
                          const occ::handle<V3d_View>& view,const std::string& mode) {
  auto group=structure->Groups().First(); group->Clear(false);
  group->SetGroupPrimitivesAspect(unlit_aspects()); cap_cube(group,0); group->SetClosed(mode!="cap-open");
  auto plane=mode=="cap-oblique"?cap_plane(.6,0,-.8,0):cap_plane(0,0,-1,0);
  if(mode=="cap-off") plane->SetCapping(false);
  if(mode=="cap-disabled") plane->SetOn(false);
  if(mode=="cap-moved") plane->SetEquation(NCollection_Vec4<double>(0,0,-1,-2));
  if(mode=="cap-outside") plane->SetEquation(NCollection_Vec4<double>(0,0,-1,8));
  if(mode=="cap-empty") plane->SetEquation(NCollection_Vec4<double>(0,0,-1,-8));
  if(mode=="cap-object"||mode=="cap-overlap-object") plane->SetUseObjectMaterial(true);
  occ::handle<Graphic3d_SequenceOfHClipPlane> planes=new Graphic3d_SequenceOfHClipPlane(); planes->Append(plane);
  if(mode=="cap-chain"||mode=="cap-chain-disabled-child") {
    auto child=cap_plane(1,0,0,0); if(mode=="cap-chain-disabled-child") child->SetOn(false); plane->SetChainNextPlane(child);
  }
  if(mode=="cap-intersection") planes->Append(cap_plane(1,0,0,0));
  if(mode=="cap-local") structure->SetClipPlanes(planes); else view->SetClipPlanes(planes);
  if(mode.rfind("cap-overlap",0)==0) {
    auto second=structure->NewGroup(); auto style=unlit_aspects();
    style->SetInteriorColor(Quantity_Color(1,1,0,Quantity_TOC_RGB)); second->SetGroupPrimitivesAspect(style); cap_cube(second,3);
  }
  if(mode=="cap-transformed") { gp_Trsf t; t.SetTranslation(gp_Vec(2,1,1)); group->SetTransformation(t); }
  if(mode=="cap-override") { occ::handle<Graphic3d_SequenceOfHClipPlane> local=new Graphic3d_SequenceOfHClipPlane();
    local->SetOverrideGlobal(true); structure->SetClipPlanes(local); }
  if(mode.find("pers")!=std::string::npos) {
    occ::handle<Graphic3d_TransformPers> pers=new Graphic3d_TransformPers(Graphic3d_TMF_ZoomPers,gp_Pnt(1,0,0));
    if(mode=="cap-group-pers") group->SetTransformPersistence(pers); else structure->SetTransformPersistence(pers);
    if(mode=="cap-pers-local") structure->SetClipPlanes(planes);
    view->ChangeRenderingParams().FrustumCullingState=Graphic3d_RenderingParams::FrustumCulling_Off;
  }
  if(mode=="cap-front") view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_FrontCulled);
  if(mode=="cap-back") view->SetBackFacingModel(Graphic3d_TypeOfBackfacingModel_BackCulled);
  if(mode=="cap-mirrored"||mode=="cap-mirror-uncut") {
    gp_Trsf t; t.SetScale(gp_Pnt(0,0,0),-1); structure->SetTransformation(new TopLoc_Datum3D(t));
  }
  if(mode=="cap-mirror-uncut") view->SetClipPlanes(occ::handle<Graphic3d_SequenceOfHClipPlane>());
  structure->CalculateBoundBox();
}

static occ::handle<Graphic3d_Structure> cap_shadow(const occ::handle<Graphic3d_Structure>& source,
                                                 const occ::handle<V3d_Viewer>& viewer) {
  occ::handle<Graphic3d_Structure> shadow=new Graphic3d_Structure(viewer->StructureManager(),source);
  auto group=shadow->NewGroup(); auto style=unlit_aspects();
  style->SetInteriorColor(Quantity_Color(1,1,0,Quantity_TOC_RGB));
  group->SetGroupPrimitivesAspect(style); cap_cube(group,3);
  source->Erase(); shadow->CalculateBoundBox(); shadow->Display(); return shadow;
}
