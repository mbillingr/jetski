from utils import gensym, is_list, match
from phase1_scheme import stringify


def Tc(expr, c):
    if is_atomic(expr):
        return (c, M(expr))
    elif expr[0] == 'begin' and len(expr) == 2:
        return Tc(expr[1], c)
    elif expr[0] == 'begin':
        return Tk(expr[1], lambda _: Tc(('begin',) + expr[2:], c))
    elif expr[0] == 'if':
        k = gensym('k', namespace='cps')
        return (('lambda', (k,), Tk(expr[1], lambda aexp: ('if', aexp,
                                                           Tc(expr[2], k),
                                                           Tc(expr[3], k)))),
                c)
    elif expr[0] == 'set!':
        return Tk(expr[2],
                  lambda aexp: ('set-then!', expr[1], aexp, (c, 'undef')))
    # WARNING: This transformation is not hygienic if the continuation c
    #          references any of the bound variables!
    # Use this transformation only on alphatized programs!
    elif expr[0] == 'letrec':
        return (
         'letrec', tuple((v, M(a)) for v, a in expr[1]), Tc(expr[2], c))
    elif is_primitive(expr[0]):
        return Tsk(expr[1:], lambda es_: (('cps', expr[0])) + es_ + (c,))
    else:
        f, es = expr[0], expr[1:]
        return Tk(f, lambda f_: Tsk(es, lambda es_: (f_,) + es_ + (c,)))


def Tk(expr, k):
    if not is_list(expr):
        return k(M(expr))
    elif expr[0] == 'lambda':
        return k(M(expr))

    elif expr[0] == 'if':
        rv = gensym('rv', namespace='cps')
        cont = ('lambda', (rv,), k(rv))
        k = gensym('k', namespace='cps')
        return (('lambda', (k,), Tk(expr[1], lambda aexp: ('if', aexp,
                                                           Tc(expr[2], k),
                                                           Tc(expr[3], k)))),
                cont)
    # WARNING: This transformation is not hygienic if the continuation c
    #          references any of the bound variables!
    # Use this transformation only on alphatized programs!
    elif expr[0] == 'letrec':
        rv = gensym('rv', namespace='cps')
        cont = ('lambda', (rv,), k(rv))
        return (
            'letrec', tuple((v, M(a)) for v, a in expr[1]), Tc(expr[2], cont))
    elif is_primitive(expr[0]):
        rv = gensym('rv', namespace='cps')
        cont = ('lambda', (rv,), k(rv))
        return Tsk(expr[1:], lambda es_: (('cps', expr[0])) + es_ + (cont,))
    else:
        f, es = expr[0], expr[1:]
        rv = gensym('rv', namespace='cps')
        cont = ('lambda', (rv,), k(rv))
        return Tk(f, lambda f_: Tsk(es, lambda es_: (f_,) + es_+ (cont,)))


def Tsk(exprs, k):
    if len(exprs) == 0:
        return k(())
    return Tk(exprs[0], lambda hd: Tsk(exprs[1:], lambda tl: k((hd,) + tl)))


def M(expr):
    lam = match(expr, ('lambda', ('..args',), '..body'))
    if lam is not None:
        k = gensym('k', namespace='cps')
        return ['lambda', lam['args'] + (k,), Tc(('begin',) + lam['body'], k)]
    else:
        # TODO: distinguish non-atomic expressions and raise an error
        return expr


def is_atomic(expr):
    return (not is_list(expr)) or expr[0] == 'lambda'


def is_primitive(expr):
    return expr in ['+', '-', '*', '/']


if __name__ == '__main__':
    print(Tc(('g', 'a'), 'halt'))
    print(stringify(Tc(('begin', ('print', '"A"'), ('print', '"B"'), ('print', '"C"')), 'halt')))
    print(stringify(M(('lambda', ('x',), ('begin', ('print', '"A"'), ('print', '"B"'), ('print', '"C"'))))))
    print(stringify(Tc((('lambda', ('x', 'y'), ('print', 'x'), ('print', 'y')), 5, 3), 'print')))
    print(stringify(Tc((('lambda', ('x', 'y'), ('sqrt', ('+', ('*', 'x', 'x'), ('*', 'y', 'y')))), 2, 3), 'print')))
    print(stringify(Tc(('letrec', (('a', 42), ('b', ('+', 1, 2))), ('+', 'a', 'b')), 'print')))

    # This should print the value of x passed to the lambda and 3
    # but after the transformation it will print 3 3
    # because letrec rebinds x and the print is moved into the continuation of letrec.
    # This demonstrates the unhygienic behavior of the letrec transform
    print(stringify(M(('lambda', ('x',), ('print', 'x', ('letrec', (('x', 3),), 'x'))))))

    print(stringify(M(('lambda', ('x',), ('if', ('even?', 'x'), ('print', '"even"'), ('print', '"odd"'))))))
    print(stringify(M(('lambda', ('x',), ('print', ('if', ('even?', 'x'), '"even"', '"odd"'))))))
