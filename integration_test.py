import os
import sys
import subprocess
import time


def p(message: str):
    print(message, file=sys.stderr)


class Color:
    CLEAR = "\033[0m"
    RED = "\033[1m\033[31m"
    GREEN = "\033[1m\033[32m"
    BLUE = "\033[1m\033[34m"


def test_x64_compiler():
    p(f"{Color.GREEN}++++++++++++++++test-x64-compiler++++++++++++++++{Color.CLEAR}")
    test_files = os.listdir("examples/x64")
    expects = {"intlit.go": 42}

    for test_file in test_files:
        # Peachili Compiler --asm.s-> clang --a.out-> OSの順にプロセスを展開
        expect_status = expects[test_file]

        actual = subprocess.run(
            ["./target/debug/peachili", "compile", f"examples/x64/{test_file}"]
        ).returncode

        if actual != 0:
            p(f"{test_file}: peachili compiler failed to compile")
            exit(1)

        actual = subprocess.run(["clang", "-static", "asm.s"]).returncode
        if actual != 0:
            p("asm.s: clang failed to link")
            exit(1)

        actual = subprocess.run(["./a.out"]).returncode
        if actual != expect_status:
            p(f"{test_file}: expected {expect_status} but got {actual}")
            exit(1)


if __name__ == "__main__":
    os.environ["PEACHILI_LIB_PATH"] = f"{os.getcwd()}/lib"

    start = time.time()
    test_x64_compiler()
    x64_compiler_time = time.time() - start
    p(
        f"test-x64-compiler time -> {Color.BLUE}{round(x64_compiler_time, 2)}{Color.CLEAR}s"
    )

    p(f"{Color.GREEN}All Test Passed.{Color.CLEAR}")