"""
File that describes what deployments will be used for the benchmark.
Design decisions, to get commands we call a lambda function to give more expressive power
"""
import json
from itertools import chain
from typing import Optional

from .shared import *

"""
***************************************************************************************************
helper functions to get deployments
***************************************************************************************************
"""


class RFVParams:
    def __init__(
            self, default=True,
            er=False,
            er_fp=None,
            er_impl=None,
            er_gen=None,
            largest_inductive_clause=False,
            ctg=False,
            ev_delta: Optional[int] = None,
            decay: Optional[float] = None,
            branch="main"
    ):
        self.default = default
        self.er = er
        self.er_fp = er_fp
        self.er_impl = er_impl
        self.er_gen = er_gen
        self.largest_inductive_clause = largest_inductive_clause
        self.ctg = ctg
        self.ev_delta = ev_delta
        self.decay = decay
        self.branch = branch

    def __str__(self):
        r = f"rfv PDR"
        if self.default:
            r += " Default"
            return r
        if self.er:
            r += " ER"

        for tag, value in [("F", self.er_fp), ("G", self.er_gen), ("I", self.er_impl)]:
            if value is True:
                r += f"+{tag}"
            elif value is False:
                r += f"-{tag}"

        if self.largest_inductive_clause:
            r += " LIC"
        if self.ev_delta is not None:
            r += f" EVD {self.ev_delta}"
        if self.decay is not None:
            r += f" DECAY {self.decay}"
        if self.ctg:
            r += f" CTG"
        return r

    def get_run_command(
            self, aig_file: str
    ) -> str:
        e = "./pdr_engine_for_hwmcc "
        e += f"'{aig_file}' "
        e += '--counterexample "" --certificate "" --verbose on --check-result on '
        if self.default:
            return e

        e += "--er "
        if self.er:
            e += "on "
        else:
            e += "off "

        for tag, value in [("er-fp", self.er_fp), ("er-gen", self.er_gen),
                           ("er-impl", self.er_impl)]:
            if value is True:
                e += f"--{tag} on "
            elif value is False:
                e += f"--{tag} off "

        e += "--lic "
        if self.largest_inductive_clause:
            e += "on "
        else:
            e += "off "

        e += "--ctg "
        if self.ctg:
            e += "on "
        else:
            e += "off "

        if self.ev_delta is not None:
            e += f"--ev-delta {self.ev_delta} "

        if self.decay is not None:
            e += f"--decay {self.decay}"

        return e


