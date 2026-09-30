// ShapeConstruct_Curve: op 0 adjust3d, 1 segment3d, 2 adjust2d,
// 3 convert3d, 4 convert2d, 5 FixKnots(array), 6 FixKnots(handle).
// Curve records use tp_geom_curve/tp_geom2d_curve; knot records: lower count
// values. Digests include the original after failures, output aliasing, and
// every spline pole/knot.
#include <NCollection_HArray1.hxx>
#include <ShapeConstruct_Curve.hxx>

void sh_construct_spline3d(const occ::handle<Geom_Curve> &input) {
  auto spline = occ::down_cast<Geom_BSplineCurve>(input);
  if (spline.IsNull())
    return;
  ga_int(spline->Degree());
  ga_int(spline->NbPoles());
  ga_int(spline->NbKnots());
  ga_flag(spline->IsPeriodic());
  for (int i = 1; i <= spline->NbKnots(); ++i) {
    ga_real(spline->Knot(i));
    ga_int(spline->Multiplicity(i));
  }
  for (int i = 1; i <= spline->NbPoles(); ++i) {
    ga_pnt(spline->Pole(i));
    ga_real(spline->Weight(i));
  }
}

void sh_construct_spline2d(const occ::handle<Geom2d_Curve> &input) {
  auto spline = occ::down_cast<Geom2d_BSplineCurve>(input);
  if (spline.IsNull())
    return;
  ga_int(spline->Degree());
  ga_int(spline->NbPoles());
  ga_int(spline->NbKnots());
  ga_flag(spline->IsPeriodic());
  for (int i = 1; i <= spline->NbKnots(); ++i) {
    ga_real(spline->Knot(i));
    ga_int(spline->Multiplicity(i));
  }
  for (int i = 1; i <= spline->NbPoles(); ++i) {
    ga_pnt2d(spline->Pole(i));
    ga_real(spline->Weight(i));
  }
}

void sh_construct_adjust3d(GaReader &r, int operation) {
  auto input = tp_geom_curve(r);
  const auto first = r.pnt(), last = r.pnt();
  const double a = r.real(), b = r.real();
  ShapeConstruct_Curve tool;
  GA_TRY(ga_flag(operation == 0
                     ? tool.AdjustCurve(input, first, last, a != 0., b != 0.)
                     : tool.AdjustCurveSegment(input, first, last, a, b)))
  shc_curve(input);
  sh_construct_spline3d(input);
}

void sh_construct_adjust2d(GaReader &r) {
  auto input = tp_geom2d_curve(r);
  const auto first = r.pnt2d(), last = r.pnt2d();
  const bool a = r.flag(), b = r.flag();
  GA_TRY(
      ga_flag(ShapeConstruct_Curve().AdjustCurve2d(input, first, last, a, b)))
  shc_curve2d(input);
  sh_construct_spline2d(input);
}

void sh_construct_convert3d(GaReader &r) {
  auto input = tp_geom_curve(r);
  const double first = r.real(), last = r.real(), precision = r.real();
  GA_TRY(auto result = ShapeConstruct_Curve().ConvertToBSpline(input, first,
                                                               last, precision);
         ga_flag(result == input); shc_curve(result);
         sh_construct_spline3d(result))
  shc_curve(input);
  sh_construct_spline3d(input);
}

void sh_construct_convert2d(GaReader &r) {
  auto input = tp_geom2d_curve(r);
  const double first = r.real(), last = r.real(), precision = r.real();
  GA_TRY(auto result = ShapeConstruct_Curve().ConvertToBSpline(input, first,
                                                               last, precision);
         ga_flag(result == input); shc_curve2d(result);
         sh_construct_spline2d(result))
  shc_curve2d(input);
  sh_construct_spline2d(input);
}

void sh_construct_knots_handle(const NCollection_Array1<double> &knots) {
  occ::handle<NCollection_HArray1<double>> handle =
      new NCollection_HArray1<double>(knots);
  GA_TRY(ga_flag(ShapeConstruct_Curve::FixKnots(handle)))
  for (int index = knots.Lower(); index <= knots.Upper(); ++index)
    ga_real(handle->Value(index));
}

void sh_construct_knots(GaReader &r, bool use_handle) {
  const int lower = r.integer(), count = r.integer();
  NCollection_Array1<double> knots(lower, lower + count - 1);
  for (int i = lower; i < lower + count; ++i)
    knots(i) = r.real();
  if (use_handle) {
    sh_construct_knots_handle(knots);
    return;
  }
  GA_TRY(ga_flag(ShapeConstruct_Curve::FixKnots(knots)))
  for (int i = lower; i < lower + count; ++i)
    ga_real(knots(i));
}

void sh_construct_case(std::istream &input) {
  GaReader reader{input};
  const int operation = reader.integer();
  if (operation == 0 || operation == 1)
    sh_construct_adjust3d(reader, operation);
  else if (operation == 2)
    sh_construct_adjust2d(reader);
  else if (operation == 3)
    sh_construct_convert3d(reader);
  else if (operation == 4)
    sh_construct_convert2d(reader);
  else
    sh_construct_knots(reader, operation == 6);
  ga_end();
}
