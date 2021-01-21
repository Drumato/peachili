# Peachili

## Architecture

```
String ==(Parser)==> Higher AST ==(Type Resolver && Translator)==> Typed AST(Architecture-dependent) ==(Generator)==> assembly
```

## usage

```
$ peachili compile <peachili-file> # generate an assembly-file for X86_64
```

## [Documents](https://github.com/Drumato/peachili/blob/master/docs/main.md)

## Run all tests

### Unit tests

```
cargo test
```

### Integration tests

```
cargo build
python integration_test.py
```

## Debug

```
# aarch64
qemu-aarch64-static -g <port> a.out
gdb-multiarch a.out
target remote :<port>
```