SAT_STRING = "Unsafe, Counter example found of depth"
UN_SAT_STRING = "Safe, Proof found with"
FUNCTIONS_TO_TRACK = """
| rust_formal_verification::engines::pdr::block_cube::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::add_clause_to_frame_at_least                                | 0.156 | 390   | 0.000399 | 14.367     |
| rust_formal_verification::engines::pdr::block_cube::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::make_simplified_cube_non_initial                            | 0.001 | 390   | 0.000004 | 0.131      |
| rust_formal_verification::engines::pdr::block_cube::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::no_predecessor_of_cube                                      | 0.359 | 390   | 0.000920 | 33.077     |
| rust_formal_verification::engines::pdr::block_cube::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::recursively_block_cube                                      | 0.427 | 104   | 0.004108 | 39.394     |
| rust_formal_verification::engines::pdr::definition_library::api::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::add_definition                        | 0.000 | 22    | 0.000014 | 0.028      |
| rust_formal_verification::engines::pdr::definition_library::api::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::save_definition_in_bdd                | 0.000 | 22    | 0.000002 | 0.005      |
| rust_formal_verification::engines::pdr::definition_library::solve::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::bdd_solve_implies                   | 0.091 | 22167 | 0.000004 | 8.354      |
| rust_formal_verification::engines::pdr::definition_library::solve::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::bdd_solve_is_clause_a_contradiction | 0.012 | 493   | 0.000024 | 1.095      |
| rust_formal_verification::engines::pdr::definition_library::solve::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::clause_to_bdd                       | 0.027 | 2691  | 0.000010 | 2.465      |
| rust_formal_verification::engines::pdr::definition_library::solve::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::clause_to_bdd_cached                | 0.039 | 6637  | 0.000006 | 3.605      |
| rust_formal_verification::engines::pdr::definition_library::solve::<impl rust_formal_verification::engines::pdr::definition_library::DefinitionLibrary<_, _>>::is_in_coi_of_some_ev                | 0.004 | 3371  | 0.000001 | 0.356      |
| rust_formal_verification::engines::pdr::frames::complex::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::get_cnf_of_frame                                                     | 0.000 | 1     | 0.000013 | 0.001      |
| rust_formal_verification::engines::pdr::frames::extension_variables::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::call_condense                                            | 0.031 | 723   | 0.000042 | 2.829      |
| rust_formal_verification::engines::pdr::frames::extension_variables::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::condense_frames_by_defining_new_variables                | 0.030 | 22    | 0.001356 | 2.751      |
| rust_formal_verification::engines::pdr::frames::extension_variables::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::fix_redundancy                                           | 0.016 | 22    | 0.000717 | 1.455      |
| rust_formal_verification::engines::pdr::frames::extension_variables::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::perform_bva                                              | 0.002 | 22    | 0.000086 | 0.175      |
| rust_formal_verification::engines::pdr::frames::generalize::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::_generalize_using_definitions_relative_to_frame_new               | 0.141 | 390   | 0.000362 | 13.009     |
| rust_formal_verification::engines::pdr::frames::generalize::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::generalize                                                        | 0.200 | 390   | 0.000512 | 18.404     |
| rust_formal_verification::engines::pdr::frames::generalize::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::generalize_relative_to_frame                                      | 0.057 | 390   | 0.000145 | 5.227      |
| rust_formal_verification::engines::pdr::frames::insert::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::does_a_imply_b                                                        | 0.236 | 83243 | 0.000003 | 21.776     |
| rust_formal_verification::engines::pdr::frames::insert::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::insert_clause_to_exact_frame                                          | 0.252 | 1124  | 0.000224 | 23.267     |
| rust_formal_verification::engines::pdr::frames::insert::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::insert_clause_to_highest_frame_possible                               | 0.153 | 390   | 0.000391 | 14.076     |
| rust_formal_verification::engines::pdr::frames::insert::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::make_delta_element                                                    | 0.054 | 6144  | 0.000009 | 4.944      |
| rust_formal_verification::engines::pdr::frames::insert::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::mark_clause_added                                                     | 0.249 | 1124  | 0.000221 | 22.939     |
| rust_formal_verification::engines::pdr::frames::propagate::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::propagate                                                          | 0.238 | 18    | 0.013226 | 21.955     |
| rust_formal_verification::engines::pdr::frames::propagate::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::propagate_blocked_cubes_in_range                                   | 0.274 | 48    | 0.005716 | 25.301     |
| rust_formal_verification::engines::pdr::frames::propagate_f_inf::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::propagate_to_infinite_frame                                  | 0.040 | 17    | 0.002378 | 3.727      |
| rust_formal_verification::engines::pdr::frames::sat_calls::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::is_clause_satisfied_by_all_initial_states                          | 0.009 | 5052  | 0.000002 | 0.868      |
| rust_formal_verification::engines::pdr::frames::sat_calls::<impl rust_formal_verification::engines::pdr::frames::Frames<_, _>>::is_cube_initial                                                    | 0.002 | 1550  | 0.000001 | 0.199      |
| rust_formal_verification::engines::pdr::prove::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::call_propagate                                                   | 0.465 | 18    | 0.025806 | 42.835     |
| rust_formal_verification::engines::pdr::prove::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::increase_pdr_depth                                               | 0.465 | 18    | 0.025818 | 42.856     |
| rust_formal_verification::engines::pdr::prove::<impl rust_formal_verification::engines::pdr::PropertyDirectedReachability<_, _>>::perform_proof_iteration                                          | 1.053 | 122   | 0.008627 | 97.060     |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::do_ternary_simulation_on_bad_cube_using_sat_solver                 | 0.001 | 104   | 0.000008 | 0.078      |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::do_ternary_simulation_on_predecessor_using_sat_solver              | 0.006 | 333   | 0.000017 | 0.512      |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::extract_variables_from_solver                                      | 0.002 | 755   | 0.000003 | 0.212      |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::get_bad_cube                                                       | 0.004 | 122   | 0.000033 | 0.375      |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::get_predecessor_of_cube                                            | 0.031 | 723   | 0.000043 | 2.885      |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::is_clause_guaranteed_after_transition                              | 0.078 | 1489  | 0.000053 | 7.227      |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::is_clause_guaranteed_after_transition_if_assumed                   | 0.115 | 5304  | 0.000022 | 10.624     |
| rust_formal_verification::engines::pdr::solvers::sat_calls::<impl rust_formal_verification::engines::pdr::solvers::Solvers<_>>::solve_is_cube_blocked                                              | 0.003 | 235   | 0.000012 | 0.258      |
"""
PARAMETERS_TO_TRACK = """
| Depth                                            | 18                   |
| Total memory used (MB)                           | 19                   |
| Count UNSAT core on bad cube reduction           | 104                  |
| Average UNSAT core on bad cube reduction         | 0.6528846153846153   |
| Count UNSAT core on predecessor reduction        | 333                  |
| Average UNSAT core on predecessor reduction      | 0.1471471471471471   |
| Count UNSAT core when no predecessor reduction   | 390                  |
| Average UNSAT core when no predecessor reduction | 0.5630493576741041   |
| Count generalization EV reduction                | 390                  |
| Average generalization EV reduction              | -0.03573023671281825 |
| Count generalization no EV reduction             | 390                  |
| Average generalization no EV reduction           | 0.13384912959381048  |
| Fractional Propagation Successful                | 172                  |
| Fractional Propagation Unsuccessful              | 709                  |
| Frame propagation skipped due to no changes      | 33                   |
| Number of SAT calls                              | 7873                 |
| Number of SAT calls (SAT)                        | 5421                 |
| Number of SAT calls (UNSAT)                      | 2452                 |
| Propagation Successful                           | 562                  |
| Propagation Unsuccessful                         | 24                   |
| Total Proof Obligations                          | 723                  |
| Violating State Developed                        | 104                  |
| does_a_imply_b COI subtraction                   | 45711                |
| does_a_imply_b d_lib empty                       | 190                  |
| does_a_imply_b neither clause contains EVs       | 3807                 |
| does_a_imply_b solved by subsumption (true)      | 147                  |
| does_a_imply_b total calls                       | 83243                |
| solve_implies cache hit                          | 11221                |
| solve_implies solved with DDs.                   | 22167                |
| Proof obligations                                | 0                    |
| Trace Tree size                                  | 288                  |
| Extension Variables                              | 22                   |
| Delta of Infinite Frame                          | 0                    |
| Total Clauses                                    | 187                  |
| BVA count                                        | ?                    |
| BVA unsuccessful, no matches                     | ?                    |
| BVA unsuccessful, insufficient matches           | ?                    |
"""


