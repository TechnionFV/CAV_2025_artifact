"""
File that describes what deployments will be used for the benchmark.
Design decisions, to get commands we call a lambda function to give more expressive power
"""
from itertools import chain

from .shared import *

"""
***************************************************************************************************
helper functions to get deployments
***************************************************************************************************
"""


class ABCParams:
    def __init__(
            self, r=False, n=False, c=False, q=False
    ):
        self.r = r
        self.n = n
        self.c = c
        self.q = q

    def __str__(self):
        cmd = self.command()
        cmd = cmd.replace(" -", "")
        cmd = cmd[3:]
        cmd = cmd.upper()
        return f"ABC {cmd}"

    def command(self):
        r_str = "pdr -v"
        if self.r:
            r_str += " -r"
        if self.n:
            r_str += " -n"
        if self.c:
            r_str += f" -c"
        if self.q:
            r_str += f" -q"
        return r_str


SAT_STRING = "was asserted in frame"
UN_SAT_STRING = "Property proved"

"""
***************************************************************************************************
column extractors
***************************************************************************************************
"""


# def o_is_contradiction_to_hwmcc20(output: str) -> str:
#     return is_contradiction_to_hwmcc20(output=output, sat_str=SAT_STRING,
#                                        un_sat_str=UN_SAT_STRING,
#                                        problem_name=get_problem_name_from_output(output))
#
#
# def o_is_unique_win_in_hwmcc20(output: str) -> str:
#     return is_unique_win_in_hwmcc20(output=output, sat_str=SAT_STRING, un_sat_str=UN_SAT_STRING,
#                                     problem_name=get_problem_name_from_output(output))


def o_get_time_from_output(output: str) -> str:
    return get_time_from_output(output=output, sat_str=SAT_STRING, un_sat_str=UN_SAT_STRING)


def o_get_result(output) -> str:
    return get_result(output=output, sat_str=SAT_STRING, un_sat_str=UN_SAT_STRING)


def get_clauses_in_cnf(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string=". Clauses ="
    )


def get_variables_in_cnf(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string="CNF stats: Vars ="
    )


def get_literals_in_cnf(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string=". Literals ="
    )


def get_invariant_size(output: str) -> float:
    if SAT_STRING in output:
        return -float('inf')
    return get_number_after_last_appearance_of_string(
        output=output,
        string="Verification of invariant with",
        default=float('inf')
    )


def get_depth(output: str):
    lines = output.splitlines()
    lines.reverse()
    for line in lines:
        line_after_split = line.split(':')
        if len(line_after_split) == 1:
            continue
        potential_depth = line_after_split[0].split()[0]
        if potential_depth[0].isnumeric():
            return int(potential_depth)
    return -1


def get_compilation_cmd() -> str:
    import multiprocessing
    c = multiprocessing.cpu_count()
    return f"make ABC_USE_NO_READLINE=1 -j{c}"


def analyze_output(output) -> dict[str, str]:
    return {
        "TimeError": o_get_time_from_output(output=output),
        "Result": o_get_result(output=output),
        "Clauses": get_clauses_in_cnf(output=output),
        "Variables": get_variables_in_cnf(output=output),
        "Literals": get_literals_in_cnf(output=output),
        "MemoryKB": get_memory_usage_using_bin_time(output=output),
        "Depth": get_depth(output=output),
        "InvariantSize": get_invariant_size(output=output),
    }


"""
***************************************************************************************************
deployment
***************************************************************************************************
"""


def abc_pdr_deployment(params: ABCParams) -> Deployment:
    pdr_c = params.command()
    r = Deployment()
    r.name = str(params).lower().replace(" ", "_")
    r.fetch_command = lambda: "git clone --depth 1 https://github.com/berkeley-abc/abc.git"
    r.cd_after_fetch = lambda: "abc"
    r.compilation_command = get_compilation_cmd
    r.run_command = lambda \
            f: f'./abc -c "&read "{f}" ; &put; write_cnf /dev/null ; {pdr_c}"'
    r.version = lambda: get_current_commit_hash()
    r.individual_analysis = analyze_output
    return r


def abc_pdr_aggregate_analysis(i: int, params: ABCParams):
    n = str(params)
    return [
        {"name": f"{n} SOLVED",
         "cmd": lambda csv_dict: count(
             csv_dict=csv_dict, i=i, column_name="Result", target="SAT"
         ) + count(
             csv_dict=csv_dict, i=i, column_name="Result", target="UN_SAT"
         )},
        {"name": f"{n} Error",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
                                       column_name="TimeError",
                                       target="ERROR")},
        {"name": f"{n} Memory Error",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict,
                                       i=i,
                                       column_name="TimeError",
                                       target="MEMORY ERROR")},
        {"name": f"{n} SAT",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i, column_name="Result",
                                       target="SAT")},
        {"name": f"{n} UN-SAT",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
                                       column_name="Result", target="UN_SAT")},
    ]


"""
***************************************************************************************************
deployments
***************************************************************************************************
"""


def make_deployment_profile(params: list[ABCParams]) -> DeploymentProfiles:
    return DeploymentProfiles(
        name=f"ABC {len(params)} runs ({' vs. '.join(str(p) for p in params)})",
        deployments=[abc_pdr_deployment(params=p) for p in params],
        aggregate_analysis=list(chain.from_iterable(
            [abc_pdr_aggregate_analysis(i=i, params=p) for (i, p) in enumerate(params)]
        )),
    )


"""
***************************************************************************************************
profiles
***************************************************************************************************
"""

abc_runs = [
    make_deployment_profile(params=[ABCParams()]),
    make_deployment_profile(params=[ABCParams(r=True, n=True)]),
    make_deployment_profile(params=[ABCParams(r=True, n=True, c=True)]),
    make_deployment_profile(params=[ABCParams(c=True)]),
    make_deployment_profile(params=[ABCParams(n=True, c=True)]),
    make_deployment_profile(params=[ABCParams(n=True, c=True, q=True)])
]
