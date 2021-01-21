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

```bash
# aarch64
$ sudo apt install gcc-aarch64-linux-gnu qemu qemu-user gdb-multiarch
$ aarch64-linux-gnu-gcc -static asm.s
$ qemu-aarch64 ./a.out
```
