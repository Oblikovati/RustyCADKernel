// Actual pinned AIS_RubberBand overlays match Martensite's retained scenes.
#include <AIS_RubberBand.hxx>
struct MartensiteOracleRubberBandScene {
  occ::handle<AIS_InteractiveContext> context;
  occ::handle<AIS_RubberBand> object;
  MartensiteOracleRubberBandScene(const occ::handle<V3d_Viewer>& viewer) {
    context=new AIS_InteractiveContext(viewer);
    object=new AIS_RubberBand();object->SetRectangle(8,8,36,36);
    context->Display(object,false);
  }
  void transition(const std::string& mode,double offset) {
    gp_Trsf transform;transform.SetTranslation(gp_Vec(offset,0,0));object->SetLocalTransformation(transform);
    if(mode=="rubber-open")object->SetPolygonClosed(false);
    if(mode=="rubber-filled"||mode=="rubber-concave")object->SetFilling(Quantity_NOC_RED,0.);
    if(mode=="rubber-transparent")object->SetFilling(Quantity_NOC_RED,0.5);
    if(mode=="rubber-dash")object->SetLineType(Aspect_TOL_DASH);
    if(mode=="rubber-wide")object->SetLineWidth(3.);
    if(mode=="rubber-updated")object->SetRectangle(12,10,40,34);
    if(mode=="rubber-concave") {
      object->ClearPoints();
      for(const auto& point: {NCollection_Vec2<int>(8,8),NCollection_Vec2<int>(8,36),NCollection_Vec2<int>(20,20),NCollection_Vec2<int>(36,36),NCollection_Vec2<int>(36,8)})object->AddPoint(point);
    }
    context->Redisplay(object,false);
  }
};
