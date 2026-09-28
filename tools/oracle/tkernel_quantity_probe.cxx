// Quantity probe operations (twin of crates/tkernel/examples/oracle_probe/quantity.rs).
//
// Floats travel as 8-digit hex IEEE bits (`f`), doubles as 16-digit hex bits (`d`); outputs
// print the same way so the comparison is bit-exact. A raised Standard_Failure is caught by
// the operation and printed as `raise <type> <message>` inside the OK line.
//
// qc_named <index>                 every observable of the named color
// qc_ctor <type 0..4> d d d        Quantity_Color(c1, c2, c3, type)
// qc_vec3 f f f                    Quantity_Color(NCollection_Vec3<float>)
// qc_name d d d                    Quantity_Color::Name(r, g, b)
// qc_from_name <token|EMPTY>       ColorFromName for NameOfColor, Color and ColorRGBA
// qc_hex <token|EMPTY>             ColorFromHex for Color and ColorRGBA (alpha on/off)
// qc_argb <int>                    Argb2color then Color2argb
// qc_hls f f f                     Convert_HLS_To_sRGB / _LinearRGB, HlsRgb
// qc_srgb_hls f f f                Convert_sRGB_To_HLS, Convert_LinearRGB_To_HLS, RgbHls
// qc_lab f f f                     Lab/Lch/linear RGB conversions
// qc_gamma f f f d d d             scalar and vector gamma conversions, approx22
// qc_pair d d d d d d              distances, equality, Delta, DeltaE2000
// qc_change d d d d                ChangeIntensity and ChangeContrast
// qc_epsilon d                     SetEpsilon, Epsilon, equality and Lch under it
// qc_json <text...>                Quantity_Color::InitFromJson
// qrgba_ctor f f f f               Quantity_ColorRGBA(r, g, b, a) and the Vec4 constructor
// qrgba_pair f f f f f f f f       RGBA equality
// qrgba_gamma f f f f              Vec4 gamma conversions
// qrgba_json <text...>             Quantity_ColorRGBA::InitFromJson
// qperiod_ops s u s u              Quantity_Period operators
// qdate_ops <8 ints> s u           Quantity_Date operators against a period and the epoch
#include <NCollection_Vec3.hxx>
#include <NCollection_Vec4.hxx>
#include <Quantity_Color.hxx>
#include <Quantity_ColorRGBA.hxx>
#include <Quantity_Date.hxx>
#include <Quantity_Period.hxx>
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <functional>
#include <istream>
#include <ostream>
#include <sstream>
#include <string>

static float quantity_float(std::istream& in)
{
  std::string aToken;
  in >> aToken;
  const uint32_t aBits = static_cast<uint32_t>(std::stoul(aToken, nullptr, 16));
  float          aValue;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  return aValue;
}

static double quantity_double(std::istream& in)
{
  std::string aToken;
  in >> aToken;
  const uint64_t aBits = static_cast<uint64_t>(std::stoull(aToken, nullptr, 16));
  double         aValue;
  std::memcpy(&aValue, &aBits, sizeof(aValue));
  return aValue;
}

static std::string quantity_f(float theValue)
{
  uint32_t aBits;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  char aText[16];
  std::snprintf(aText, sizeof(aText), "%08x", aBits);
  return aText;
}

static std::string quantity_d(double theValue)
{
  uint64_t aBits;
  std::memcpy(&aBits, &theValue, sizeof(aBits));
  char aText[24];
  std::snprintf(aText, sizeof(aText), "%016llx", static_cast<unsigned long long>(aBits));
  return aText;
}

static std::string quantity_vec3(const NCollection_Vec3<float>& theVec)
{
  return quantity_f(theVec.r()) + ":" + quantity_f(theVec.g()) + ":" + quantity_f(theVec.b());
}

static std::string quantity_token(std::istream& in)
{
  std::string aToken;
  in >> aToken;
  return aToken == "EMPTY" ? std::string() : aToken;
}

static std::string quantity_rest(std::istream& in)
{
  std::string aRest;
  std::getline(in, aRest);
  const size_t aStart = aRest.find_first_not_of(' ');
  return aStart == std::string::npos ? std::string() : aRest.substr(aStart);
}

static void quantity_raise(std::ostream& out, const Standard_Failure& theFailure)
{
  out << "raise " << theFailure.ExceptionType() << ' ' << theFailure.what();
}

