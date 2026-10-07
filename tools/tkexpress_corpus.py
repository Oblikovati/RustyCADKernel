#!/usr/bin/env python3
"""The TKExpress request corpus of tools/oracle/tkexpress_probe.cxx and its Rust twin.

Texts travel as `x` and hexadecimal bytes. The corpus covers the grammar (every rule and
action of ExprIntrp.yacc, the flex rules and the echo of unmatched characters, the
10000-state limit of the Bison stack), the simplification, derivation and evaluation of
every Expr class, the values of the standard functions over a range of inputs, relations,
and the direct API scenarios. A seeded generator adds random token sequences, so the
LALR tables are exercised far from the hand-written cases.

Usage: requests: list[str] = []; append_tkexpress_requests(requests)
"""
from __future__ import annotations
import math
import random
import struct

# Every standard function name ExprIntrp_StandardFunction knows.
STANDARD_NAMES = ['abs', 'Abs', 'acos', 'ACos', 'asin', 'ASin', 'atan', 'ATan', 'acosh', 'ACosh',
                  'asinh', 'ASinh', 'atanh', 'ATanh', 'cosh', 'Cosh', 'cos', 'Cos', 'exp', 'Exp',
                  'log', 'Log', 'Ln', 'sign', 'Sign', 'sin', 'Sin', 'sinh', 'Sinh', 'sqrt', 'Sqrt',
                  'tan', 'Tan', 'tanh', 'Tanh']

# Single texts: tokens, operators, simplifications and errors.
SINGLE_TEXTS = [
    'x', '2', '0.5', '.5', '1e3', '1.5e-2', '1e+2', '2e|3', '1e', '1ex', '00012', '1.2.3', '.', '..',
    '1e400', '1e-400', 'x1_y2', 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa',
    'x+y', 'x-y', 'x*y', 'x/y', 'x^2', 'x**3', '-x', '+x', '--x', '-(-x)', '(x)', '[x+1]', 'x + y',
    'x+y*z^2-w/3', '(x+y)*(x-y)', 'x^y^z', 'x-y-z', 'x/y/z', '2+3', '2*3', '2-2', '2/0', '0/0', '0/x',
    'x/1', 'x^0', 'x^1', '1^x', '2^0.5', '2^-1', '-2^2', '2*x*3*4', '1*x', '-1*x', '0*x', 'x*1*1',
    '2+x+3', 'x+0', '0+x', '0-x', 'x-0', '-x-(-y)', '-x-y', 'x-(-y)', 'x+(-3)', '(x+y)+(z+w)',
    '(x*y)*(z*w)', '2*(3*x)', '(2+x)+(3+y)', 'x*y+x*y', 'x^2^3', '(x^2)^2', 'Sqrt(x)^2',
    'Sin(ASin(x))', 'ASin(Sin(x))', 'Cos(ACos(x))', 'ACos(Cos(x))', 'Tan(ATan(x))', 'ATan(Tan(x))',
    'Exp(Ln(x))', 'Ln(Exp(x))', 'Cosh(ACosh(x))', 'ACosh(Cosh(x))', 'Sinh(ASinh(x))', 'ASinh(Sinh(x))',
    'Tanh(ATanh(x))', 'ATanh(Tanh(x))', 'Sqrt(x*x)', 'Abs(-x)', 'Abs(-3)', 'Abs(x)', 'Sign(-0)',
    'Sign(x)', 'Sign(-2.5)', 'log(100)', 'Log(2)', 'Ln(2)', 'log(x)', 'Sin(x)*Cos(x)', 'Tan(x)/Tanh(y)',
    'ACosh(x)+ASinh(x)+ATanh(x)', 'Exp(2*Sin(x^2))', 'Exp(5*x)', 'x^y', 'y^(x*2)', 'Sqrt(1-x^2)',
    'ASin(x/2)', 'ACos(2*x)', 'ATan(x^2)', 'Sinh(x)*Cosh(y)', 'Exp(-x^2/2)', 'log(x^2+1)',
    'Abs(Sin(x))', 'Sign(x-1)*x', '(x+1)/(x-1)', 'x/(y*z)', 'Sqrt(Sqrt(x))', '1/Sqrt(x)',
    'Deriv[x^3, x]', 'Deriv[Sin(x)*x, x, 3]', 'Deriv[Sin(x), y]', 'Deriv[x, x, 0]', 'Deriv[x^2, x, 2]',
    'Deriv[x*y^2, y, 2]', 'Deriv[Exp(x)/x, x]', 'Deriv[Tan(x), x, 2]', 'Deriv[x, x, .5]',
    'Sin\'(x)', 'Sin\'\'(x^2)', 'Exp\'(2*x)', 'Cos\'\'\'(x)', 'Sin\'(2)', 'Sin\'(x*y)', 'Ln\'(x)',
    'Sqrt\'(x+1)', 'ATan\'\'(x)', 'Abs\'(x)', 'foo\'(x)',
    'Const[Pi, 3.14159]', 'Const[Pi, 3.14]*x', 'Const[e, 2.718]+1', 'Const[K, 1e3]/x',
    'Sum[x^2, x, 1, 1, 4]', 'Sum[x, x, 0, 0.5, 3]', 'Prod[x+1, x, 1, 1, 3]', 'Prod[y, x, 1, 1, 3]',
    'Sum[x*y, x, y, 1, 2]', 'Sum[Sin(x), x, 0, 1, 3]', 'Prod[x, x, 2, 2, 4]', 'Sum[x, x, 1, 1, 1]',
    'x <- 3', 'x <- x+1', 'x <- 2*y', 'Deassign[x]', 'x = y', 'x+1 = 2; y = 3', 'x=1;y=2;z=3',
    'f[x] <- x^2', 'f[x,y] <- x*y', 'f[x] <- x',
    '', 'x+', '(x', 'x)', '1 2', 'Sin', 'Sin()', 'foo(x)', '\t', 'x$y', 'a\nb', 'x#y', 'x<y', 'x < - y',
    'x\0y', '\0x', '@', '@f/@X1(x)', ',', 'x,y', '[x)', 'Deriv[x]', 'Sum[x]', 'x <- ', '<- x', 'x = ',
    'x;y', 'x\'', 'x\'\'(y)', 'é', '(((((((((((x)))))))))))', '-(-(-(-x)))', 'x' + '+x' * 60,
] + [name + '(x)' for name in STANDARD_NAMES] + [name + '(0.25)' for name in STANDARD_NAMES] + [
    name + '(2)' for name in STANDARD_NAMES]

