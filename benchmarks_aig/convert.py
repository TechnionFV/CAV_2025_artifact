import datetime
import os
import pathlib
import sys
import subprocess
from multiprocessing import Pool


def my_print(s: str):
    print(s, flush=True)


def run_cmd(cmd: str):
    my_print(f"--- RUNNING {cmd}")
    cmd_result = os.system(cmd)
    assert cmd_result == 0


def run(i_cmd: tuple[int, str]) -> float:
    i, cmd = i_cmd
    my_print(f"i = {i}, RUNNING {cmd}")
    first_time = datetime.datetime.now()
    subprocess.run(cmd, shell=True, check=False)
    passed = datetime.datetime.now() - first_time
    return passed.seconds


def convert_cmd(src: str, dest: str) -> str:
    return f'abc -c " &read {src} ; &put ; fold -v ; write_aiger {dest} ; &get ; &fraig -y ; &put; write_aiger {dest} ; orchestrate ; write_aiger {dest} "'


def convert_suit(src: str, dest: str):
    THREADS = os.cpu_count()
    MAX_SECONDS_FOR_COMMAND = 120
    commands = []

    for root, dirs, files in os.walk(src):
        for file in files:
            if file.endswith(".aig"):
                f = os.path.join(root, file)

                # Destination path
                destination = f.replace(src, dest)

                # Ensure destination directory exists
                os.makedirs(os.path.dirname(destination), exist_ok=True)

                # Command to execute
                cmd = convert_cmd(f, destination)
                cmd = f"/usr/bin/timeout {MAX_SECONDS_FOR_COMMAND} {cmd}"

                commands.append((len(commands), cmd))

    my_print(f"Number of threads: {THREADS}")
    with Pool(THREADS) as p:
        my_print(p.map(run, commands))


def main():
    src = sys.argv[1]
    dest = sys.argv[2]
    convert_suit(src=src, dest=dest)


if __name__ == "__main__":
    main()