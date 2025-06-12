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
from inspect import isclass
from typing import Any, Type, get_origin

from msgspec import ValidationError

from antelope_rs.meta import builtin_classes


_convertible_classes: set[Type[Any]] = set(builtin_classes)


def _is_type_alias(tp: Type[Any]) -> bool:
    from antelope_rs.abi._struct import TypeAlias
    return isclass(tp) and issubclass(tp, TypeAlias)


def enc_hook(obj: Any) -> Any:
    if (
        type(obj) in _convertible_classes
        or
    _is_type_alias(type(obj))
    ):
        return obj.to_builtins()

    else:
        raise NotImplementedError(f"Objects of type {type} are not supported")


def dec_hook(type_: Type, obj):
    origin = get_origin(type_) or type_

    if (
        origin in _convertible_classes
        or
        _is_type_alias(origin)
    ):
        try:
            return type_.try_from(obj)
        except ValidationError as e:
            pretty = getattr(type_, 'pretty_def_str', None)
            if callable(pretty):
                e.add_note(f'While decoding:\n{pretty(indent=1)}')
            else:
                e.add_note(f'While decoding {type_!r}')
            raise

    raise NotImplementedError(f'Objects of type {type_} are not supported')
