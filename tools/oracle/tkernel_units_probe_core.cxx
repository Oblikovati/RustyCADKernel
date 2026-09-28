// Units package functions, sentences and conversions.
#include <Units.hxx>
#include <Units_Lexicon.hxx>
#include <Units_MathSentence.hxx>
#include <Units_Operators.hxx>
#include <Units_Quantity.hxx>
#include <Units_Sentence.hxx>
#include <Units_UnitSentence.hxx>
#include <Units_UnitsDictionary.hxx>
#include <Units_UnitsLexicon.hxx>

//! Sequence after construction, evaluated token, sequence length after evaluation.
static void units_sentence_report(std::ostream& out, Units_Sentence& theSentence)
{
  out << theSentence.IsDone() << ' ';
  units_tokens(out, theSentence.Sequence());
  out << " = ";
  units_token(out, theSentence.Evaluate());
  out << " left " << theSentence.Sequence()->Length();
}

static void units_sentence_op(const std::string& theKind, std::istream& in, std::ostream& out)
{
  const std::string anExpr = units_arg(in);
  units_run(out, [&](std::ostream& res) {
    if (theKind == "unit")
    {
      Units_UnitSentence aSentence(anExpr.c_str());
      units_sentence_report(res, aSentence);
    }
    else if (theKind == "math")
    {
      Units_MathSentence aSentence(anExpr.c_str());
      units_sentence_report(res, aSentence);
    }
    else
    {
      occ::handle<Units_Lexicon> aLexicon = theKind == "formula" ? Units::LexiconFormula()
                                            : theKind == "units" ? Units::LexiconUnits(false)
                                                                  : new Units_Lexicon();
      Units_Sentence aSentence(aLexicon, anExpr.c_str());
      aSentence.SetConstants();
      units_sentence_report(res, aSentence);
    }
  });
}

static void units_tosi_op(std::istream& in, std::ostream& out)
{
  const double      aValue = units_real(in);
  const std::string aUnit  = units_arg(in);
  units_run(out, [&](std::ostream& res) {
    occ::handle<Units_Dimensions> aTo, aFrom;
    res << units_bits(Units::ToSI(aValue, aUnit.c_str(), aTo)) << ' ';
    units_dims(res, aTo);
    res << ' ' << units_bits(Units::FromSI(aValue, aUnit.c_str(), aFrom)) << ' ';
    units_dims(res, aFrom);
    res << ' ' << units_bits(Units::ToSI(aValue, aUnit.c_str())) << ' '
        << units_bits(Units::FromSI(aValue, aUnit.c_str()));
  });
}

static void units_convert_op(std::istream& in, std::ostream& out)
{
  const double      aValue = units_real(in);
  const std::string aFrom = units_arg(in), aTo = units_arg(in);
  units_run(out, [&](std::ostream& res) {
    res << units_bits(Units::Convert(aValue, aFrom.c_str(), aTo.c_str()));
  });
}

static void units_quantity_op(std::istream& in, std::ostream& out)
{
  const std::string aName = units_arg(in);
  units_run(out, [&](std::ostream& res) {
    occ::handle<Units_Quantity> aQuantity = Units::Quantity(aName.c_str());
    if (aQuantity.IsNull())
      res << "null";
    else
    {
      res << units_hex(aQuantity->Name().ToCString()) << ' ' << aQuantity->Sequence()->Length() << ' ';
      units_dims(res, aQuantity->Dimensions());
    }
    res << " active " << units_hex(Units::DictionaryOfUnits()->ActiveUnit(aName.c_str()).ToCString());
    const char* aFirst = Units::FirstQuantity(aName.c_str());
    res << " first " << (aFirst == nullptr ? std::string("null") : units_hex(aFirst)) << " dims ";
    units_dims(res, Units::Dimensions(aName.c_str()));
  });
}

static void units_first_op(std::istream& in, std::ostream& out)
{
  const std::string aUnit = units_arg(in);
  units_run(out, [&](std::ostream& res) {
    const char* aFirst = Units::FirstQuantity(aUnit.c_str());
    res << (aFirst == nullptr ? std::string("null") : units_hex(aFirst));
  });
}

static occ::handle<Units_Dimensions> units_read_dims(std::istream& in)
{
  double aValues[9];
  for (double& aValue : aValues)
    aValue = units_real(in);
  return new Units_Dimensions(aValues[0], aValues[1], aValues[2], aValues[3], aValues[4], aValues[5],
                              aValues[6], aValues[7], aValues[8]);
}