# Sequences of texts on one interpreter: definitions, assignments and their uses.
SEQUENCES = [
    ['x <- 2', 'x+1', 'Deassign[x]', 'x+1'],
    ['x <- y+1', 'x*2', 'y <- 3', 'x*x', 'Deassign[y]', 'x'],
    ['Const[Pi, 3.14159]', 'Pi*2', 'Pi <- 3', 'Deassign[Pi]', 'Deriv[x, Pi]'],
    ['f[x] <- x^2', 'f(3)', 'f(y)', 'f\'(y)', 'f\'\'(2)', 'f(1,2)', 'f(Sin(z))', 'Deriv[f(z), z]'],
    ['g[x,y] <- x*y+x^2', 'g(1,2)', 'g(a,b)', '@g/@X1(a,b)', '@2g/@2X2(a,b)', '@g/@X3(a,b)',
     '@@g/@X1/@X2(a,b)', '@2g/@3X1(a,b)', 'g(a)', 'g\'(a)', '@g/@x2(a,b)', '@g/@Y1(a,b)'],
    ['h[x,y,z] <- x+y*z', 'h(1,2,3)', 'h(a,b,c)', '@h/@X2(a,b,c)', 'h(a,b)', '@h/@X0(a,b,c)'],
    ['f[x] <- 2', 'f(x)', 'f\'(x)', '@f/@X1(x)'],
    ['f[x] <- x', 'f(y)', 'f\'(y)'],
    ['f[x] <- Sin(x)', 'g[y] <- f(y)*y', 'g(2)', 'g(z)', 'g\'(z)'],
    ['x <- 1', 'f[x] <- x+1', 'f(5)', 'x'],
    ['Const[Pi, 3.14]', 'f[Pi] <- 1', 'Pi'],
    ['f[x,y] <- x/y', 'f(1,0)', 'f(0,0)', '@f/@X2(u,v)', 'Deriv[f(u,v), v]'],
    ['f[a,b,c,d] <- a*b-c/d', 'f(1,2,3,4)', '@f/@X4(p,q,r,s)', '@3f/@3X4(p,q,r,s)'],
    ['x = 2', 'x', 'y <- x', 'y'],
    ['Sum[x^2, x, 1, 1, 3]', 'Prod[x, x, 1, 1, 5]', 'Sum[x, x, 0, 0, 2]'],
]

