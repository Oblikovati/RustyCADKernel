// Rust translation of the parser driver of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp.tab.c
// (yyparse of the Bison 3.7.4 yacc.c skeleton) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The LALR(1) parser of the expression grammar (`ExprIntrpparse`).
//!
//! It walks the Bison tables exactly as `yyparse` does: a state reads its lookahead
//! only when its `YYPACT` entry asks for one, so each semantic action sees the
//! scanner results the native one sees. The grammar declares no `error` token and
//! `ExprIntrperror` throws, so the first syntax error ends the parse with an
//! ExprIntrp_SyntaxError; there is no error recovery to port. The semantic values
//! are never read, so no value stack is kept.
use super::context::{ExprIntrpParseContext, expr_intrp_error};
use super::tables::{
    YY_CHECK, YY_DEFACT, YY_DEFGOTO, YY_FINAL, YY_INITDEPTH, YY_LAST, YY_MAXDEPTH, YY_MAXUTOK,
    YY_NTOKENS, YY_PACT, YY_PACT_NINF, YY_PGOTO, YY_R1, YY_R2, YY_TABLE, YY_TRANSLATE,
};
use super::token::ExprIntrpTokenKind;
use rustycad_tkernel::standard::Failure;

/// The action of an expression rule (the first half of the `switch (yyn)` of
/// `yyparse`); `None` when the rule has none here.
fn expression_action(
    rule: usize,
    context: &mut ExprIntrpParseContext<'_>,
) -> Option<Result<(), Failure>> {
    let outcome: Result<(), Failure> = match rule {
        5 => context.end_of_func_def(),
        6 => context.end_of_relation(),
        7 => context.assign_variable(),
        8 => context.end_of_assign(),
        9 => context.deassign(),
        11 => context.sum_operator(),
        12 => context.minus_operator(),
        13 => context.product_operator(),
        14 => context.divide_operator(),
        15 => context.exp_operator(),
        18 => context.unary_minus_operator(),
        19 => context.unary_plus_operator(),
        27 | 52 | 61 | 64 => context.variable_identifier(),
        28 | 62 | 65 => context.num_value(),
        33 | 34 | 49 | 50 => context.count_argument(),
        70 => context.end_of_equal(),
        _ => return None,
    };
    Some(outcome)
}

/// The action of a function, derivation or series rule (the second half of the
/// `switch (yyn)`); `None` when the rule has none here.
fn definition_action(
    rule: usize,
    context: &mut ExprIntrpParseContext<'_>,
) -> Option<Result<(), Failure>> {
    let outcome: Result<(), Failure> = match rule {
        29 => context.end_function(),
        30 => context.end_der_function(),
        31 => context.end_differential(),
        32 => context.end_diff_function(),
        36 | 47 => context.start_function(),
        37 => context.define_function(),
        39 => context.start_derivate(),
        40 => context.end_derivate(),
        41 => context.diff_var(),
        42 => context.diff_degree(),
        43 => context.ver_diff_degree(),
        44 => context.diff_degree_var(),
        45 => context.start_differential(),
        53 | 55 => context.derivation(),
        54 | 57 => context.end_derivation(),
        56 => context.derivation_value(),
        58 => context.constant_identifier(),
        59 => context.constant_definition(),
        63 => context.sumator(),
        66 => context.productor(),
        _ => return None,
    };
    Some(outcome)
}

/// Run the semantic action of `rule`, if it has one.
fn run_action(rule: usize, context: &mut ExprIntrpParseContext<'_>) -> Result<(), Failure> {
    if let Some(outcome) = expression_action(rule, context) {
        return outcome;
    }
    definition_action(rule, context).unwrap_or(Ok(()))
}

/// `YYTRANSLATE`: the symbol number of an external token number.
fn translate(token: i32) -> i32 {
    if !(0..=YY_MAXUTOK).contains(&token) {
        return 2; // YYSYMBOL_YYUNDEF
    }
    i32::from(YY_TRANSLATE[token as usize])
}

