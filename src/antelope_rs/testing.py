# pyo3-antelope-rs
# Copyright 2025-eternity Guillermo Rodriguez

# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.

# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU Affero General Public License for more details.

# You should have received a copy of the GNU Affero General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.
from __future__ import annotations

import os
import json
import string
import logging

from typing import (
    Any,
    Generator,
    Sequence,
    Type,
    Callable,
)
from pathlib import Path

import hypothesis.strategies as st

from antelope_rs import ABIView


logger = logging.getLogger(__name__)


inside_ci = any(
    os.getenv(v)
    for v in [
        'CI', 'GITHUB_ACTIONS', 'GITLAB_CI', 'TRAVIS', 'CIRCLECI'
    ]
)

tests_dir = Path(__file__).parent.parent.parent / 'tests'
abi_dir = Path(__file__).parent / 'abi'

SynthABI = ABIView.from_file(abi_dir / 'synth.json')

StdABI = ABIView.from_file(
    abi_dir / 'standard.json',
    cls_alias='std'
)

EosioABI = ABIView.from_file(abi_dir / 'eosio.json')
MsigABI = ABIView.from_file(abi_dir / 'msig.json')


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


name_chars   = 'abcdefghijklmnopqrstuvwxyz12345.'
symbol_chars = string.ascii_uppercase


@st.composite
def name_str(draw) -> str:
    ln = draw(st.integers(min_value=1, max_value=12))
    return ''.join(draw(
        st.sampled_from(name_chars))
        for _ in range(ln)
    )


@st.composite
def symbol_code_str(draw) -> str:
    ln = draw(st.integers(min_value=1, max_value=7))
    return ''.join(draw(
        st.sampled_from(symbol_chars))
        for _ in range(ln)
    )


@st.composite
def symbol_str(draw) -> str:
    precision = draw(st.integers(min_value=0, max_value=18))
    return f'{precision},{draw(symbol_code_str())}'


@st.composite
def asset_str(draw) -> str:
    prec  = draw(st.integers(min_value=0, max_value=8))
    code  = draw(symbol_code_str())
    whole = draw(st.integers(min_value=-1_000_000, max_value=1_000_000))
    if prec:
        frac   = draw(st.integers(min_value=0, max_value=10 ** prec - 1))
        amount = f'{whole}.{frac:0{prec}d}'
    else:
        amount = f'{whole}'
    return f'{amount} {code}'


@st.composite
def extended_asset_str(draw) -> str:
    return f'{draw(asset_str())}@{draw(name_str())}'


@st.composite
def public_key_bytes(draw) -> bytes:
    return b'\0' + draw(st.binary(min_size=33, max_size=33))


@st.composite
def signature_bytes(draw) -> bytes:
    return b'\0' + draw(st.binary(min_size=65, max_size=65))

