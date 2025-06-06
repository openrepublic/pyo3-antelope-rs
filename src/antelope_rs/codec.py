from typing import Any, Type

from antelope_rs import (
    VarUInt32,
    VarInt32,
    Float128,
    Name,
    Checksum160,
    Checksum256,
    Checksum512,
    PublicKey,
    Signature,
    SymbolCode,
    Symbol,
    Asset,
    ExtendedAsset,
    TimePoint,
    TimePointSec,
    BlockTimestamp
)

def enc_hook(obj: Any) -> Any:
    match obj:
        case Float128():
            return str(obj)

        case (
            VarUInt32() |
            VarInt32() |
            Name() |
            SymbolCode() |
            Symbol() |
            TimePoint() |
            TimePointSec() |
            BlockTimestamp()
        ):
            return int(obj)

        case (
            Checksum160() |
            Checksum256() |
            Checksum512() |
            PublicKey() |
            Signature()
        ):
            return obj.raw

        case (
            Asset() | ExtendedAsset()
        ):
            return obj.encode()

        case _:
            raise NotImplementedError(f"Objects of type {type(obj)} are not supported")


def dec_hook(type: Type, obj: Any) -> Any:
    if hasattr(type, 'try_from'):
        return type.try_from(obj)
    else:
        raise NotImplementedError(f"Objects of type {type} are not supported")