static void quantity_color_values(std::ostream& out, const Quantity_Color& theColor)
{
  for (int aType = Quantity_TOC_RGB; aType <= Quantity_TOC_CIELch; ++aType)
  {
    double aC1, aC2, aC3;
    theColor.Values(aC1, aC2, aC3, (Quantity_TypeOfColor)aType);
    out << ' ' << quantity_d(aC1) << ':' << quantity_d(aC2) << ':' << quantity_d(aC3);
  }
}

static void quantity_color_out(std::ostream& out, const Quantity_Color& theColor)
{
  int anArgb = 0;
  Quantity_Color::Color2argb(theColor, anArgb);
  std::ostringstream aDump;
  theColor.DumpJson(aDump);
  out << quantity_vec3(theColor.Rgb());
  quantity_color_values(out, theColor);
  out << ' ' << quantity_d(theColor.Hue()) << ' ' << quantity_d(theColor.Light()) << ' '
      << quantity_d(theColor.Saturation()) << ' ' << (int)theColor.Name() << ' '
      << Quantity_Color::ColorToHex(theColor).ToCString() << ' '
      << Quantity_Color::ColorToHex(theColor, false).ToCString() << ' ' << anArgb << ' '
      << std::hash<Quantity_Color>()(theColor) << " [" << aDump.str() << ']';
}

static void quantity_rgba_out(std::ostream& out, const Quantity_ColorRGBA& theColor)
{
  std::ostringstream aDump;
  theColor.DumpJson(aDump);
  out << quantity_vec3(theColor.GetRGB().Rgb()) << ' ' << quantity_f(theColor.Alpha()) << ' '
      << Quantity_ColorRGBA::ColorToHex(theColor).ToCString() << ' '
      << Quantity_ColorRGBA::ColorToHex(theColor, false).ToCString() << ' '
      << std::hash<Quantity_ColorRGBA>()(theColor) << " [" << aDump.str() << ']';
}

static void quantity_named(std::istream& in, std::ostream& out)
{
  int anIndex = 0;
  in >> anIndex;
  const Quantity_Color aColor((Quantity_NameOfColor)anIndex);
  out << Quantity_Color::StringName((Quantity_NameOfColor)anIndex) << ' ';
  quantity_color_out(out, aColor);
  Quantity_ColorRGBA anRgba(aColor);
  out << ' ';
  quantity_rgba_out(out, anRgba);
}

