// Units classes used directly: tokens, measurements, units, quantities, explorers and
// units systems.
#include <Units_Explorer.hxx>
#include <Units_Measurement.hxx>
#include <Units_ShiftedUnit.hxx>
#include <Units_Unit.hxx>
#include <Units_UnitsSystem.hxx>

static occ::handle<Units_Token> units_evaluated(const std::string& theExpr)
{
  Units_UnitSentence aSentence(theExpr.c_str());
  return aSentence.IsDone() ? aSentence.Evaluate() : occ::handle<Units_Token>();
}

static void units_token_arith(std::ostream& res, const occ::handle<Units_Token>& a,
                              const occ::handle<Units_Token>& b, double theValue)
{
  for (const occ::handle<Units_Token>& aResult :
       {a->Add(b), a->Subtract(b), a->Multiply(b), a->Divide(b), a->Power(b), a->Power(theValue),
        a->Add(3), a + b, a - b, a * b, a / b, pow(a, b), pow(a, theValue), a + 7})
  {
    units_token(res, aResult);
    res << (aResult == a ? " same" : "") << " | ";
  }
  res << units_bits(a->Multiplied(theValue)) << ' ' << units_bits(a->Divided(theValue)) << ' ';
}

static void units_token_compare(std::ostream& res, const occ::handle<Units_Token>& a,
                                const occ::handle<Units_Token>& b, const std::string& theText)
{
  const char* aText = theText.c_str();
  res << a->IsEqual(aText) << a->IsNotEqual(aText) << a->IsLessOrEqual(aText) << a->IsGreater(aText)
      << a->IsGreater(b) << a->IsGreaterOrEqual(b) << a->IsEqual(b) << a->IsNotEqual(b)
      << (a == aText) << (a != aText) << (a <= aText) << (a > aText) << (a > b) << (a >= b) << ' '
      << a->Length() << ' ';
}

static void units_token_copies(std::ostream& res, const occ::handle<Units_Token>& a,
                               const std::string& theMean, double theValue)
{
  occ::handle<Units_Token> aCreated = a->Creates();
  occ::handle<Units_Token> aCopy    = new Units_Token(a);
  units_token(res, aCreated);
  res << ' ' << (aCopy->Dimensions() == a->Dimensions()) << (aCreated->Dimensions() == a->Dimensions())
      << ' ';
  aCreated->Update(theMean.c_str());
  aCreated->Update(theMean.c_str());
  units_token(res, aCreated);
  occ::handle<Units_ShiftedToken> aShifted =
    new Units_ShiftedToken("sh", theMean.c_str(), theValue, 2.5, a->Dimensions());
  res << ' ' << units_bits(aShifted->Multiplied(theValue)) << ' '
      << units_bits(aShifted->Divided(theValue)) << ' ';
  units_token(res, aShifted->Creates());
  aShifted->Dump(1, 1);
  a->Dump(0, 1);
}

static void units_token_ctors(std::ostream& res, const std::string& theText, double theValue)
{
  occ::handle<Units_Token> aTokens[] = {
    new Units_Token(), new Units_Token(theText.c_str()), new Units_Token(theText.c_str(), "U"),
    new Units_Token(theText.c_str(), "M", theValue),
    new Units_Token(theText.c_str(), "K", theValue, occ::handle<Units_Dimensions>())};
  for (const occ::handle<Units_Token>& aToken : aTokens)
  {
    units_token(res, aToken);
    res << " | ";
  }
  aTokens[0]->Word("w");
  aTokens[0]->Mean("m");
  aTokens[0]->Value(theValue);
  aTokens[0]->Dimensions(Units_Dimensions::ALength());
  units_token(res, aTokens[0]);
  res << ' ' << (aTokens[0]->Dimensions() == Units_Dimensions::ALength());
  aTokens[0]->Dimensions(occ::handle<Units_Dimensions>());
  res << ' ';
  units_token(res, aTokens[0]);
  aTokens[1]->Dump(1, 0);
}

static void units_token_op(std::istream& in, std::ostream& out)
{
  const std::string anA = units_arg(in), aB = units_arg(in);
  const double      aValue = units_real(in);
  const std::string aText = units_arg(in), aMean = units_arg(in);
  units_run(out, [&](std::ostream& res) {
    units_token_ctors(res, aText, aValue);
    res << " || ";
    occ::handle<Units_Token> a = units_evaluated(anA), b = units_evaluated(aB);
    if (a.IsNull() || b.IsNull())
    {
      res << "null";
      return;
    }
    units_token_compare(res, a, b, aText);
    units_token_arith(res, a, b, aValue);
    units_token_copies(res, a, aMean, aValue);
  });
}

static void units_measure_print(std::ostream& res, const Units_Measurement& theMeasure)
{
  res << theMeasure.HasToken() << ' ' << units_bits(theMeasure.Measurement()) << ' ';
  units_token(res, theMeasure.Token());
  res << " | ";
}