def functions_to_track() -> list[str]:
    r = list(set([
        f.split("|")[1].strip().split("::")[-1]
        for f in FUNCTIONS_TO_TRACK.splitlines() if len(f) > 0
    ]))
    r.sort()
    assert all([a < b for a, b in zip(r, r[1:])]), "duplicates"
    return r


def parameters_to_track() -> list[str]:
    r = list(set([
        f.split("|")[1].strip()
        for f in PARAMETERS_TO_TRACK.splitlines() if len(f) > 0
    ]))
    r.sort()
    assert all([a < b for a, b in zip(r, r[1:])]), "duplicates"
    return r


"""
***************************************************************************************************
helper functions for individual analysis
***************************************************************************************************
"""


def numeric_columns() -> list[str]:
    r = ["Clauses", "Variables", "Literals", "TimeError", "Depth", "Memory (kB)", "AuxVars"]
    r += functions_to_track()
    r += parameters_to_track()
    return r


def get_numeric_aggregations(i: int, d_name: str) -> list[dict]:
    names = numeric_columns()
    r = []
    for c, cond_col in [
        ("AuxVars", "AuxVars"),
        ("AndAuxVars", "AuxVars"),
        ("XorAuxVars", "AuxVars"),
        ("UsedAuxVars", "InvariantSize"),
        ("UsedAndAuxVars", "InvariantSize"),
        ("UsedXorAuxVars", "InvariantSize"),
        ("InvariantSize", "InvariantSize")
    ]:
        m = f"if 0 < {cond_col} < inf"
        r += [
            {
                "name": f"{r} Average {c} ({m})",
                "cmd": lambda csv_dict, name=c: average_or_median(
                    csv_dict=csv_dict, i=i, calculation="average", condition_columns=[cond_col],
                    column_name=f"{name}", ignore_zero=False, ignore_inf=False
                )
            },
            {
                "name": f"{r} Median {c} ({m})",
                "cmd": lambda csv_dict, name=c: average_or_median(
                    csv_dict=csv_dict, i=i, calculation="median", condition_columns=[cond_col],
                    column_name=f"{name}", ignore_zero=False, ignore_inf=False
                )
            },
        ]
    for n in names:
        r += [
            {
                "name": f"{d_name} Average {n}",
                "cmd": lambda csv_dict, name=n: average_or_median(
                    csv_dict=csv_dict, i=i, calculation="average", condition_columns=[],
                    column_name=f"{name}"
                )
            },
        ]
    return r


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


