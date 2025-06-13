# pyo3-antelope-rs

Minimal Python bindings for the `antelope-rs` library built with PyO3.

It exposes most Antelope (EOSIO/Telos) primitive types, ABI packing/unpacking helpers and a few utilities.

## Status

Works for Telos tooling. API may still change.

## Requirements

- Python 3.9 or newer
- Rust toolchain (only to build from source)

## Installing

```bash
# from PyPI
pip install pyo3-antelope-rs

# or build locally
git clone https://github.com/openrepublic/pyo3-antelope-rs
cd pyo3-antelope-rs
maturin build --release
pip install target/wheels/pyo3_antelope_rs-*.whl
```

## Quick example

```python
from antelope_rs.abi import StdABI


packed = StdABI.Transaction.pack({
    'expiration': 0,
    'ref_block_num': 0,
    'ref_block_prefix': 0,
    'max_net_usage_words': 0,
    'max_cpu_usage_ms': 0,
    'delay_sec': 0,
    'context_free_actions': [],
    'actions': [],
    'extension': []
})

print('bytes:', len(packed))
```

## Running tests

```bash
uv sync --dev
uv run pytest -n auto
```

The tests fuzz pack/unpack equivalence between the Rust and Python implementations.

## License

AGPL-3.0-or-later.

