// The scene script of sv_scene (included by tkservice_probe_driver.cxx); the Rust twin is
// crates/tkservice/examples/sv_probe/driver.rs.
// Prints a Bnd_Box: void and whole flags, then the limits of a non-void box.
void sv_bnd(const Bnd_Box& theBox)
{
  ga_flag(theBox.IsVoid()); ga_flag(theBox.IsWhole());
  if (theBox.IsVoid()) return;
  double aB[6]; theBox.Get(aB[0], aB[1], aB[2], aB[3], aB[4], aB[5]);
  for (double aV : aB) ga_real(aV);
}
// Prints a Graphic3d_BndBox3d: valid flag and corners of a valid box.
void sv_bvh_box(const Graphic3d_BndBox3d& theBox)
{
  ga_flag(theBox.IsValid());
  if (!theBox.IsValid()) return;
  for (int i = 0; i < 3; ++i) ga_real(theBox.CornerMin()[i]);
  for (int i = 0; i < 3; ++i) ga_real(theBox.CornerMax()[i]);
}
// Replaces every printed address ("0x...") by "?", then masks the uninitialised render
// parameter and camera tile values (as sv_params and sv_camera do).
std::string sv_mask_dump(std::string text)
{
  const std::string head = "\": \"0x";
  for (size_t at = text.find(head); at != std::string::npos; at = text.find(head, at + 1))
  { const size_t start = at + 4; const size_t end = text.find('"', start); text.replace(start, end - start, "?"); }
  // The camera tile dumps its integer vectors through the double macro, and SetTile compares
  // with the uninitialised tile of a new camera (an extra projection state count).
  return sv_masked(sv_masked(sv_masked(sv_masked(text, "ChartSize"), "Params.Params3d"), "Tile"), "WorldViewProjState");
}
template<class T> void sv_dump_masked(const T& theObject)
{ std::ostringstream anOut; theObject.DumpJson(anOut, -1); ga_text(sv_mask_dump(anOut.str())); }
struct SvScene
{
  occ::handle<SvDriver> Driver;
  occ::handle<Graphic3d_StructureManager> Manager;
  occ::handle<SvView> View;
  std::vector<occ::handle<Graphic3d_Structure>> Structs;
  int ViewW = 0, ViewH = 0;
};
occ::handle<Graphic3d_Structure> sv_struct(SvScene& theScene, GaReader& r)
{
  const int anIndex = r.integer();
  return anIndex >= 0 && anIndex < (int)theScene.Structs.size() ? theScene.Structs[anIndex] : occ::handle<Graphic3d_Structure>();
}
occ::handle<Graphic3d_TransformPers> sv_pers(GaReader& r)
{
  const int aMode = r.integer(); const gp_Pnt aPnt = r.pnt();
  if (aMode == 0) return occ::handle<Graphic3d_TransformPers>();
  if (aMode == Graphic3d_TMF_2d || aMode == Graphic3d_TMF_TriedronPers)
    return new Graphic3d_TransformPers((Graphic3d_TransModeFlags)aMode, Aspect_TOTP_LEFT_LOWER, NCollection_Vec2<int>((int)aPnt.X(), (int)aPnt.Y()));
  return new Graphic3d_TransformPers((Graphic3d_TransModeFlags)aMode, aPnt);
}
// Updates the culling of every view layer with a culling tool of the view camera.
void sv_cull(SvScene& theScene, int theState)
{
  Graphic3d_CullingTool aTool; aTool.SetViewVolume(theScene.View->Camera());
  aTool.SetViewportSize(theScene.ViewW, theScene.ViewH, 1.0); aTool.CacheClipPtsProjections();
  for (NCollection_List<occ::handle<Graphic3d_Layer>>::Iterator anIt(theScene.View->Layers()); anIt.More(); anIt.Next())
    anIt.Value()->UpdateCulling(theScene.View->Identification(), aTool, (Graphic3d_RenderingParams::FrustumCulling)theState);
}
// Runs one operation of the script.
void sv_scene_op(SvScene& theScene, GaReader& r)
{
  const int anOp = r.integer();
  if (anOp == 1)
  {
    occ::handle<Graphic3d_Structure> aStruct = new Graphic3d_Structure(theScene.Manager);
    const gp_Pnt aMin = r.pnt(), aMax = r.pnt();
    aStruct->NewGroup()->SetMinMaxValues(aMin.X(), aMin.Y(), aMin.Z(), aMax.X(), aMax.Y(), aMax.Z());
    theScene.Structs.push_back(aStruct);
    return;
  }
  if (anOp == 19) { sv_cull(theScene, r.integer()); return; }
  if (anOp == 24) { const bool aMode = r.flag(); theScene.View->SetComputedMode(aMode); return; }
  if (anOp == 25) { if (r.flag()) theScene.View->Activate(); else theScene.View->Deactivate(); return; }
  occ::handle<Graphic3d_Structure> aS = sv_struct(theScene, r);
  switch (anOp)
  {
    case 2: aS->Display(); break;
    case 3: aS->Erase(); break;
    case 4: { const int aP = r.integer(); GA_TRY(aS->SetDisplayPriority((Graphic3d_DisplayPriority)aP)); break; }
    case 5: aS->SetZLayer(r.integer()); break;
    case 6: aS->SetInfiniteState(r.flag()); break;
    case 7: { occ::handle<Graphic3d_Structure> aT = sv_struct(theScene, r); const int aType = r.integer(); const bool aCheck = r.flag(); aS->Connect(aT.get(), (Graphic3d_TypeOfConnection)aType, aCheck); break; }
    case 8: { occ::handle<Graphic3d_Structure> aT = sv_struct(theScene, r); aS->Disconnect(aT.get()); break; }
    case 9: aS->SetVisible(r.flag()); break;
    case 10: aS->Highlight(occ::handle<Graphic3d_PresentationAttributes>()); break;
    case 11: aS->UnHighlight(); break;
    case 12: aS->SetTransformPersistence(sv_pers(r)); break;
    case 13: { gp_Trsf aT; aT.SetTranslation(gp_Vec(r.xyz())); aS->SetTransformation(new TopLoc_Datum3D(aT)); break; }
    case 14: aS->Remove(); break;
    case 15: aS->SetMutable(r.flag()); break;
    case 17: { occ::handle<Graphic3d_Text> aText = new Graphic3d_Text(10.0f); const gp_Pnt aP = r.pnt(); aText->SetPosition(aP);
               if (r.flag()) aText->SetOrientation(gp_Ax2(aP, gp::DZ())); aS->CurrentGroup()->AddText(aText); break; }
    case 18: { const int aNb = r.integer(); occ::handle<Graphic3d_ArrayOfPoints> anArr = new Graphic3d_ArrayOfPoints(aNb);
               for (int i = 0; i < aNb; ++i) { const gp_Pnt aP = r.pnt(); anArr->AddVertex(aP.X(), aP.Y(), aP.Z()); }
               aS->NewGroup()->AddPrimitiveArray(anArr); break; }
    case 20: aS->SetIsForHighlight(r.flag()); break;
    case 21: aS->Clear(r.flag()); break;
    case 22: { const int aG = r.integer(); if (aG < 1 || aG > aS->NumberOfGroups()) break;
               // A held handle: the group must outlive its own Remove().
               occ::handle<Graphic3d_Group> aGroup = aS->Groups().Value(aG); aGroup->Remove(); break; }
    case 23: aS->SetVisual((Graphic3d_TypeOfStructure)r.integer()); break;
  }
}
// Prints a structure: removed flag, then its state, boxes and descendants.
void sv_scene_struct(SvScene& theScene, const occ::handle<Graphic3d_Structure>& theS)
{
  ga_flag(theS->IsDeleted());
  if (theS->IsDeleted()) return;
  ga_int(theS->Identification()); ga_flag(theS->IsDisplayed()); ga_flag(theS->IsVisible()); ga_flag(theS->IsHighlighted());
  ga_int(theS->DisplayPriority()); ga_int(theS->GetZLayer()); ga_flag(theS->IsEmpty()); ga_flag(theS->IsInfinite());
  ga_flag(theS->IsTransformed()); ga_int(theS->NumberOfGroups()); ga_flag(theS->CStructure()->IsCulled());
  ga_flag(theS->CStructure()->Is2dText); ga_flag(theS->CStructure()->IsAlwaysRendered()); NCollection_Map<occ::handle<Graphic3d_Structure>> aShown; theScene.View->DisplayedStructures(aShown); ga_flag(aShown.Contains(theS));
  sv_bvh_box(theS->CStructure()->BoundingBox()); sv_bnd(theS->MinMaxValues(false)); sv_bnd(theS->MinMaxValues(true));
  for (NCollection_Sequence<occ::handle<Graphic3d_Group>>::Iterator anIt(theS->Groups()); anIt.More(); anIt.Next())
  { double aB[6]; anIt.Value()->MinMaxValues(aB[0], aB[1], aB[2], aB[3], aB[4], aB[5]); for (double aV : aB) ga_real(aV); ga_flag(anIt.Value()->IsEmpty()); }
  NCollection_Map<occ::handle<Graphic3d_Structure>> aSet; theS->Descendants(aSet); ga_int(aSet.Extent());
}
// Prints a layer: id, counts, the structure ids of each priority and the boxes.
void sv_scene_layer(SvScene& theScene, const occ::handle<Graphic3d_Layer>& theLayer)
{
  ga_int(theLayer->LayerId()); ga_int(theLayer->NbStructures()); ga_int(theLayer->NbStructuresNotCulled());
  ga_int(theLayer->NbOfTransformPersistenceObjects()); ga_flag(theLayer->IsImmediate());
  for (int aP = 0; aP < Graphic3d_DisplayPriority_NB; ++aP)
  {
    const NCollection_IndexedMap<const Graphic3d_CStructure*>& aMap = theLayer->Structures((Graphic3d_DisplayPriority)aP);
    ga_int(aMap.Extent());
    for (int i = 1; i <= aMap.Extent(); ++i) ga_int(aMap(i)->Identification());
  }
  for (int anAux = 0; anAux < 2; ++anAux)
    GA_TRY(sv_bnd(theLayer->BoundingBox(theScene.View->Identification(), theScene.View->Camera(), theScene.ViewW, theScene.ViewH, anAux == 1)));
  GA_TRY(ga_real(theLayer->considerZoomPersistenceObjects(theScene.View->Identification(), theScene.View->Camera(), theScene.ViewW, theScene.ViewH)));
}
// `camera W H N op...`: a scene on a null driver, a view of W x H on the camera, N script
// operations, then the structures, the view bounds and the layers.
void sv_scene_case(std::istream& input)
{
  GaReader r{input}; SvScene aScene;
  occ::handle<Graphic3d_Camera> aCam = sv_camera_read(r);
  aScene.ViewW = r.integer(); aScene.ViewH = r.integer();
  aScene.Driver = new SvDriver(); aScene.Manager = new Graphic3d_StructureManager(aScene.Driver);
  aScene.View = occ::down_cast<SvView>(aScene.Driver->CreateView(aScene.Manager));
  occ::handle<Aspect_NeutralWindow> aWin = new Aspect_NeutralWindow(); aWin->SetSize(aScene.ViewW, aScene.ViewH);
  aScene.View->SetSvWindow(aWin); aScene.View->SetCamera(aCam); aScene.View->Activate();
  const int aNbOps = r.integer();
  for (int i = 0; i < aNbOps; ++i) sv_scene_op(aScene, r);
  ga_int((int)aScene.Structs.size());
  for (const occ::handle<Graphic3d_Structure>& aS : aScene.Structs) sv_scene_struct(aScene, aS);
  GA_TRY(sv_bnd(aScene.View->MinMaxValues(false))); GA_TRY(sv_bnd(aScene.View->MinMaxValues(true)));
  GA_TRY(ga_real(aScene.View->ConsiderZoomPersistenceObjects()));
  ga_int(aScene.View->NumberOfDisplayedStructures());
  for (NCollection_List<occ::handle<Graphic3d_Layer>>::Iterator anIt(aScene.View->Layers()); anIt.More(); anIt.Next()) sv_scene_layer(aScene, anIt.Value());
  for (const occ::handle<Graphic3d_Structure>& aS : aScene.Structs) if (!aS->IsDeleted()) sv_dump_masked(*aS);
  sv_dump_masked(*aScene.Driver); sv_dump_masked(*aScene.View);
  // The manager dumps its pointer-hashed maps: only with one displayed structure at most.
  NCollection_Map<occ::handle<Graphic3d_Structure>> aShown; aScene.Manager->DisplayedStructures(aShown);
  NCollection_Map<occ::handle<Graphic3d_Structure>> aLit; aScene.Manager->HighlightedStructures(aLit);
  if (aShown.Extent() <= 1 && aLit.Extent() <= 1) sv_dump_masked(*aScene.Manager);
  ga_end();
}