# builtin antelope types
_builtin_alts: dict[str, st.SearchStrategy[Any]] = {
    'bool': st.booleans(),
    # unsigned ints
    **{
        n: st.one_of(
            st.integers(min_value=0, max_value=(1 << b) - 1),
            st.binary(min_size=b // 8, max_size=b // 8),
        )
        for n, b in (
            ('uint8', 8), ('uint16', 16), ('uint32', 32),
            ('uint64', 64), ('uint128', 128),
        )
    },
    # signed ints
    **{
        n: st.one_of(
            st.integers(min_value=-(1 << (b - 1)), max_value=(1 << (b - 1)) - 1),
            st.binary(min_size=b // 8, max_size=b // 8),
        )
        for n, b in (
            ('int8', 8), ('int16', 16), ('int32', 32),
            ('int64', 64), ('int128', 128),
        )
    },

    'varuint32': st.integers(min_value=0, max_value=(1 << 32) - 1),
    'varint32': st.integers(min_value=-(1 << 31), max_value=(1 << 31) - 1),

    'float32': st.floats(allow_infinity=False, allow_nan=False, width=32),
    'float64': st.floats(allow_infinity=False, allow_nan=False, width=64),
    'float128': st.one_of(
        st.binary(min_size=16, max_size=16),
        st.binary(min_size=16, max_size=16).map(lambda b: b.hex())
    ),

    'time_point': st.integers(min_value=0, max_value=(1 << 64) - 1),
    'time_point_sec': st.integers(min_value=0, max_value=(1 << 32) - 1),
    'block_timestamp_type': st.integers(min_value=0, max_value=(1 << 32) - 1),

    'name': st.one_of(
        st.integers(min_value=0, max_value=(1 << 64) - 1),
        name_str(),
        st.binary(min_size=8, max_size=8),
    ),

    'bytes': st.binary(),
    'string': st.text(),

    'checksum160': st.binary(min_size=20, max_size=20),
    'checksum256': st.binary(min_size=32, max_size=32),
    'checksum512': st.binary(min_size=64, max_size=64),

    'public_key': public_key_bytes(),
    'signature': signature_bytes(),

    'symbol_code': symbol_code_str(),
    'symbol': symbol_str(),
    'asset': asset_str(),
    'extended_asset': extended_asset_str(),
}


def _optional_or_extension(inner: st.SearchStrategy[Any], chance_of_none: float) -> st.SearchStrategy[Any]:
    '''
    Return a strategy choosing `None` with probability ~chance_of_none.

    '''
    if chance_of_none >= 1.0:
        return st.just(None)
    weight = max(1, min(10, int(round(chance_of_none * 10))))
    return st.one_of(*([st.just(None)] * weight + [inner]))


def _array_of(inner: st.SearchStrategy[Any], min_len: int, max_len: int) -> st.SearchStrategy[Sequence[Any]]:
    if max_len <= 0:
        return st.just([])
    return st.lists(inner, min_size=min_len, max_size=max_len)


def _lazy_cell() -> tuple[st.SearchStrategy[Any], Callable[[st.SearchStrategy[Any]], None]]:
    '''
    Return `(placeholder, commit)` where placeholder is deferred.

    '''
    cell: list[st.SearchStrategy[Any]] = [st.nothing()]

    def getter() -> st.SearchStrategy[Any]:
        return cell[0]

    def commit(value: st.SearchStrategy[Any]) -> None:
        # mutate in-place
        cell[0] = value

    return st.deferred(getter), commit


def _field_strategies(
    abi: Type[ABIView],
    struct_name: str,
    **kwargs,
) -> dict[str, st.SearchStrategy[Any]]:
    sdef = abi.struct_map[struct_name]
    out: dict[str, st.SearchStrategy[Any]] = {}

    if sdef.base:
        out.update(
            _field_strategies(
                abi, sdef.base, **kwargs
            )
        )

    for fld in sdef.fields:
        out[str(fld.name)] = strategy_for_type(
            abi, fld.type_, **kwargs
        )

    return out


def strategy_for_type(
    abi: Type[ABIView],
    type_name: str,
    *,
    tag_field: str = 'antelope_type',
    # list tuning
    min_list_size: int = 0,
    max_list_size: int = 3,
    list_delta   : int = 0,
    # optional / extension tuning
    chance_of_none: float = 0.50,
    chance_delta: float = 0.50,
    # per-type overrides
    type_args: dict[str, dict] = {},
    # internal memo table (for cycle breaking)
    memo: dict = {},
) -> st.SearchStrategy[Any]:
    '''
    Build a Hypothesis strategy that can generate any value valid for *type_name*.
    Parameters mimic `random_abi_type()`, so you can share a config dict:
      * list sizes shrink by `list_delta` on each recursion
      * chance-of-None increases by `chance_delta` on each recursion

    '''
    # kwargs that may change as we peel modifiers
    kwargs = {
        'min_list_size': min_list_size,
        'max_list_size': max_list_size,
        'list_delta': list_delta,
        'chance_of_none': chance_of_none,
        'chance_delta': chance_delta,
        'type_args': type_args,
    }

    # override via *type_args*
    if type_name in type_args:
        kwargs |= type_args[type_name]

    min_list_size = kwargs.get('min_list_size', min_list_size)
    max_list_size = kwargs.get('max_list_size', max_list_size)
    list_delta = kwargs.get('list_delta',    list_delta)
    chance_of_none = kwargs.get('chance_of_none',        chance_of_none)
    chance_delta = kwargs.get('chance_delta',  chance_delta)

    # cycle-breaker using a lazy cell
    key = (abi, type_name)
    if key in memo:
        return memo[key]

    # reserve slot before recursion
    placeholder, _commit = _lazy_cell()
    memo[key] = placeholder

    # resolve ABI type and peel modifiers
    resolved = abi.resolve_type(type_name)
    base, modifiers = str(resolved.resolved_name), list(resolved.modifiers)

    # handle array / optional / extension
    if modifiers:
        outer, *rest = modifiers
        rest_type = base + ''.join({'array': '[]', 'optional': '?', 'extension': '$'}[m] for m in rest)

        match outer:
            case 'array':
                kwargs['min_list_size'] = max(0, min_list_size - list_delta)
                kwargs['max_list_size'] = max(0, max_list_size - list_delta)
                inner = strategy_for_type(
                    abi,
                    rest_type,
                    **kwargs,
                )
                result = _array_of(
                    inner, min_list_size, max_list_size
                )
                _commit(result)
                return result

            case 'optional' | 'extension':
                kwargs['chance_of_none'] = min(1.0, chance_of_none + chance_delta)
                inner = strategy_for_type(
                    abi,
                    rest_type,
                    **kwargs,
                )
                result = _optional_or_extension(
                    inner, chance_of_none
                )
                _commit(result)
                return result

            case _:
                raise AssertionError(f'unknown modifier {outer!r}')

    # builtins
    if base in _builtin_alts:
        result = _builtin_alts[base]
        _commit(result)
        return result

    # variant
    if base in abi.variant_map:
        v = abi.variant_map[base]

        alts: list[st.SearchStrategy[Any]] = []
        for t in v.types:
            inner = strategy_for_type(abi, t, **kwargs)
            meta  = abi.resolve_type(t)

            # Only struct-alts need the tag field
            if meta.is_struct is not None:
                tag_value = t
                inner = inner.map(
                    lambda d, _tag=tag_value: (
                        {tag_field: str(_tag), **d}  # struct branch -> dict
                        if isinstance(d, dict)
                        else d  # builtin branch -> passthrough
                    )
                )

            alts.append(inner)

        result = st.one_of(*alts)
        _commit(result)
        return result

    # struct
    if base in abi.struct_map:
        fields = _field_strategies(
            abi, base, **kwargs
        )
        result = st.fixed_dictionaries(fields)
        _commit(result)
        return result

    raise KeyError(f'Unknown ABI type {base!r}')


def generate_strategies(
    abi_whitelist: set[str],
    type_whitelist: set[str]
) -> Generator:
    '''
    Based on ABI & types whitelist iterate over all different test cases

    '''
    abis: dict[str, Type[ABIView]] = {
        name: abi
        for name, abi in (
            # synthetic test abi
            ('synth', SynthABI),
            # ship standard abi
            ('std', StdABI),
            # eosio system & msig contracts
            ('eosio', EosioABI),
            ('msig', MsigABI)
        )
        if (
            len(abi_whitelist) == 0
            or
            name in abi_whitelist
        )
    }

    pairs: list[tuple[str, str]] = [
        (abi_key, str(t))
        for abi_key, view in abis.items()
        for t in sorted(set(view.struct_map) | set(view.variant_map))
        if (
            (
                len(abi_whitelist) == 0
                or
                abi_key in abi_whitelist
            )
            and
            (
                len(type_whitelist) == 0
                or
                t in type_whitelist
            )
        )
    ]

    for abi_name, type_name in pairs:
        abi = abis[abi_name]
        strat = strategy_for_type(abi, type_name)
        struct_cls = getattr(abi, type_name)

        yield abi_name, type_name, abi, struct_cls, strat