static void units_dims_op(std::istream& in, std::ostream& out)
{
  occ::handle<Units_Dimensions> aLeft = units_read_dims(in), aRight = units_read_dims(in);
  const double                  aPower = units_real(in);
  units_run(out, [&](std::ostream& res) {
    units_dims(res, aLeft * aRight);
    res << ' ';
    units_dims(res, aLeft / aRight);
    res << ' ';
    units_dims(res, pow(aLeft, aPower));
    res << ' ' << aLeft->IsEqual(aRight) << aLeft->IsNotEqual(aRight) << ' ';
    const char* aQuantity = aLeft->Quantity();
    res << (aQuantity == nullptr ? std::string("null") : units_hex(aQuantity));
    aLeft->Dump(1);
  });
}

static void units_basic_op(std::ostream& out)
{
  units_run(out, [&](std::ostream& res) {
    for (const occ::handle<Units_Dimensions>& aDims :
         {Units_Dimensions::ALess(), Units_Dimensions::AMass(), Units_Dimensions::ALength(),
          Units_Dimensions::ATime(), Units_Dimensions::AElectricCurrent(),
          Units_Dimensions::AThermodynamicTemperature(), Units_Dimensions::AAmountOfSubstance(),
          Units_Dimensions::ALuminousIntensity(), Units_Dimensions::APlaneAngle(),
          Units_Dimensions::ASolidAngle(), Units::NullDimensions(), Units::Dimensions(nullptr)})
    {
      units_dims(res, aDims);
      res << ' ';
    }
    res << (Units_Dimensions::ALess() == Units_Dimensions::ALess())
        << (Units::NullDimensions() == Units::NullDimensions())
        << (Units::Dimensions(nullptr) == Units_Dimensions::ALess());
  });
}

static void units_lexicon_op(std::istream& in, std::ostream& out)
{
  std::string aKind;
  in >> aKind;
  units_run(out, [&](std::ostream& res) {
    occ::handle<Units_Lexicon> aLexicon =
      aKind == "formula" ? Units::LexiconFormula() : Units::LexiconUnits(false);
    units_tokens(res, aLexicon->Sequence());
    aLexicon->Dump();
  });
}

static void units_lexicon_add_op(std::istream& in, std::ostream& out)
{
  const std::string aWord = units_arg(in), aMean = units_arg(in);
  const double      aValue = units_real(in);
  units_run(out, [&](std::ostream& res) {
    occ::handle<Units_Lexicon> aLexicon = new Units_Lexicon();
    res << aLexicon->Sequence().IsNull() << ' ';
    aLexicon->Creates();
    aLexicon->AddToken(aWord.c_str(), aMean.c_str(), aValue);
    aLexicon->AddToken(aWord.c_str(), aMean.c_str(), aValue);
    units_tokens(res, aLexicon->Sequence());
    occ::handle<Units_UnitsLexicon> aUnits = new Units_UnitsLexicon();
    aUnits->Creates(false);
    res << ' ' << aUnits->Sequence()->Length();
    aUnits->Dump();
  });
}

static void units_dict_op(std::istream& in, std::ostream& out)
{
  std::string aKind;
  in >> aKind;
  units_run(out, [&](std::ostream& res) {
    const occ::handle<Units_UnitsDictionary>& aDict = Units::DictionaryOfUnits();
    if (aKind == "dump1")
      aDict->Dump(1);
    else if (aKind == "dump2")
      aDict->Dump(2);
    else if (aKind == "dumpdims")
      aDict->Dump(units_read_dims(in));
    else if (aKind == "recreate")
      res << (Units::DictionaryOfUnits(true) == aDict);
    else if (aKind == "fresh")
    {
      occ::handle<Units_UnitsDictionary> aFresh = new Units_UnitsDictionary();
      res << aFresh->Sequence().IsNull() << ' ';
      aFresh->Creates();
      res << aFresh->Sequence()->Length();
    }
    res << " n=" << aDict->Sequence()->Length();
  });
}

static bool units_core_probe(const std::string& op, std::istream& in, std::ostream& out)
{
  if (op == "units_sentence") units_sentence_op("unit", in, out);
  else if (op == "units_math") units_sentence_op("math", in, out);
  else if (op == "units_raw") { std::string aKind; in >> aKind; units_sentence_op(aKind, in, out); }
  else if (op == "units_tosi") units_tosi_op(in, out);
  else if (op == "units_convert") units_convert_op(in, out);
  else if (op == "units_quantity") units_quantity_op(in, out);
  else if (op == "units_first") units_first_op(in, out);
  else if (op == "units_dims") units_dims_op(in, out);
  else if (op == "units_basic") units_basic_op(out);
  else if (op == "units_lexicon") units_lexicon_op(in, out);
  else if (op == "units_lexicon_add") units_lexicon_add_op(in, out);
  else if (op == "units_dict") units_dict_op(in, out);
  else return false;
  return true;
}
