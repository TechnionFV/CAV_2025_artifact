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

SAT_STRING = "result: unsafe"
UN_SAT_STRING = "result: safe"
configurations = [
    "-e ic3",
    "-e ic3 --ic3-inn",
    "-e ic3 --ic3-dynamic --rseed 55",
    "-e ic3 --ic3-ctp --rseed 5555",
    "-e ic3 --ic3-ctg",
    "-e ic3 --ic3-ctg --ic3-ctg-limit 5",
    "-e ic3 --ic3-ctg --ic3-ctg-max 5 --ic3-ctg-limit 15",
    "-e ic3 --ic3-ctg --ic3-abs-cst --rseed 55",
    "-e ic3 --ic3-ctg --ic3-ctp",
    "-e ic3 --ic3-ctg --ic3-inn",
    "-e ic3 --ic3-ctg --ic3-ctg-limit 5 --ic3-inn",
    "-e bmc --step 10",
    "-e bmc --bmc-kissat --step 70",
    "-e bmc --bmc-kissat --step 135",
    "-e bmc --bmc-kissat --bmc-time-limit 100 --step 100",
    "-e kind --step 1",
]


class RIC3Params:
    def __init__(
            self, engine="ic3", inn=False, ctg=False, ctp=False, dyn=False, abc_simp=False,
            cnf_simp=True, repo="gipsyh"
    ):
        self.engine = engine
        self.inn = inn
        self.ctg = ctg
        self.ctp = ctp
        self.dyn = dyn
        self.abc_simp = abc_simp
        self.cnf_simp = cnf_simp
        self.repo = repo

    def __str__(self):
        cmd = self.command()
        cmd = cmd[3:]
        cmd = cmd.replace("--no-abc", "NAS")
        cmd = cmd.replace("--no-cnf", "NCS")
        cmd = cmd.replace("--ic3-dynamic", "DYN")
        cmd = cmd.replace("--ic3-", "")
        cmd = cmd.replace(" -v 2", "")
        cmd = cmd.upper()
        return f"rIC3 ({self.repo}) {cmd}"

    def command(self):
        r_str = f"-e {self.engine} -v 2"
        if self.inn:
            r_str += " --ic3-inn"
        if self.ctg:
            r_str += " --ic3-ctg"
        if self.ctp:
            r_str += f" --ic3-ctp"
        if self.dyn:
            r_str += f" --ic3-dynamic"
        if not self.cnf_simp:
            r_str += " --no-cnf"
        if not self.abc_simp:
            r_str += " --no-abc"
        return r_str


"""
***************************************************************************************************
column extractors
***************************************************************************************************
"""


#
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


def get_compilation_cmd():
    return lambda: f"cargo +nightly build --release && mv ./target/release/rIC3 ./rIC3 && cargo +nightly clean"


def get_name(params: RIC3Params) -> str:
    return str(params)


def get_cmd(params: RIC3Params):
    return lambda f: f"./rIC3 {f} {params.command()}"


def analyze_output(output: str) -> dict[str, str]:
    result = o_get_result(output=output)
    inv_size = float("inf")
    if result == "UN_SAT":
        lines = output.splitlines()
        i = lines.index("SolverStatistic {")
        frames = lines[i - 1].split()
        inv_size = frames[frames.index("0") + 1]
    elif result == "SAT":
        inv_size = float("-inf")

    return {
        "TimeError": o_get_time_from_output(output=output),
        "Result": result,
        "Depth": get_number_after_last_appearance_of_string(
            output=output, string="frame:", default=0, ignore_comma=True
        ),
        "InvariantSize": inv_size,
        "Memory (kB)": get_memory_usage_using_bin_time(output=output),
    }


"""
***************************************************************************************************
deployment
***************************************************************************************************
"""


def ric3_deployment(params: RIC3Params) -> Deployment:
    r = Deployment()
    r.name = get_name(params=params).lower().replace(" ", "_")
    r.fetch_command = lambda: f"git clone --recurse-submodules --depth 1 https://github.com/{params.repo}/rIC3.git"
    r.cd_after_fetch = lambda: "rIC3"
    r.compilation_command = get_compilation_cmd()
    r.run_command = get_cmd(params=params)
    r.version = lambda: get_current_commit_hash()
    r.individual_analysis = analyze_output
    return r


def ric3_aggregate_analysis(i: int, params: RIC3Params):
    n = get_name(params=params)
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
        # {"name": f"{n} contradictions to HWMCC20",
        #  "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
        #                                column_name="Contradiction to HWMCC20",
        #                                target="True")},
        # {"name": f"{n} not Solved in HWMCC20",
        #  "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
        #                                column_name="Not Solved in HWMCC20",
        #                                target="True")},
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


def make_deployment_profile(params: list[RIC3Params]) -> DeploymentProfiles:
    return DeploymentProfiles(
        name=f"rIC3 {len(params)} runs ({' vs. '.join(str(p) for p in params)})",
        deployments=[ric3_deployment(params=p) for p in params],
        aggregate_analysis=list(chain.from_iterable(
            [ric3_aggregate_analysis(i=i, params=p) for (i, p) in enumerate(params)]
        )),
    )


"""
***************************************************************************************************
profiles
***************************************************************************************************
"""

ric3_runs = [
    make_deployment_profile(params=[RIC3Params()]),
    make_deployment_profile(params=[RIC3Params(inn=True)]),
    make_deployment_profile(params=[RIC3Params(ctp=True)]),
    make_deployment_profile(params=[RIC3Params(ctg=True)]),
    make_deployment_profile(params=[RIC3Params(dyn=True)]),
    make_deployment_profile(params=[RIC3Params(repo='sirandreww')]),
    make_deployment_profile(params=[RIC3Params(repo='sirandreww', inn=True)]),
    make_deployment_profile(params=[RIC3Params(repo='sirandreww', ctp=True)]),
    make_deployment_profile(params=[RIC3Params(repo='sirandreww', ctg=True)]),
    make_deployment_profile(params=[RIC3Params(repo='sirandreww', dyn=True)])
]
