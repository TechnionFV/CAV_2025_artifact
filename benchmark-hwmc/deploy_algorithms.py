import resource
import subprocess


def main():
    import argparse
    import utils
    import time
    from deployments import deployment_profiles

    # parse inputs
    parser = argparse.ArgumentParser(
        prog='Slurm Deploy Script',
        description='Container that runs the deployment on file',
        epilog='For more features open a issue the github repo.'
    )
    parser.add_argument(
        '-d',
        help='Deployment profile to run.',
        type=int,
        required=True,
    )
    parser.add_argument(
        '-i',
        '--index',
        help='The index in deployments you wish to run.',
        type=int,
        required=True
    )
    parser.add_argument(
        '--file',
        help='The path to the file you want to run.',
        type=str,
        required=True
    )
    parser.add_argument(
        '--timeout',
        help='The time limit of the test.',
        type=str,
        required=True
    )
    # --workdir {work_dir}
    parser.add_argument(
        '--workdir',
        help='The working directory to use for finding the executable to run.',
        type=str,
        required=True
    )

    args = parser.parse_args()
    index = args.index
    file = args.file
    # repo_path = utils.get_path_to_repo()

    deployments = deployment_profiles[args.d].deployments
    deployment = deployments[index]

    # change directory to correct location
    utils.change_directory(f"{args.workdir}/repos/deployment_{index}")
    cd_path = deployment.cd_after_fetch()
    utils.change_directory(cd_path)

    start = time.time()

    print(f"--- AIG file name = {file}")
    run_command = deployment.run_command(file)
    memory_limit = 20 * (2 ** 30)  # 20 GB

    def set_memory_limit():
        resource.setrlimit(resource.RLIMIT_AS, (memory_limit, memory_limit))

    cmd = f"/usr/bin/time -v /usr/bin/timeout {args.timeout} {run_command}"
    print(f"--- RUNNING {cmd}", flush=True)
    subprocess.run(
        cmd,
        shell=True,
        preexec_fn=set_memory_limit
    )
    end = time.time()
    print("DONE, OVERALL TOTAL TIME = ", end - start, flush=True)


if __name__ == "__main__":
    main()
