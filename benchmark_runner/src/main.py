"""
***************************************************************************************************
import
***************************************************************************************************
"""

import os
import pathlib
import random
from datetime import datetime
from multiprocessing import Pool
from time import sleep

import compilation
import utils
from arguments import parse_arguments
from deployments import deployment_profiles

"""
***************************************************************************************************
run
***************************************************************************************************
"""


def __custom_run_cmd(i_cmd: tuple[int, str]):
    i, cmd = i_cmd
    print(f"--- i = {i} RUNNING {cmd}")
    os.system(cmd)


def __run(repo_path: str, work_dir: str, aig_files: list[str], args):
    # create directories for outputs
    utils.run_cmd(f"mkdir {work_dir}/outputs")
    deployments = deployment_profiles[args.d].deployments
    for i, _ in enumerate(deployments):
        utils.run_cmd(f"mkdir {work_dir}/outputs/deployment_{i}")

    i_files = [(x, y) for x, y in enumerate(aig_files)]
    # shuffle them to avoid unexpected performance differences when similar tests run close
    # together
    random.shuffle(i_files)

    # make commands to send
    commands = []
    for k, (i, test_path) in enumerate(i_files):
        output_file = test_path.split("/")[-1]
        for j, deployment in enumerate(deployments):
            name = deployment.name
            out = f"'{work_dir}/outputs/deployment_{j}/{name}_{i}_{output_file}.out.txt'"
            cmd = f"python3 src/deploy_algorithms.py -d {args.d} -i {j} --file {test_path} --timeout {args.timeout} --workdir {work_dir}"
            if args.slurm:
                job_name = f"{j}_{k}"
                # timeout is used just for making sure no job goes overboard, but timeout should
                # be the one to kill the process inside the job
                timeout_min = ((args.timeout / 60 + 1) * 1.1).__ceil__()
                p = args.partition
                command_to_send = f"sbatch -p {p} --cpus-per-task=1 --time={timeout_min} -J {job_name} --output={out} --wrap='{cmd}' "
            else:
                command_to_send = f"{cmd} > {out} 2>&1"
            commands.append(command_to_send)

    utils.change_directory(repo_path)
    # deploy local processes if local
    if args.local:
        with Pool(args.threads) as p:
            p.map(__custom_run_cmd, list(enumerate(commands)))

    # deploy slurm jobs if slurm is enabled
    if args.slurm:
        for cmd in commands:
            utils.run_cmd(cmd)


def get_aig_files_in_test(suite: str, tests: list[str], repo_path: str) -> list[str]:
    aig_files = utils.get_all_file_paths_in_dir_that_have_desired_ending(
        rootdir=f"{repo_path}/aig_inputs/{suite}",
        desired_ending=".aig",
    )

    aig_files = [x for x in aig_files if any([(y in x) for y in tests])]
    return aig_files


def __analyze(args, repo_path: str, work_dir: str):
    cmd = f"python3 {repo_path}/src/deploy_analysis.py -t {args.timeout} -d {args.d} --suite {args.suite} --tests {args.tests} --workdir {work_dir}"
    if args.slurm:
        if not args.analyze_only:
            cmd = f"{cmd} --slurm"
        p = args.partition
        utils.run_cmd(
            f"sbatch -p {p} --cpus-per-task=1 -J analysis --output='{repo_path}/deploy_analysis_output.txt' --wrap='{cmd}'")
    else:
        utils.run_cmd(cmd)


def make_work_dir(repo_path: str) -> str:
    workdir = f"{datetime.now().strftime('%Y_%m_%d_%H_%M_%S')}"
    utils.change_directory(repo_path)
    if not pathlib.Path("work_dirs").is_dir():
        utils.run_cmd("mkdir work_dirs")
    utils.change_directory("work_dirs")
    assert not pathlib.Path(workdir).is_dir()
    utils.run_cmd(f"mkdir {workdir}")
    return workdir


"""
***************************************************************************************************
main
***************************************************************************************************
"""


def main():
    # parsing arguments
    args = parse_arguments()

    # print message
    print(utils.WELCOME_MSG)

    # get repo path
    repo_path = utils.get_path_to_repo()

    # print parameters
    print(args)
    print(f"Deployment Profile name: {deployment_profiles[args.d].name}")
    aig_files = get_aig_files_in_test(suite=args.suite, tests=args.tests, repo_path=repo_path)
    print(f"Number of tests to run on: {len(aig_files)}")
    work_dir = make_work_dir(repo_path=repo_path) if args.analyze_only == "" else args.analyze_only
    print(f"Working Directory: {work_dir}")
    full_work_dir = f"{repo_path}/work_dirs/{work_dir}"

    # wait to show parameters
    sleep(5)

    # deploy runs
    if args.analyze_only == '':
        aig_files.sort()

        # compile
        if not args.saved:
            deployments = deployment_profiles[args.d].deployments
            compilation.compile_deployments(work_dir=full_work_dir, deployments=deployments)

        # deploy runs
        __run(repo_path=repo_path, work_dir=full_work_dir, aig_files=aig_files, args=args)

    # perform analysis
    __analyze(args=args, repo_path=repo_path, work_dir=work_dir)


"""
***************************************************************************************************
call main
***************************************************************************************************
"""

# this is better than simply defining main here because this avoids global variables
if __name__ == "__main__":
    main()
