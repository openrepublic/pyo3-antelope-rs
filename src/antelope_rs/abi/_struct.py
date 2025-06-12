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

import types
from typing import (
    Any,
    ClassVar,
    ForwardRef,
    Self,
    Sequence,
    Type,
    Union,
    get_args,
    get_origin,
    get_type_hints
)

import msgspec

from antelope_rs.codec import dec_hook
from antelope_rs.meta import BytesLike, IOTypes, VarUInt32
from antelope_rs.utils import to_snake

from ._defs import ABIResolvedType


class TypeAlias:
    __abi_type__: str
    __value__:    type
    __slots__ = ('_inner',)

    def __init__(self, inner):
        self._inner = inner

    # class factory
    @staticmethod
    def from_target(alias: str, target: type) -> type['TypeAlias']:
        return types.new_class(
            alias, (TypeAlias,),
            exec_body=lambda ns: ns.update({
                '__module__' : __name__,
                '__qualname__': alias,
                '__abi_type__': to_snake(alias),
                '__value__'   : target,
            })
        )

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        return cls.__value__.from_bytes(raw)

    @classmethod
    def try_from(cls, obj):
        inner = msgspec.convert(obj, type=cls.__value__, dec_hook=dec_hook)
        return inner if isinstance(inner, cls) else cls(inner)

    def to_builtins(self):
        return self._inner.to_builtins()

    def encode(self):
        return self._inner.encode()

    def __repr__(self):
        return f'{type(self).__name__}({self._inner!r})'

    def __eq__(self, o):
        return self._inner == (o._inner if isinstance(o, TypeAlias) else o)

    def __hash__(self):
        return hash(self._inner)

    @classmethod
    def pretty_def_str(cls, **kwargs) -> str:
        target = cls.__value__
        real = getattr(target, '__name__', str(target))
        return f'TypeAlias[{real}]'


class Variant(TypeAlias):
    '''
    Discriminated-union wrapper.

    * ``__value__`` is a ``typing.Union[...]`` whose arms are **all**
      ``msgspec.Struct`` subclasses (scalars get promoted via a wrapper).
    * ``encode`` adds a `VarUInt32` discriminator in *declaration* order.

    '''
    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        '''
        Decode *raw* into the appropriate union arm.

        Layout (by convention - see ``encode``):

            VarUInt32  variant-index
            <bytes>    payload encoded by the selected arm type
        '''

        # read the tag
        tag_vu = VarUInt32.from_bytes(raw)
        tag  = int(tag_vu)
        offset = tag_vu.encode_length  # bytes consumed

        arms = get_args(cls.__value__) or (cls.__value__,)
        if tag >= len(arms):
            raise ValueError(
                f'{cls.__name__}: tag {tag} outside valid range 0..{len(arms)-1}'
            )

        arm_cls = arms[tag]
        payload = raw[offset:]

        # delegate to the arm's decoder
        if hasattr(arm_cls, 'from_bytes'):
            inner = arm_cls.from_bytes(payload)

        # scalar wrapper Struct (single ``value`` field)
        elif (
            isinstance(arm_cls, type)
            and
            issubclass(arm_cls, msgspec.Struct)
            and
            getattr(arm_cls, '__struct_fields__', ()) == ('value',)
        ):
            val_ann = arm_cls.__annotations__['value']
            if not hasattr(val_ann, 'from_bytes'):
                raise TypeError(
                    f'{cls.__name__}: inner type {val_ann} lacks from_bytes()'
                )
            inner = arm_cls(  # wrap scalar
                val_ann.from_bytes(payload)
            )

        else:
            raise TypeError(
                f'{cls.__name__}: cannot decode arm {arm_cls!r}'
            )

        return inner if isinstance(inner, cls) else cls(inner)

    @classmethod
    def try_from(cls, obj):
        target = cls.__value__

        try:
            inner = msgspec.convert(obj, type=target, dec_hook=dec_hook)

        except msgspec.ValidationError as first_err:
            # fallback: scalar -> wrapper Struct
            origin = get_origin(target)
            if origin is Union:  # only for unions
                for alt in get_args(target):
                    if (
                        isinstance(alt, type)  # is a class
                        and
                        issubclass(alt, msgspec.Struct)
                        and
                        getattr(alt, '__struct_fields__', ()) == ('value',)
                    ):
                        try:
                            val_type = alt.__annotations__['value']
                            val = msgspec.convert(
                                obj, type=val_type,
                                dec_hook=dec_hook
                            )
                            inner = alt(val)  # wrap scalar -> Struct
                            break

                        except msgspec.ValidationError:
                            pass
                else:
                    # no arm matched the scalar -> re-raise original error
                    raise first_err
            else:
                raise first_err

        return inner if isinstance(inner, cls) else cls(inner)

    def encode(self):
        union_ann = type(self).__value__
        # typing.get_args() is empty when the variant has a single arm,
        # so fall back to treating the annotation itself as that arm.
        arms = get_args(union_ann) or (union_ann,)
        arm_type = type(self._inner)

        # exact-type match or subclass (covers wrapper structs)
        try:
            idx = arms.index(arm_type)
        except ValueError:
            for i, a in enumerate(arms):
                if issubclass(arm_type, a):
                    idx = i
                    break
            else:
                raise TypeError(f'{arm_type.__name__} not in Variant arms')

        return VarUInt32.from_int(idx).encode() + self._inner.encode()

    def to_builtins(self):
        out = self._inner.to_builtins()

        if isinstance(out, dict):
            tag = getattr(type(self._inner), '__abi_type__')
            out = {'antelope_type': tag, **out}

        return out

    @classmethod
    def pretty_def_str(cls, **kwargs) -> str:
        target = cls.__value__

        arms = get_args(target) or (target,)
        arm_names = ', '.join(
            getattr(a, '__name__', str(a)) for a in arms
        )
        return f'Variant[{arm_names}]'


