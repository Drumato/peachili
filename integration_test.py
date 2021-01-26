import os
import sys
import subprocess
import time

X86_64 = "x86_64"
AARCH64 = "aarch64"

EXPECTS = {
    "intlit.go": 42,
    "simple_arithmetic1.go": 1,
    "simple_arithmetic2.go": 3,
    "prefix_minus.go": 9,
    "prefix_plus.go": 10,
    "simple_autovar1.go": 30,
    "simple_autovar2.go": 15,
}


def p(message: str):
    print(message, file=sys.stderr)


class Color:
    CLEAR = "\033[0m"
    RED = "\033[1m\033[31m"
    GREEN = "\033[1m\033[32m"
    BLUE = "\033[1m\033[34m"


def test_compiler(arch_name: str):
    test_files = os.listdir(f"examples/{arch_name}")

    for test_file in test_files:
        # Peachili Compiler --asm.s-> clang --a.out-> OSの順にプロセスを展開
        expect_status = EXPECTS[test_file]

        actual = generate_an_assembly(arch_name, test_file)
        if actual != 0:
            p(f"{test_file}: peachili compiler failed to compile")
            exit(1)

        actual = link_assembly(arch_name)
        if actual != 0:
            p("asm.s: failed to link")
            exit(1)

        actual = execute_binary(arch_name)
        if actual != expect_status:
            p(f"{test_file}: expected {expect_status} but got {actual}")
            exit(1)


def generate_an_assembly(arch_name: str, test_file: str) -> int:
    return subprocess.run(
        [
            "./target/debug/peachili",
            "--target",
            arch_name,
            "compile",
            f"examples/{arch_name}/{test_file}",
        ]
    ).returncode


def link_assembly(arch_name) -> int:
    if arch_name == X86_64:
        return subprocess.run(["clang", "-static", "asm.s"]).returncode
    else:
        return subprocess.run(["aarch64-linux-gnu-gcc", "-static", "asm.s"]).returncode


def execute_binary(arch_name: str) -> int:
    if arch_name == X86_64:
        return subprocess.run(["./a.out"]).returncode
    else:
        return subprocess.run(["qemu-aarch64", "./a.out"]).returncode


def test_aarch64_compiler():
    test_compiler(AARCH64)


def test_x64_compiler():
    test_compiler(X86_64)


def profile_procedure(fn_name):
    start = time.time()
    globals()[fn_name]()
    total_time = time.time() - start

    p(f"{fn_name} time -> {Color.BLUE}{round(total_time, 2)}{Color.CLEAR}s")
    p(f"{Color.GREEN}{fn_name}: All Test Passed.{Color.CLEAR}\n")


if __name__ == "__main__":
    os.environ["PEACHILI_LIB_PATH"] = f"{os.getcwd()}/lib"

    profile_procedure("test_x64_compiler")
    profile_procedure("test_aarch64_compiler")
