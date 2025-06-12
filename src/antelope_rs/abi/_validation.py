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

from collections import deque

from antelope_rs._lowlevel import builtin_types
from antelope_rs.meta import TypeNameStr


class ABIValidationError(Exception):
    pass


class UndefinedTypeError(ABIValidationError):
    def __init__(self, where: str, typename: str) -> None:
        msg = f'undefined type {typename!r} referenced from {where}'
        super().__init__(msg)


class DuplicateNameError(ABIValidationError):
    def __init__(self, kind: str, name: str) -> None:
        super().__init__(f'duplicate {kind} name {name!r}')


class ExtensionOrderError(ABIValidationError):
    def __init__(self, struct: str, field: str) -> None:
        msg = (
            f'struct {struct!r} has non-extension field {field!r} after an '
            'extension field'
        )
        super().__init__(msg)


class BaseExtensionFieldError(ABIValidationError):
    def __init__(self, base: str, field: str) -> None:
        super().__init__(
            f'base struct {base!r} contains extension field {field!r} '
            '(base structs may not carry $-fields)'
        )


class CyclicAliasError(ABIValidationError):
    def __init__(self, chain: list[str]) -> None:
        super().__init__(f'cyclic alias detected: {" -> ".join(chain)}')


def _collect_defined(defn) -> set[str]:
    return {
        *builtin_types,
        *(t.new_type_name for t in defn.types),
        *(s.name for s in defn.structs),
        *(v.name for v in defn.variants),
    }


def _bare(name: TypeNameStr | str) -> str:
    '''Return *name* without trailing ?, $, or [] modifiers (recursive).'''
    if isinstance(name, TypeNameStr):
        name = str(name)

    # strip ?, $
    while name and name[-1] in ('?', '$'):
        name = name[:-1]

    # strip any number of [] suffixes
    while name.endswith('[]'):
        name = name[:-2]
    return name


def _check_duplicates(defn) -> None:
    seen: dict[str, str] = {}
    for kind, coll in (
        ('type',    defn.types),
        ('struct',  defn.structs),
        ('variant', defn.variants),
    ):
        for obj in coll:
            name = obj.name if kind != 'type' else obj.new_type_name
            if name in seen:
                raise DuplicateNameError(seen[name], name)
            seen[name] = kind


def _check_alias_targets(defn, defined: set[str]) -> None:
    for alias in defn.types:
        target = _bare(alias.type_)
        if target not in defined:
            raise UndefinedTypeError(
                f'alias {alias.new_type_name!r}', alias.type_
            )


def _check_structs(defn, defined: set[str]) -> None:
    for s in defn.structs:
        # base
        if s.base and s.base not in defined:
            raise UndefinedTypeError(f'struct {s.name!r} base', s.base)

        # extension ordering + field types
        seen_ext = False
        for fld in s.fields:
            if fld.type_.endswith('$'):
                seen_ext = True
            elif seen_ext:
                raise ExtensionOrderError(s.name, fld.name)

            bare_type = _bare(fld.type_)
            if bare_type not in defined:
                raise UndefinedTypeError(
                    f'field {s.name!r}.{fld.name!r}', bare_type
                )

def _check_base_struct_extensions(defn) -> None:
    # gather every struct that is used as a base
    base_names = {s.base for s in defn.structs if s.base}
    # quick lookup: name -> struct object
    by_name = {s.name: s for s in defn.structs}

    for base in base_names:
        s = by_name[base]
        for fld in s.fields:
            if fld.type_.endswith('$'):
                raise BaseExtensionFieldError(base, fld.name)


def _check_variants(defn, defined: set[str]) -> None:
    for v in defn.variants:
        for t in v.types:
            bare = _bare(t)
            if bare not in defined:
                raise UndefinedTypeError(f'variant {v.name!r}', bare)


def _detect_alias_cycles(defn) -> None:
    graph = {a.new_type_name: _bare(a.type_) for a in defn.types}

    for root in graph:
        path: deque[str] = deque()
        node = root
        while node in graph:
            if node in path:
                cycle = list(path)
                cycle.append(node)
                raise CyclicAliasError(cycle)
            path.append(node)
            node = graph[node]


def validate_definition(defn) -> None:
    '''
    Raise *ABIValidationError* (or subclass) if *defn* is malformed.

    The function performs, in order:

    1. duplicate-name detection
    2. alias-target existence & cyclic-alias detection
    3. struct checks
       * base type exists
       * every field type exists
       * once a field marked ``$`` appears, all following fields *must* also
         be extensions
    4. variant arm type existence
    '''
    defined = _collect_defined(defn)

    _check_duplicates(defn)
    _check_alias_targets(defn, defined)
    _detect_alias_cycles(defn)
    _check_structs(defn, defined)
    _check_base_struct_extensions(defn)
    _check_variants(defn, defined)

    # If we made it here, the ABI is structurally sound.
