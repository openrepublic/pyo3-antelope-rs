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

from typing import Literal
from pathlib import Path

from msgspec import (
    Struct,
    field,
    convert,
)
from frozendict import frozendict

from antelope_rs._lowlevel import (
    ABI,
    ShipABI,
)

from antelope_rs.meta import TypeNameStr, FieldNameStr, AntelopeNameStr, BaseTypeNameStr
from antelope_rs.codec import dec_hook


TypeModifier = Literal['optional'] | Literal['extension'] | Literal['array']
TypeSuffix = Literal['?'] | Literal['$'] | Literal['[]']


_suffixes: frozendict[TypeModifier, TypeSuffix] = frozendict(
    {
        'array': '[]',
        'optional': '?',
        'extension': '$',
    }
)


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
    alias_chain: list[str]
    is_std: bool
    is_struct: StructDef | None
    is_variant: VariantDef | None
    modifiers: list[TypeModifier]


# ABI & ShipABI highlevel patching

ABILike = bytes | str | dict | ABI | ShipABI


# classmethods
def _from_file(cls, p: Path | str):
    return cls.from_str(Path(p).read_text())


def _try_from(cls, abi: ABILike):
    if isinstance(abi, cls):
        return abi

    if isinstance(abi, bytes):
        return cls.from_bytes(abi)

    if isinstance(abi, dict):
        abi = json.dumps(abi)

    if isinstance(abi, str):
        return cls.from_str(abi)

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

    return h.digest() if as_bytes else h.hexdigest()


def _resolve_type(self, type_name: str) -> ABIResolvedType:
    return convert(
        self.resolve_type_into_dict(str(type_name)),
        type=ABIResolvedType,
        dec_hook=dec_hook,
    )


# finally monkey patch ABI & ShipABI


def _apply_to_abi_classes(attr_name: str, fn):
    setattr(ABI, attr_name, fn)
    setattr(ShipABI, attr_name, fn)


_class_methods = [('from_file', _from_file), ('try_from', _try_from)]

_properties = [
    ('types', _types),
    ('structs', _structs),
    ('variants', _variants),
    ('actions', _actions),
    ('tables', _tables),
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
