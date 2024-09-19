import contextlib
import glob
import os
import subprocess
import sys


def rm_f(path):
    try:
        os.remove(path)
    except FileNotFoundError:
        pass


def glob_norm(path, recursive):
    return [os.path.normpath(p) for p in glob.glob(path, recursive=recursive)]


def rm_glob_f(path, exclude=None, recursive=True):
    if exclude is not None:
        for f in list(
            set(glob_norm(path, recursive=recursive))
            - set(glob_norm(exclude, recursive=recursive))
        ):
            rm_f(f)
    else:
        for f in glob.glob(path, recursive=recursive):
            rm_f(f)


@contextlib.contextmanager
def working_dir(path):
    cwd = os.getcwd()
    os.chdir(path)
    try:
        yield
    finally:
        os.chdir(cwd)


@contextlib.contextmanager
def with_env(**kwargs):
    env = os.environ.copy()
    for key, value in kwargs.items():
        os.environ[key] = value
    try:
        yield
    finally:
        os.environ.clear()
        os.environ.update(env)


argv = sys.argv[1:]

format = "lcov"
if len(argv) > 0:
    format = argv[0]

with working_dir(os.path.dirname(os.path.abspath(__file__))):
    with with_env(
        RUSTFLAGS="-C instrument-coverage",
        LLVM_PROFILE_FILE="%m-%p.profraw",
    ):
        command = ["cargo", "build"]
        subprocess.run(command).check_returncode()
        command[1] = "test"
        subprocess.run(command).check_returncode()

        command = [
            "grcov",
            ".",
            "-s",
            ".",
            "--binary-path",
            "./target/debug",
            "--llvm",
            "--branch",
            "--ignore-not-existing",
            "-o",
            "./coverage",
            "-t",
            format,
            "--excl-line",
            r"GRCOV_EXCL_LINE|#\[derive|#\[error|unreachable!|unimplemented!|tracing::(debug|trace|info|warn|error)!\([\s\S]*\);",
            "--keep-only",
            "src/**/*.rs",
            "--excl-start",
            "GRCOV_EXCL_START",
            "--excl-stop",
            "GRCOV_EXCL_STOP",
        ]
        subprocess.run(command).check_returncode()
        rm_glob_f("**/*.profraw")