class Struct(msgspec.Struct, frozen=True):
    __abi_type__: ClassVar[str]
    __abi_fields__: ClassVar[list[tuple[str, ABIResolvedType]]]

    # NOTE: the heavy lifting (per-field modifier resolution) happens **once**
    #       and is cached on the *class* - instances just reuse it.
    _ENC_PIPELINES: ClassVar[list[tuple[str, list[str]]]]
    _BLT_PIPELINES: ClassVar[list[tuple[str, list[str]]]]

    def _encode_val(self, val: Any, modifiers: Sequence[str]) -> bytes:  # noqa: C901
        if not modifiers:
            if hasattr(val, 'encode'):
                return val.encode()

            if isinstance(val, list):
                return b''.join(
                    self._encode_val(item, ()) for item in val
                )

            if val is None:  # only happens for '$'
                return b''

            raise ValueError(
                f'Cannot encode value {val!r} ({type(val).__name__})'
            )

        outer, *inner = modifiers
        match outer:
            case 'optional':
                return (
                    b'\x00'
                    if val is None
                    else b'\x01' + self._encode_val(val, inner)
                )

            case 'extension':
                return (
                    b''
                    if val is None
                    else self._encode_val(val, inner)
                )

            case 'array':
                if val is None:
                    raise ValueError('Array value cannot be None')

                prefix = VarUInt32.from_int(len(val)).encode()
                payload = b''.join(
                    self._encode_val(item, inner) for item in val
                )
                return prefix + payload

        raise AssertionError(f'Unknown modifier {outer!r}')

    def _to_builtins_val(self, val: Any, modifiers: Sequence[str]) -> IOTypes:  # noqa: C901
        if not modifiers:
            if hasattr(val, 'to_builtins'):
                return val.to_builtins()
            if isinstance(val, list):
                return [
                    self._to_builtins_val(item, ())
                    for item in val
                ]
            return val

        outer, *inner = modifiers
        match outer:
            case 'optional' | 'extension':
                return (
                    None
                    if val is None
                    else self._to_builtins_val(val, inner)
                )

            case 'array':
                if val is None:
                    raise ValueError('Array field cannot be None')
                if not isinstance(val, list):
                    raise TypeError(
                        f'Expected list, got {type(val).__name__}'
                    )
                return [
                    self._to_builtins_val(item, inner)
                    for item in val
                ]

        raise AssertionError(f'Unknown modifier {outer!r}')

    @classmethod
    def from_bytes(cls: Type[S], raw: BytesLike) -> S:
        '''
        Reconstruct *cls* from the binary representation produced by
        :py:meth:`encode`.

        The decoding algorithm is the exact inverse of `_encode_val`:

        * iterate over ``_ENC_PIPELINES`` in order;
        * consume from a growing offset inside the input buffer;
        * honour the same ``optional | extension | array`` modifiers;
        * delegate leaf-level decoding to the *base* ABI type's
          ``from_bytes`` constructor.

        '''
        buf = memoryview(raw)
        off = 0
        kv = {}  # kwargs for cls(...)
        hints = get_type_hints(cls, include_extras=True)

        def _base_ann(ann: Any, mods: Sequence[str]) -> Type[Any]:
            '''
            Strip *mods* (outer-to-inner) from *ann* and return the
            underlying ABI type.
            '''
            for m in mods:
                match m:
                    case 'array':
                        # list[T]  ->  T
                        ann = get_args(ann)[0]
                    case 'optional' | 'extension':
                        # Union[T, None]  ->  T
                        ann = next(a for a in get_args(ann)
                                   if a is not type(None))
            # ForwardRef? - resolve lazily from the current module
            if isinstance(ann, ForwardRef):
                ann = eval(ann.__forward_arg__, cls.__module__.__dict__)  # type: ignore
            return ann   # type: ignore[return-value]

        def _decode_val(
            base: Type[Any],
            view: memoryview,
            mods: Sequence[str],
        ) -> tuple[Any, int]:
            '''
            Decode a single field and return *(value, bytes_consumed)*.
            '''
            if not mods:                       # leaf
                val = base.from_bytes(view)    # ABINamespaceType API
                return val, len(val.encode())  # round-trip for length

            outer, *inner = mods
            match outer:
                # --------------------------- optional ------------------ #
                case 'optional':
                    flag = view[0]
                    if flag == 0:
                        return None, 1
                    val, used = _decode_val(base, view[1:], inner)
                    return val, 1 + used

                # --------------------------- extension ----------------- #
                case 'extension':
                    # present only if there is *anything* left
                    if len(view) == 0:
                        return None, 0
                    return _decode_val(base, view, inner)

                # --------------------------- array --------------------- #
                case 'array':
                    size_v = VarUInt32.from_bytes(view)
                    cnt    = int(size_v)
                    head   = size_v.encode_length
                    items  = []
                    pos    = head
                    for _ in range(cnt):
                        item, used = _decode_val(base, view[pos:], inner)
                        items.append(item)
                        pos += used
                    return items, pos

            raise AssertionError(f'Unknown modifier {outer!r}')

        # -----------------------------------------------------------------
        # main loop - walk the pipelines in order
        # -----------------------------------------------------------------
        for fname, mods in cls._ENC_PIPELINES:
            ann_base   = _base_ann(hints[fname], mods)
            val, used  = _decode_val(ann_base, buf[off:], mods)
            kv[fname]  = val
            off       += used

        return cls(**kv)

    @classmethod
    def try_from(cls: Type[S], obj: Any) -> S:        # noqa: N805
        return msgspec.convert(obj, type=cls, dec_hook=dec_hook)

    @classmethod
    def pretty_def_str(cls: Type[S], **kwargs) -> str:
        from antelope_rs.abi._struct_ns import pretty_abi_type_def
        return pretty_abi_type_def(
            cls, **kwargs
        )

    def encode(self) -> bytes:
        buf = bytearray()
        for fname, mods in self._ENC_PIPELINES:
            buf.extend(
                self._encode_val(getattr(self, fname), mods)
            )
        return bytes(buf)

    def to_builtins(self) -> IOTypes:
        out: dict[str, IOTypes] = {}
        for fname, mods in self._BLT_PIPELINES:
            out[fname] = self._to_builtins_val(
                getattr(self, fname),
                mods,
            )
        return out

    def __repr__(self) -> str:
        fields = ', '.join(
            f'{f}={getattr(self, f)!r}' for f, _ in self._ENC_PIPELINES
        )
        return f'{type(self).__name__}({fields})'

    def __eq__(self, other: object) -> bool:
        cls = type(self)
        if not isinstance(other, cls):
            return False

        for name in cls.__struct_fields__:
            self_val = getattr(self, name)
            other_val = getattr(other, name)
            if self_val != other_val:
                return False

        return True

S = Struct
