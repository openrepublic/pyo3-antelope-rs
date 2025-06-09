import pytest

from antelope_rs.abi.structs import to_camel
from antelope_rs.testing import StdABI


# instantiate the sample struct
sample = StdABI.Action.try_from({
    'account': 'eosio.token',
    'name': 'transfer',
    'authorization': [],
    'data': '00010203040506070809',
})

sample_builtins = sample.to_builtins()


@pytest.mark.parametrize(
    'sample_input',
    (
        {
            field: (
                str(getattr(sample, field))
                if field in ('account', 'name')
                else getattr(sample, field)
            )
            for field in type(sample).__struct_fields__
        },
        {
            field: (
                int(getattr(sample, field))
                if field in ('account', 'name')
                else getattr(sample, field)
            )
            for field in type(sample).__struct_fields__
        },
        {
            field: (
                getattr(sample, field).encode()
                if field in ('account', 'name')
                else getattr(sample, field)
            )
            for field in type(sample).__struct_fields__
        }
    ),
    ids=(
        'str_names',
        'int_names',
        'raw_names',
    )
)
def test_action_struct(sample_input):
    # convert an ensure result is eq to sample
    convert_result_0 = StdABI.Action.try_from(sample_input)
    assert convert_result_0 == sample

    # convert to builtins and ensure result is eq
    builtins_result_0 = convert_result_0.to_builtins()
    assert builtins_result_0 == sample_builtins


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
