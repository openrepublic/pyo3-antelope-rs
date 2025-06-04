import pytest
import msgspec

from antelope_rs import Name
from antelope_rs.codec import dec_hook, enc_hook


class Action(msgspec.Struct):
    account: Name
    name: Name



# start with names as str
sample_str: dict[str, str] = {
    'account': 'eosio.token',
    'name': 'transfer'
}

# convert dict values to Name
sample: dict[str, Name] = {
    k: Name.from_str(v)
    for k, v in sample_str.items()
}

# convert dict values to ints
sample_ints: dict[str, int] = {
    k: int(n)
    for k, n in sample.items()
}

# instantiate the sample struct
sample_struct = Action(**sample)


@pytest.mark.parametrize(
    'sample_input',
    (
        sample_str,
        sample_ints,
        {
            k: n.encode()
            for k, n in sample.items()
        },
        {
            **sample_str,
            'account': sample_ints['account']
        },
        {
            **sample_str,
            'name': sample_ints['name'],
        }
    ),
    ids=(
        'str_names',
        'int_names',
        'raw_names',
        'mixed_int_acc_str_name',
        'mixed_str_acc_int_name'
    )
)
def test_action_struct(sample_input):
    # convert an ensure result is eq to sample_struct
    convert_result_0 = msgspec.convert(
        sample_input, type=Action, dec_hook=dec_hook
    )
    assert convert_result_0 == sample_struct

    # convert to builtins and ensure result is eq to sample_ints
    builtins_result_0 = msgspec.to_builtins(
        convert_result_0, enc_hook=enc_hook
    )
    assert builtins_result_0 == sample_ints
