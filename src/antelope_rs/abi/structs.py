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

import re
import inspect

from typing import (
    Any,
    ForwardRef,
    Type,
    Union,
    get_type_hints
)

import msgspec

import antelope_rs
from antelope_rs.meta import (
    TypeAlias,
    TypeNameStr,
    builtin_class_map,
    builtin_classes,
)
from ..codec import dec_hook, enc_hook


# ABI namespace type debug str helpers

def _fmt_type(tp: Any) -> str:
    '''
    Render type annotations, including generics like list[X] or dict[K, V].

    '''
    if isinstance(tp, ForwardRef):
        return tp.__forward_arg__

    origin = getattr(tp, '__origin__', None)
    args = getattr(tp, '__args__', tuple())

    # typing.Union | X | Y
    if origin is Union:
        return ' | '.join(_fmt_type(a) for a in args)

    # Built-in collection generics (PEP-585)
    if origin is list:
        return f'list[{_fmt_type(args[0])}]'
    if origin is tuple:
        return 'tuple[' + ', '.join(_fmt_type(a) for a in args) + ']'
    if origin is dict:
        return f'dict[{_fmt_type(args[0])}, {_fmt_type(args[1])}]'
    if origin is set:
        return f'set[{_fmt_type(args[0])}]'

    # Annotated, Literal, etc. fall back to repr
    if origin is not None:
        return str(tp).replace('typing.', '')

    # Plain classes
    if getattr(tp, '__module__', '') == 'builtins':
        return tp.__name__
    return getattr(tp, '__name__', str(tp))


def _is_type_alias(cls: Any) -> bool:
    '''
    True if *cls* is a TypeAlias subclass.

    '''
    return (
        inspect.isclass(cls)
        and
        issubclass(cls, TypeAlias)
    )


def _is_msgspec_struct(cls: Any) -> bool:
    '''
    True if *cls* looks like a msgspec.Struct subclass (module-agnostic).

    '''
    return (
        inspect.isclass(cls)
        and
        hasattr(cls, "__struct_fields__")
    )


def pretty_abi_type_def(
    obj: Type,
    *,
    indent: int = 0,
    indent_char: str = ' ',
    indent_word: int = 4,
) -> str:
    '''
    Get a pretty string representation of an ABI namespace type.

    For four spaces of extra indentation:
        - indent=1
        - indent_char=' '
        - indent_word=4

    Or for an extra tabulation:
        - indent=1
        - indent_char='\t'
        - indent_word=1

    '''

    # alias / Union
    if _is_type_alias(obj):
        rhs = _fmt_type(obj.__value__)
        return f'{obj.__name__} = {rhs}\n'

    if getattr(obj, '__origin__', None) is Union:
        return _fmt_type(obj) + '\n'

    # Struct
    if not _is_msgspec_struct(obj):
        # builtins
        if obj in builtin_classes:
            return f'{obj.__name__}\n'

        raise TypeError(
            f'# {obj!r} is neither Struct nor recognised alias (type={type(obj)})'
        )

    # figure out bases, tag, tag_field
    bases = [
        b.__name__
        for b in obj.__bases__
        if _is_msgspec_struct(b) and b is not msgspec.Struct
    ]
    bases_str = ', '.join(bases) or 'msgspec.Struct'

    cfg = getattr(obj, '__struct_config__', None)
    if cfg is not None:  # msgspec >= 0.18
        tag_val = getattr(cfg, 'tag', None)
        tag_field = getattr(cfg, 'tag_field', None)
    else:  # legacy fallback
        tag_val = getattr(obj, '__struct_tag__', None)
        tag_field = None

    # extra will contain any additional struct def args if applicable
    extra = (
        f', tag="{tag_val}"'
        if tag_val is not None else ''
    )

    extra += (
        f', tag_field="{tag_field}"'
        if tag_field is not None and tag_field != 'type' else ''
    )

    # finally create class def
    prefix = indent_char * indent * indent_word
    txt = f'{prefix}class {obj.__name__}({bases_str}{extra}):\n'

    # fields
    try:
        hints = get_type_hints(obj, include_extras=True)
    except NameError:
        # unresolved ForwardRef? fallback
        hints = obj.__annotations__

    prefix = indent_char * (indent + 1) * indent_word
    for fld in obj.__struct_fields__:
        typ = _fmt_type(hints.get(fld, Any))
        txt += f'{prefix}{fld}: {typ}\n'

    return txt


# modifier helpers

def _apply_modifiers(ann, modifiers: list):
    '''
    Wrap *ann* with list/Optional according to rust‐side modifiers.

    '''
    for m in reversed(modifiers):  # inner-first in Python
        m = (
            m.as_str()
            if hasattr(m, 'as_str')
            else
            str(m)
        )
        if m == 'array':
            ann = list[ann]  # PEP-585

        elif m in ('optional', 'extension'):
            ann = Union.__getitem__((ann, type(None)))  # type: ignore

    return ann


def _union(*xs):
    '''
    Return a typing.Union that copes with ForwardRefs.

    '''
    conv = [
        ForwardRef(x)  # maybe promote str -> FwdRef
        if isinstance(x, str)
        else x
        for x in xs
    ]

    if len(conv) == 1:
        return conv[0]

    return Union.__getitem__(tuple(conv))  # type: ignore


