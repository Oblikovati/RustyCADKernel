// Supplemental protected-hook oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Adaptor3d_Surface.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <GeomAdaptor_TransformedSurface.hxx>
#include <GeomGridEval_Surface.hxx>
#include <Geom_Plane.hxx>
#include <Geom_RectangularTrimmedSurface.hxx>
#include <Standard_NotImplemented.hxx>
#include <gp.hxx>
#include <iostream>

class SurfaceReinitializationProbe : public GeomGridEval_Surface
{
public:
  using GeomGridEval_Surface::GeomGridEval_Surface;
  using GeomGridEval_Surface::initialization;
};

static void print_state(const SurfaceReinitializationProbe& evaluator)
{
  NCollection_Array1<double> parameters(-1, -1);
  parameters(-1) = 0.0;
  const auto points = evaluator.EvaluateGrid(parameters, parameters);
  std::cout << int(evaluator.GetType()) << ' ' << evaluator.HasTransformation()
            << ' ' << points.Size();
  if (!points.IsEmpty())
  {
    const auto& point = points(1, 1);
    std::cout << ' ' << point.X() << ' ' << point.Y() << ' ' << point.Z();
  }
  std::cout << '\n';
}

static void clear_and_fail(SurfaceReinitializationProbe& evaluator,
                           const occ::handle<Geom_Surface>& plane)
{
  evaluator.initialization(occ::handle<Geom_Surface>());
  print_state(evaluator);
  evaluator.initialization(plane);
  print_state(evaluator);
  Adaptor3d_Surface throwing;
  try { evaluator.initialization(throwing); }
  catch (const Standard_NotImplemented&) { std::cout << "GetType raised\n"; }
  print_state(evaluator);
}

int main()
{
  occ::handle<Geom_Surface> plane = new Geom_Plane(gp::XOY());
  gp_Trsf translation;
  translation.SetTranslation(gp_Vec(10, 20, 30));
  GeomAdaptor_TransformedSurface transformed(plane, translation);
  SurfaceReinitializationProbe evaluator(transformed);
  print_state(evaluator);
  evaluator.initialization(new Geom_Plane(gp_Pnt(1, 2, 3), gp::DZ()));
  print_state(evaluator);
  clear_and_fail(evaluator, plane);
  evaluator.initialization(transformed);
  print_state(evaluator);
  GeomAdaptor_Surface plain(plane);
  evaluator.initialization(plain);
  print_state(evaluator);
  evaluator.initialization(new Geom_RectangularTrimmedSurface(plane, 4.0, 5.0, 6.0, 7.0));
  print_state(evaluator);
}
