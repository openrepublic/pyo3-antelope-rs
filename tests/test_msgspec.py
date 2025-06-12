import pytest

from antelope_rs.utils import to_camel
from antelope_rs.testing import StdABI


@pytest.mark.parametrize(
    'type_name',
    (
        'signed_block',
        'transaction_variant',
        'transaction_trace',
        'transaction_trace_v0',
        'partial_transaction',
        'abi_table',
        'transaction_header'
    ),
    ids=lambda x: x
)
def test_show_autogen_defs(type_name: str):
    cls = getattr(StdABI, type_name)
    assert (
        cls
        ==
        getattr(StdABI, to_camel(type_name))
    )

    print(f'\n\"{type_name}\" is: ')
    print(cls.pretty_def_str(indent=1))
