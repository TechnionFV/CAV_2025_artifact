"""
Shared functions between deployments
"""
import math
import subprocess
from collections.abc import Callable

"""
***************************************************************************************************
types
***************************************************************************************************
"""


class Deployment:
    name: str
    fetch_command: Callable[[], str]
    cd_after_fetch: Callable[[], str]
    compilation_command: Callable[[], str]
    run_command: Callable[[str], str]
    version: Callable[[], str]
    individual_analysis: Callable[[str], dict[str, str]]


class DeploymentProfiles:
    name: str
    deployment: list[Deployment]
    cross_examinations: list[dict[str, any]]
    aggregate_analysis: list[dict[str, any]]

    def __init__(
            self, name: str,
            deployments: list[Deployment],
            aggregate_analysis: list
    ):
        self.name = name
        self.deployments = deployments
        self.aggregate_analysis = aggregate_analysis


"""
***************************************************************************************************
helper functions to get deployments
***************************************************************************************************
"""

#
# def get_hwmcc20_solution_dict() -> dict[str, str]:
#     result = {}
#     cwd = str(pathlib.Path(__file__).parent.resolve())
#     p = f'{cwd}/../../../aig_inputs/hwmcc20/hwmcc20-bv-all.csv'
#     with open(p, newline='') as csvfile:
#         spamreader = csv.reader(csvfile, delimiter=';')
#
#         for row in spamreader:
#             if row[0] == 'benchmark':
#                 continue
#             prob_name = row[0]
#             prob_results = [x for x in row if x in ["sat", "uns", "unk", "time"]]
#
#             r = None
#             match ("sat" in prob_results, "uns" in prob_results):
#                 case (True, False):
#                     r = "SAT"
#                 case (False, True):
#                     r = "UN-SAT"
#                 case (False, False):
#                     r = "UNKNOWN"
#                 case _:
#                     s = prob_results.count("sat")
#                     u = prob_results.count("uns")
#                     if s > u:
#                         r = "SAT"
#                     elif u > s:
#                         r = "UN-SAT"
#                     else:
#                         assert False
#
#             result[prob_name] = r
#     return result
#
#
# solution_dict = get_hwmcc20_solution_dict()

"""
***************************************************************************************************
helper functions for individual analysis
***************************************************************************************************
"""


def get_problem_name_from_output(output: str) -> str:
    aig_file_line = [x for x in output.splitlines() if ("--- AIG file name = " in x)][0]
    problem_name = aig_file_line.split('/')[-1].replace(".aig", "").replace(" ", "")
    return problem_name


def get_number_after_last_appearance_of_string(
        output: str, string: str,
        default: float = float("inf"),
        offset=0,
        ignore_comma=False
) -> float:
    index = output.rfind(string)
    if index == -1:
        return default
    output = output[index + len(string):]
    split = output.split()
    s = split[offset]
    if ignore_comma:
        s = s.strip(",")
    num = float(s)
    return num


def get_number_before_last_appearance_of_string(
        output: str, string: str,
        default: float = float("inf"),
        offset=-1,
) -> float:
    index = output.rfind(string)
    if index == -1:
        return default
    output = output[:index]
    split = output.split()
    num = float(split[offset])
    return num


def count(csv_dict, i: int, column_name: str, target: str) -> int:
    r = [csv_dict[x][i][column_name] for x in csv_dict]
    r = r.count(target)
    return r


def is_float(x: str):
    try:
        float(x)
        return True
    except ValueError:
        return False


def average_or_median(csv_dict, i: int, column_name: str, calculation: str,
                      condition_columns: list[str],
                      ignore_inf=True, ignore_zero=False,
                      ignore_negative=False, default=float('inf')) -> float:
    assert calculation in ["average", "median"]
    r = [float(csv_dict[x][i][column_name]) for x in csv_dict if
         all(0 < float(csv_dict[x][i][c]) < float('inf') for c in condition_columns)]
    if ignore_inf:
        r = [x for x in r if math.isfinite(x)]
    if ignore_zero:
        r = [x for x in r if x != 0]
    if ignore_negative:
        r = [x for x in r if x >= 0]
    if len(r) == 0:
        return default
    elif calculation == "average":
        avg = sum(r) / len(r)
        return avg
    else:
        r.sort()
        return r[len(r) // 2]


def get_current_commit_hash() -> str:
    output = subprocess.run(
        ["git", "log", "--pretty=format:'%h'", "-n 1"],
        stdout=subprocess.PIPE
    )
    return output.stdout.decode('utf-8')


"""
***************************************************************************************************
column extractors
***************************************************************************************************
"""


def get_time_from_output(output: str, sat_str: str, un_sat_str: str) -> str:
    e = get_error(output=output)
    if e != "":
        return e
    if "Command exited with non-zero status 124" in output:
        return "TIMEOUT BY /usr/bin/timeout"
    if "DUE TO TIME LIMIT ***" in output:
        return "TIMEOUT BY SLURM"
    if not (un_sat_str in output or sat_str in output):
        return "ERROR"
    r = get_number_after_last_appearance_of_string(
        output=output,
        string="User time (seconds):",
    )
    assert isinstance(r, float)
    assert not math.isinf(r)
    return str(r)


def get_error(output: str) -> str:
    if "memory allocation of" in output and "bytes failed" in output:
        return "MEMORY ERROR"
    if "terminate called after throwing an instance of 'std::bad_alloc'" in output:
        return "MEMORY ERROR"
    if "fatal runtime error: Rust cannot catch foreign exceptions" in output:
        return "C++ EXCEPTION IN RUST PROGRAM"
    if "Some of your processes may have been killed by the cgroup out-of-memory handler" in output:
        return "MEMORY ERROR"
    lower_out = output.lower()
    if ("panic" in lower_out) or ("failed" in lower_out):
        return "ERROR"
    elif "err" in lower_out:
        slurm = "err" in lower_out.replace("error: *** JOB", "", 1)
        if not slurm:
            return "ERROR"
    return ""


def get_result(output, sat_str: str, un_sat_str: str) -> str:
    is_sat = sat_str in output
    is_un_sat = un_sat_str in output
    assert not (is_sat and is_un_sat), f"Output deemed SAT and UNSAT at the same time:\n{output}"
    if is_sat:
        return "SAT"
    if is_un_sat:
        return "UN_SAT"
    return ""


# def is_contradiction_to_hwmcc20(output: str, sat_str: str, un_sat_str: str,
#                                 problem_name: str) -> str:
#     is_un_sat = "True" if un_sat_str in output else ""
#     is_sat = "True" if sat_str in output else ""
#     if is_un_sat and is_sat:
#         return "True"
#     match solution_dict[problem_name]:
#         case "SAT":
#             if is_un_sat:
#                 return "True"
#         case "UN-SAT":
#             if is_sat:
#                 return "True"
#         case "UNKNOWN":
#             pass
#         case _:
#             assert False
#     return ""
#
#
# def is_unique_win_in_hwmcc20(output: str, sat_str: str, un_sat_str: str, problem_name: str) -> str:
#     is_un_sat = "True" if un_sat_str in output else ""
#     is_sat = "True" if sat_str in output else ""
#     if is_un_sat and is_sat:
#         return ""
#     match solution_dict[problem_name]:
#         case "SAT":
#             if is_un_sat:
#                 return ""
#         case "UN-SAT":
#             if is_sat:
#                 return ""
#         case "UNKNOWN":
#             if is_sat or is_un_sat:
#                 return "True"
#         case _:
#             assert False
#     return ""


def get_memory_usage_using_bin_time(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string="Maximum resident set size (kbytes):"
    )
