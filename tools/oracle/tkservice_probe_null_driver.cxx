// The Graphic3d null driver of the native probes: SvDriver, SvCStructure, SvGroup and SvView
// mirror rustycad_tkservice::null_driver; they keep the scene and draw nothing. Included by
// tkservice_probe_driver.cxx and by the TKV3d probe, whose viewers run on it.
#include <Aspect_NeutralWindow.hxx>
#include <BVH_LinearBuilder.hxx>
#include <Graphic3d_ArrayOfPoints.hxx>
#include <Graphic3d_CStructure.hxx>
#include <Graphic3d_CView.hxx>
#include <Graphic3d_CullingTool.hxx>
#include <Graphic3d_GraphicDriver.hxx>
#include <Graphic3d_Group.hxx>
#include <Graphic3d_Layer.hxx>
#include <Graphic3d_Structure.hxx>
#include <Graphic3d_StructureManager.hxx>
#include <Graphic3d_Text.hxx>
#include <Graphic3d_TransformPers.hxx>
#include <TopLoc_Datum3D.hxx>
class SvGroup : public Graphic3d_Group
{
public:
  SvGroup(const occ::handle<Graphic3d_Structure>& theStruct) : Graphic3d_Group(theStruct) {}
  occ::handle<Graphic3d_Aspects> Aspects() const override { return myAspects; }
  void SetGroupPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& theAspect) override { myAspects = theAspect; }
  void SetPrimitivesAspect(const occ::handle<Graphic3d_Aspects>& theAspect) override { myAspects = theAspect; }
  void SynchronizeAspects() override {}
  void ReplaceAspects(const NCollection_DataMap<occ::handle<Graphic3d_Aspects>, occ::handle<Graphic3d_Aspects>>&) override {}
  void SetStencilTestOptions(const bool) override {}
  void SetFlippingOptions(const bool, const gp_Ax2&) override {}
