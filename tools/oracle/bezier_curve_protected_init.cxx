#include <Geom2d_BezierCurve.hxx>
#include <Geom_BezierCurve.hxx>
#include <Standard_Failure.hxx>
#include <iostream>

template<class Curve, class Point>
class InitCurve : public Curve {
public:
  using Curve::init;
  InitCurve(const NCollection_Array1<Point>& poles, const NCollection_Array1<double>& weights)
    : Curve(poles, weights) {}
};

template<class Point>
NCollection_Array1<Point> poles(int count, double end) {
  NCollection_Array1<Point> result(5, 4 + count);
  for (int i = 0; i < count; ++i) result(5 + i).SetX(i == count - 1 ? end : i);
  return result;
}

template<class Curve>
void snapshot(const Curve& curve) {
  std::cout << curve.NbPoles() << ' ' << curve.Degree() << ' '
            << curve.IsRational() << ' ' << curve.IsClosed() << ' '
            << curve.Pole(1).X() << ' ' << curve.Weight(1) << '\n';
}

template<class Curve, class Point>
void finish(InitCurve<Curve, Point>& curve, const NCollection_Array1<Point>& three) {
  NCollection_Array1<double> shifted(8, 10); shifted(8) = -1; shifted(9) = 2; shifted(10) = 3;
  curve.init(three, &shifted); snapshot(curve);
  shifted.Init(0); curve.init(three, &shifted); snapshot(curve);
  curve.init(poles<Point>(1, 7.0), nullptr); snapshot(curve);
  try { curve.init(NCollection_Array1<Point>(), nullptr); }
  catch (const Standard_Failure& failure) { std::cout << failure.ExceptionType() << '\n'; }
  snapshot(curve);
  curve.init(poles<Point>(2, 1.0e-8), nullptr); std::cout << curve.IsClosed() << '\n';
}

template<class Curve, class Point>
void run() {
  auto initial = poles<Point>(2, 2.0);
  NCollection_Array1<double> weights(1, 2); weights(1) = 1; weights(2) = 2;
  InitCurve<Curve, Point> curve(initial, weights);
  auto three = poles<Point>(3, 0.0);
  try { curve.init(three, &weights); }
  catch (const Standard_Failure& failure) {
    std::cout << failure.ExceptionType() << ' ' << curve.NbPoles() << ' '
              << curve.IsRational() << ' ' << curve.IsClosed() << '\n';
  }
  finish<Curve, Point>(curve, three);
}

int main() {
  std::cout << "2d\n"; run<Geom2d_BezierCurve, gp_Pnt2d>();
  std::cout << "3d\n"; run<Geom_BezierCurve, gp_Pnt>();
}
