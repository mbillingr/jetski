

def gensym(name, namespace=None):
    global _gensym_counters
    count = _gensym_counters.setdefault((name, namespace), 0)
    new_name = name + str(count)
    _gensym_counters[name, namespace] = count + 1
    return new_name
_gensym_counters = {}


def match(expr, pattern):
    if is_list(pattern) and is_list(expr):
        return match_list(expr, pattern)
    else:
        return match_element(expr, pattern)


def match_list(expr, pattern):
    bindings = {}
    for i, (p, e) in enumerate(zip(pattern, expr)):
        if isinstance(p, str) and p.startswith('..'):
            bindings[p[2:]] = expr[i:]
            break
        inner_match = match(e, p)
        if inner_match is None:
            return None
        bindings.update(inner_match)
    return bindings


def match_element(expr, pattern):
    if pattern == '_':
        pass
    elif pattern == expr:
        pass
    elif isinstance(pattern, str) and pattern[0] == '$':
        return {pattern[1:]: expr}
    else:
        return None
    return {}


def is_list(expr):
    return isinstance(expr, list) or isinstance(expr, tuple)


if __name__ == '__main__':
    from phase1_scheme import read

    print(match(read('(define (sqr x) (* x x))'), ('define', ('$name', '..args'), '$body')))