private:
  occ::handle<Graphic3d_Aspects> myAspects;
};
class SvCStructure : public Graphic3d_CStructure
{
public:
  SvCStructure(const occ::handle<Graphic3d_StructureManager>& theMgr) : Graphic3d_CStructure(theMgr) {}
  void OnVisibilityChanged() override {}
  void Clear() override { myGroups.Clear(); Is2dText = false; IsForHighlight = false; }
  void Connect(Graphic3d_CStructure&) override {}
  void Disconnect(Graphic3d_CStructure&) override {}
  void GraphicHighlight(const occ::handle<Graphic3d_PresentationAttributes>& theStyle) override { myHighlightStyle = theStyle; highlight = 1; }
  void GraphicUnhighlight() override { highlight = 0; myHighlightStyle.Nullify(); }
  occ::handle<SvCStructure> myShadowParent;
  occ::handle<Graphic3d_CStructure> ShadowLink(const occ::handle<Graphic3d_StructureManager>& theMgr) const override
  {
    occ::handle<SvCStructure> aShadow = new SvCStructure(theMgr);
    // OpenGl_StructureShadow retains its root parent without adopting removable group nodes.
    aShadow->myShadowParent = myShadowParent.IsNull() ? this : myShadowParent;
    aShadow->IsInfinite = aShadow->myShadowParent->IsInfinite;
    aShadow->ChangeBoundingBox() = aShadow->myShadowParent->BoundingBox();
    aShadow->SetTransformation(aShadow->myShadowParent->Transformation());
    aShadow->SetTransformPersistence(aShadow->myShadowParent->TransformPersistence());
    return aShadow;
  }
  occ::handle<Graphic3d_Group> NewGroup(const occ::handle<Graphic3d_Structure>& theStruct) override
  { occ::handle<Graphic3d_Group> aGroup = new SvGroup(theStruct); myGroups.Append(aGroup); return aGroup; }
  void RemoveGroup(const occ::handle<Graphic3d_Group>& theGroup) override
  {
    if (theGroup.IsNull()) return;
    for (NCollection_Sequence<occ::handle<Graphic3d_Group>>::Iterator anIt(myGroups); anIt.More(); anIt.Next())
      if (anIt.Value() == theGroup) { theGroup->Clear(false); myGroups.Remove(anIt); return; }
  }
};
class SvView : public Graphic3d_CView
{
public:
  SvView(const occ::handle<Graphic3d_StructureManager>& theMgr, const NCollection_List<occ::handle<Graphic3d_Layer>>& theLayers) : Graphic3d_CView(theMgr)
  {
    // The culling builder of OpenGl_LayerList.
    occ::handle<BVH_Builder3d> aBuilder = new BVH_LinearBuilder<double, 3>(BVH_Constants_LeafNodeSizeSingle, BVH_Constants_MaxTreeDepth);
    for (NCollection_List<occ::handle<Graphic3d_Layer>>::Iterator anIt(theLayers); anIt.More(); anIt.Next())
    { occ::handle<Graphic3d_Layer> aCopy = new Graphic3d_Layer(anIt.Value()->LayerId(), aBuilder); aCopy->SetLayerSettings(anIt.Value()->LayerSettings()); myLayers.Append(aCopy); }
  }
  void SetSvWindow(const occ::handle<Aspect_Window>& theWindow) { myWindow = theWindow; }
  void Redraw() override {}
  void RedrawImmediate() override {}
  void Invalidate() override {}
  bool IsInvalidated() override { return false; }
  void Resized() override { Graphic3d_CView::Resized(); }
  bool SetImmediateModeDrawToFront(const bool) override { return false; }
  void SetWindow(const occ::handle<Graphic3d_CView>&, const occ::handle<Aspect_Window>& theWindow, const Aspect_RenderingContext) override { myWindow = theWindow; }
  occ::handle<Aspect_Window> Window() const override { return myWindow; }
  bool IsDefined() const override { return !myWindow.IsNull(); }
  bool BufferDump(Image_PixMap&, const Graphic3d_BufferType&) override { return false; }
  bool ShadowMapDump(Image_PixMap&, const TCollection_AsciiString&) override { return false; }
  void InvalidateBVHData(const Graphic3d_ZLayerId theLayerId) override { occ::handle<Graphic3d_Layer> aLayer = Layer(theLayerId); if (!aLayer.IsNull()) aLayer->InvalidateBVHData(); }
  void InsertLayerBefore(const Graphic3d_ZLayerId, const Graphic3d_ZLayerSettings&, const Graphic3d_ZLayerId) override {}
  void InsertLayerAfter(const Graphic3d_ZLayerId, const Graphic3d_ZLayerSettings&, const Graphic3d_ZLayerId) override {}
  int ZLayerMax() const override { return 0; }
  const NCollection_List<occ::handle<Graphic3d_Layer>>& Layers() const override { return myLayers; }
  occ::handle<Graphic3d_Layer> Layer(const Graphic3d_ZLayerId theLayerId) const override
  { for (NCollection_List<occ::handle<Graphic3d_Layer>>::Iterator anIt(myLayers); anIt.More(); anIt.Next()) if (anIt.Value()->LayerId() == theLayerId) return anIt.Value(); return occ::handle<Graphic3d_Layer>(); }
  void RemoveZLayer(const Graphic3d_ZLayerId) override {}
  void SetZLayerSettings(const Graphic3d_ZLayerId, const Graphic3d_ZLayerSettings&) override {}
  occ::handle<Standard_Transient> FBO() const override { return occ::handle<Standard_Transient>(); }
  void SetFBO(const occ::handle<Standard_Transient>&) override {}
  occ::handle<Standard_Transient> FBOCreate(const int, const int) override { return occ::handle<Standard_Transient>(); }
  void FBORelease(occ::handle<Standard_Transient>&) override {}
  void FBOGetDimensions(const occ::handle<Standard_Transient>&, int&, int&, int&, int&) override {}
  void FBOChangeViewport(const occ::handle<Standard_Transient>&, const int, const int) override {}
  Aspect_GradientBackground GradientBackground() const override { return Aspect_GradientBackground(); }
  void SetGradientBackground(const Aspect_GradientBackground&) override {}
  void SetBackgroundImage(const occ::handle<Graphic3d_TextureMap>&, bool) override {}
  Aspect_FillMethod BackgroundImageStyle() const override { return Aspect_FM_NONE; }
  void SetBackgroundImageStyle(const Aspect_FillMethod) override {}
  void SetImageBasedLighting(bool) override {}
  void SetTextureEnv(const occ::handle<Graphic3d_TextureEnv>&) override {}
  const occ::handle<Graphic3d_LightSet>& Lights() const override { return myLights; }
  void SetLights(const occ::handle<Graphic3d_LightSet>&) override {}
  const occ::handle<Graphic3d_SequenceOfHClipPlane>& ClipPlanes() const override { return myPlanes; }
  void SetClipPlanes(const occ::handle<Graphic3d_SequenceOfHClipPlane>& thePlanes) override { myPlanes = thePlanes; }
  void DiagnosticInformation(NCollection_IndexedDataMap<TCollection_AsciiString, TCollection_AsciiString>& theDict, Graphic3d_DiagnosticInfo theFlags) const override
  { Graphic3d_CView::DiagnosticInformation(theDict, theFlags); }
  TCollection_AsciiString StatisticInformation() const override { return TCollection_AsciiString(); }
  void StatisticInformation(NCollection_IndexedDataMap<TCollection_AsciiString, TCollection_AsciiString>&) const override {}
private:
  occ::handle<Graphic3d_Layer> layerOf(Graphic3d_ZLayerId theId) const { occ::handle<Graphic3d_Layer> aLayer = Layer(theId); return aLayer.IsNull() ? Layer(Graphic3d_ZLayerId_Default) : aLayer; }
  Graphic3d_DisplayPriority removeFromLayers(const occ::handle<Graphic3d_CStructure>& theStruct, Graphic3d_ZLayerId theId, bool theForChange)
  {
    occ::handle<Graphic3d_Layer> anOwn = layerOf(theId);
    Graphic3d_DisplayPriority aPriority = Graphic3d_DisplayPriority_INVALID;
    if (!anOwn.IsNull()) anOwn->Remove(theStruct.get(), aPriority, theForChange);
    if (aPriority != Graphic3d_DisplayPriority_INVALID) return aPriority;
    for (NCollection_List<occ::handle<Graphic3d_Layer>>::Iterator anIt(myLayers); anIt.More(); anIt.Next())
    { if (anIt.Value() == anOwn) continue; anIt.Value()->Remove(theStruct.get(), aPriority, theForChange); if (aPriority != Graphic3d_DisplayPriority_INVALID) break; }
    return aPriority;
  }
  void displayStructure(const occ::handle<Graphic3d_CStructure>& theStruct, const Graphic3d_DisplayPriority thePriority) override
  { occ::handle<Graphic3d_Layer> aLayer = layerOf(theStruct->ZLayer()); if (!aLayer.IsNull()) aLayer->Add(theStruct.get(), thePriority, false); }
  void eraseStructure(const occ::handle<Graphic3d_CStructure>& theStruct) override { removeFromLayers(theStruct, theStruct->ZLayer(), false); }
  void changeZLayer(const occ::handle<Graphic3d_CStructure>& theStruct, const Graphic3d_ZLayerId theNewLayerId) override
  {
    Graphic3d_DisplayPriority aPriority = removeFromLayers(theStruct, theStruct->ZLayer(), false);
    occ::handle<Graphic3d_Layer> aLayer = layerOf(theNewLayerId); if (!aLayer.IsNull()) aLayer->Add(theStruct.get(), aPriority, false);
  }
  void changePriority(const occ::handle<Graphic3d_CStructure>& theStruct, const Graphic3d_DisplayPriority thePriority) override
  {
    if (removeFromLayers(theStruct, theStruct->ZLayer(), true) == Graphic3d_DisplayPriority_INVALID) return;
    occ::handle<Graphic3d_Layer> aLayer = layerOf(theStruct->ZLayer()); if (!aLayer.IsNull()) aLayer->Add(theStruct.get(), thePriority, true);
  }
  NCollection_List<occ::handle<Graphic3d_Layer>> myLayers;
  occ::handle<Aspect_Window> myWindow;
  occ::handle<Graphic3d_LightSet> myLights;
  occ::handle<Graphic3d_SequenceOfHClipPlane> myPlanes;
};
class SvDriver : public Graphic3d_GraphicDriver
{
public:
  SvDriver() : Graphic3d_GraphicDriver(occ::handle<Aspect_DisplayConnection>()) {}
  int InquireLimit(const Graphic3d_TypeOfLimit theType) const override
  { return theType == Graphic3d_TypeOfLimit_MaxNbLights || theType == Graphic3d_TypeOfLimit_MaxNbClipPlanes ? 8 : (theType == Graphic3d_TypeOfLimit_MaxNbViews ? 32 : 0); }
  occ::handle<Graphic3d_CStructure> CreateStructure(const occ::handle<Graphic3d_StructureManager>& theMgr) override { return new SvCStructure(theMgr); }
  void RemoveStructure(occ::handle<Graphic3d_CStructure>& theStruct) override { theStruct.Nullify(); }
  occ::handle<Graphic3d_CView> CreateView(const occ::handle<Graphic3d_StructureManager>& theMgr) override { return new SvView(theMgr, myLayers); }
  void RemoveView(const occ::handle<Graphic3d_CView>&) override {}
  void EnableVBO(const bool) override {}
  bool IsVerticalSync() const override { return true; }
  void SetVerticalSync(bool) override {}
  bool MemoryInfo(size_t& theFree, TCollection_AsciiString& theInfo) const override { theFree = 0; theInfo.Clear(); return false; }
  float DefaultTextHeight() const override { return 16.0f; }
  void TextSize(const occ::handle<Graphic3d_CView>&, const char* const, const float, float& theW, float& theA, float& theD) const override { theW = theA = theD = 0.0f; }
  void InsertLayerBefore(const Graphic3d_ZLayerId theId, const Graphic3d_ZLayerSettings& theSettings, const Graphic3d_ZLayerId theAfter) override { Graphic3d_GraphicDriver::InsertLayerBefore(theId, theSettings, theAfter); }
  void InsertLayerAfter(const Graphic3d_ZLayerId theId, const Graphic3d_ZLayerSettings& theSettings, const Graphic3d_ZLayerId theBefore) override { Graphic3d_GraphicDriver::InsertLayerAfter(theId, theSettings, theBefore); }
  void RemoveZLayer(const Graphic3d_ZLayerId theId) override { Graphic3d_GraphicDriver::RemoveZLayer(theId); }
  void SetZLayerSettings(const Graphic3d_ZLayerId theId, const Graphic3d_ZLayerSettings& theSettings) override { Graphic3d_GraphicDriver::SetZLayerSettings(theId, theSettings); }
  bool ViewExists(const occ::handle<Aspect_Window>&, occ::handle<Graphic3d_CView>&) override { return false; }
};
