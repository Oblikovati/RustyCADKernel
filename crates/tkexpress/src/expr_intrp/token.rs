// Rust translation of ModelingAlgorithms/TKExpress/ExprIntrp/ExprIntrp.tab.h at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The token kinds (`enum yytokentype`) that the scanner returns to the parser.

/// `enum yytokentype` of the Bison parser: the external token numbers;
/// `ExprIntrpTokenKind::Sumop as i32 == 258`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum ExprIntrpTokenKind {
    /// `YYEMPTY`: no lookahead read yet.
    Yyempty = -2,
    /// `YYEOF`: the end of the input.
    Yyeof = 0,
    /// `YYerror`: the error token.
    Yyerror = 256,
    /// `YYUNDEF`: an invalid token.
    Yyundef = 257,
    /// `SUMOP`: `+`.
    Sumop = 258,
    /// `MINUSOP`: `-`.
    Minusop = 259,
    /// `DIVIDEOP`: `/`.
    Divideop = 260,
    /// `EXPOP`: `^` or `**`.
    Expop = 261,
    /// `MULTOP`: `*`.
    Multop = 262,
    /// `PARENTHESIS`: `(`.
    Parenthesis = 263,
    /// `BRACKET`: `[`.
    Bracket = 264,
    /// `ENDPARENTHESIS`: `)`.
    Endparenthesis = 265,
    /// `ENDBRACKET`: `]`.
    Endbracket = 266,
    /// `VALUE`: a number.
    Value = 267,
    /// `IDENTIFIER`: a name.
    Identifier = 268,
    /// `COMMA`: `,`.
    Comma = 269,
    /// `DIFFERENTIAL`: `@`.
    Differential = 270,
    /// `DERIVATE`: one or more `'`.
    Derivate = 271,
    /// `DERIVKEY`: `Deriv`.
    Derivkey = 272,
    /// `ASSIGNOP`: `<-`.
    Assignop = 273,
    /// `DEASSIGNKEY`: `Deassign`.
    Deassignkey = 274,
    /// `EQUALOP`: `=`.
    Equalop = 275,
    /// `RELSEPARATOR`: `;`.
    Relseparator = 276,
    /// `CONSTKEY`: `Const`.
    Constkey = 277,
    /// `SUMKEY`: `Sum`.
    Sumkey = 278,
    /// `PRODKEY`: `Prod`.
    Prodkey = 279,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_numbers_follow_the_bison_header() {
        assert_eq!(ExprIntrpTokenKind::Yyempty as i32, -2);
        assert_eq!(ExprIntrpTokenKind::Yyeof as i32, 0);
        assert_eq!(ExprIntrpTokenKind::Sumop as i32, 258);
        assert_eq!(ExprIntrpTokenKind::Prodkey as i32, 279);
    }
}