/// `yytable[index]` when `yycheck[index] == symbol` and `index` is in the table.
fn checked_entry(index: i32, symbol: i32) -> Option<i32> {
    if !(0..=YY_LAST).contains(&index) || i32::from(YY_CHECK[index as usize]) != symbol {
        return None;
    }
    Some(i32::from(YY_TABLE[index as usize]))
}

/// What the parser does next in a state.
enum ParserStep {
    Shift(i32),
    Reduce(usize),
    Error,
}

/// The parser of one input (`ExprIntrpparse`), over a parse context;
/// `ExprIntrpParser::new(context).parse()?`.
pub struct ExprIntrpParser<'c, 'a> {
    context: &'c mut ExprIntrpParseContext<'a>,
    states: Vec<i32>,
    lookahead: i32,
}
impl<'c, 'a> ExprIntrpParser<'c, 'a> {
    /// A parser at its initial state over `context`; `ExprIntrpParser::new(&mut context)`.
    pub fn new(context: &'c mut ExprIntrpParseContext<'a>) -> Self {
        let mut states: Vec<i32> = Vec::with_capacity(YY_INITDEPTH);
        states.push(0);
        Self {
            context,
            states,
            lookahead: ExprIntrpTokenKind::Yyempty as i32,
        }
    }
    /// Parse the whole input (`ExprIntrpparse`): `Ok(0)` on acceptance; the first syntax
    /// error, the stack exhaustion or a failing action is the error;
    /// `ExprIntrpParser::new(&mut context).parse()? == 0`.
    pub fn parse(&mut self) -> Result<i32, Failure> {
        loop {
            // yysetstate: the stack holds YYMAXDEPTH states at most.
            if self.states.len() >= YY_MAXDEPTH {
                return Err(expr_intrp_error("memory exhausted"));
            }
            let state: i32 = self.top();
            if state == YY_FINAL {
                return Ok(0);
            }
            match self.step(state) {
                ParserStep::Shift(next) => {
                    self.lookahead = ExprIntrpTokenKind::Yyempty as i32;
                    self.states.push(next);
                }
                ParserStep::Reduce(rule) => self.reduce(rule)?,
                ParserStep::Error => return Err(expr_intrp_error("syntax error")),
            }
        }
    }
    /// The current state.
    fn top(&self) -> i32 {
        self.states.last().copied().unwrap_or(0)
    }
    /// `yybackup` and `yydefault`: shift, reduce or fail in `state`, reading the
    /// lookahead only when the state needs it.
    fn step(&mut self, state: i32) -> ParserStep {
        let base: i32 = i32::from(YY_PACT[state as usize]);
        if base != YY_PACT_NINF {
            let symbol: i32 = self.lookahead_symbol();
            if let Some(action) = checked_entry(base + symbol, symbol) {
                return match action {
                    next if next > 0 => ParserStep::Shift(next),
                    rule => ParserStep::Reduce((-rule) as usize),
                };
            }
        }
        match YY_DEFACT[state as usize] {
            0 => ParserStep::Error,
            rule => ParserStep::Reduce(rule as usize),
        }
    }
    /// The symbol of the lookahead, read from the scanner when none is held.
    fn lookahead_symbol(&mut self) -> i32 {
        if self.lookahead == ExprIntrpTokenKind::Yyempty as i32 {
            self.lookahead = self.context.scanner.lex();
        }
        if self.lookahead <= ExprIntrpTokenKind::Yyeof as i32 {
            self.lookahead = ExprIntrpTokenKind::Yyeof as i32;
            return 0;
        }
        translate(self.lookahead)
    }
    /// `yyreduce`: run the action of `rule`, pop its right side and go to the state
    /// of its left side.
    fn reduce(&mut self, rule: usize) -> Result<(), Failure> {
        run_action(rule, self.context)?;
        let length: usize = YY_R2[rule] as usize;
        self.states
            .truncate(self.states.len().saturating_sub(length));
        let left: usize = (i32::from(YY_R1[rule]) - YY_NTOKENS) as usize;
        let state: i32 = self.top();
        let next: i32 = checked_entry(i32::from(YY_PGOTO[left]) + state, state)
            .unwrap_or_else(|| i32::from(YY_DEFGOTO[left]));
        self.states.push(next);
        Ok(())
    }
}
