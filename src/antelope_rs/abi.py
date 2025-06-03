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

import hashlib

from typing import (
    Annotated,
    Callable,
    Literal,
    Type
)
from pathlib import Path

from msgspec import (
    Meta,
    Struct,
    field,
    convert,
)
from frozendict import frozendict

from ._lowlevel import (
    builtin_types,
    ABI,
    ShipABI,
)


# msgspec compatible type hints for different string fields in ABI definitions

# type names, alphanumeric can have multiple modifiers (?, $ & []) at the end
regex_type_name = r'^([A-Za-z_][A-Za-z0-9_]*)(?:\[\]|\?|\$)*$'

TypeNameStr = Annotated[
    str,
    Meta(
        pattern=regex_type_name,
        min_length=1,
    )
]

# ABI struct fields, alphanumeric + '_'
regex_field_name = r'^[A-Za-z_][A-Za-z0-9_]*$'

FieldNameStr = Annotated[
    str,
    Meta(
        pattern=regex_field_name,
        min_length=1,
    )
]

# Antelope account name(uint64), empty string, lowercase letters, only numbers
# 1-5 & '.'
regex_antelope_name = r'^(?:$|[a-z][a-z1-5\.]{0,11}[a-j1-5\.]?)$'

AntelopeNameStr = Annotated[
    str,
    Meta(
        pattern=regex_antelope_name,
        min_length=0,
        max_length=13
    )
]

# ABI struct, base type name string, like TypeNameStr but with no modifiers,
# and empty string is allowed
regex_base_type_name = r'^$|^[A-Za-z_][A-Za-z0-9_]*$'

BaseTypeNameStr = Annotated[
    str,
    Meta(
        pattern=regex_base_type_name,
        min_length=0,
    )
]

# hex strings
regex_hex_str = r'^[A-Za-z0-9_]*$'
Sum160Str = Annotated[
    str,
    Meta(
        pattern=regex_hex_str,
        min_length=40,
        max_length=40
    )
]

Sum256Str = Annotated[
    str,
    Meta(
        pattern=regex_hex_str,
        min_length=64,
        max_length=64
    )
]

Sum512Str = Annotated[
    str,
    Meta(
        pattern=regex_hex_str,
        min_length=128,
        max_length=128
    )
]


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

# classmethods
def _from_file(cls, p: Path | str):
    return cls.from_str(
        Path(p).read_text()
    )

# properties
def _types(self) -> list[AliasDef]:
    return [
        convert(type_dict, type=AliasDef)
        for type_dict in self._types
    ]

def _structs(self) -> list[StructDef]:
    return [
        convert(struct_dict, type=StructDef)
        for struct_dict in self._structs
    ]

def _variants(self) -> list[VariantDef]:
    return [
        convert(variant_dict, type=VariantDef)
        for variant_dict in self._variants
    ]

def _actions(self) -> list[VariantDef]:
    return [
        convert(action_dict, type=VariantDef)
        for action_dict in self._actions
    ]

def _tables(self) -> list[VariantDef]:
    return [
        convert(table_dict, type=VariantDef)
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
    return convert(self.resolve_type_into_dict(type_name), type=ABIResolvedType)

# finally monkey patch ABI & ShipABI

def _apply_to_abi_classes(attr_name: str, fn):
    setattr(ABI, attr_name, fn)
    setattr(ShipABI, attr_name, fn)

_class_methods = [
    ('from_file', _from_file),
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


ABILike = bytes | str | dict | ABI | ShipABI


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


class ABIView:

    _def: ABI | ShipABI

    alias_map: frozendict[TypeNameStr, TypeNameStr]
    variant_map: frozendict[TypeNameStr, VariantDef]
    struct_map: frozendict[TypeNameStr, StructDef]
    valid_types: frozenset[TypeNameStr]

    # only available if antelope_rs.testing is imported
    make_canonical: Callable
    canonical_diff: Callable
    assert_deep_eq: Callable
    random_of: Callable

    def __init__(
        self,
        definition: ABI | ShipABI
    ):
        self._def = definition

        self.alias_map = frozendict({
            a.new_type_name: a.type_
            for a in definition.types
        })
        self.struct_map = frozendict({
            s.name: s
            for s in definition.structs
        })
        self.variant_map = frozendict({
            e.name: e
            for e in definition.variants
        })
        self.valid_types = frozenset([
            *builtin_types,
            *list(self.struct_map.keys()),
            *list(self.variant_map.keys()),
            *list(self.alias_map.keys()),
        ])

    @staticmethod
    def from_str(s: str, cls: ABIClassOrAlias = None) -> ABIView:
        cls = _solve_cls_alias(cls)
        return ABIView(cls.from_str(s))

    @staticmethod
    def from_file(p: Path | str, cls: ABIClassOrAlias = None) -> ABIView:
        cls = _solve_cls_alias(cls)
        return ABIView(cls.from_file(p))

    @staticmethod
    def from_abi(abi: ABI | ShipABI | ABIView) -> ABIView:
        if isinstance(abi, ABIView):
            return abi

        return ABIView(abi)

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

    def hash(self, *, as_bytes: bool = False) -> str | bytes:
        return self._def.hash(as_bytes=as_bytes)

    def resolve_type(self, type_name: str) -> ABIResolvedType:
        return self._def.resolve_type(type_name)

    def pack(self, *args, **kwargs) -> bytes:
        return self._def.pack(*args, **kwargs)

    def unpack(self, *args, **kwargs) -> bytes:
        return self._def.unpack(*args, **kwargs)
