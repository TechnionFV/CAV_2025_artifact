import utils


def compile_deployments(work_dir: str, deployments):
    print("Compiling...")
    utils.run_cmd(f"mkdir {work_dir}/repos")

    previous_fetch_commands = {}
    # now compile all the deployments
    for i, deployment in enumerate(deployments):
        print(f"Compiling deployment #{i}")

        # make directory for deployment
        utils.run_cmd(f"mkdir {work_dir}/repos/deployment_{i}")
        utils.change_directory(f"{work_dir}/repos/deployment_{i}")

        # get commands
        fetch_command = deployment.fetch_command()
        cd_path = deployment.cd_after_fetch()
        compilation_command = deployment.compilation_command()
        k = (fetch_command, cd_path, compilation_command)

        if k in previous_fetch_commands:
            j = previous_fetch_commands[k]
            utils.run_cmd(f"cp -r ../deployment_{j}/* .")
        else:
            utils.run_cmd(fetch_command)
            utils.change_directory(cd_path)
            utils.run_cmd(compilation_command)
            previous_fetch_commands[k] = i

    print("Finished compiling.")
