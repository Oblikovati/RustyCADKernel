#pragma once

#include "Schema.h"
#include "clang/Lex/PPCallbacks.h"
#include <map>
#include <set>
#include <string>

namespace api_inventory {
class PreprocessorEvents final : public clang::PPCallbacks {
public:
  PreprocessorEvents(const Config &config, clang::SourceManager &sources,
                     const clang::LangOptions &language);
  void MacroDefined(const clang::Token &name,
                    const clang::MacroDirective *directive) override;
  void Ifndef(clang::SourceLocation location, const clang::Token &name,
              const clang::MacroDefinition &definition) override;
  void SourceRangeSkipped(clang::SourceRange range,
                         clang::SourceLocation endifLocation) override;
  void InclusionDirective(clang::SourceLocation hashLocation,
                          const clang::Token &includeToken,
                          llvm::StringRef fileName,
                          bool isAngled,
                          clang::CharSourceRange filenameRange,
                          clang::OptionalFileEntryRef file,
                          llvm::StringRef searchPath,
                          llvm::StringRef relativePath,
                          const clang::Module *imported,
                          bool isModuleImport,
                          clang::SrcMgr::CharacteristicKind fileType) override;
  void emitHeaders() const;

private:
  const Config &config_;
  clang::SourceManager &sources_;
  const clang::LangOptions &language_;
  std::set<std::filesystem::path> seenHeaders_;
  std::map<std::filesystem::path, std::set<std::string>> includeGuardNames_;
};
} // namespace api_inventory
