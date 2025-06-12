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
import types
import struct
import base64
import binascii

from typing import (
    Any,
    ClassVar,
    Protocol,
    Self,
    Type,
    runtime_checkable
)

from typing_extensions import TypeAlias as TypingAlias

from antelope_rs.utils import validate_protocol

from ._lowlevel import (
    VarUInt32,
    VarInt32,
    Float128,
    Name,
    Checksum160,
    Checksum256,
    Checksum512,
    PrivateKey,
    PublicKey,
    Signature,
    Asset,
    ExtendedAsset,
    SymbolCode,
    Symbol,
    TimePoint,
    TimePointSec,
    BlockTimestamp,
    ABI,
    ShipABI
)


IOTypes = (
    None | bool | int | float | bytes | str | list | dict
)


@runtime_checkable
class ABINamespaceType(Protocol):

    __abi_type__: ClassVar[str]

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        ...

    @classmethod
    def try_from(cls, obj: Any) -> Self:
        ...

    @classmethod
    def pretty_def_str(cls, **kwargs) -> str:
        ...

    def to_builtins(self) -> IOTypes:
        ...

    def encode(self) -> bytes:
        ...

    def __eq__(self, value: object, /) -> bool:
        ...


BytesLike = bytes | bytearray | memoryview


BoolLike = BytesLike | int | bool

class Bool:
    value: bool

    def __init__(self, value: bool):
        self.value = value

    def __repr__(self) -> str:
        return self.value.__repr__()

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        return cls(bool(raw[0]))

    @classmethod
    def try_from(cls, obj: BoolLike):
        if isinstance(obj, BytesLike):
            obj = obj[0]

        if isinstance(obj, cls):
            obj = obj.value

        return cls(bool(obj))

    @classmethod
    def pretty_def_str(cls) -> str:
        return cls.__name__

    def to_builtins(self) -> bool:
        return self.value

    def encode(self) -> bytes:
        return (
            b'\1'
            if self.value
            else b'\0'
        )

    def __eq__(self, value: object, /) -> bool:
        return self.value == value

    def __hash__(self) -> int:
        return self.value.__hash__()


class Bytes:
    value: bytes

    def __init__(self, n: bytes | bytearray | memoryview):
        self.value = bytes(n)

    def __len__(self) -> int:
        return len(self.value)

    def __repr__(self) -> str:
        return self.value.__repr__()

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        length = VarUInt32.from_bytes(raw).encode_length
        return cls(raw[length:])

    @classmethod
    def try_from(cls, obj: BytesLike):
        if isinstance(obj, str):
            # 1.  recognise base-64 first
            try:
                return cls(base64.b64decode(obj, validate=True))
            except binascii.Error:
                pass                                    # not base-64

            # 2.  then fall back to hex
            try:
                return cls(bytes.fromhex(obj))
            except ValueError:
                raise ValueError("Bytes: string is neither valid hex nor base-64") from None

        if isinstance(obj, cls):
            obj = obj.value

        return cls(obj)

    @classmethod
    def pretty_def_str(cls) -> str:
        return cls.__name__

    def to_builtins(self) -> bytes:
        return self.value

    def encode(self) -> bytes:
        return VarUInt32.from_int(len(self)).encode() + self.value

    def __eq__(self, value: object, /) -> bool:
        return self.value == value

    def __hash__(self) -> int:
        return self.value.__hash__()


StringLike = BytesLike | str


class StrBase(str):
    re: ClassVar[str | None]
    min_length: ClassVar[int | None]
    max_length: ClassVar[int | None]

    __abi_type__: ClassVar[str] = 'string'

    def __init__(self, s: str):
        self.value = s

    def __len__(self) -> int:
        return len(self.value)

    def __str__(self) -> str:
        return self.value

    def __repr__(self) -> str:
        return self.value.__repr__()

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        length = VarUInt32.from_bytes(raw).encode_length
        if isinstance(raw, memoryview):
            raw = bytes(raw)

        return cls(raw[length:].decode('utf-8'))

    @classmethod
    def try_from(cls, obj: StringLike):
        if isinstance(obj, bytes | bytearray | memoryview):
            obj = bytearray(obj).decode(encoding='utf-8')

        if isinstance(obj, cls):
            obj = obj.value

        if (
            isinstance(cls.min_length, int)
            and
            len(obj) < cls.min_length
        ):
            raise ValueError(
                f'{cls.__name__} has minimun length of {cls.min_length} but got: \"{obj}\"'
            )

        if (
            isinstance(cls.max_length, int)
            and
            len(obj) > cls.max_length
        ):
            raise ValueError(
                f'{cls.__name__} has maximun length of {cls.max_length} but got: \"{obj}\"'
            )

        if (
            isinstance(cls.re, str)
            and
            re.fullmatch(cls.re, obj) is None
        ):
            raise ValueError(
                f'{cls.__name__} requires input match regex \"{cls.re}\"'
            )

        return cls(obj)

    @classmethod
    def pretty_def_str(cls) -> str:
        return cls.__name__

    def to_builtins(self) -> str:
        return self.value

    def encode(self, *args, **kwargs) -> bytes:
        utf8 = self.value.encode('utf-8')
        return VarUInt32.from_int(len(utf8)).encode() + utf8

    def __eq__(self, value: object, /) -> bool:
        return self.value == value

    def __lt__(self, value: object, /) -> bool:
        return str(self) < str(value)

    def __gt__(self, value: object, /) -> bool:
        return str(self) > str(value)

    def __hash__(self) -> int:
        return self.value.__hash__()


