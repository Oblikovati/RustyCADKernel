// Camera persistence reference uses actual OpenGl_Structure and group state at
// OCCT 3d097a0.
#include <Graphic3d_TransformPers.hxx>
#include <Graphic3d_TransformPersScaledAbove.hxx>
static occ::handle<Graphic3d_TransformPers>
persistence_corner(Graphic3d_TransModeFlags flags) {
  return new Graphic3d_TransformPers(flags, Aspect_TOTP_LEFT_LOWER,
                                     NCollection_Vec2<int>(18, 18));
}
static occ::handle<Graphic3d_TransformPers>
persistence_mode(const std::string &mode) {
  Graphic3d_TransModeFlags flags = Graphic3d_TMF_ZoomPers;
  if (mode == "pers-rotate" || mode == "pers-group-rotate")
    flags = Graphic3d_TMF_RotatePers;
  else if (mode == "pers-zoom-rotate")
    flags = Graphic3d_TMF_ZoomRotatePers;
  else if (mode == "pers-axial")
    flags = Graphic3d_TMF_AxialScalePers;
  else if (mode == "pers-zoom-axial")
    flags = Graphic3d_TMF_AxialZoomPers;
  else if (mode == "pers-ortho")
    flags = Graphic3d_TransModeFlags(Graphic3d_TMF_ZoomPers |
                                     Graphic3d_TMF_OrthoPers);
  else if (mode == "pers-2d" || mode == "pers-group-2d")
    return persistence_corner(Graphic3d_TMF_2d);
  else if (mode == "pers-triedron")
    return persistence_corner(Graphic3d_TMF_TriedronPers);
  else if (mode == "pers-2d-ortho")
    return persistence_corner(
        Graphic3d_TransModeFlags(Graphic3d_TMF_2d | Graphic3d_TMF_OrthoPers));
  else if (mode == "pers-triedron-ortho")
    return persistence_corner(Graphic3d_TransModeFlags(
        Graphic3d_TMF_TriedronPers | Graphic3d_TMF_OrthoPers));
  else if (mode == "pers-scaled-above")
    return new Graphic3d_TransformPersScaledAbove(0.5, gp_Pnt(1, -1, 0));
  else if (mode.find("camera") != std::string::npos)
    return new Graphic3d_TransformPers(Graphic3d_TMF_CameraPers);
  return new Graphic3d_TransformPers(flags, gp_Pnt(1, -1, 0));
}
static void persistence_scene(const occ::handle<Graphic3d_Structure> &structure,
                              const occ::handle<V3d_View> &view,
                              const std::string &mode) {
  view->Camera()->SetEyeAndCenter(gp_Pnt(8, 6, 20), gp_Pnt(0, 0, 0));
  view->Camera()->SetAxialScale(gp_XYZ(1.5, 0.75, 1.25));
  structure->Clear();
  auto group = structure->NewGroup();
  group->SetGroupPrimitivesAspect(unlit_aspects());
  auto vertices = triangle_vertices();
  if (mode.find("camera") != std::string::npos) {
    const double z = mode.find("near") != std::string::npos  ? -0.5
                     : mode.find("far") != std::string::npos ? -150.0
                                                             : -20.0;
    vertices->SetVertice(1, -8, -8, z);
    vertices->SetVertice(2, 8, -8, z);
    vertices->SetVertice(3, 0, 8, z);
  }
  group->AddPrimitiveArray(vertices);
  auto persistence = persistence_mode(mode);
  if (mode.rfind("pers-group-", 0) == 0 || mode == "pers-nested") {
    group->SetTransformPersistence(persistence);
    gp_Trsf transform;
    transform.SetTranslation(gp_Vec(1, 2, 0));
    group->SetTransformation(transform);
  } else {
    structure->SetTransformPersistence(persistence);
  }
  if (mode == "pers-nested")
    structure->SetTransformPersistence(persistence_corner(Graphic3d_TMF_2d));
}
