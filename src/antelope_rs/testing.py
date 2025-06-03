'''
Fast, test-oriented generators for Antelope “std” types with RNG-aware cache.

* `random_std_type(type_name, rng=<Random>)`
  - deterministic if you supply your own `random.Random`.

The dispatch table for a given RNG is built at most **once** and then looked
up from `_GEN_CACHE` (a `weakref.WeakKeyDictionary`).  The default global RNG
gets its own immutable table `_GLOBAL_GENERATORS`.

'''

from __future__ import annotations

import os
import json
import random
import string
import weakref
import logging

from typing import Callable

from deepdiff import DeepDiff

from antelope_rs import (
    IOTypes,
    Symbol,
    ABIView,
    builtin_types,
    builtin_class_map
)
from antelope_rs.abi import (
    TypeModifier,
    suffix_for
)


logger = logging.getLogger(__name__)


inside_ci = any(
    os.getenv(v)
    for v in [
        'CI', 'GITHUB_ACTIONS', 'GITLAB_CI', 'TRAVIS', 'CIRCLECI'
    ]
)


class AntelopeDebugEncoder(json.JSONEncoder):
    def default(self, o):
        # hex string on bytes
        if isinstance(o, (bytes, bytearray)):
            return f'bytes({o.hex()})'

        if isinstance(o, type):
            return f'type({str(o)}'

        if hasattr(o, '__str__'):
            return str(o)

        try:
            return super().default(o)

        except Exception:
            return f'err({type(o).__name__})'


# deepdiff powered type comparator

def make_canonical(abi: ABIView, val: IOTypes, type_name: str) -> IOTypes:
    """
    Convert *val* to a canonical representation according to *type_name*.

    Handles:
      - array (`[]`), optional (`?`) and extension (`$`) suffixes
      - ABI aliases
      - ABI structs (recursive on every field)
      - built-in wrappers via `try_from`
    """
    # peel modifiers ([], ?, $)
    if type_name.endswith('[]'):  # array
        if not isinstance(val, list):
            raise TypeError(f'Expected a list for {type_name}')
        type_name = type_name[:-2]
        return [make_canonical(abi, v, type_name) for v in val]

    if type_name[-1] in ('?', '$'):  # optional / extension
        type_name = type_name[:-1]
        return (
            None
            if val is None
            else make_canonical(abi, val, type_name)
        )

    # follow aliases
    alias = abi.alias_map.get(type_name)
    if alias:
        return make_canonical(abi, val, alias)

    # built-ins
    builtin_cls = builtin_class_map.get(type_name)
    if builtin_cls:
        return builtin_cls.try_from(val)

    # variants
    variant = abi.variant_map.get(type_name)
    if variant:
        # struct variant
        if isinstance(val, dict):
            tag  = val['type']
            if tag not in variant.types:
                raise ValueError(
                    f'Unknown tag "{tag}" for variant "{type_name}"'
                )
            canon = make_canonical(abi, val, tag)
            if not isinstance(canon, dict):
                raise TypeError(f'Expected a dict for variant {type_name}')
            return {'type': tag, **canon}

        else:
            # std type variant
            candidates = []
            for t in variant.types:
                meta = abi.resolve_type(t)
                if meta.is_std:
                    candidates.append(t)

            if len(candidates) != 1:
                raise ValueError(
            f'Ambiguous value for variant "{type_name}"'
                )

            tag= candidates[0]
            return make_canonical(abi, val, tag)

    # structs
    sdef = abi.struct_map.get(type_name)
    if sdef:
        if not isinstance(val, dict):
            raise TypeError(
                f'Expected dict for struct "{type_name}", got {type(val).__name__}'
            )

        out = {}

        # flatten inheritance
        if sdef.base:
            base_obj = make_canonical(abi, val, sdef.base)
            if not isinstance(base_obj, dict):
                raise TypeError(
                    f'Expected dict for struct base "{sdef.base}", got {type(base_obj).__name__}'
                )
            out.update(base_obj)

        for f in sdef.fields:  # canonicalise every field
            field_val = val.get(f.name)
            out[f.name] = make_canonical(abi, field_val, f.type_)

        return out

    # passthrough for primitives / unknowns
    return val

ABIView.make_canonical = make_canonical

def canonical_diff(
    abi: ABIView,
    type_name: str,
    old: IOTypes,
    new: IOTypes,
    **kwargs,
) -> DeepDiff:
    '''
    Deep compare two IOTypes based on an ABI type.

    '''
    _kwargs: dict = {
        'ignore_order': True,
        'ignore_encoding_errors': True,
        'significant_digits': 4,
    }
    _kwargs.update(kwargs)

    old_can = make_canonical(abi, old, type_name)
    new_can = make_canonical(abi, new, type_name)

    return DeepDiff(old_can, new_can, **_kwargs)