def o_get_trace_sizes(output: str) -> str:
    r = []
    for p in output.split("| Depth "):
        x = get_number_after_last_appearance_of_string(
            output=p,
            default=float('inf'),
            string=f"| Total Clauses ",
            offset=1
        )
        r.append(x)
    return "_".join([str(int(x)) if math.isfinite(x) else "inf" for x in r])


def o_get_proof_obligation_sizes(output: str) -> str:
    r = []
    for p in output.split("| Depth "):
        x = get_number_after_last_appearance_of_string(
            output=p,
            default=float('inf'),
            string=f"| Total Proof Obligations ",
            offset=1
        )
        r.append(x)
    return "_".join([str(int(x)) if math.isfinite(x) else "inf" for x in r])


def o_get_invariant_size(output: str) -> float:
    if SAT_STRING in output:
        return -float('inf')
    return get_number_after_last_appearance_of_string(
        output=output,
        string="Safe, Proof found with",
        default=float('inf')
    )


def o_get_result(output) -> str:
    return get_result(output=output, sat_str=SAT_STRING, un_sat_str=UN_SAT_STRING)


def get_clauses_in_cnf(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string="Number of clauses in CNF = "
    )


def o_get_number_of_defined_auxiliary_vars(output: str, var: str) -> int:
    assert var in ["XOR", "AND"]
    return int(get_number_after_last_appearance_of_string(
        output=output,
        default=0,
        string=f"| {var} Extension Variables ",
        offset=1
    ))


def o_get_number_of_auxiliary_variables(output: str) -> float:
    a = o_get_number_of_defined_auxiliary_vars(output=output, var="AND")
    b = o_get_number_of_defined_auxiliary_vars(output=output, var="XOR")
    return a + b


def o_get_number_of_used_auxiliary_variables_custom(output: str, var: str) -> int:
    assert var in ["XOR", "AND"]
    output.splitlines()
    return len([lin for lin in output.splitlines() if
                "Extension Variable used in the invariant" in lin and var in lin])


def o_get_number_of_used_auxiliary_variables(output: str) -> int:
    return output.count("Extension Variable used in the invariant")


def get_variables_in_cnf(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string="Number of variables in CNF = "
    )


def get_literals_in_cnf(output: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        string="Number of literals in CNF = "
    )


def get_function_runtime(output: str, function_name: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        default=0,
        string=f"::{function_name} ",
        offset=1
    )


def get_parameter_value(output: str, parameter: str):
    return get_number_after_last_appearance_of_string(
        output=output,
        default=0,
        string=f"| {parameter}  ",
        offset=1
    ) + get_number_after_last_appearance_of_string(
        output=output,
        default=0,
        string=f"| {parameter} |",
        offset=0
    )