static void quantity_ctor(std::istream& in, std::ostream& out)
{
  int          aType = 0;
  in >> aType;
  const double aC1 = quantity_double(in), aC2 = quantity_double(in), aC3 = quantity_double(in);
  try
  {
    quantity_color_out(out, Quantity_Color(aC1, aC2, aC3, (Quantity_TypeOfColor)aType));
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
}

static void quantity_vec3_ctor(std::istream& in, std::ostream& out)
{
  const float aR = quantity_float(in), aG = quantity_float(in), aB = quantity_float(in);
  try
  {
    quantity_color_out(out, Quantity_Color(NCollection_Vec3<float>(aR, aG, aB)));
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
}

static void quantity_name(std::istream& in, std::ostream& out)
{
  const double aR = quantity_double(in), aG = quantity_double(in), aB = quantity_double(in);
  try
  {
    out << (int)Quantity_Color::Name(aR, aG, aB);
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
}

static void quantity_from_name(std::istream& in, std::ostream& out)
{
  const std::string    aName = quantity_token(in);
  Quantity_NameOfColor anEnum = Quantity_NOC_WHITE;
  Quantity_Color       aColor(Quantity_NOC_WHITE);
  Quantity_ColorRGBA   anRgba(Quantity_Color(Quantity_NOC_WHITE), 0.25f);
  const bool           isEnum  = Quantity_Color::ColorFromName(aName.c_str(), anEnum);
  const bool           isColor = Quantity_Color::ColorFromName(aName.c_str(), aColor);
  const bool           isRgba  = Quantity_ColorRGBA::ColorFromName(aName.c_str(), anRgba);
  out << isEnum << ' ' << (int)anEnum << ' ' << isColor << ' ' << quantity_vec3(aColor.Rgb())
      << ' ' << isRgba << ' ' << quantity_vec3(anRgba.GetRGB().Rgb()) << ' '
      << quantity_f(anRgba.Alpha());
}

static void quantity_hex(std::istream& in, std::ostream& out)
{
  const std::string aText = quantity_token(in);
  Quantity_Color    aColor(Quantity_NOC_WHITE);
  const bool        isColor = Quantity_Color::ColorFromHex(aText.c_str(), aColor);
  out << isColor << ' ' << quantity_vec3(aColor.Rgb());
  for (const bool isAlphaOff : {false, true})
  {
    Quantity_ColorRGBA anRgba(Quantity_Color(Quantity_NOC_WHITE), 0.25f);
    const bool isRgba = Quantity_ColorRGBA::ColorFromHex(aText.c_str(), anRgba, isAlphaOff);
    out << ' ' << isRgba << ' ' << quantity_vec3(anRgba.GetRGB().Rgb()) << ' '
        << quantity_f(anRgba.Alpha());
  }
}

static void quantity_argb(std::istream& in, std::ostream& out)
{
  int anArgb = 0;
  in >> anArgb;
  Quantity_Color aColor(Quantity_NOC_WHITE);
  Quantity_Color::Argb2color(anArgb, aColor);
  quantity_color_out(out, aColor);
}

static void quantity_hls(std::istream& in, std::ostream& out)
{
  const float aH = quantity_float(in), aL = quantity_float(in), aS = quantity_float(in);
  const NCollection_Vec3<float> aHls(aH, aL, aS);
  try
  {
    out << quantity_vec3(Quantity_Color::Convert_HLS_To_sRGB(aHls));
  }
  catch (const Standard_Failure& theFailure)
  {
    // The noexcept wrappers below would call std::terminate on this input.
    quantity_raise(out, theFailure);
    return;
  }
  double aR, aG, aB;
  Quantity_Color::HlsRgb(aH, aL, aS, aR, aG, aB);
  out << ' ' << quantity_vec3(Quantity_Color::Convert_HLS_To_LinearRGB(aHls)) << ' '
      << quantity_d(aR) << ':' << quantity_d(aG) << ':' << quantity_d(aB);
}

static void quantity_srgb_hls(std::istream& in, std::ostream& out)
{
  const float aR = quantity_float(in), aG = quantity_float(in), aB = quantity_float(in);
  const NCollection_Vec3<float> aRgb(aR, aG, aB);
  double                        aH, aL, aS;
  Quantity_Color::RgbHls(aR, aG, aB, aH, aL, aS);
  out << quantity_vec3(Quantity_Color::Convert_sRGB_To_HLS(aRgb)) << ' '
      << quantity_vec3(Quantity_Color::Convert_LinearRGB_To_HLS(aRgb)) << ' ' << quantity_d(aH)
      << ':' << quantity_d(aL) << ':' << quantity_d(aS);
}

static void quantity_lab(std::istream& in, std::ostream& out)
{
  const float                   aX = quantity_float(in), aY = quantity_float(in), aZ = quantity_float(in);
  const NCollection_Vec3<float> aVec(aX, aY, aZ);
  out << quantity_vec3(Quantity_Color::Convert_Lab_To_LinearRGB(aVec)) << ' '
      << quantity_vec3(Quantity_Color::Convert_Lab_To_Lch(aVec)) << ' '
      << quantity_vec3(Quantity_Color::Convert_Lch_To_Lab(aVec)) << ' '
      << quantity_vec3(Quantity_Color::Convert_LinearRGB_To_Lab(aVec));
}

static void quantity_gamma(std::istream& in, std::ostream& out)
{
  const float  aF1 = quantity_float(in), aF2 = quantity_float(in), aF3 = quantity_float(in);
  const double aD1 = quantity_double(in), aD2 = quantity_double(in), aD3 = quantity_double(in);
  const NCollection_Vec3<float>  aFloats(aF1, aF2, aF3);
  const NCollection_Vec3<double> aDoubles(aD1, aD2, aD3);
  const NCollection_Vec3<double> aToSrgb   = Quantity_Color::Convert_LinearRGB_To_sRGB(aDoubles);
  const NCollection_Vec3<double> aToLinear = Quantity_Color::Convert_sRGB_To_LinearRGB(aDoubles);
  out << quantity_vec3(Quantity_Color::Convert_LinearRGB_To_sRGB(aFloats)) << ' '
      << quantity_vec3(Quantity_Color::Convert_sRGB_To_LinearRGB(aFloats)) << ' '
      << quantity_vec3(Quantity_Color::Convert_LinearRGB_To_sRGB_approx22(aFloats)) << ' '
      << quantity_vec3(Quantity_Color::Convert_sRGB_To_LinearRGB_approx22(aFloats)) << ' '
      << quantity_f(Quantity_Color::Convert_LinearRGB_To_sRGB_approx22(aF1)) << ' '
      << quantity_f(Quantity_Color::Convert_sRGB_To_LinearRGB_approx22(aF1)) << ' '
      << quantity_f(Quantity_Color::Convert_LinearRGB_To_sRGB(aF1)) << ' '
      << quantity_f(Quantity_Color::Convert_sRGB_To_LinearRGB(aF1)) << ' '
      << quantity_d(Quantity_Color::Convert_LinearRGB_To_sRGB(aD1)) << ' '
      << quantity_d(Quantity_Color::Convert_sRGB_To_LinearRGB(aD1)) << ' '
      << quantity_d(aToSrgb.x()) << ':' << quantity_d(aToSrgb.y()) << ':' << quantity_d(aToSrgb.z())
      << ' ' << quantity_d(aToLinear.x()) << ':' << quantity_d(aToLinear.y()) << ':'
      << quantity_d(aToLinear.z());
}

static void quantity_pair(std::istream& in, std::ostream& out)
{
  const double aR1 = quantity_double(in), aG1 = quantity_double(in), aB1 = quantity_double(in);
  const double aR2 = quantity_double(in), aG2 = quantity_double(in), aB2 = quantity_double(in);
  const Quantity_Color aFirst(aR1, aG1, aB1, Quantity_TOC_RGB);
  const Quantity_Color aSecond(aR2, aG2, aB2, Quantity_TOC_RGB);
  double               aDC, aDI;
  aFirst.Delta(aSecond, aDC, aDI);
  out << quantity_d(aFirst.Distance(aSecond)) << ' ' << quantity_d(aFirst.SquareDistance(aSecond))
      << ' ' << aFirst.IsEqual(aSecond) << aFirst.IsDifferent(aSecond) << (aFirst == aSecond)
      << (aFirst != aSecond) << ' ' << quantity_d(aDC) << ' ' << quantity_d(aDI) << ' '
      << quantity_d(aFirst.DeltaE2000(aSecond));
}

static void quantity_change(std::istream& in, std::ostream& out)
{
  const double aR = quantity_double(in), aG = quantity_double(in), aB = quantity_double(in);
  const double aDelta = quantity_double(in);
  Quantity_Color anIntensity(aR, aG, aB, Quantity_TOC_RGB);
  Quantity_Color aContrast = anIntensity;
  anIntensity.ChangeIntensity(aDelta);
  aContrast.ChangeContrast(aDelta);
  out << quantity_vec3(anIntensity.Rgb()) << ' ' << quantity_vec3(aContrast.Rgb());
}

static void quantity_epsilon(std::istream& in, std::ostream& out)
{
  const double         anEpsilon = quantity_double(in);
  const Quantity_Color aRed(Quantity_NOC_RED);
  const Quantity_Color aNear(1.0, 0.001, 0.0, Quantity_TOC_RGB);
  Quantity_Color::SetEpsilon(anEpsilon);
  out << quantity_d(Quantity_Color::Epsilon()) << ' ' << aRed.IsEqual(aNear) << ' '
      << quantity_vec3(Quantity_Color::Convert_Lab_To_Lch(NCollection_Vec3<float>(50.0f, 0.001f, 0.001f)))
      << ' ' << quantity_d(aRed.DeltaE2000(aNear));
  Quantity_Color::SetEpsilon(0.0001);
}

static void quantity_json(std::istream& in, std::ostream& out)
{
  const std::string aText = quantity_rest(in);
  Standard_SStream  aStream;
  aStream << aText;
  Quantity_Color aColor(Quantity_NOC_WHITE);
  int            aPos = 1;
  try
  {
    const bool isDone = aColor.InitFromJson(aStream, aPos);
    out << isDone;
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
  out << ' ' << aPos << ' ' << quantity_vec3(aColor.Rgb());
}

static void quantity_rgba_ctor(std::istream& in, std::ostream& out)
{
  const float aR = quantity_float(in), aG = quantity_float(in), aB = quantity_float(in);
  const float anA = quantity_float(in);
  try
  {
    quantity_rgba_out(out, Quantity_ColorRGBA(aR, aG, aB, anA));
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
  out << ' ';
  try
  {
    quantity_rgba_out(out, Quantity_ColorRGBA(NCollection_Vec4<float>(aR, aG, aB, anA)));
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
}

static Quantity_ColorRGBA quantity_rgba(std::istream& in)
{
  const float aR = quantity_float(in), aG = quantity_float(in), aB = quantity_float(in);
  return Quantity_ColorRGBA(aR, aG, aB, quantity_float(in));
}

static void quantity_rgba_pair(std::istream& in, std::ostream& out)
{
  const Quantity_ColorRGBA aFirst  = quantity_rgba(in);
  const Quantity_ColorRGBA aSecond = quantity_rgba(in);
  out << aFirst.IsEqual(aSecond) << aFirst.IsDifferent(aSecond) << (aFirst == aSecond)
      << (aFirst != aSecond);
}

static void quantity_rgba_gamma(std::istream& in, std::ostream& out)
{
  const float aR = quantity_float(in), aG = quantity_float(in), aB = quantity_float(in);
  const NCollection_Vec4<float> aVec(aR, aG, aB, quantity_float(in));
  for (const NCollection_Vec4<float>& aResult : {Quantity_ColorRGBA::Convert_LinearRGB_To_sRGB(aVec),
                                                 Quantity_ColorRGBA::Convert_sRGB_To_LinearRGB(aVec)})
  {
    out << quantity_vec3(aResult.rgb()) << ':' << quantity_f(aResult.a()) << ' ';
  }
}

static void quantity_rgba_json(std::istream& in, std::ostream& out)
{
  const std::string aText = quantity_rest(in);
  Standard_SStream  aStream;
  aStream << aText;
  Quantity_ColorRGBA aColor(Quantity_Color(Quantity_NOC_WHITE), 0.25f);
  int                aPos = 1;
  try
  {
    const bool isDone = aColor.InitFromJson(aStream, aPos);
    out << isDone;
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
  out << ' ' << aPos << ' ' << quantity_vec3(aColor.GetRGB().Rgb()) << ' '
      << quantity_f(aColor.Alpha());
}

static void quantity_period_out(std::ostream& out, const Quantity_Period& thePeriod)
{
  int aSec, aUSec;
  thePeriod.Values(aSec, aUSec);
  out << aSec << ':' << aUSec;
}

static void quantity_period_ops(std::istream& in, std::ostream& out)
{
  int aS1, aU1, aS2, aU2;
  in >> aS1 >> aU1 >> aS2 >> aU2;
  const Quantity_Period aFirst(aS1, aU1), aSecond(aS2, aU2);
  out << (aFirst == aSecond) << (aFirst < aSecond) << (aFirst > aSecond) << ' ';
  quantity_period_out(out, aFirst + aSecond);
  out << ' ';
  try
  {
    quantity_period_out(out, aFirst - aSecond);
  }
  catch (const Standard_Failure& theFailure)
  {
    quantity_raise(out, theFailure);
  }
}

static void quantity_date_ops(std::istream& in, std::ostream& out)
{
  int aMM, aDD, aYY, aHH, aMN, aSS, aMIS, aMICS, aSec, aUSec;
  in >> aMM >> aDD >> aYY >> aHH >> aMN >> aSS >> aMIS >> aMICS >> aSec >> aUSec;
  Quantity_Date         aDate(aMM, aDD, aYY, aHH, aMN, aSS, aMIS, aMICS);
  const Quantity_Date   anEpoch;
  const Quantity_Period aPeriod(aSec, aUSec);
  out << (aDate == anEpoch) << (aDate < anEpoch) << (aDate > anEpoch);
  for (int anOp = 0; anOp < 2; ++anOp)
  {
    try
    {
      Quantity_Date aResult = anOp == 0 ? aDate + aPeriod : aDate - aPeriod;
      out << ' ';
      quantity_period_out(out, aResult.Difference(anEpoch));
    }
    catch (const Standard_Failure& theFailure)
    {
      out << ' ';
      quantity_raise(out, theFailure);
    }
  }
}

static bool quantity_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  using Operation = void (*)(std::istream&, std::ostream&);
  static const std::pair<const char*, Operation> THE_OPERATIONS[] = {
    {"qc_named", quantity_named},         {"qc_ctor", quantity_ctor},
    {"qc_vec3", quantity_vec3_ctor},      {"qc_name", quantity_name},
    {"qc_from_name", quantity_from_name}, {"qc_hex", quantity_hex},
    {"qc_argb", quantity_argb},           {"qc_hls", quantity_hls},
    {"qc_srgb_hls", quantity_srgb_hls},   {"qc_lab", quantity_lab},
    {"qc_gamma", quantity_gamma},         {"qc_pair", quantity_pair},
    {"qc_change", quantity_change},       {"qc_epsilon", quantity_epsilon},
    {"qc_json", quantity_json},           {"qrgba_ctor", quantity_rgba_ctor},
    {"qrgba_pair", quantity_rgba_pair},   {"qrgba_gamma", quantity_rgba_gamma},
    {"qrgba_json", quantity_rgba_json},   {"qperiod_ops", quantity_period_ops},
    {"qdate_ops", quantity_date_ops}};
  for (const auto& anOperation : THE_OPERATIONS)
  {
    if (op == anOperation.first)
    {
      anOperation.second(in, out);
      return true;
    }
  }
  return false;
}
