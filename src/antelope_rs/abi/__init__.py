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

import json
import hashlib

from typing import (
    Callable,
    Literal,
    Type,
)
from pathlib import Path

from msgspec import (
    Struct,
    field,
    convert,
)
from frozendict import frozendict

from antelope_rs._lowlevel import (
    builtin_types,
    ABI,
    ShipABI,
)

from antelope_rs.codec import dec_hook
from antelope_rs.meta import (
    TypeNameStr,
    FieldNameStr,
    AntelopeNameStr,
    BaseTypeNameStr
)
from .structs import build_struct_namespace


TypeModifier = Literal['optional'] | Literal['extension'] | Literal['array']
TypeSuffix = Literal['?'] | Literal['$'] | Literal['[]']


_suffixes: frozendict[TypeModifier, TypeSuffix] = frozendict({
    'array': '[]',
    'optional': '?',
    'extension': '$',
})


def suffix_for(type_mod: TypeModifier) -> TypeSuffix:
    return _suffixes[type_mod]


class AliasDef(Struct, frozen=True):
    new_type_name: TypeNameStr
    type_: TypeNameStr = field(name='type')


class VariantDef(Struct, frozen=True):
    name: TypeNameStr
    types: list[TypeNameStr]


class FieldDef(Struct, frozen=True):
    name: FieldNameStr
    type_: TypeNameStr = field(name='type')


class StructDef(Struct, frozen=True):
    name: TypeNameStr
    fields: list[FieldDef]
    base: BaseTypeNameStr | None = None


class ActionDef(Struct, frozen=True):
    name: AntelopeNameStr
    type_: TypeNameStr = field(name='type')
    ricardian_contract: str


class TableDef(Struct, frozen=True):
    name: AntelopeNameStr
    key_names: list[FieldNameStr]
    key_types: list[TypeNameStr]
    index_type: TypeNameStr
    type_: TypeNameStr = field(name='type')


class ABIResolvedType(Struct, frozen=True):
    original_name: TypeNameStr
    resolved_name: TypeNameStr
    is_std: bool
    is_alias: bool
    is_struct: StructDef | None
    is_variant: VariantDef | None
    modifiers: list[TypeModifier]


# ABI & ShipABI highlevel patching

ABILike = bytes | str | dict | ABI | ShipABI

# classmethods
def _from_file(cls, p: Path | str):
    return cls.from_str(
        Path(p).read_text()
    )

def _try_from(cls, abi: ABILike):
    if isinstance(abi, cls):
        return abi

    if isinstance(abi, bytes):
        return cls.from_bytes(abi)

    if isinstance(abi, dict):
        abi = json.dumps(abi)

    if isinstance(abi, str):
        return cls.from_bytes(abi)

    raise TypeError(
        f'Wrong type for abi creation expected ABILike but got {type(abi).__name__}'
    )


# properties
def _types(self) -> list[AliasDef]:
    return [
        convert(type_dict, type=AliasDef, dec_hook=dec_hook)
        for type_dict in self._types
    ]

def _structs(self) -> list[StructDef]:
    return [
        convert(struct_dict, type=StructDef, dec_hook=dec_hook)
        for struct_dict in self._structs
    ]

def _variants(self) -> list[VariantDef]:
    return [
        convert(variant_dict, type=VariantDef, dec_hook=dec_hook)
        for variant_dict in self._variants
    ]

def _actions(self) -> list[VariantDef]:
    return [
        convert(action_dict, type=VariantDef, dec_hook=dec_hook)
        for action_dict in self._actions
    ]

def _tables(self) -> list[VariantDef]:
    return [
        convert(table_dict, type=VariantDef, dec_hook=dec_hook)
        for table_dict in self._tables
    ]

# methods
def _hash(self, *, as_bytes: bool = False) -> str | bytes:
    '''
    Get a sha256 of the types definition

    '''
    h = hashlib.sha256()

    h.update(b'structs')
    for s in self.structs:
        h.update(s.name.encode())
        for f in s.fields:
            h.update(f.name.encode())
            h.update(f.type_.encode())

    h.update(b'enums')
    for e in self.variants:
        h.update(e.name.encode())
        for v in e.types:
            h.update(v.encode())

    h.update(b'aliases')
    for a in self.types:
        h.update(a.new_type_name.encode())
        h.update(a.type_.encode())

    return (
        h.digest() if as_bytes
        else h.hexdigest()
    )

def _resolve_type(self, type_name: str) -> ABIResolvedType:
    return convert(self.resolve_type_into_dict(str(type_name)), type=ABIResolvedType, dec_hook=dec_hook)

# finally monkey patch ABI & ShipABI

def _apply_to_abi_classes(attr_name: str, fn):
    setattr(ABI, attr_name, fn)
    setattr(ShipABI, attr_name, fn)

_class_methods = [
    ('from_file', _from_file),
    ('try_from', _try_from)
]