def make_string_type(
    name: str,
    re: str | None = None,
    min_length: int | None = None,
    max_length: int | None = None
) -> Type[StrBase]:
    return type(name, (StrBase,), {
        're': re,
        'min_length': min_length,
        'max_length': max_length,
        '__annotations__': {'value': str}
    })


String: TypingAlias = make_string_type('String')  # type: ignore

# type names, alphanumeric can have multiple modifiers (?, $ & []) at the end
regex_type_name = r'^([A-Za-z_][A-Za-z0-9_]*)(?:\[\]|\?|\$)*$'

TypeNameStr: TypingAlias = make_string_type(  # type: ignore
    'TypeNameStr',
    re=regex_type_name,
    min_length=1
)

# ABI struct fields, alphanumeric + '_'
regex_field_name = r'^[A-Za-z_][A-Za-z0-9_]*$'

FieldNameStr: TypingAlias = make_string_type(  # type: ignore
    'FieldNameStr',
    re=regex_field_name,
    min_length=1
)

# Antelope account name(uint64), empty string, lowercase letters, only numbers
# 1-5 & '.'
regex_antelope_name = r'^(?:$|[a-z][a-z1-5\.]{0,11}[a-j1-5\.]?)$'

AntelopeNameStr: TypingAlias = make_string_type(  # type: ignore
    'AntelopeNameStr',
    re=regex_antelope_name,
    min_length=0,
    max_length=13
)

# ABI struct, base type name string, like TypeNameStr but with no modifiers,
# and empty string is allowed
regex_base_type_name = r'^$|^[A-Za-z_][A-Za-z0-9_]*$'

BaseTypeNameStr: TypingAlias = make_string_type(  # type: ignore
    'BaseTypeNameStr',
    re=regex_base_type_name,
    min_length=0,
)

class BitsBase:
    bit_length: ClassVar[int]
    length: ClassVar[int]

    __abi_type__: ClassVar[str]

    def __init__(self, n: bytes | bytearray | memoryview):
        self.value = bytes(n)

    def __len__(self) -> int:
        return self.length

    def __repr__(self) -> str:
        return self.value.__repr__()

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        return cls(raw)

    @classmethod
    def try_from(cls, obj: BytesLike):
        if isinstance(obj, str):
            obj = bytes.fromhex(obj)

        if len(obj) != cls.length:
            raise ValueError(
                f'{cls.__name__} requires exactly {cls.length} bytes but {len(obj)} where provided'
            )

        if isinstance(obj, cls):
            obj = obj.value

        return cls(obj)

    @classmethod
    def pretty_def_str(cls) -> str:
        return cls.__name__

    def to_builtins(self) -> bytes:
        return self.value

    def encode(self) -> bytes:
        return self.value

    def __eq__(self, other: Any) -> bool:
        return (
            hasattr(other, 'value')
            and
            self.value == other.value
        )

    def __hash__(self) -> int:
        return self.value.__hash__()


def make_bits_type(bit_length: int) -> type:
    name = f'Bits{bit_length}'
    return type(name, (BitsBase,), {
        'bit_length': bit_length,
        'length': bit_length // 8,
        '__abi_type__': f'fixed_bytes[{bit_length // 8}]',
        '__annotations__': {'value': bytes}
    })

Bits8: TypingAlias = make_bits_type(8)  # type: ignore
Bits16: TypingAlias = make_bits_type(16)  # type: ignore
Bits32: TypingAlias = make_bits_type(32)  # type: ignore
Bits64: TypingAlias = make_bits_type(64)  # type: ignore
Bits128: TypingAlias = make_bits_type(128)  # type: ignore
Bits160: TypingAlias = make_bits_type(160)  # type: ignore
Bits192: TypingAlias = make_bits_type(192)  # type: ignore
Bits256: TypingAlias = make_bits_type(256)  # type: ignore
Bits512: TypingAlias = make_bits_type(512)  # type: ignore


# msgspec compatible type hints for all ABI builtin types