def analyze_output(output: str) -> dict[str, str]:
    a = {
        "TimeError": o_get_time_from_output(output=output),
        "Result": o_get_result(output=output),
        "Clauses": get_clauses_in_cnf(output=output),
        "Variables": get_variables_in_cnf(output=output),
        "Literals": get_literals_in_cnf(output=output),
        "Memory (kB)": get_memory_usage_using_bin_time(output=output),
        "InvariantSize": o_get_invariant_size(output=output),
        "AuxVars": o_get_number_of_auxiliary_variables(output=output),
        "AndAuxVars": lambda o: o_get_number_of_defined_auxiliary_vars(
            output=o, var="AND"
        ),
        "XorAuxVars": lambda o: o_get_number_of_defined_auxiliary_vars(
            output=o, var="XOR"
        ),
        "UsedAuxVars": o_get_number_of_used_auxiliary_variables(output=output),
        "UsedAndAuxVars": lambda o: o_get_number_of_used_auxiliary_variables_custom(
            output=o, var="AND"
        ),
        "UsedXorAuxVars": lambda o: o_get_number_of_used_auxiliary_variables_custom(
            output=o, var="XOR"
        ),
        "TraceSizes": o_get_trace_sizes(output=output),
        "POSizes": o_get_proof_obligation_sizes(output=output),
    }
    for f in functions_to_track():
        a[f] = get_function_runtime(output, function_name=f)
    for p in parameters_to_track():
        a[p] = get_parameter_value(output, parameter=p)
    return a


def analyze_output_json(output: str) -> dict[str, str]:
    print("output =", get_problem_name_from_output(output=output))
    dec = json.JSONDecoder()
    pos = 0
    jsons = []
    while not pos == len(output):
        if output[pos] != "{":
            pos += 1
            continue
        try:
            j, json_len = dec.raw_decode(output[pos:])
            pos += json_len
            try:
                if 'json_tag' in j:
                    jsons.append(j)
            except TypeError:
                continue
        except json.JSONDecodeError:
            pos += 1
    # print(f"Found {len(jsons)} JSONs in RFV output")
    time_stats = [j for j in jsons if j['json_tag'] == "TimeStats"]
    pdr_stats = [j for j in jsons if j['json_tag'] == "PDRStats"]
    pdr_params = [j for j in jsons if j['json_tag'] == "PDRParameters"]

    a = {
        "TimeError": o_get_time_from_output(output=output),
        "Result": o_get_result(output=output),
        "Clauses": get_clauses_in_cnf(output=output),
        "Variables": get_variables_in_cnf(output=output),
        "Literals": get_literals_in_cnf(output=output),
        "Memory (kB)": get_memory_usage_using_bin_time(output=output),
        "InvariantSize": o_get_invariant_size(output=output),
    }

    if len(pdr_stats) > 0:
        last_pdr_stat = pdr_stats[-1]
        for k in last_pdr_stat:
            if k == 'json_tag':
                continue
            a[k] = last_pdr_stat[k]

    if len(time_stats) > 0:
        last_time_stats = time_stats[-1]
        for k in last_time_stats:
            if k == 'json_tag':
                continue
            # print("function name", k)
            function_stats = last_time_stats[k]
            a[f"function {k} total"] = function_stats["Total"]
            a[f"function {k} count"] = function_stats["Count"]
            a[f"function {k} average"] = function_stats["Average"]
            a[f"function {k} percentage"] = function_stats["Percentage"]

    # fix empty depth
    if 'Depth' not in a:
        a['Depth'] = -1

    return a


"""
***************************************************************************************************
deployment
***************************************************************************************************
"""


def rfv_pdr_deployment(
        params: RFVParams
) -> Deployment:
    r = Deployment()
    r.name = str(params).lower().replace(" ", "_")
    fetch_cmd = f"cp -r /usr/src/pdrer_crate ./rust-formal-verification"
    r.fetch_command = lambda: fetch_cmd
    r.cd_after_fetch = lambda: "rust-formal-verification"
    r.compilation_command = lambda: "RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --example pdr_engine_for_hwmcc --target x86_64-unknown-linux-gnu && cp ./target/x86_64-unknown-linux-gnu/release/examples/pdr_engine_for_hwmcc ./pdr_engine_for_hwmcc && cargo clean"
    r.run_command = lambda aig_file: params.get_run_command(aig_file=aig_file)
    r.version = lambda: get_current_commit_hash()
    if False:
        r.individual_analysis = analyze_output_json
    else:
        r.individual_analysis = analyze_output
    return r


