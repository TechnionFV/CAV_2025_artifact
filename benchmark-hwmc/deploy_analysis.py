"""
***************************************************************************************************
import
***************************************************************************************************
"""

import argparse
import csv
import json
import sys
from time import sleep

import utils
from deployments import deployment_profiles
from graph_maker import make_plots

"""
***************************************************************************************************
Analysis results
***************************************************************************************************
"""

individual_result = dict[str, dict[str, str]]
aggregate_result = dict[str, str]

"""
***************************************************************************************************
helper functions
***************************************************************************************************
"""


def __individual_analysis(work_dir: str, deployments) -> (
        list[str], list[individual_result]):
    versions = []
    paths = []
    for i, deployment in enumerate(deployments):
        # change directory
        utils.change_directory(f"{work_dir}/repos/deployment_{i}")
        cd_path = deployment.cd_after_fetch()
        utils.change_directory(cd_path)
        # get version
        version = deployment.version()
        versions.append(version)

        # change directory to outputs
        utils.change_directory(f"{work_dir}/outputs/deployment_{i}")
        # get all output files
        output_files = utils.get_all_file_paths_in_dir_that_have_desired_ending(
            rootdir=".",
            desired_ending=".txt"
        )
        output_files.sort()

        # analyze each file
        header = []
        csv_results = {}
        for file_name in output_files:
            example_name = file_name.replace(f"./{deployment.name}_", "")
            output = utils.read_file(path=file_name)
            analysis = deployment.individual_analysis
            output_datapoints: dict[str, str] = analysis(output)
            for datapoint_name in output_datapoints:
                if datapoint_name in header:
                    continue
                simp_datapoint_name = datapoint_name.replace(' ', '').replace(',', '').replace('\\',
                                                                                               '')
                if simp_datapoint_name != datapoint_name:
                    print(
                        f"Warning: The key `{datapoint_name}` contains spaces, commas or other characters that prevent SQL queries on this column",
                        file=sys.stderr, flush=True)
                header += [datapoint_name]
            csv_results[example_name] = output_datapoints

        file_p = f"{work_dir}/results/deployment_{i}.csv"
        paths.append(file_p)
        with open(file_p, 'w', encoding='UTF8') as f:
            writer = csv.writer(f)
            writer.writerow(["model"] + header)
            for row in csv_results:
                r_dict = csv_results[row]
                line = [row] + [r_dict[x] if x in r_dict else "" for x in header]
                writer.writerow(line)

    individual_results = utils.read_csv_files(paths=paths)
    return versions, individual_results


def __aggregate_analysis(
        ce_result: dict[str, list[dict[str, str]]],
        aggregate_analysis
) -> aggregate_result:
    aggregate_results = {}
    for i, r in enumerate(aggregate_analysis):
        k = r["name"]
        v = r["cmd"](ce_result)
        aggregate_results[k] = v
    return aggregate_results


def __cross_examination(
        work_dir: str,
        individual_results: list[individual_result],
) -> dict[str, list[dict[str, str]]]:
    shared_keys = list(set.intersection(*[set(x.keys()) for x in individual_results]))
    shared_keys.sort()
    result = {}
    for key in shared_keys:
        data = [x[key] for x in individual_results]
        ce = {}
        data.append(ce)
        result[key] = data
    # write csv
    with open(f"{work_dir}/results/cross_examination.csv", 'w', encoding='UTF8') as f:
        writer = csv.writer(f)
        header = [(i, k) for i, d in enumerate(result[shared_keys[0]]) for k in d]
        writer.writerow(["model"] + header)
        for row in result:
            list_of_dicts = result[row]
            line = [row] + [list_of_dicts[i][k] for i, k in header]
            writer.writerow(line)
    return result


"""
***************************************************************************************************
API
***************************************************************************************************
"""


# def __analyze_slurm(repo_path, timeout) -> AnalysisResults:
#     results = __get_results(repo_path=repo_path, timeout=timeout)
#     return __analyze_results(repo_path=repo_path, results=results)


def main():
    parser = argparse.ArgumentParser(
        prog='Analysis tool for files in outputs',
        description='',
        epilog='For more features open a issue the github repo.'
    )
    parser.add_argument(
        '--slurm',
        help='To be used when deployed in slurm, in such a case this script waits till squeue is free of tasks that were deployed by the user',
        action='store_true',
        default=False,
    )
    parser.add_argument(
        '-t',
        help='The test time in seconds.',
        type=int,
        required=True
    )
    parser.add_argument(
        '-d',
        help='Deployment profile to run.',
        type=int,
        required=True,
    )
    parser.add_argument(
        '--suite',
        help='The suite of aig files to run (name of folder in aig_inputs)',
        choices=utils.__get_possible_suites(),
        required=True,
    )
    parser.add_argument(
        '--workdir',
        help='The working directory that was made for the run.',
        type=str,
        required=True,
    )
    parser.add_argument(
        '--tests',
        type=str,
        required=True
    )
    args = parser.parse_args()

    # wait for all deployed processes
    if args.slurm:
        sleep_time = args.t * 1.1 + 60
        print(f"SLEEPING FOR {sleep_time} SECONDS")
        sleep(sleep_time)
        print("AWAKE")

    repo_path = utils.get_path_to_repo()
    work_dir = f"{repo_path}/work_dirs/{args.workdir}"
    utils.change_directory(work_dir)
    utils.run_cmd("rm -rf results")
    utils.run_cmd("mkdir results")
    utils.run_cmd("mkdir results/graphs")

    deployments = deployment_profiles[args.d].deployments
    # cross_examinations = deployment_profiles[args.d].cross_examinations
    aggregate_analysis = deployment_profiles[args.d].aggregate_analysis

    # analyze each deployment individually
    versions, individual_results = __individual_analysis(work_dir=work_dir, deployments=deployments)

    ce_result = __cross_examination(work_dir=work_dir, individual_results=individual_results)

    # aggregate analysis
    aggregate_results = __aggregate_analysis(ce_result=ce_result,
                                             aggregate_analysis=aggregate_analysis)

    make_plots(work_dir=work_dir, time_limit=args.t, deployment_names=[d.name for d in deployments])

    # build commit message
    option = "slurm" if args.slurm else "local"
    commit_msg = f"OPTIONS: --{option} -t {args.t} --suite {args.suite} --tests {args.tests} --workdir {args.workdir} ("
    commit_msg += " vs. ".join(
        [f"{deployments[i].name} {versions[i]}" for i in range(len(deployments))])
    commit_msg += ")\n"
    commit_msg += json.dumps(aggregate_results, indent=4)

    # write commit message to file
    with open(f"{work_dir}/results/commit_message.txt", 'w', encoding='utf-8') as f:
        f.write(commit_msg)

    # copy to repo results
    utils.change_directory(repo_path)
    utils.run_cmd("rm -rf results")
    utils.run_cmd(f"cp -r {work_dir}/results ./results")

    print(commit_msg , flush=True)
    # commit
    # utils.run_cmd("git add .")
    # utils.run_cmd('git config user.email "automatic.commit@notreal.notreal"')
    # utils.run_cmd('git config user.name "Automatic Commit"')
    # utils.run_cmd(f"git commit -m '{commit_msg}'")


if __name__ == "__main__":
    main()