# name helpers

_CAMEL_RE = re.compile(r"(?:^|_)([a-zA-Z0-9]+)")

def to_camel(s: TypeNameStr) -> str:
    '''
    Convert snake_case string to CamelCase

    '''
    return ''.join(
        m.group(1).capitalize()
        for m in _CAMEL_RE.finditer(str(s))
    )


# main builder

def build_struct_namespace(
    abi: 'antelope_rs.abi.ABIView',
    *,
    tag_field: str = 'type',
) -> dict[str, type]:
    '''
    Generate python classes mirroring *abi*’s structs/aliases.

    '''
    structs: dict[str, Type] = {}
    pyname_map: dict[str, str] = {}  # ABI type name -> CamelCase

    tagged_structs = {
        abi.resolve_type(t).resolved_name
        for v in abi.variants
        for t in v.types
        if abi.resolve_type(t).is_struct
    }

    def resolve_ann(type_str: str):
        info = abi.resolve_type(type_str)

        # base
        if info.is_std:
            ann = builtin_class_map[info.resolved_name]

        elif info.is_struct is not None:
            py_name = pyname_map.setdefault(
                info.resolved_name, to_camel(info.resolved_name)
            )
            ann = structs.get(info.resolved_name) or ForwardRef(py_name)

        elif info.is_variant is not None:
            # forward-ref to the alias – will be patched later
            py_name = pyname_map.setdefault(
                info.resolved_name, to_camel(info.resolved_name)
            )
            ann = ForwardRef(py_name)

        else:  # pragma: no cover - rust guarantees we never get here
            raise KeyError(f"Unhandled ABI kind for {type_str}")

        # wrap with modifiers
        return _apply_modifiers(ann, info.modifiers)

    # build dependency graph                                               #
    pending = {s.name: s for s in abi.structs}
    deps = {name: set() for name in pending}

    for name, s in pending.items():
        if s.base:
            base_info = abi.resolve_type(s.base)
            if base_info.is_struct:
                deps[name].add(base_info.resolved_name)

        for f in s.fields:
            finfo = abi.resolve_type(f.type_)
            if finfo.is_struct:
                deps[name].add(finfo.resolved_name)

    while pending:
        ready = [n for n, d in deps.items() if d <= structs.keys()]
        if not ready:  # break cycles with fwd refs
            ready = list(pending.keys())

        for name in ready:
            sdef = pending.pop(name)
            deps.pop(name, None)

            py_name = pyname_map.setdefault(name, to_camel(name))

            bases = ((structs[sdef.base],) if sdef.base else ())

            fields = [
                (str(fld.name), resolve_ann(fld.type_))
                for fld in sdef.fields
            ]

            kw = {}
            if name in tagged_structs:
                kw = dict(tag_field=tag_field, tag=str(name))

            cls = msgspec.defstruct(
                py_name,
                fields,
                bases=bases,
                module=__name__,
                **kw,
            )

            def gen_eq_for_cls(cls):
                field_names = cls.__struct_fields__
                def _eq(self, other) -> bool:
                    for name in field_names:
                        if not (
                            hasattr(self, name)
                            or
                            hasattr(other, name)
                            or
                            getattr(self, name) == getattr(other, name)
                        ):
                            return False

                    return True

                return _eq

            cls.__eq__ = gen_eq_for_cls(cls)

            # export to namespace
            structs[name] = cls

    # variant aliases
    for vdef in abi.variants:
        v_name = vdef.name  # snake_case
        py_name  = pyname_map.setdefault(v_name, to_camel(v_name))

        # build the Union (may be one member only)
        union_ann = _union(*(resolve_ann(t) for t in vdef.types))

        # keep *alias* identity even for 1-item variants
        alias_obj = TypeAlias.from_target(py_name, union_ann)

        structs[v_name] = alias_obj

    # publish aliases
    for alias in abi.types:
        alias_name = alias.new_type_name
        if alias_name in structs:  # already registered
            continue

        # If something with that snake_case name is already present
        # (e.g. a real struct), just add the CamelCase mirror and move on.
        if alias_name in structs:
            continue

        resolved_ann = resolve_ann(alias_name)
        alias_obj = TypeAlias.from_target(
            to_camel(alias_name), resolved_ann
        )

        structs[alias_name] = alias_obj

    # add converters & pretty_str as well as re-export CamelCase type in namespace
    for name, cls in list(structs.items()):
        if not hasattr(cls, 'try_from'):
            def _try_from(cls, obj: Any) -> msgspec.Struct:
                return msgspec.convert(
                    obj,
                    type=cls,
                    dec_hook=dec_hook
                )

            cls.try_from = classmethod(_try_from)

        def _to_builtins(self) -> Any:
            return msgspec.to_builtins(
                self,
                enc_hook=enc_hook
            )

        cls.to_builtins = _to_builtins

        def _pretty_def_str(cls, **kwargs) -> str:
            return pretty_abi_type_def(cls, **kwargs)

        cls.pretty_def_str = classmethod(_pretty_def_str)
        cls.__abi_type__ = name

        cam_name = to_camel(name)
        structs[cam_name] = cls

    return structs