ABIView.canonical_diff = canonical_diff

def assert_deep_eq(
    abi: ABIView,
    type_name: str,
    old: IOTypes,
    new: IOTypes,
    **kwargs,
) -> None:
    '''
    Assert no differences found between the canonical form of old & new

    '''
    diff = abi.canonical_diff(type_name, old, new, **kwargs)
    if diff:
        raise AssertionError(f'Differences found:\n{diff.pretty()}')


ABIView.assert_deep_eq = assert_deep_eq


# ABI types generators

_LETTERS = string.ascii_letters


def _randbits(rng, bits: int, nonzero: bool = False) -> int:
    num = rng.getrandbits(bits)
    return (
        num
        if not nonzero
        else max(1, num)
    )


def _rand(rng) -> float:
    return rng.random()


def _randbytes(rng, n: int) -> bytes:
    return rng.randbytes(n)


def _signed_int(rng, bits: int) -> int:
    val = _randbits(rng, bits)
    sign_bit = 1 << (bits - 1)
    return val - (sign_bit << 1) if val & sign_bit else val


def _randsym(rng) -> int:
    while True:
        try:
            sym = _randbits(rng, 64, nonzero=True)
            Symbol.from_int(sym)
            return sym

        except ValueError:
            continue


def _make_generators(rng) -> dict[str, Callable[[], IOTypes]]:
    '''
    Build a fresh dispatch table that closes over *rng*.

    Called exactly once per unique RNG thanks to the cache below.
    '''
    return {
        # booleans
        'bool': lambda: bool(_randbits(rng, 1)),

        # unsigned ints
        **{f'uint{b}': (lambda b=b: _randbits(rng, b))
           for b in (8, 16, 32, 64, 128)},

        # signed ints
        **{f'int{b}': (lambda b=b: _signed_int(rng, b))
           for b in (8, 16, 32, 64, 128)},

        # LEB128 helpers
        'varuint32': lambda: _randbits(rng, 32),
        'varint32': lambda: _signed_int(rng, 32),

        # floats in small ranges
        'float32': lambda: _rand(rng) * 2.0 - 1.0,
        'float64': lambda: _rand(rng) * 2.0e4 - 1.0e4,

        # float128 just 16 random bytes
        'float128': lambda: _randbytes(rng, 16),

        # time
        'time_point': lambda: _randbits(rng, 64),
        'time_point_sec': lambda: _randbits(rng, 32),
        'block_timestamp_type': lambda: _randbits(rng, 32),

        # account name
        'name': lambda: _randbits(rng, 64),

        # blobs & ASCII strings
        'bytes': lambda: _randbytes(rng, _randbits(rng, 4) & 0x0F),
        'string': lambda: ''.join(
            rng.choices(
                _LETTERS,
                k=(_randbits(rng, 4) & 0x0C) | _randbits(rng, 2)
            )
        ),

        # checksums
        'checksum160': lambda: _randbytes(rng, 20),
        'checksum256': lambda: _randbytes(rng, 32),
        'checksum512': lambda: _randbytes(rng, 64),

        'public_key': lambda: b'\0' + _randbytes(rng, 33),
        'signature': lambda: b'\0' + _randbytes(rng, 65),

        # asset related
        'symbol': lambda: _randsym(rng),
        'symbol_code': lambda: _randbits(rng, 64),
        'asset': lambda: {
            'amount': _signed_int(rng, 62),
            'symbol': _randsym(rng)
        },
        'extended_asset': lambda: {
            'quantity': {
                'amount': _signed_int(rng, 62),
                'symbol': _randsym(rng)
            },
            'contract': _randbits(rng, 64)
        },
    }


# RNG-aware cache
_RNG = random.Random()
_GLOBAL_GENERATORS = _make_generators(random)

_GEN_CACHE: 'weakref.WeakKeyDictionary[random.Random, dict[str, Callable]]'
_GEN_CACHE = weakref.WeakKeyDictionary()


def _generators_for(rng) -> dict[str, Callable[[], IOTypes]]:
    '''
    Return the dispatch table for *rng*, building it once if necessary.
    '''
    if rng is _RNG:  # stdlib’s singleton module
        return _GLOBAL_GENERATORS

    try:
        return _GEN_CACHE[rng]
    except KeyError:
        table = _make_generators(rng)
        _GEN_CACHE[rng] = table
        return table