def rfv_pdr_aggregate_analysis(
        i: int, params: RFVParams, add_numeric=False
) -> list[dict[str, any]]:
    r = str(params)
    a = [
        {"name": f"{r} SOLVED",
         "cmd": lambda csv_dict: count(
             csv_dict=csv_dict, i=i, column_name="Result", target="SAT"
         ) + count(
             csv_dict=csv_dict, i=i, column_name="Result", target="UN_SAT"
         )},
        {"name": f"{r} Error",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
                                       column_name="TimeError",
                                       target="ERROR")},
        {"name": f"{r} Memory Error",
         "cmd": lambda csv_dict: count(
             csv_dict=csv_dict,
             i=i,
             column_name="TimeError",
             target="MEMORY ERROR"
         ) + count(
             csv_dict=csv_dict,
             i=i,
             column_name="TimeError",
             target="C++ EXCEPTION IN RUST PROGRAM"
         )},
        {"name": f"{r} SAT",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
                                       column_name="Result",
                                       target="SAT")},
        {"name": f"{r} UN-SAT",
         "cmd": lambda csv_dict: count(csv_dict=csv_dict, i=i,
                                       column_name="Result", target="UN_SAT")},
    ]
    if add_numeric:
        a += get_numeric_aggregations(i, d_name=r)
    return a


def get_cross_examination(i: int, j: int) -> list[dict[str, any]]:
    r = []
    for f in numeric_columns():
        r.append(
            {
                "name": f"delta {f}",
                "cmd": lambda data: data[i][f] / data[j][f]
            }
        )
    return []


def make_deployment_profile(params: list[RFVParams]) -> DeploymentProfiles:
    return DeploymentProfiles(
        name=f"RFV {len(params)} runs ({' vs. '.join(str(p) for p in params)})",
        deployments=[rfv_pdr_deployment(params=p) for p in params],
        aggregate_analysis=list(chain.from_iterable(
            [rfv_pdr_aggregate_analysis(i=i, params=p) for (i, p) in enumerate(params)]
        )),
    )


"""
***************************************************************************************************
profiles
***************************************************************************************************
"""

rfv_runs = [
    make_deployment_profile(params=[RFVParams()]),
    make_deployment_profile(params=[RFVParams(default=False)]),
    make_deployment_profile(params=[RFVParams(default=False, er=True)]),
    make_deployment_profile(params=[RFVParams(default=False, ctg=True)]),
    make_deployment_profile(params=[RFVParams(default=False, er=True, ctg=True)]),
    make_deployment_profile(params=[RFVParams(branch="dev")]),
    make_deployment_profile(params=[RFVParams(branch="dev", default=False)]),
    make_deployment_profile(
        params=[RFVParams(branch="dev", default=False, er=True)]),
    make_deployment_profile(params=[RFVParams(branch="dev", default=False, ctg=True)]),
    make_deployment_profile(
        params=[RFVParams(branch="dev", default=False, er=True, ctg=True)]),

    make_deployment_profile(
        params=[RFVParams(branch="jan_2025_cav_pdr_er")]
    ),
    make_deployment_profile(
        params=[RFVParams(branch="jan_2025_cav_pdr_er", default=False)]
    ),
    make_deployment_profile(
        params=[RFVParams(branch="jan_2025_cav_pdr_er", default=False, er=True)]
    ),
    make_deployment_profile(
        params=[RFVParams(branch="jan_2025_cav_pdr_er", default=False, ctg=True)]
    ),
    make_deployment_profile(
        params=[RFVParams(
            branch="jan_2025_cav_pdr_er",
            default=False,
            er=True,
            ctg=True
        )]
    ),

    # all off
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er",
            default=False, er=True, er_impl=False,
            er_gen=False, er_fp=False
        )]
    ),

    # one on
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=True, er_gen=False, er_fp=False
        )]
    ),
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=False, er_gen=True, er_fp=False
        )]
    ),
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=False, er_gen=False, er_fp=True
        )]
    ),

    # two on
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=True, er_gen=True, er_fp=False
        )]
    ),
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=False, er_gen=True, er_fp=True
        )]
    ),
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=True, er_gen=False, er_fp=True
        )]
    ),

    # Three on
    make_deployment_profile(
        params=[RFVParams(
            branch="mar_2025_cav_pdr_er", default=False, er=True,
            er_impl=True, er_gen=True, er_fp=True
        )]
    ),
]
