# Peachili

This programming language is inspired Golang strongly.

## how to build

### with Ninja

```
mkdir build && cd build
export CC=<whatever you what to use> cmake -GNinja ..
ninja

# or you can use
./scripts/ninja_build.sh
```

### with GNU make

```
mkdir build && cd build
export CC=<whatever you what to use> cmake ..
make

# or you can use
./scripts/make_build.sh
```
