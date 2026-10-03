#include <BRepOffset.hxx>
#include <BRepOffset_Status.hxx>
#include <Geom_BezierSurface.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_OffsetSurface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Geom_SphericalSurface.hxx>
#include <Geom_ToroidalSurface.hxx>
#include <NCollection_Array2.hxx>
#include <Precision.hxx>
#include <Standard_Failure.hxx>

#include <cmath>
#include <iomanip>
#include <iostream>
#include <string>
#include <vector>

namespace
{
constexpr double SampleU = 0.3;
constexpr double SampleV = 0.4;

const char* statusName(const BRepOffset_Status status)
{
  switch (status)
  {
    case BRepOffset_Good: return "Good";
    case BRepOffset_Reversed: return "Reversed";
    case BRepOffset_Degenerated: return "Degenerated";
    case BRepOffset_Unknown: return "Unknown";
  }
  return "InvalidStatus";
}

gp_Ax3 frame(const bool direct)
{
  gp_Ax3 axis(gp::Origin(), gp::DZ(), gp::DX());
  if (!direct)
  {
    axis.YReverse();
  }
  return axis;
}

occ::handle<Geom_Surface> makeBezier()
{
  NCollection_Array2<gp_Pnt> poles(1, 2, 1, 2);
  poles(1, 1) = gp_Pnt(0.0, 0.0, 0.0);
  poles(2, 1) = gp_Pnt(1.0, 0.0, 0.0);
  poles(1, 2) = gp_Pnt(0.0, 1.0, 0.0);
  poles(2, 2) = gp_Pnt(1.0, 1.0, 0.0);
  return new Geom_BezierSurface(poles);
}

occ::handle<Geom_Surface> makeSurface(const std::string& kind, const bool direct)
{
  const gp_Ax3 axis = frame(direct);
  if (kind == "plane") return new Geom_Plane(axis);
  if (kind == "cylinder") return new Geom_CylindricalSurface(axis, 2.0);
  if (kind == "cone") return new Geom_ConicalSurface(axis, 0.4, 2.0);
  if (kind == "sphere") return new Geom_SphericalSurface(axis, 2.0);
  if (kind == "torus") return new Geom_ToroidalSurface(axis, 5.0, 2.0);
  if (kind == "spindle") return new Geom_ToroidalSurface(axis, 2.0, 3.0);
  if (kind == "trimmed")
  {
    occ::handle<Geom_Surface> cylinder = new Geom_CylindricalSurface(axis, 2.0);
    return new Geom_RectangularTrimmedSurface(cylinder, 0.2, 1.0, -1.0, 2.0);
  }
  return makeBezier();
}

void writeVector(const gp_XYZ& value)
{
  std::cout << value.X() << ',' << value.Y() << ',' << value.Z();
}

void writeSample(const occ::handle<Geom_Surface>& surface)
{
  gp_Pnt point;
  try
  {
    surface->D0(SampleU, SampleV, point);
    std::cout << " point=";
    writeVector(point.XYZ());
  }
  catch (const Standard_Failure&)
  {
    std::cout << " point_error=Standard_Failure";
  }
}

void writeCylinderParameters(const occ::handle<Geom_Surface>& surface)
{
  const auto cylinder = occ::down_cast<Geom_CylindricalSurface>(surface);
  std::cout << " radius=" << cylinder->Radius() << " direct="
            << cylinder->Position().Direct();
}

void writeConeParameters(const occ::handle<Geom_Surface>& surface)
{
  const auto cone = occ::down_cast<Geom_ConicalSurface>(surface);
  std::cout << " radius=" << cone->RefRadius() << " angle=" << cone->SemiAngle()
            << " direct=" << cone->Position().Direct();
}

void writeSphereParameters(const occ::handle<Geom_Surface>& surface)
{
  const auto sphere = occ::down_cast<Geom_SphericalSurface>(surface);
  std::cout << " radius=" << sphere->Radius() << " direct="
            << sphere->Position().Direct();
}

void writeTorusParameters(const occ::handle<Geom_Surface>& surface)
{
  const auto torus = occ::down_cast<Geom_ToroidalSurface>(surface);
  std::cout << " major=" << torus->MajorRadius() << " minor=" << torus->MinorRadius()
            << " direct=" << torus->Position().Direct();
}

void writeFallbackParameters(const occ::handle<Geom_Surface>& surface)
{
  if (surface->IsKind(STANDARD_TYPE(Geom_OffsetSurface)))
  {
    const auto offsetSurface = occ::down_cast<Geom_OffsetSurface>(surface);
    std::cout << " offset=" << offsetSurface->Offset() << " basis="
              << offsetSurface->BasisSurface()->DynamicType()->Name();
  }
  if (surface->IsKind(STANDARD_TYPE(Geom_RectangularTrimmedSurface)))
  {
    const auto trimmed = occ::down_cast<Geom_RectangularTrimmedSurface>(surface);
    std::cout << " basis=" << trimmed->BasisSurface()->DynamicType()->Name();
  }
}

void writeAnalyticParameters(const occ::handle<Geom_Surface>& surface)
{
  if (surface->IsKind(STANDARD_TYPE(Geom_CylindricalSurface)))
    return writeCylinderParameters(surface);
  if (surface->IsKind(STANDARD_TYPE(Geom_ConicalSurface)))
    return writeConeParameters(surface);
  if (surface->IsKind(STANDARD_TYPE(Geom_SphericalSurface)))
    return writeSphereParameters(surface);
  if (surface->IsKind(STANDARD_TYPE(Geom_ToroidalSurface)))
    return writeTorusParameters(surface);
  writeFallbackParameters(surface);
}

void writeSurface(const occ::handle<Geom_Surface>& surface)
{
  std::cout << " type=" << surface->DynamicType()->Name();
  double uMin, uMax, vMin, vMax;
  surface->Bounds(uMin, uMax, vMin, vMax);
  std::cout << " bounds=" << uMin << ',' << uMax << ',' << vMin << ',' << vMax;
  writeSample(surface);
  writeAnalyticParameters(surface);
}

double collapseOffset(const std::string& kind, const bool direct)
{
  const double direction = direct ? 1.0 : -1.0;
  if (kind == "cylinder" || kind == "sphere" || kind == "trimmed" || kind == "torus")
    return -2.0 * direction;
  if (kind == "cone") return -2.0 / std::cos(0.4);
  return 0.0;
}

void appendThresholdOffsets(std::vector<double>& result, const double collapse,
                            const bool direct)
{
  if (collapse == 0.0) return;
  const double direction = direct ? 1.0 : -1.0;
  const double tolerance = Precision::Confusion();
  result.push_back(collapse);
  result.push_back(collapse + tolerance * 0.999 * direction);
  result.push_back(collapse - tolerance * 0.999 * direction);
  result.push_back(collapse + tolerance * 1.001 * direction);
  result.push_back(collapse - tolerance * 1.001 * direction);
}

std::vector<double> offsets(const std::string& kind, const bool direct)
{
  std::vector<double> result{1.0, -2.0, -3.0, 1.0e-7, -1.0e-7};
  appendThresholdOffsets(result, collapseOffset(kind, direct), direct);
  return result;
}

void probe(const std::string& kind, const bool direct, const double offset, const bool allowC0)
{
  std::cout << "case=" << kind << " direct=" << direct << " offset=" << offset
            << " allow_c0=" << allowC0;
  try
  {
    const occ::handle<Geom_Surface> input = makeSurface(kind, direct);
    BRepOffset_Status status = BRepOffset_Unknown;
    const occ::handle<Geom_Surface> result = BRepOffset::Surface(input, offset, status, allowC0);
    std::cout << " status=" << statusName(status);
    writeSurface(result);
    std::cout << '\n';
  }
  catch (const Standard_Failure&)
  {
    std::cout << " exception=Standard_Failure\n";
  }
}
} // namespace

int main()
{
  std::cout << std::setprecision(17);
  for (const std::string kind : {"plane", "cylinder", "cone", "sphere", "torus", "spindle",
                                 "trimmed", "bezier"})
  {
    for (const bool direct : {true, false})
    {
      const std::vector<double> sweep = offsets(kind, direct);
      for (const double offset : sweep)
      {
        probe(kind, direct, offset, false);
      }
    }
  }
  probe("bezier", true, 1.0, true);
}
