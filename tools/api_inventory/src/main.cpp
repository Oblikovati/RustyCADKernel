#include "InventoryVisitor.h"
#include "PreprocessorEvents.h"
#include "clang/Basic/Diagnostic.h"
#include "clang/Frontend/CompilerInstance.h"
#include "clang/Frontend/FrontendAction.h"
#include "clang/Lex/Preprocessor.h"
#include "clang/Tooling/Tooling.h"
#include "llvm/Support/FormatVariadic.h"
#include "llvm/Support/raw_ostream.h"
#include <memory>
#include <string>
#include <vector>

#ifndef CLANG_DRIVER_PATH
#define CLANG_DRIVER_PATH "clang++"
#endif
#ifndef CLANG_RESOURCE_DIR
#define CLANG_RESOURCE_DIR ""
#endif

namespace api_inventory {
namespace {
struct RunState {
  bool unhandledDeclKind = false;
  unsigned diagnosticErrors = 0;
};

std::string severityName(clang::DiagnosticsEngine::Level level) {
  switch (level) {
  case clang::DiagnosticsEngine::Ignored: return "ignored";
  case clang::DiagnosticsEngine::Note: return "note";
  case clang::DiagnosticsEngine::Remark: return "remark";
  case clang::DiagnosticsEngine::Warning: return "warning";
  case clang::DiagnosticsEngine::Error: return "error";
  case clang::DiagnosticsEngine::Fatal: return "fatal";
  }
  return "unknown";
}

class JsonDiagnosticConsumer final : public clang::DiagnosticConsumer {
public:
  explicit JsonDiagnosticConsumer(const Config &config) : config_(config) {}

  void HandleDiagnostic(clang::DiagnosticsEngine::Level level,
                        const clang::Diagnostic &diagnostic) override {
    clang::DiagnosticConsumer::HandleDiagnostic(level, diagnostic);
    llvm::SmallString<256> message;
    diagnostic.FormatDiagnostic(message);
    llvm::json::Object event{{"event", "diagnostic"},
                             {"main_header", config_.mainHeader.string()},
                             {"severity", severityName(level)},
                             {"message", message.str().str()}};
    if (diagnostic.hasSourceManager())
      event["location"] = locationObject(diagnostic.getLocation(),
                                          diagnostic.getSourceManager());
    writeEvent(std::move(event));
    if (level >= clang::DiagnosticsEngine::Error) ++errors_;
  }

  unsigned errors() const { return errors_; }

private:
  const Config &config_;
  unsigned errors_ = 0;
};

class InventoryConsumer final : public clang::ASTConsumer {
public:
  InventoryConsumer(clang::ASTContext &context, const Config &config,
                    RunState &state)
      : visitor_(context, config), state_(state) {}

  void HandleTranslationUnit(clang::ASTContext &context) override {
    visitor_.TraverseDecl(context.getTranslationUnitDecl());
    visitor_.emitSummary();
    state_.unhandledDeclKind = visitor_.hasUnhandledKinds();
  }

private:
  InventoryVisitor visitor_;
  RunState &state_;
};

class InventoryAction final : public clang::ASTFrontendAction {
public:
  InventoryAction(const Config &config, RunState &state,
                  JsonDiagnosticConsumer &diagnostics)
      : config_(config), state_(state), diagnostics_(diagnostics) {}

  bool BeginSourceFileAction(clang::CompilerInstance &compiler) override {
    compiler.getDiagnostics().setClient(&diagnostics_, false);
    return true;
  }

  std::unique_ptr<clang::ASTConsumer>
  CreateASTConsumer(clang::CompilerInstance &compiler,
                    llvm::StringRef) override {
    auto callbacks = std::make_unique<PreprocessorEvents>(
        config_, compiler.getSourceManager(), compiler.getLangOpts());
    events_ = callbacks.get();
    compiler.getPreprocessor().addPPCallbacks(std::move(callbacks));
    return std::make_unique<InventoryConsumer>(compiler.getASTContext(),
                                               config_, state_);
  }

  void EndSourceFileAction() override {
    if (events_) events_->emitHeaders();
    state_.diagnosticErrors = diagnostics_.errors();
  }

private:
  const Config &config_;
  RunState &state_;
  JsonDiagnosticConsumer &diagnostics_;
  PreprocessorEvents *events_ = nullptr;
};

bool parseArguments(int argc, char **argv, Config &config,
                    std::vector<std::string> &compilerArguments,
                    std::string &error) {
  bool compilerMode = false;
  for (int index = 1; index < argc; ++index) {
    const std::string argument = argv[index];
    if (argument == "--") {
      compilerMode = true;
      continue;
    }
    if (compilerMode) {
      compilerArguments.push_back(argument);
      continue;
    }
    if (argument == "--source-root" || argument == "--main-header" ||
        argument == "--emit-header") {
      if (++index >= argc) {
        error = "missing value after " + argument;
        return false;
      }
      const auto path = canonicalPath(argv[index]);
      if (argument == "--source-root") config.sourceRoot = path;
      if (argument == "--main-header") config.mainHeader = path;
      if (argument == "--emit-header") config.extraEmitHeaders.insert(path);
      continue;
    }
    error = "unexpected argument " + argument + "; expected --source-root, --main-header, --emit-header, then -- compiler flags";
    return false;
  }
  if (config.sourceRoot.empty() || config.mainHeader.empty()) {
    error = "source root and main header are required";
    return false;
  }
  return true;
}

std::vector<std::string> frontendArguments(
    std::vector<std::string> compilerArguments) {
  if (compilerArguments.empty()) compilerArguments.push_back("-std=c++17");
  compilerArguments.push_back("-fsyntax-only");
  compilerArguments.push_back("-resource-dir=" CLANG_RESOURCE_DIR);
  return compilerArguments;
}

std::string includeCode(const Config &config) {
  return "#include \"" + config.mainHeader.string() + "\"\n";
}
} // namespace
} // namespace api_inventory

int main(int argc, char **argv) {
  api_inventory::Config config;
  std::vector<std::string> compilerArguments;
  std::string error;
  if (!api_inventory::parseArguments(argc, argv, config, compilerArguments, error)) {
    llvm::errs() << error << "\n";
    return 2;
  }
  api_inventory::RunState state;
  api_inventory::JsonDiagnosticConsumer diagnostics(config);
  std::unique_ptr<clang::FrontendAction> action =
      std::make_unique<api_inventory::InventoryAction>(config, state, diagnostics);
  const bool parsed = clang::tooling::runToolOnCodeWithArgs(
      std::move(action), api_inventory::includeCode(config),
      api_inventory::frontendArguments(std::move(compilerArguments)),
      "occt-api-extract-main.cxx", CLANG_DRIVER_PATH);
  if (!parsed) return 1;
  if (state.unhandledDeclKind) return 3;
  return state.diagnosticErrors == 0 ? 0 : 1;
}
