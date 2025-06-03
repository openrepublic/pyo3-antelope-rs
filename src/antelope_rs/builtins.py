# pyo3-antelope-rs: PyO3 bindings to telosnetwork/antelope-rs + some QoL addons
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
'''
Add ``try_from`` style converters to all ``antelope_rs._lowlevel`` builtin
types

'''
from typing import (
    Any,
    Type,
    Iterable,
)

from antelope_rs._lowlevel import (
    ABI,
    ShipABI,
    Name,
    Checksum160,
    Checksum256,
    Checksum512,
    PrivateKey,
    PublicKey,
    Signature,
    SymbolCode,
    Symbol,
    Asset,
    ExtendedAsset,
)
from antelope_rs.abi import (
    AntelopeNameStr,
    Sum160Str,
    Sum256Str,
    Sum512Str
)


# monkey patch try_from into builtin classes

_converters: list[tuple[Type, str]] = [
    (str,   'from_str'),
    (int,   'from_int'),
    (dict,  'from_dict'),
    (bytes, 'from_bytes'),
]

def _maybe_patch_converters_for(cls: Type[Any]):
    if hasattr(cls, 'try_from'):
        return

    # figure out which converters does the class have
    converters = []
    for py_type, ctor_name in _converters:
        if hasattr(cls, ctor_name):
            converters.append((py_type, getattr(cls, ctor_name)))

    if converters:
        # build try_from that uses specific class converters
        def try_from(val: Any) -> Any:
            if isinstance(val, cls):
                return val

            for t, conv in converters:
                if isinstance(val, t):
                    return conv(val)

            raise TypeError(
                f'Can\'t instantiate {cls.__name__} from {type(val).__name__}'
            )

        cls.try_from = staticmethod(try_from)


def _patch_wrappers_converters(classes: Iterable[Type[Any]]) -> None:
    for cls in classes:
        _maybe_patch_converters_for(cls)


builtin_classes: tuple[Type[Any], ...] = (
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
    ABI,
    ShipABI
)

_patch_wrappers_converters(builtin_classes)


# typing hints for each builtin class supporting try_from
NameLike = int | AntelopeNameStr | Name
Sum160Like = bytes | Sum160Str | Checksum160
Sum256Like = bytes | Sum256Str | Checksum256
Sum512Like = bytes | Sum512Str | Checksum512
PrivKeyLike = bytes | str | PrivateKey
PubKeyLike = bytes | str | PublicKey
SigLike = bytes | str | Signature
SymCodeLike = int | str | SymbolCode
SymLike = int | str | Symbol
AssetLike = str | Asset
ExtAssetLike = str | ExtendedAsset

IOTypes = (
    None | bool | int | float | bytes | str | list | dict
)


# map std names to builtin_classes
builtin_class_map: dict[str, Type[Any]] = {
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
}
