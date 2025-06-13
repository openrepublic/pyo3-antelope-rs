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

import inspect

from types import ModuleType
import types
from typing import (
    Any,
    ForwardRef,
    Type,
    Union,
    get_type_hints,
)

import msgspec

import antelope_rs

from antelope_rs.meta import (
    ABINamespaceType,
    BytesLike,
    IOTypes,
    builtin_class_map,
    builtin_classes,
)
from antelope_rs.utils import to_camel, to_snake, validate_protocol

from ._struct import (
    TypeAlias,
    Variant,
    Struct as AntelopeStruct,
)


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
    return inspect.isclass(cls) and issubclass(cls, TypeAlias)


def _is_msgspec_struct(cls: Any) -> bool:
    '''
    True if *cls* looks like a msgspec.Struct subclass (module-agnostic).

    '''
    return inspect.isclass(cls) and hasattr(cls, '__struct_fields__')


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
    extra = f', tag="{tag_val}"' if tag_val is not None else ''

    extra += (
        f', tag_field="{tag_field}"'
        if tag_field is not None and tag_field != 'type'
        else ''
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


# automatic msgspec struct from ABI

# modifier helpers


def _apply_modifiers(ann, modifiers: list) -> Type[Any]:
    '''
    Wrap *ann* with list/Optional according to rust‐side modifiers.

    '''
    for m in reversed(modifiers):  # inner-first in Python
        m = m.as_str() if hasattr(m, 'as_str') else str(m)
        match m:
            case 'array':
                ann = list[ann]  # PEP-585

            case 'optional' | 'extension':
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


class BuiltinWrapper(msgspec.Struct, frozen=True):
    __abi_type__: str
    value: ABINamespaceType

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> ABINamespaceType: ...

    def encode(self) -> bytes: ...

    def to_builtins(self) -> IOTypes: ...


def _wrap_as_struct(
    name: str,
    inner_ann: Any,
    module: ModuleType,
    tag: str,
    tag_field: str,
) -> type[msgspec.Struct]:
    '''
    Produce a 1-field Struct so builtin/alias types can sit inside a tagged union.

    '''
    if isinstance(inner_ann, ForwardRef):
        inner_ann = module.__dict__[to_snake(inner_ann.__forward_arg__)]

    cls: Type[BuiltinWrapper] = msgspec.defstruct(  # type: ignore
        to_camel(name),
        [('value', inner_ann)],
        tag=tag,
        tag_field=tag_field,
        module=module.__name__,
    )

    cls.encode = lambda self: self.value.encode()
    cls.to_builtins = lambda self: self.value.to_builtins()
    cls.__abi_type__ = name

    @classmethod
    def _from_bytes(inner_cls, raw: BytesLike) -> ABINamespaceType:
        val_ann = inner_cls.__annotations__['value']
        return inner_cls(val_ann.from_bytes(raw))

    cls.from_bytes = _from_bytes

    return cls


def _expand_struct_fields(
    abi: antelope_rs.abi.ABIView, sdef: antelope_rs.abi.StructDef
) -> list[tuple[str, antelope_rs.abi.ABIResolvedType]]:
    '''
    Recursivly expand struct base fields

    '''
    abi_fields = []
    if sdef.base:
        base_sdef = abi.struct_map[sdef.base]
        abi_fields = _expand_struct_fields(abi, base_sdef)

    abi_fields += [
        (str(fld.name), abi.resolve_type(fld.type_))
        for fld in abi.struct_map[sdef.name].fields
    ]

    return abi_fields


# main builder


def build_struct_namespace(
    abi: 'antelope_rs.abi.ABIView',
    module: ModuleType,
    *,
    tag_field: str = 'antelope_type',
) -> dict[str, type]:
    '''
    Generate python classes mirroring *abi*’s structs/aliases.

    '''
    structs: dict[str, Type] = {}
    pyname_map: dict[str, str] = {}  # ABI type name -> CamelCase

    tagged_structs: dict[str, tuple[str, int]] = {
        str(abi.resolve_type(t).resolved_name): (str(v.name), i)
        for v in abi.variants
        for i, t in enumerate(v.types)
        if abi.resolve_type(t).is_struct
    }

    def resolve_ann(type_str: str) -> tuple[antelope_rs.abi.ABIResolvedType, Type[Any]]:
        info = abi.resolve_type(type_str)

        # base
        if info.is_std:
            ann = builtin_class_map[info.resolved_name]

        elif info.is_struct is not None:
            py_name = pyname_map.setdefault(
                info.resolved_name, to_camel(info.resolved_name)
            )
            ann = structs.get(str(info.resolved_name)) or ForwardRef(py_name)

        elif info.is_variant is not None:
            # forward-ref to the alias - will be patched later
            py_name = pyname_map.setdefault(
                info.resolved_name, to_camel(info.resolved_name)
            )
            ann = ForwardRef(py_name)

        else:  # pragma: no cover - rust guarantees we never get here
            raise KeyError(f'Unhandled ABI kind for {type_str}')

        # wrap with modifiers
        return info, _apply_modifiers(ann, info.modifiers)

    # build dependency graph                                               #
    pending = {str(s.name): s for s in abi.structs}
    deps = {name: set() for name in pending}

    for name, s in pending.items():
        if s.base:
            base_info = abi.resolve_type(s.base)
            if base_info.is_struct:
                deps[name].add(str(base_info.resolved_name))

        for f in s.fields:
            finfo = abi.resolve_type(f.type_)
            if finfo.is_struct:
                deps[name].add(str(finfo.resolved_name))

    while pending:
        ready = [n for n, d in deps.items() if d <= structs.keys()]
        if not ready:  # break cycles with fwd refs
            ready = list(pending.keys())

        for name in ready:
            sdef = pending.pop(name)
            deps.pop(name, None)

            py_name = pyname_map.setdefault(name, to_camel(name))

            base = structs[str(sdef.base)] if sdef.base else None

            fields = {str(fld.name): resolve_ann(fld.type_) for fld in sdef.fields}

            kw: dict[str, Any] = dict(tag=False, tag_field=None)
            if name in tagged_structs:
                # only the true union variants get a tag
                kw = dict(tag_field=tag_field, tag=str(name))

            bases = tuple(
                b
                for b in (
                    base,
                    AntelopeStruct,
                )  # base first, mix-in last to avoid MRO issues
                if b is not None
            )

            cls: Type[AntelopeStruct] = msgspec.defstruct(  # type: ignore
                py_name,
                [(key, info[1]) for key, info in fields.items()],
                bases=bases if bases else (AntelopeStruct,),
                module=module.__name__,
                **kw,
            )

            cls.__abi_type__ = name
            cls.__abi_fields__ = _expand_struct_fields(abi, sdef)

            pipelines: list[tuple[str, list[str]]] = [
                (fname, list(meta.modifiers)) for fname, meta in cls.__abi_fields__
            ]
            cls._ENC_PIPELINES = pipelines
            cls._BLT_PIPELINES = pipelines

            setattr(module, str(name), cls)
            structs[str(name)] = cls

        # variant aliases
        for vdef in abi.variants:
            v_name = str(vdef.name)
            py_name = pyname_map.setdefault(v_name, to_camel(v_name))

            union_members = []
            for i, t in enumerate(vdef.types):
                info, ann = resolve_ann(t)

                # scalars -> wrapper structs, structs stay untouched
                if info.is_struct:
                    union_members.append(ann)
                else:
                    wrapper = _wrap_as_struct(
                        name=f'{v_name}_{i}',
                        inner_ann=ann,
                        module=module,
                        tag=v_name,
                        tag_field=tag_field,
                    )
                    union_members.append(wrapper)

            union_ann = _union(*union_members)

            # concrete subclass of Variant
            alias_obj = types.new_class(
                py_name,
                (Variant,),
                exec_body=lambda ns: ns.update(
                    {
                        '__module__': module.__name__,
                        '__qualname__': py_name,
                        '__abi_type__': v_name,
                        '__value__': union_ann,
                    }
                ),
            )

            setattr(module, v_name, alias_obj)
            structs[v_name] = alias_obj

    # publish aliases
    for alias in abi.types:
        alias_name = str(alias.new_type_name)
        if alias_name in structs:  # already registered
            continue

        _, resolved_ann = resolve_ann(alias_name)
        alias_obj = TypeAlias.from_target(to_camel(alias_name), resolved_ann)

        setattr(module, alias_name, alias_obj)
        structs[alias_name] = alias_obj

    # add converters & pretty_str as well as re-export CamelCase type in namespace
    for name, cls in list(structs.items()):
        validate_protocol(cls, ABINamespaceType)
        cam_name = to_camel(name)
        setattr(module, cam_name, cls)
        structs[cam_name] = cls

    return structs
