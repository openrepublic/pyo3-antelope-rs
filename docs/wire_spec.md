# Antelope Serialization Protocol Specification

## Scope
> This document defines the canonical wire-format for *every* value that can
> appear in an Antelope ABI, including:
>
> * all built-in scalar types
> * the three type-modifiers (`[]`, `?`, `$`)
> * struct composition (with inheritance)
> * variant / union encoding

---

## Table of Contents
1. [General conventions](#1-general-conventions)
2. [Built-in scalar types](#2-built-in-scalar-types)
   2.1 [Booleans](#21-boolean-bool)
   2.2 [Fixed-width integers](#22-fixed-width-integers)
   2.3 [Variable-length integers](#23-variable-length-integers)
   2.4 [Floating-point numbers](#24-floating-point-numbers)
   2.5 [Time types](#25-time-types)
   2.6 [String-likes](#26-string-likes)
   2.7 [Name](#27-name)
   2.8 [Checksums](#28-checksums)
   2.9 [Cryptographic material](#29-cryptographic-material)
   2.10 [Financial primitives](#210-financial-primitives)
       2.10.1 [Symbol code](#2101-symbol-code)
       2.10.2 [Symbol](#2102-symbol)
       2.10.3 [Asset](#2103-asset)
       2.10.4 [Extended asset](#2104-extended-asset)
3. [Type modifiers](#3-type-modifiers)
4. [Struct encoding](#4-struct-encoding)
5. [Variant / union encoding](#5-variant--union-encoding)
6. [Alias resolution & tag propagation](#6-alias-resolution--tag-propagation)
7. [Change-log & compatibility notes](#7-change-log--compatibility-notes)

---

## 1  General conventions
* Endianness - All fixed-width integer and floating-point scalars are
  little-endian.
* Length prefixes - Variable-length fields (`array`, `string`, `bytes`) are
  preceded by a `varuint32` length (see 2.3).
* Composition order - When a type is wrapped in modifiers, the outermost
  modifier appears first on the wire (pre-order traversal).
* Struct inheritance - If struct B derives from A, encode all fields of A
  first, then the fields of B in declaration order.

---

## 2  Built-in scalar types

### 2.1  Boolean `bool`
| Logical | Byte |
|---------|------|
| false   | 0x00 |
| true    | 0x01 |

---

### 2.2  Fixed-width integers
All integers are two's-complement (signed) or unsigned, little-endian.

| ABI          | Bits | Rust type | Size |
|--------------|------|-----------|------|
| int8 / uint8 | 8    | i8 / u8   | 1 B  |
| int16 / uint16 | 16 | i16 / u16 | 2 B  |
| int32 / uint32 | 32 | i32 / u32 | 4 B  |
| int64 / uint64 | 64 | i64 / u64 | 8 B  |
| int128 / uint128 | 128 | i128 / u128 | 16 B |

---

### 2.3  Variable-length integers
Both kinds use unsigned LEB128 on the wire:

* `varuint32` - unsigned, range 0 .. 2^32-1
* `varint32`  - signed, range -2^31 .. 2^31-1, encoded as:

```
zigzag(n) = (n << 1) ^ (n >> 31)
uLEB128(zigzag(n))
```

---

### 2.4  Floating-point numbers
| ABI      | Bits | Encoding                       |
|----------|------|--------------------------------|
| float32  | 32   | IEEE-754 binary32, little-end  |
| float64  | 64   | IEEE-754 binary64, little-end  |
| float128 | 128  | Raw 16 bytes, little-end       |

---

### 2.5  Time types
| ABI                  | Underlying | Meaning                                   |
|----------------------|------------|-------------------------------------------|
| time_point           | u64        | microseconds since Unix epoch             |
| time_point_sec       | u32        | seconds since Unix epoch                  |
| block_timestamp_type | u32        | half second slots, slot 0 = 2000-01-01    |

---

### 2.6  String-likes

* `string` - UTF-8, prefixed by `varuint32` length.
* `bytes`  - arbitrary bytes, prefixed by `varuint32` length.

### 2.7  Name

* Wire: uint64, little-endian.
* Allowed chars: '.', '1'..'5', 'a'..'z'.
* Max length: 13 chars.

Encoding:

1. For chars 0-11 store 5 bits each, high to low.
2. Char 12 (if present) uses 4 bits.
3. Write result as little-endian uint64.

---

### 2.8  Checksums
| ABI          | Size | Encoding  |
|--------------|------|-----------|
| checksum160  | 20 B | raw bytes |
| checksum256  | 32 B | raw bytes |
| checksum512  | 64 B | raw bytes |

---

### 2.9  Cryptographic material
* `public_key` - 1 byte `KeyType` + 33 byte key data.
* `signature`  - 65 raw bytes (curve dependent).

---

### 2.10  Financial primitives

Antelope follows the legacy EOSIO token design.

#### 2.10.1  Symbol code

* Wire: uint64, little-endian, always 8 bytes.
* Content: up to 7 ASCII uppercase letters (A-Z).
  Each letter occupies one byte starting at bit 8.
* Unused high-order bytes are zero.


String form: `"ABC"` where length is 1..7.

#### 2.10.2  Symbol

* Wire: uint64, little-endian.
  Low byte 0..18 = precision (number of decimals).
  Higher bytes = `symbol_code`.
* String form: `"<precision>,<SYMCODE>"`
  Example: `"4,EOS"`.

#### 2.10.3  Asset

* Wire: `i64 amount` (little-end) then `symbol` (8 bytes).
* `amount` represents raw units: real = amount / 10^precision.
* String form: `"<decimal> <SYMCODE>"`.
  Fraction digits equal precision, e.g. `"10.0000 EOS"`.

Limits: `amount` must be in range -(2^62-1) .. 2^62-1.
Precision must be 0..18.

#### 2.10.4  Extended asset

* Wire: `asset` (16 bytes) followed by `name contract` (8 bytes).
* String form: `"<asset>@<contract>"`,
  e.g. `"10.0000 EOS@eosio.token"`.

---

## 3  Type modifiers

### 3.1  Array `T[]`

```
<varuint32 N> <elem-0> ... <elem-N-1>
```

A null array is invalid.

### 3.2  Optional `T?`

```
<byte flag>
if flag == 0x00 -> value absent
if flag == 0x01 -> <T payload>
```

### 3.3  Extension `T$`

`T$` lets newer schemas append optional tail fields.

Encode: if value is None write nothing, else write `<T>`.

Decode: when input ends, remaining `$` fields are None. Once one `$`
field is absent, all following `$` fields are also absent.

---

## 4  Struct encoding
1. Encode base struct first (if any).
2. Encode fields in declaration order.
3. `$` fields that are None are skipped.
4. Final byte stream is simple concatenation.

---

## 5  Variant / union encoding

```
<varuint32 arm_index> <payload>
```

`arm_index` is zero based.
Payload is encoded as if the chosen arm type were encoded alone.
Nested variants write a discriminator for each level.

---

## 6  Alias resolution & tag propagation
Aliases are resolved before applying modifiers.

---

## 7  Change-log & compatibility notes

* 2025-06-11 - initial draft

---

End of document
