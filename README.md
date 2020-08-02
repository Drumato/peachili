# Peachili

This programming language is inspired Golang strongly.

## usage

```
$ peachili compile <peachili-file> # generate an assembly-file for X86_64
$ peachili compile <peachili-file> # generate an assembly-file for aarch64
```

## [Documents](https://github.com/Drumato/peachili/blob/master/docs/main.md)

## Run all tests

### Unit tests

```
cargo test
```

### Integration tests on x86_64

```
./x64_test.sh
```

### Integration tests on Arm64

```
./aarch64_test.sh
```
