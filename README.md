# Peachili

## Architecture

```
String ==(Parser)==> Higher AST ==(Translator)==> Typed AST(Architecture-dependent) ==(Generator)==> assembly
```

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
cargo build
./x64_test.sh
```

### Integration tests on Arm64

```
cargo build
./aarch64_test.sh
```

## Debug

```
# aarch64
qemu-aarch64-static -g <port> a.out
gdb-multiarch a.out
target remote :<port>
```
