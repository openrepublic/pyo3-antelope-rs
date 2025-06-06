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
from typing import (
    Any,
    Type,
)

from .abi import (
    Int32Bytes,
    Int64Bytes as Int64Bytes,
    AntelopeNameStr as AntelopeNameStr,
    AssetBytes as AssetBytes,
    ExtAssetBytes as ExtAssetBytes,
    NameBytes as NameBytes,
    Sum160Bytes as Sum160Bytes,
    Sum160Str as Sum160Str,
    Sum256Bytes as Sum256Bytes,
    Sum256Str as Sum512Bytes,
    Sum512Bytes as Sum256Str,
    Sum512Str as Sum512Str,
    SymCodeBytes as SymCodeBytes,
    SymbolBytes as SymbolBytes,
    ABILike as ABILike,
    ABIView as ABIView
)

from ._lowlevel import (
    VarUInt32 as VarUInt32,
    VarInt32 as VarInt32,

    Float128 as Float128,

    Name as Name,

    PrivateKey as PrivateKey,
    PublicKey as PublicKey,
    Signature as Signature,

    Checksum160 as Checksum160,
    Checksum256 as Checksum256,
    Checksum512 as Checksum512,

    SymbolCode as SymbolCode,
    Symbol as Symbol,
    Asset as Asset,
    ExtendedAsset as ExtendedAsset,

    TimePoint as TimePoint,
    TimePointSec as TimePointSec,
    BlockTimestamp as BlockTimestamp,

    ABI as ABI,
    ShipABI as ShipABI,

    builtin_types as builtin_types,

    sign_tx as sign_tx
)

builtin_classes: tuple[Type[Any], ...] = (
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

# typing hints for each builtin class supporting try_from
VarUInt32Like = Int32Bytes | int | str | VarUInt32
VarInt32Like = Int32Bytes | int | str | VarInt32
Float128Like = bytes | str | float | Float128
NameLike = NameBytes | int | AntelopeNameStr | Name
Sum160Like = Sum160Bytes | Sum160Str | Checksum160
Sum256Like = Sum256Bytes | Sum256Str | Checksum256
Sum512Like = Sum512Bytes | Sum512Str | Checksum512
PrivKeyLike = bytes | str | PrivateKey
PubKeyLike = bytes | str | PublicKey
SigLike = bytes | str | Signature
SymCodeLike = SymCodeBytes | int | str | SymbolCode
SymLike = SymbolBytes | int | str | Symbol
AssetLike = AssetBytes | str | Asset
ExtAssetLike = ExtAssetBytes | str | ExtendedAsset
TimePointLike = Int64Bytes | int | str | TimePoint
TimePointSecLike = Int64Bytes | int | str | TimePointSec
BlockTimestampLike = Int32Bytes | int | str | BlockTimestamp

IOTypes = (
    None | bool | int | float | bytes | str | list | dict
)


# map std names to builtin_classes
builtin_class_map: dict[str, Type[Any]] = {
    'varuint32': VarUInt32,
    'varint32': VarInt32,
    'float128': Float128,
    'name': Name,
    'checksum160': Checksum160,
    'checksum256': Checksum256,
    'checksum512': Checksum512,
    'public_key': PublicKey,
    'signature': Signature,
    'symbol': Symbol,
    'symbol_code': SymbolCode,
    'asset': Asset,
    'extended_asset': ExtendedAsset,
    'time_point': TimePoint,
    'time_point_sec': TimePointSec,
    'block_timestamp_type': BlockTimestamp,
}
