from __future__ import annotations

import json
import os

import hypothesis.strategies as st
from hypothesis import HealthCheck, given, settings
import msgspec

from antelope_rs.codec import dec_hook
from antelope_rs.testing import AntelopeDebugEncoder


EXAMPLES_PER_TYPE: int = int(os.getenv('EXAMPLES_PER_TYPE', '256'))


@settings(
    max_examples=EXAMPLES_PER_TYPE,
    deadline=None,
    suppress_health_check=[HealthCheck.too_slow],
)
@given(data=st.data())
def test_roundtrip_equivalence(
    abi_case,
    data
):
    (
        abi_key,
        type_name,
        abi,
        struct_cls,
        strat
    ) = abi_case

    # random value for this ABI type
    val = data.draw(strat)

    py_encoded = rs_encoded = b''
    obj = None
    obj_builtins = None

    try:
        # python-side conversions
        obj = msgspec.convert(val, type=struct_cls, dec_hook=dec_hook)
        assert struct_cls.try_from(val) == obj

        # built-ins
        obj_builtins = obj.to_builtins()
        assert struct_cls.try_from(val) == struct_cls.try_from(obj_builtins)

        # binary round-trip
        py_encoded = obj.encode()
        rs_encoded = abi.pack(type_name, obj_builtins)
        assert py_encoded == rs_encoded

    except (AssertionError, ValueError) as e:
        abi_fields = getattr(struct_cls, '__abi_fields__', None)
        if (
            not abi_fields
            and hasattr(struct_cls, '__value__')
            and hasattr(struct_cls.__value__, '__abi_fields__')
        ):
            abi_fields = getattr(struct_cls.__value__, '__abi_fields__')

        e.add_note(
            json.dumps(
                {
                    'abi': abi_key,
                    'type': type_name,
                    'val': val,
                    'obj': obj,
                    'obj_builtins': obj_builtins,
                    'abi_fields': abi_fields,
                    'python': (len(py_encoded), py_encoded),
                    'rust': (len(rs_encoded), rs_encoded),
                },
                cls=AntelopeDebugEncoder,
                indent=4,
            )
        )
        raise