_uint_ranges: dict[int, tuple[int, int]] = {
    bits: (0, (1 << bits) - 1)
    for bits in (8, 16, 32, 64, 128)
}

_int_ranges: dict[int, tuple[int, int]] = {
    bits: (-(1 << (bits - 1)), (1 << (bits - 1)) - 1)
    for bits in (8, 16, 32, 64, 128)
}

_float_ranges: dict[int, tuple[float, float]] = {
    32: (-3.4028234663852886e38, 3.4028234663852886e38),
    64: (-1.7976931348623157e308, 1.7976931348623157e308),
}


IntLike = bytes | str | int


class IntBase:
    bits: ClassVar[int]
    signed: ClassVar[bool]
    _min: ClassVar[int]
    _max: ClassVar[int]

    __abi_type__: ClassVar[str]

    def __init__(self, n: int):
        self.value = n

    def __int__(self):
        return self.value

    def __repr__(self) -> str:
        return self.value.__repr__()

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        return cls(int.from_bytes(
      raw,
            byteorder='little',
            signed=cls.signed
        ))

    @classmethod
    def try_from(cls, obj: IntLike):
        if isinstance(obj, BytesLike):
            return cls.from_bytes(obj)

        num = int(obj)
        if not (cls._min <= num <= cls._max):
            kind = 'signed' if cls.signed else 'unsigned'
            raise ValueError(
                f'{num} out of range ({cls._min}, {cls._max}) for {cls.bits}-bit {kind}'
            )

        if isinstance(obj, cls):
            obj = obj.value

        return cls(num)

    @classmethod
    def pretty_def_str(cls) -> str:
        return cls.__name__

    def to_builtins(self) -> int:
        return self.value

    def encode(self) -> bytes:
        return self.value.to_bytes(
            length=self.bits // 8,
            byteorder='little',
            signed=self.signed
        )

    def __eq__(self, value: object, /) -> bool:
        return self.value == value

    def __hash__(self) -> int:
        return self.value


def make_int_type(bits: int, signed: bool) -> type:
    name = f'Int{bits}' if signed else f'UInt{bits}'
    _min, _max = (_int_ranges if signed else _uint_ranges)[bits]
    return type(name, (IntBase,), {
        'bits': bits,
        'signed': signed,
        '_min': _min,
        '_max': _max,
        '__abi_type__': name.lower(),
        '__annotations__': {'value': int}
    })


FloatLike = bytes | str | float


class FloatBase:
    bits: ClassVar[int]
    _min: ClassVar[float]
    _max: ClassVar[float]

    __abi_type__: ClassVar[str]

    def __init__(self, x: float):
        self.value = x

    def __float__(self):
        return self.value

    def __repr__(self) -> str:
        return self.value.__repr__()

    @classmethod
    def from_bytes(cls, raw: BytesLike) -> Self:
        fstr = 'f' if cls.bits == 32 else 'd'
        return cls(struct.unpack(  # type: ignore
            f'<{fstr}',
            raw
        ))

    @classmethod
    def try_from(cls, obj: FloatLike):
        if isinstance(obj, bytes):
            fstr = 'f' if cls.bits == 32 else 'd'
            _, obj = struct.unpack(f'<{fstr}', obj)

        num = float(obj)
        if not (cls._min <= num <= cls._max):
            raise ValueError(
                f'{num} out of range ({cls._min}, {cls._max}) for {cls.bits}-bit float'
            )

        if isinstance(obj, cls):
            obj = obj.value

        return cls(num)

    @classmethod
    def pretty_def_str(cls) -> str:
        return cls.__name__

    def to_builtins(self) -> float:
        return self.value

    def encode(self) -> bytes:
        fstr = 'f' if self.bits == 32 else 'd'
        return struct.pack(
            f'<{fstr}',
            self.value
        )

    def __eq__(self, value: object, /) -> bool:
        return self.value == value

    def __hash__(self) -> int:
        return self.value.__hash__()


def make_float_type(bits: int) -> type:
    _min, _max = _float_ranges[bits]
    return type(f'Float{bits}', (FloatBase,), {
        'bits': bits,
        '_min': _min,
        '_max': _max,
        '__abi_type__': f'float{bits}',
        '__annotations__': {'value': float}
    })


Int8: TypingAlias = make_int_type(8, True)  # type: ignore
Int16: TypingAlias = make_int_type(16, True)  # type: ignore
Int32: TypingAlias = make_int_type(32, True)  # type: ignore
Int64: TypingAlias = make_int_type(64, True)  # type: ignore
Int128: TypingAlias = make_int_type(128, True)  # type: ignore

