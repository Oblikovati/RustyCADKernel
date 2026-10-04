// Supplemental protected-hook oracle for OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
#include <Adaptor2d_Curve2d.hxx>
#include <Adaptor2d_Line2d.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2dGridEval_Curve.hxx>
#include <Geom2d_Circle.hxx>
#include <Geom2d_Line.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <Standard_NotImplemented.hxx>
#include <iostream>

class ReinitializationProbe : public Geom2dGridEval_Curve
{
public:
  using Geom2dGridEval_Curve::Geom2dGridEval_Curve;
  using Geom2dGridEval_Curve::initialization;
};

static void print_state(const ReinitializationProbe& evaluator)
{
  NCollection_Array1<double> parameters(-1, 0);
  parameters(-1) = 0.0;
  parameters(0) = 1.0;
  const auto points = evaluator.EvaluateGrid(parameters);
  std::cout << int(evaluator.GetType()) << ' ' << points.Lower() << ' ' << points.Upper();
  for (const auto& point : points) std::cout << ' ' << point.X() << ' ' << point.Y();
  std::cout << '\n';
}

static void clear_and_restore(ReinitializationProbe& evaluator,
                              const occ::handle<Geom2d_Curve>& line)
{
  evaluator.initialization(occ::handle<Geom2d_Curve>());
  print_state(evaluator);
  Geom2dAdaptor_Curve adaptor(line, 4.0, 5.0);
  evaluator.initialization(adaptor);
  print_state(evaluator);
  Adaptor2d_Curve2d throwing;
  try { evaluator.initialization(throwing); }
  catch (const Standard_NotImplemented&) { std::cout << "GetType raised\n"; }
  print_state(evaluator);
}

int main()
{
  occ::handle<Geom2d_Curve> line = new Geom2d_Line(gp_Pnt2d(3, 4), gp_Dir2d(1, 0));
  ReinitializationProbe evaluator(line);
  print_state(evaluator);
  evaluator.initialization(new Geom2d_Circle(gp_Ax2d(gp_Pnt2d(0, 0), gp_Dir2d(1, 0)), 2));
  NCollection_Array1<double> origin(1, 1);
  origin(1) = 0.0;
  const auto point = evaluator.EvaluateGrid(origin)(1);
  std::cout << int(evaluator.GetType()) << ' ' << point.X() << ' ' << point.Y() << '\n';
  evaluator.initialization(new Geom2d_TrimmedCurve(line, 4.0, 5.0));
  print_state(evaluator);
  clear_and_restore(evaluator, line);
  Adaptor2d_Line2d fallback(gp_Pnt2d(6, 7), gp_Dir2d(0, 1), 4.0, 5.0);
  evaluator.initialization(fallback);
  print_state(evaluator);
}
