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

import sys
import time
import types

from typing import Type
from pathlib import Path

from frozendict import frozendict

from antelope_rs._lowlevel import (
    builtin_types,
    ABI,
    ShipABI,
)

from antelope_rs.meta import TypeNameStr

from ._defs import (
    ABILike as ABILike,
    ABIResolvedType as ABIResolvedType,
    VariantDef as VariantDef,
    StructDef as StructDef,
    ActionDef as ActionDef,
    AliasDef as AliasDef,
    TableDef as TableDef
)
from ._struct_ns import (
    build_struct_namespace
)
from ._validation import validate_definition

# ABIView:
# Wraps ABI or ShipABI to provide a unified & fast API into the type namespace
# defined

ABIClass = Type[ABI] | Type[ShipABI]
ABIClassOrAlias = ABIClass | str | None


def _solve_cls_alias(cls: ABIClassOrAlias = None) -> ABIClass:
    if cls is None:
        return ABI

    if isinstance(cls, str):
        if cls in ['std', 'standard']:
            return ShipABI

        return ABI

    if not (
        isinstance(cls, ABI)
        and
        isinstance(cls, ShipABI)
    ):
        cls_str = 'None' if not cls else cls.__name__
        raise TypeError(f'Unknown class to init ABIView: {cls_str}')

    return cls


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

    def __new__(mcls, name, bases, ns, *, definition=None, **kw) -> Type[ABIView]:
        cls = super().__new__(mcls, name, bases, ns, **kw)

        # Base ABIView itself has no definition - skip the heavy lifting
        if definition is None:
            return cls

        # validate
        validate_definition(definition)

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

        # give each ABIView a unique module name
        mod_name = f'antelope_rs.abi._view_{name}_{int(time.time())}'
        mod = types.ModuleType(mod_name)
        sys.modules[mod_name] = mod

        # Re-use the existing helper to autogenerate Python structs/aliases
        tmp_view = object.__new__(ABIView)
        tmp_view._def = definition
        tmp_view.alias_map   = cls.alias_map
        tmp_view.struct_map  = cls.struct_map
        tmp_view.variant_map = cls.variant_map
        tmp_view.valid_types = cls.valid_types
        tmp_view.resolve_type = definition.resolve_type

        for k, v in build_struct_namespace(tmp_view, mod).items():
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
    ) -> Type[ABIView]:
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