UInt8: TypingAlias = make_int_type(8, False)  # type: ignore
UInt16: TypingAlias = make_int_type(16, False)  # type: ignore
UInt32: TypingAlias = make_int_type(32, False)  # type: ignore
UInt64: TypingAlias = make_int_type(64, False)  # type: ignore
UInt128: TypingAlias = make_int_type(128, False)  # type: ignore

integer_classes: tuple[Type[IntBase], ...] = (
    Int8, Int16, Int32, Int64, Int128,
    UInt8, UInt16, UInt32, UInt64, UInt128,
)

Float32: TypingAlias = make_float_type(32)  # type: ignore
Float64: TypingAlias = make_float_type(64)  # type: ignore

float_classes: tuple[Type[FloatBase], ...] = (Float32, Float64)


# map std names to builtin_classes
builtin_class_map: dict[str, Type[Any]] = {
    'bool': Bool,

    'int8': Int8,
    'int16': Int16,
    'int32': Int32,
    'int64': Int64,
    'int128': Int128,

    'uint8': UInt8,
    'uint16': UInt16,
    'uint32': UInt32,
    'uint64': UInt64,
    'uint128': UInt128,

    'varuint32': VarUInt32,
    'varint32': VarInt32,

    'float32': Float32,
    'float64': Float64,
    'float128': Float128,

    'time_point': TimePoint,
    'time_point_sec': TimePointSec,
    'block_timestamp_type': BlockTimestamp,

    'name': Name,

    'bytes': Bytes,
    'string': String,

    'checksum160': Checksum160,
    'checksum256': Checksum256,
    'checksum512': Checksum512,

    'public_key': PublicKey,
    'signature': Signature,

    'symbol': Symbol,
    'symbol_code': SymbolCode,

    'asset': Asset,
    'extended_asset': ExtendedAsset
}

for type_name, cls in builtin_class_map.items():
    cls.__abi_type__ = type_name


lowlevel_builtin_classes: tuple[Type[Any], ...] = (
    VarUInt32,
    VarInt32,
    Float128,
    Name,
    Checksum160,
    Checksum256,
    Checksum512,
    PrivateKey,
    PublicKey,
    Signature,
    Asset,
    ExtendedAsset,
    SymbolCode,
    Symbol,
    TimePoint,
    TimePointSec,
    BlockTimestamp,
    ABI,
    ShipABI
)

builtin_classes: tuple[Type[Any], ...] = tuple(set((
    Bool,
    Bytes,
    Bits8,
    Bits16,
    Bits32,
    Bits64,
    Bits128,
    Bits160,
    Bits192,
    Bits256,
    Bits512,
    String,
    TypeNameStr,
    FieldNameStr,
    AntelopeNameStr,
    BaseTypeNameStr,
    *integer_classes,
    *float_classes,
    *lowlevel_builtin_classes
)))

# sanity check
for cls in builtin_classes:
    if cls in (PrivateKey, ShipABI, ABI):
        continue

    validate_protocol(cls, ABINamespaceType)


# try_from hints for all classes
Int8Like = Bits8 | str | int
Int16Like = Bits16 | str | int
Int32Like = Bits32 | str | int
Int64Like = Bits64 | str | int
Int128Like = Bits128 | str | int

UInt8Like = Bits8 | str | int
UInt16Like = Bits16 | str | int
UInt32Like = Bits32 | str | int
UInt64Like = Bits64 | str | int
UInt128Like = Bits128 | str | int

NameBytes = Bits64
Sum160Bytes = Bits160
Sum256Bytes = Bits256
Sum512Bytes = Bits512
SymCodeBytes = Bits64
SymbolBytes = Bits64
AssetBytes = Bits128
ExtAssetBytes = Bits192
TimePointBytes = Bits64
TimePointSecBytes = Bits32
BlockTimestampBytes = Bits32


# typing hints for each builtin class supporting try_from
VarUInt32Like = Bits32 | int | str | VarUInt32
VarInt32Like = Bits32 | int | str | VarInt32
Float128Like = Bits128 | str | float | Float128
NameLike = NameBytes | int | str | Name
Sum160Like = Sum160Bytes | str | Checksum160
Sum256Like = Sum256Bytes | str | Checksum256
Sum512Like = Sum512Bytes | str | Checksum512
PrivKeyLike = bytes | str | PrivateKey
PubKeyLike = bytes | str | PublicKey
SigLike = bytes | str | Signature
SymCodeLike = SymCodeBytes | int | str | SymbolCode
SymLike = SymbolBytes | int | str | Symbol
AssetLike = AssetBytes | str | Asset
ExtAssetLike = ExtAssetBytes | str | ExtendedAsset
TimePointLike = TimePointBytes | int | str | TimePoint
TimePointSecLike = TimePointSecBytes | int | str | TimePointSec
BlockTimestampLike = BlockTimestampBytes | int | str | BlockTimestamp
