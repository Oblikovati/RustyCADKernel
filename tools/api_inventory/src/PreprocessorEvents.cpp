#include "PreprocessorEvents.h"
#include "clang/Lex/Lexer.h"
#include "clang/Lex/MacroInfo.h"
#include "clang/Lex/Preprocessor.h"

namespace api_inventory {
namespace {
std::filesystem::path fileAt(clang::SourceLocation location,
                             const clang::SourceManager &sources) {
  auto fileLocation = sources.getFileLoc(location);
  return canonicalPath(sources.getFilename(fileLocation).str());
}

std::string macroReplacement(const clang::MacroInfo &macro,
                             const clang::SourceManager &sources,
                             const clang::LangOptions &language) {
  std::string replacement;
  for (const clang::Token &token : macro.tokens()) {
    if (!replacement.empty()) replacement.push_back(' ');
    replacement += clang::Lexer::getSpelling(token, sources, language);
  }
  return replacement;
}

std::string macroName(const clang::Token &token,
                      const clang::SourceManager &sources,
                      const clang::LangOptions &language) {
  return clang::Lexer::getSpelling(token, sources, language);
}

llvm::json::Object macroEvent(const clang::Token &name,
                              const clang::MacroInfo &macro,
                              const clang::SourceManager &sources,
                              const clang::LangOptions &language,
                              const Config &config,
                              bool includeGuard) {
  llvm::json::Array parameters;
  if (macro.isFunctionLike())
    for (const clang::IdentifierInfo *parameter : macro.params())
      parameters.push_back(parameter ? parameter->getName().str() : "");
  return llvm::json::Object{
      {"event", "macro"},
      {"source", fileAt(macro.getDefinitionLoc(), sources).string()},
      {"main_header", config.mainHeader.string()},
      {"name", macroName(name, sources, language)},
      {"function_like", macro.isFunctionLike()},
      {"variadic", macro.isVariadic()},
      {"parameters", std::move(parameters)},
      {"replacement", macroReplacement(macro, sources, language)},
      {"location", locationObject(macro.getDefinitionLoc(), sources)},
      {"macro_category", includeGuard ? "include_guard" : "source_macro"},
      {"api_candidate", !includeGuard},
      {"exclusion_reason", includeGuard ? "include_guard" : ""}};
}
} // namespace

PreprocessorEvents::PreprocessorEvents(const Config &config,
                                       clang::SourceManager &sources,
                                       const clang::LangOptions &language)
    : config_(config), sources_(sources), language_(language) {
  seenHeaders_.insert(config_.mainHeader);
}

void PreprocessorEvents::Ifndef(clang::SourceLocation location,
                                const clang::Token &name,
                                const clang::MacroDefinition &) {
  includeGuardNames_[fileAt(location, sources_)].insert(
      macroName(name, sources_, language_));
}

void PreprocessorEvents::MacroDefined(const clang::Token &name,
                                      const clang::MacroDirective *directive) {
  if (!directive || !directive->getMacroInfo()) return;
  const clang::MacroInfo &macro = *directive->getMacroInfo();
  const auto path = fileAt(macro.getDefinitionLoc(), sources_);
  const std::string spelling = macroName(name, sources_, language_);
  if (!shouldEmit(path, config_)) return;
  const bool includeGuard = includeGuardNames_[path].find(spelling) !=
                            includeGuardNames_[path].end();
  writeEvent(macroEvent(name, macro, sources_, language_, config_, includeGuard));
}

void PreprocessorEvents::SourceRangeSkipped(clang::SourceRange range,
                                             clang::SourceLocation endifLocation) {
  const auto source = fileAt(range.getBegin(), sources_);
  if (!shouldEmit(source, config_)) return;
  writeEvent(llvm::json::Object{
      {"event", "skipped_range"},
      {"source", source.string()},
      {"main_header", config_.mainHeader.string()},
      {"start", locationObject(range.getBegin(), sources_)},
      {"end", locationObject(range.getEnd(), sources_)},
      {"endif_location", locationObject(endifLocation, sources_)}});
}

void PreprocessorEvents::InclusionDirective(
    clang::SourceLocation, const clang::Token &, llvm::StringRef, bool,
    clang::CharSourceRange, clang::OptionalFileEntryRef file,
    llvm::StringRef, llvm::StringRef, const clang::Module *, bool,
    clang::SrcMgr::CharacteristicKind) {
  if (!file) return;
  seenHeaders_.insert(canonicalPath(file->getName().str()));
}

void PreprocessorEvents::emitHeaders() const {
  for (const auto &path : seenHeaders_) writeEvent(headerSeenEvent(path, config_));
}
} // namespace api_inventory