_properties = [
    ('types', _types),
    ('structs', _structs),
    ('variants', _variants),
    ('actions', _actions),
    ('tables', _tables)
]

_methods = [
    ('hash', _hash),
    ('resolve_type', _resolve_type),
]

for name, fn in _class_methods:
    _apply_to_abi_classes(name, classmethod(fn))

for name, fn in _properties:
    _apply_to_abi_classes(name, property(fn))

for name, fn in _methods:
    _apply_to_abi_classes(name, fn)


# ABIView:
# Wraps ABI or ShipABI to provide a unified & fast API into the type namespace
# defined

ABIClass = Type[ABI] | Type[ShipABI]
ABIClassOrAlias = ABIClass | str | None


def _solve_cls_alias(cls: ABIClassOrAlias = None) -> ABIClass:
    if isinstance(cls, str):
        if cls in ['std', 'standard']:
            return ShipABI

        return ABI

    ret: ABIClass = cls if cls else ABI
    if not (
        isinstance(ret, ABI)
        and
        isinstance(ret, ShipABI)
    ):
        cls_str = 'None' if not cls else cls.__name__
        raise TypeError(f'Unknown class to init ABIView: {cls_str}')

    return ret


class ABIViewMeta(type):
    '''
    When a subclass is *created* we already have its ABI definition, so we
    compute all the expensive maps just once and attach them as **class
    attributes**.  Instances are never needed.

    '''
    _def: ABI | ShipABI

    alias_map: frozendict[TypeNameStr, TypeNameStr]
    variant_map: frozendict[TypeNameStr, VariantDef]
    struct_map: frozendict[TypeNameStr, StructDef]
    valid_types: frozenset[TypeNameStr]

    def __new__(mcls, name, bases, ns, *, definition=None, **kw):
        cls = super().__new__(mcls, name, bases, ns, **kw)

        # Base ABIView itself has no definition - skip the heavy lifting
        if definition is None:
            return cls

        # build metadata
        cls._def = definition  # raw Rust object

        cls.alias_map   = frozendict({
            a.new_type_name: a.type_ for a in definition.types
        })
        cls.struct_map  = frozendict({s.name: s for s in definition.structs})
        cls.variant_map = frozendict({v.name: v for v in definition.variants})
        cls.valid_types = frozenset([
            *builtin_types,
            *cls.alias_map, *cls.struct_map, *cls.variant_map
        ])

        # Re-use the existing helper to autogenerate Python structs/aliases
        tmp_view = object.__new__(ABIView)
        tmp_view._def = definition
        tmp_view.alias_map   = cls.alias_map
        tmp_view.struct_map  = cls.struct_map
        tmp_view.variant_map = cls.variant_map
        tmp_view.valid_types = cls.valid_types
        tmp_view.resolve_type = definition.resolve_type

        for k, v in build_struct_namespace(tmp_view).items():
            setattr(cls, str(k), v)

        return cls


class ABIView(metaclass=ABIViewMeta):
    '''
    Pure namespace - don’t instantiate it.  Sub-classes produced by the
    factory below have all the goodies as *class* attributes.
    '''
    # factory
    @classmethod
    def from_file(
        cls,
        path: str | Path,
        *,
        cls_alias: ABIClassOrAlias = None,
        name: str | None = None,
    ):
        '''
        Read JSON, decide ABI vs ShipABI, and *return a **new subclass*** that
        carries the parsed definition.

        Example
        -------
        Std = ABIView.from_file('standard.json', cls_alias='std')
        Std.TransactionTrace   # ready to use
        '''
        base_cls = _solve_cls_alias(cls_alias)
        definition = base_cls.from_file(path)

        qualname = (
            name
            or f'{Path(path).stem.title()}ABIView'
        )

        return ABIViewMeta(qualname, (cls,), {}, definition=definition)

    @property
    def definition(self) -> ABI | ShipABI:
        return self._def

    @property
    def structs(self) -> list[StructDef]:
        return self._def.structs

    @property
    def types(self) -> list[AliasDef]:
        return self._def.types

    @property
    def variants(self) -> list[VariantDef]:
        return self._def.variants

    @property
    def actions(self) -> list[ActionDef]:
        return self._def.actions

    @property
    def tables(self) -> list[TableDef]:
        return self._def.tables

    @classmethod
    def hash(cls, *, as_bytes: bool = False):
        return cls._def.hash(as_bytes=as_bytes)

    @classmethod
    def resolve_type(cls, type_name: str):
        return cls._def.resolve_type(type_name)

    @classmethod
    def pack(cls, *a, **kw):
        return cls._def.pack(*a, **kw)

    @classmethod
    def unpack(cls, *a, **kw):
        return cls._def.unpack(*a, **kw)

    # testing helpers added if antelope_rs.testing imported
    make_canonical: Callable 
    canonical_diff: Callable
    assert_deep_eq: Callable
