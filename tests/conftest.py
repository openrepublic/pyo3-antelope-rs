from __future__ import annotations
import os
import pdbp

from antelope_rs.testing import generate_strategies


abi_whitelist: set[str] = set(os.getenv(
    'ABI_WHITELIST', '*'
).split(','))

if '*' in abi_whitelist:
    abi_whitelist = set()

type_whitelist: set[str] = set(os.getenv(
    'TYPE_WHITELIST', '*'
).split(','))

if '*' in type_whitelist:
    type_whitelist = set()


def pytest_generate_tests(metafunc):
    '''
    Allow tests to declare a `abi_pair` parameter and get automatic
    parametrisation from the session fixture.

    '''
    if 'abi_case' in metafunc.fixturenames:
        # session fixtures are not directly available at collection time,
        # so we rebuild the small list here (cheap).
        metafunc.parametrize(
            'abi_case',
            generate_strategies(abi_whitelist, type_whitelist),
            ids=lambda p: f'{p[0]}::{p[1]}',
        )