def random_std_type(
    type_name: str,
    *,
    rng: random.Random = _RNG
) -> IOTypes:
    '''
    Generate a random value for *type_name* using *rng*.

    Parameters
    ----------
    type_name : str
        Any supported std type (e.g. 'uint64', 'float32') or 'raw(N)'.
    rng : random.Random | None
        Source of randomness.  Defaults to the stdlib global RNG.
    '''
    if type_name.startswith('raw(') and type_name.endswith(')'):
        size = int(type_name[4:-1])
        return _randbytes(rng, size)

    generators = _generators_for(rng)
    try:
        return generators[type_name]()
    except KeyError:
        raise TypeError(f'Unknown standard type “{type_name}”') from None


def _rest_type_string(base: str, mods: list[TypeModifier]) -> str:
    '''
    Re-constitute the inner-type string by appending the remaining
    modifier suffixes (inner-most first -> rightmost).

    '''
    for m in reversed(mods):
        base += suffix_for(m)
    return base

def random_abi_type(
    abi: ABIView,
    type_name: str,
    *,
    min_list_size: int = 0,
    max_list_size: int = 2,
    list_delta: int = 0,
    chance_of_none: float = 0.5,
    chance_delta: float = 0.5,
    type_args: dict[str, dict] = {},
    rng: random.Random = _RNG,
) -> IOTypes:
    '''
    Generate a random value that is valid for *type_name* according to *abi*,
    honouring **all** trailing modifiers ( `[]`, `?`, `$` ) in the correct
    outer-to-inner order.

    '''
    resolved = abi.resolve_type(type_name)

    # kwargs that may change as we peel modifiers
    kwargs = {
        'min_list_size': min_list_size,
        'max_list_size': max_list_size,
        'list_delta': list_delta,
        'chance_of_none': chance_of_none,
        'chance_delta': chance_delta,
        'type_args': type_args,
        'rng': rng
    }

    # override via *type_args*
    if type_name in type_args:
        kwargs |= type_args[type_name]

    # start with the full modifier chain (outer -> inner)
    modifiers = list(resolved.modifiers)
    base_type = resolved.resolved_name

    # handle array / optional / extension layers iteratively
    while modifiers:
        outer = modifiers.pop(0)

        if outer == 'array':
            # shrink bounds for deeper arrays
            pre_min = kwargs['min_list_size']
            pre_max = kwargs['max_list_size']
            kwargs['min_list_size'] = max(pre_min - kwargs['list_delta'], 0)
            kwargs['max_list_size'] = max(pre_max - kwargs['list_delta'], 0)

            size = rng.randint(pre_min, pre_max)
            inner = _rest_type_string(base_type, modifiers)
            return [
                random_abi_type(abi, inner, **kwargs)
                for _ in range(size)
            ]

        if outer in ('optional', 'extension'):
            # decide whether to produce None
            if rng.random() < kwargs['chance_of_none']:
                return None

            # raise None-probability for deeper optionals
            kwargs['chance_of_none'] = min(
                1.0,
                kwargs['chance_of_none'] + kwargs['chance_delta']
            )
            inner = _rest_type_string(base_type, modifiers)
            return random_abi_type(abi, inner, **kwargs)

        # unreachable: only array / optional / extension exist
        raise AssertionError(f'unknown modifier {outer!r}')

    # no more modifiers - generate concrete data
    if base_type in builtin_types:
        return random_std_type(base_type, rng=rng)

    # variant
    if base_type in abi.variant_map:
        variant_type = rng.choice(abi.variant_map[base_type].types)
        val = random_abi_type(abi, variant_type, **kwargs)
        return {'type': variant_type, **val} if isinstance(val, dict) else val

    # struct
    if base_type not in abi.struct_map:
        raise TypeError(f'Expected {type_name} to resolve to a struct')

    struct = abi.struct_map[base_type]

    # recurse into base struct first
    ioobj = (
        {} if not struct.base
        else random_abi_type(abi, struct.base, **kwargs)
    )

    if not isinstance(ioobj, dict):
        raise TypeError(f'Expected base type to be a struct, instead got {ioobj}')

    obj: dict = ioobj

    # populate fields
    obj |= {
        f.name: random_abi_type(abi, f.type_, **kwargs)
        for f in struct.fields
    }

    # enforce "$" (binary-extension) rule:
    found_ext = False
    for f in struct.fields:
        if f.type_.endswith('$'):
            if found_ext or obj[f.name] is None:
                found_ext = True
                obj[f.name] = None
        elif found_ext:
            obj[f.name] = None

    return obj

ABIView.random_of = random_abi_type