# Relations: pairs of texts (left, right), one system when several.
RELATIONS = [
    ['x', 'y'], ['x+1', '2'], ['2', '2'], ['1+1', '2'], ['x*y', 'y*x'], ['Sin(x)^2+Cos(x)^2', '1'],
    ['x', 'x'], ['x^2', 'y'], ['x+y', '3', 'x-y', '1'], ['x', '1', 'y', '2', 'z', 'x+y'],
    ['2*x', '4', 'x', '2'], ['x/y', '0.5', 'Exp(x)', 'y'], ['Pi', '3'],
]

# The values of `ex_fold` and `ex_fold2`.
SPECIAL_VALUES = [0.0, -0.0, 0.5, -0.5, 1.0, -1.0, 2.0, -2.0, 0.999999, 1.0000001, 10.0, 100.0, 710.0,
                  -710.0, 1e-300, 1e300, math.inf, -math.inf, math.nan, 3.14159, 1e-8, 0.1, 0.7, 1.5,
                  20.5, 4.0, 1e-320, -1e-8, 0.25, 2.5]
FUNCTION_CODES = 20
OPERATOR_CODES = 5

# The tokens of the random texts.
FUZZ_TOKENS = ['x', 'y', 'z', '2', '0.5', '1e2', '3', '+', '-', '*', '/', '^', '**', '(', ')', '[', ']',
               ',', '@', "'", '<-', '=', ';', ' ', 'Sin', 'Cos', 'Exp', 'Ln', 'Sqrt', 'Abs', 'f', 'g',
               'X1', 'X2', 'Deriv', '\t', '#']


def hex_text(text: str) -> str:
    """`x` and the hexadecimal UTF-8 bytes of `text`; hex_text('x') == 'x78'."""
    return 'x' + text.encode('utf-8').hex()


def bits(value: float) -> str:
    """The 64-bit pattern of a double, in decimal; bits(1.0) == '4607182418800017408'."""
    return str(struct.unpack('<Q', struct.pack('<d', value))[0])


def text_requests() -> list[str]:
    """The single texts, the sequences and the Bison stack limit."""
    requests = ['ex_text ' + hex_text(text) for text in SINGLE_TEXTS]
    requests += ['ex_text ' + ' '.join(hex_text(text) for text in sequence) for sequence in SEQUENCES]
    # The stack holds 9999 states at most: each '-' pushes one.
    for count in (9995, 9996, 9997, 9998, 9999, 10000):
        requests.append('ex_text ' + hex_text('-' * count + 'x'))
    requests.append('ex_text ' + hex_text('(' * 4000 + 'x' + ')' * 4000))
    return requests


def fold_requests(generator: random.Random) -> list[str]:
    """Every standard function and operator on the special values and random ones."""
    values = SPECIAL_VALUES + [generator.uniform(-5.0, 5.0) for _ in range(30)]
    values += [generator.uniform(-1.0, 1.0) for _ in range(30)]
    values += [10.0 ** generator.uniform(-10.0, 10.0) for _ in range(20)]
    requests = [f'ex_fold {code} {bits(value)}' for code in range(FUNCTION_CODES) for value in values]
    pairs = [(a, b) for a in SPECIAL_VALUES for b in SPECIAL_VALUES[::3]]
    pairs += [(generator.uniform(-4.0, 4.0), generator.uniform(-4.0, 4.0)) for _ in range(60)]
    requests += [f'ex_fold2 {code} {bits(a)} {bits(b)}' for code in range(OPERATOR_CODES) for a, b in pairs]
    return requests


def fuzz_requests(generator: random.Random, count: int) -> list[str]:
    """Random token sequences, some after the definitions of f and g."""
    requests = []
    for _ in range(count):
        texts = [''.join(generator.choice(FUZZ_TOKENS) for _ in range(generator.randint(1, 12)))
                 for _ in range(generator.randint(1, 3))]
        if generator.random() < 0.3:
            texts = ['f[x] <- x^2+1', 'g[x,y] <- x*y'] + texts
        requests.append('ex_text ' + ' '.join(hex_text(text) for text in texts))
    return requests


def append_tkexpress_requests(requests: list[str]) -> None:
    """Append the whole TKExpress corpus, one request per entry."""
    generator = random.Random(20261008)
    requests += text_requests()
    requests += fold_requests(generator)
    requests += ['ex_rel ' + ' '.join(hex_text(text) for text in relation) for relation in RELATIONS]
    requests += [f'ex_case {n}' for n in range(9)]
    requests += fuzz_requests(generator, 3000)
