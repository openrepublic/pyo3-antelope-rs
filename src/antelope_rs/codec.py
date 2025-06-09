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
from typing import Any, Type

from .meta import (
    builtin_classes,
    TypeAlias
)


def enc_hook(obj: Any) -> Any:
    if (
        type(obj) in _convertible_classes
        or
        issubclass(type(obj), TypeAlias)
    ):
        return obj.to_builtins()

    else:
        raise NotImplementedError(f"Objects of type {type} are not supported")


_convertible_classes: set[Type[Any]] = set(builtin_classes)


def dec_hook(type: Type, obj: Any) -> Any:
    if (
        type in _convertible_classes
        or
        issubclass(type, TypeAlias)
    ):
        return type.try_from(obj)

    else:
        raise NotImplementedError(f"Objects of type {type} are not supported")
