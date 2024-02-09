# halo2 Playground

## How to run

### Run Test

#### Run All

```bash
cargo test -- --nocapture
```

### Generate Region Maps

```bash
cargo test --features dev-graph -- --nocapture
```

## fib_1 - Fibonnacci Sequence On 1 Row

### Gate

| col_a | col_b | col_c | selector |
| ----- | ----- | ----- | -------- |
| a     | b     | c     | s        |

$s * (a + b - c)$

### Region

Each row is a region.

## fib_2 - Fibonnacci Sequence On 2 Rows

### Gate

| col_a | col_b | selector |
| ----- | ----- | -------- |
| a     | b     | s        |
|       | c     |          |

$s * (a + b - c)$

### Region

Whole table is 1 region.

## fib_3 - Fibonnacci Sequence On 3 Rows

### Gate

| advice | selector |
| ------ | -------- |
| a      | s        |
| b      |          |
| c      |          |

$s * (a + b - c)$

### Region

Whole table is 1 region.

## range_check_1 - Range Check with Expression

### Gate

| value | selector |
| ----- | -------- |
| v     | s        |

### Range Check Method

Use expression:

> Given a range R and a value v, returns the expression:
>
> $$(v)  (1 - v)  (2 - v)  ...  (R - 1 - v)$$
>
> where expression = 0 and satisfy constraint if v is in range [0, R).

## range_check_2 - Range Check with Lookup Table

### Gate

| value | q_range_check | q_lookup | table_value |
| ----- | ------------- | -------- | ----------- |
| v     | 1             | 0        | 0           |
| v'    | 0             | 1        | 1           |

### Range Check Method

Use Lookup table to check if value is in a table with values of range [0, LOOKUP_TABLE_RANGE]. If it is found, then it is in range.
