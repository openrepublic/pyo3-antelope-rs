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
import inspect
import re


_camel_re = re.compile(r'(?:^|_)([a-zA-Z0-9]+)')


def to_camel(s: str) -> str:
    """
    Convert snake_case string to CamelCase

    """
    s = str(s)

    if '_' not in s and not s.islower():  # already CamelCase
        return s

    return ''.join(m.group(1).capitalize() for m in _camel_re.finditer(s))


_snake_re_1: re.Pattern[str] = re.compile(r'(.)([A-Z][a-z]+)')
_snake_re_2: re.Pattern[str] = re.compile(r'([a-z0-9])([A-Z])')


def to_snake(s: str) -> str:
    """
    Convert CamelCase string to snake_case

    """
    s = str(s)

    if s.islower():  # already snake_case
        return s

    s = _snake_re_1.sub(r'\1_\2', s)
    s = _snake_re_2.sub(r'\1_\2', s)

    return s.lower()


def validate_protocol(cls: type, proto):
    """
    Check that `cls` implements protocol `proto`, and if it doesn't provide a
    reason.

    """
    errors: list[str] = []
    for name, member in proto.__dict__.items():
        if name.startswith('_'):
            continue

        # protocol method
        if inspect.isfunction(member):
            impl = getattr(cls, name, None)
            if impl is None:
                errors.append(f'missing method {name}')
            elif not callable(impl):
                errors.append(f'{name} exists but is not callable')
            else:
                sig_p = inspect.signature(member)
                sig_c = inspect.signature(impl)
                # drop 'self' or 'cls'
                params_p = list(sig_p.parameters.values())[1:]
                params_c = list(sig_c.parameters.values())[1:]
                if len(params_c) < len(params_p):
                    errors.append(
                        f'{name} has too few params ({len(params_c)}<{len(params_p)})'
                    )
        # protocol data attribute
        else:
            if not hasattr(cls, name):
                errors.append(f'missing attribute {name}')

    error_str = '\n'.join(errors)

    assert not errors, (
        f'{cls.__name__} doesnt support ABINamespaceType proto: {error_str}'
    )
