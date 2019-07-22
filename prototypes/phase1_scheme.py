import re
from collections import namedtuple

from utils import is_list


def read(input):
    tokens = Tokenizer(SCHEME_TOKENS).iter_tokens(input)
    return parse(tokens)


def stringify(expr):
    if is_list(expr):
        return '(' + ' '.join(stringify(x) for x in expr) + ')'
    else:
        return str(expr)


def parse(tokens):
    token = next(tokens)
    if token.name == 'NIL': return Nil
    elif token.name == 'TRUE': return True
    elif token.name == 'FALSE': return False
    elif token.name == 'SYMBOL': return token.text
    elif token.name == 'NUMBER': return float(token.text)
    elif token.name == 'STRING': return String(token.text[1:-1])
    elif token.name == 'DOT': return Dot
    elif token.name == 'QUOTE': return ['quote', parse(tokens)]
    elif token.name == 'LPAREN':
        sequence = []
        try:
            while True:
                sequence.append(parse(tokens))
        except ListFinished:
            for i, x in enumerate(sequence):
                if x == Dot and i != len(sequence) - 2:
                    raise SyntaxError("too many items in CDR position: {}".format(sequence))
            if sequence[-2:] == [Dot, Nil]:
                return sequence[:-2]
            else:
                return sequence
    elif token.name == 'RPAREN':
        raise ListFinished


class String(str):
    def __str__(self):
        return '"' + self + '"'


class Dot:
    def __str__(self):
        return '.'
Dot = Dot()


class Nil:
    def __str__(self):
        return "'()"
Nil = Nil()

class ListFinished(Exception): pass


SCHEME_TOKENS = [
    ('NIL'        , r"nil|'\(\)"),
    ('TRUE'       , r'true|#t'),
    ('FALSE'      , r'false|#f'),
    ('NUMBER'     , r'\d+'),
    ('STRING'     , r'"(\\.|[^"])*"'),
    ('DOT'        , r'\.'),
    ('QUOTE'      , r"'"),
    ('LPAREN'     , r'\('),
    ('RPAREN'     , r'\)'),
    ('SYMBOL'     , r'[\x21-\x26\x2a-\x7e]+'),
    ('WHITESPACE' , r'\w+'),
  ]


class Tokenizer:
    Token = namedtuple('Token', 'name text span')

    def __init__(self, tokens):
        self.tokens = tokens
        pat_list = []
        for tok, pat in self.tokens:
            pat_list.append('(?P<%s>%s)' % (tok, pat))
        self.re = re.compile('|'.join(pat_list))

    def iter_tokens(self, input, ignore=('WHITESPACE',)):
        for match in self.re.finditer(input):
            if match.lastgroup in ignore:
                continue
            yield Tokenizer.Token(match.lastgroup, match.group(0), match.span(0))

    def tokenize(self, input, ignore=('WHITESPACE',)):
        return list(self.iter_tokens(input, ignore))


if __name__ == '__main__':
    print(stringify(read("(define (sqr x) (* x x))")))