static void units_measure_op(std::istream& in, std::ostream& out)
{
  const double      aV1 = units_real(in);
  const std::string aU1 = units_arg(in);
  const double      aV2 = units_real(in);
  const std::string aU2 = units_arg(in), aU3 = units_arg(in);
  const double      anExp = units_real(in);
  units_run(out, [&](std::ostream& res) {
    Units_Measurement aFirst(aV1, aU1.c_str()), aSecond(aV2, aU2.c_str()), aDefault;
    units_measure_print(res, aDefault);
    units_measure_print(res, aFirst);
    units_measure_print(res, aSecond);
    units_measure_print(res, aFirst.Integer());
    units_measure_print(res, aFirst.Fractional());
    Units_Measurement aConverted = aFirst;
    aConverted.Convert(aU3.c_str());
    units_measure_print(res, aConverted);
    if (!aFirst.HasToken() || !aSecond.HasToken())
      return;
    for (const Units_Measurement& aResult :
         {aFirst.Add(aSecond), aFirst.Subtract(aSecond), aFirst.Multiply(aSecond),
          aFirst.Divide(aSecond), aFirst + aSecond, aFirst - aSecond, aFirst * aSecond,
          aFirst / aSecond, aFirst.Multiply(anExp), aFirst.Divide(anExp), aFirst * anExp,
          aFirst / anExp, aFirst.Power(anExp), Units_Measurement(aV2, aFirst.Token())})
      units_measure_print(res, aResult);
    aFirst.Dump();
  });
}

static void units_unit_op(std::istream& in, std::ostream& out)
{
  const std::string aName = units_arg(in), aSymbol = units_arg(in);
  const double      aValue = units_real(in), aMove = units_real(in);
  units_run(out, [&](std::ostream& res) {
    occ::handle<NCollection_HSequence<occ::handle<Units_Unit>>> aUnits =
      new NCollection_HSequence<occ::handle<Units_Unit>>();
    occ::handle<Units_Quantity> aQuantity = new Units_Quantity("TESTQ", Units_Dimensions::ALength(), aUnits);
    occ::handle<Units_Unit> aUnit = new Units_Unit(aName.c_str(), aSymbol.c_str(), aValue, aQuantity);
    aUnit->Symbol("alt");
    res << aUnit->IsEqual(aSymbol.c_str()) << aUnit->IsEqual("alt") << aUnit->IsEqual("zz")
        << (aUnit == aSymbol.c_str()) << ' ' << units_hex(aUnit->Name().ToCString()) << ' '
        << units_bits(aUnit->Value()) << ' ' << aUnit->SymbolsSequence()->Length() << ' '
        << (aUnit->Quantity() == aQuantity) << ' ';
    units_token(res, aUnit->Token());
    occ::handle<Units_ShiftedUnit> aShifted =
      new Units_ShiftedUnit(aName.c_str(), aSymbol.c_str(), aValue, aMove, aQuantity);
    res << ' ' << units_bits(aShifted->Move()) << ' ';
    units_token(res, aShifted->Token());
    occ::handle<Units_ShiftedUnit> aBare = new Units_ShiftedUnit(aName.c_str(), aSymbol.c_str());
    occ::handle<Units_ShiftedUnit> aNamed = new Units_ShiftedUnit(aName.c_str());
    aBare->Move(aMove);
    res << ' ' << units_bits(aBare->Move()) << ' ' << units_bits(aNamed->Move()) << ' '
        << aNamed->SymbolsSequence()->Length() << ' '
        << occ::handle<Units_Unit>(new Units_Unit(aName.c_str(), aSymbol.c_str()))->Quantity().IsNull()
        << ' ' << occ::handle<Units_Unit>(new Units_Unit(aName.c_str()))->SymbolsSequence()->Length();
    aUnits->Append(aUnit);
    aUnits->Append(aShifted);
    aUnit->Value(aValue * 2.0);
    aUnit->Dump(0, 0);
    aShifted->Dump(0, 0);
    aQuantity->Dump(1, 1);
    aQuantity->Dump(0, 0);
    res << ' ' << aQuantity->IsEqual("TESTQ") << (aQuantity == "TESTQ") << aQuantity->IsEqual("TEST");
    aNamed->Token();
  });
}

static void units_explore_walk(std::ostream& res, Units_Explorer& theExplorer, bool theAll)
{
  for (; theExplorer.MoreQuantity(); theExplorer.NextQuantity())
  {
    res << '{' << units_hex(theExplorer.Quantity().ToCString());
    for (; theExplorer.MoreUnit(); theExplorer.NextUnit())
      res << ' ' << units_hex(theExplorer.Unit().ToCString()) << (theExplorer.IsActive() ? "*" : "");
    res << '}';
    if (!theAll)
      return;
  }
}

static occ::handle<Units_UnitsSystem>& units_system()
{
  static occ::handle<Units_UnitsSystem> aSystem = new Units_UnitsSystem();
  return aSystem;
}

static void units_explore_op(std::istream& in, std::ostream& out)
{
  std::string aSource, aQuantity, aMode;
  in >> aSource >> aQuantity >> aMode;
  const std::string aName = units_unhex(aQuantity);
  units_run(out, [&](std::ostream& res) {
    Units_Explorer anExplorer;
    const bool     isDict = aSource == "dict";
    if (aQuantity == "-" && isDict)
      anExplorer.Init(Units::DictionaryOfUnits());
    else if (aQuantity == "-")
      anExplorer.Init(units_system());
    else if (isDict)
      anExplorer.Init(Units::DictionaryOfUnits(), aName.c_str());
    else
      anExplorer.Init(units_system(), aName.c_str());
    res << anExplorer.MoreQuantity() << ' ';
    if (aMode == "head")
      res << units_hex(anExplorer.Quantity().ToCString());
    else
      units_explore_walk(res, anExplorer, aQuantity == "-");
  });
}
