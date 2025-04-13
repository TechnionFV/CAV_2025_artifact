"""
***************************************************************************************************
File that has function that are used everywhere in the project
***************************************************************************************************
"""
import csv
import os
import pathlib
from enum import Enum

"""
***************************************************************************************************
const
***************************************************************************************************
"""

WELCOME_MSG: str = """
***************************************************************************************************
Thank you for using this tool.
The tool creates executables and races them on multiple tests to get an image of performance.
The performance testing is done on NP-Complete and PSpace problems and so the name of the game
is the number of solved instances in a reasonable given time. As opposed to normal performance 
comparison where we want the same task to be done faster. 
Author: Andrew Luka
***************************************************************************************************
"""

"""
***************************************************************************************************
Enum
***************************************************************************************************
"""


class RunResult(Enum):
    TIME = 1
    SAT = 2
    UN_SAT = 3
    ERROR = 4


"""
***************************************************************************************************
helper functions
***************************************************************************************************
"""


def __get_possible_suites() -> list[str]:
    file_path = f"{pathlib.Path(__file__).parent.parent}/aig_inputs"
    l = [f.name for f in os.scandir(file_path) if f.is_dir()]
    l.sort()
    return l


def __get_real_path_of_script() -> pathlib.Path:
    file_path = pathlib.Path(__file__)
    return file_path


def __get_all_file_paths_in_dir_recursively(rootdir: str):
    result = []
    for subdir, dirs, files in os.walk(rootdir, followlinks=True):
        for file in files:
            file_path = os.path.join(subdir, file)
            result += [file_path]
    return result


"""
***************************************************************************************************
API functions
***************************************************************************************************
"""


def get_path_to_repo() -> str:
    main_py_path = __get_real_path_of_script()
    src_path = main_py_path.parent.resolve()
    popped_path = src_path.parent.resolve().__str__()
    assert popped_path.split("/")[-1] == "benchmark-hwmc"
    return popped_path


def get_all_file_paths_in_dir_that_have_desired_ending(rootdir: str, desired_ending):
    files_in_root = __get_all_file_paths_in_dir_recursively(
        rootdir=rootdir
    )
    filtered_inputs = [f for f in files_in_root if f.endswith(desired_ending)]
    return filtered_inputs


def change_directory(path: str):
    print(f"--- RUNNING cd {path}", flush=True)
    os.chdir(path)


def run_cmd(cmd: str):
    print(f"--- RUNNING {cmd}", flush=True)
    cmd_result = os.system(cmd)
    assert cmd_result == 0


def read_file(path: str) -> str:
    with open(path) as f:
        lines = f.read()
        return lines


def make_repos_dir_if_not_exists(repo_path: str):
    change_directory(repo_path)
    if not os.path.isdir("repos"):
        run_cmd("mkdir repos")


def read_csv_files(paths: list[str]) -> list[dict[str, dict[str, any]]]:
    r = []
    for path in paths:
        df = {}
        with open(path) as csvfile:
            reader = csv.reader(csvfile)
            rows = [r for r in reader]
            header: list[str] = rows[0]
            key_index = header.index('model')
            for row in rows[1:]:
                key = row[key_index]
                value = {}
                for i, h in enumerate(header):
                    value[h] = row[i]
                df[key] = value
        r += [df]
    return r
