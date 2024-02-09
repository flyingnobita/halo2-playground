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

## fib_2 - Fibonnacci Sequence On 2 Rows

### Gate

| col_a | col_b | selector |
| ----- | ----- | -------- |
| a     | b     | s        |
|       | c     |          |

$s * (a + b - c)$

## fib_3 - Fibonnacci Sequence On 3 Rows

### Gate

| advice | selector |
| ------ | -------- |
| a      | s        |
| b      |          |
| c      |          |

$s * (a + b - c)$
