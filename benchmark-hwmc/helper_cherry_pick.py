"""
Run this script to make graphs comprised of runs from different commits
"""
import argparse
import ast

import utils
from graph_maker import make_plots
from main import make_work_dir


def compare(time_limit: int, details: list[tuple[str, int, str]]):
    repo_path = utils.get_path_to_repo()
    workdir = make_work_dir(repo_path=repo_path)
    utils.change_directory(repo_path)
    utils.run_cmd(f"mkdir ./work_dirs/{workdir}/results")
    for i, (commit, d, name) in enumerate(details):
        utils.run_cmd(
            f"git show {commit}:results/deployment_{d}.csv > ./work_dirs/{workdir}/results/deployment_{i}.csv ")
    utils.change_directory(f"./work_dirs")
    utils.run_cmd(f"mkdir {workdir}/results/graphs")
    make_plots(work_dir=workdir, time_limit=time_limit,
               deployment_names=[n for _, _, n in details])


def parse_details(details_str):
    """Parses the details string into a list of tuples."""
    try:
        details_list = ast.literal_eval(details_str)
        if not isinstance(details_list, list):
            raise ValueError("Details must be a list.")
        if not details_list:  # Check if the list is empty
            raise ValueError("Details list cannot be empty.")
        for item in details_list:
            if not isinstance(item, tuple) or len(item) != 3:
                raise ValueError("Each detail item must be a tuple of length 3.")
            if not isinstance(item[0], str) or not isinstance(item[1], int) or not isinstance(
                    item[2], str):
                raise ValueError("Each detail tuple must contain (str, int, str).")
        return details_list
    except (ValueError, SyntaxError) as e:
        raise argparse.ArgumentTypeError(f"Invalid details format: {e}")


def main():
    print(
        """Usage: python compare_script.py --time_limit 7200 --details "[('0405346df4edadaea8a4287db562747f72bc926e', 0, 'pdr'), ('e353e2aae9ff96f65e8e0f6c20df20c94d326c23', 0, 'pdrER')]" """)
    parser = argparse.ArgumentParser(description="Compare something with given details.")
    parser.add_argument("--time_limit", type=int, default=3600,
                        help="Time limit in seconds (default: 3600).")
    parser.add_argument("--details", type=parse_details, required=True,
                        help="List of tuples representing details. Example: '[(\"0405346df4edadaea8a4287db562747f72bc926e\", 0, \"pdr\"), (\"e353e2aae9ff96f65e8e0f6c20df20c94d326c23\", 0, \"pdrER\")]'")

    args = parser.parse_args()

    compare(time_limit=args.time_limit, details=args.details)

    # print("HWMCC ALL", flush=True)
    # sleep(2)
    # compare(
    #     time_limit=300,
    #     details=[
    #         # ("fffb059a2e59e7a925c7afdff72a80a08b602bb6", 0, "abc"),
    #         # ("e85dc1672beeecabd7812da8418178f529238c3b", 0, "pdr"),
    #         # ("7ccb88504b44810fada11e3db40838c8aca48d58", 0, "pdrER"),
    #         # ("6e3c5c5e2e6194ee885a121e1d2c32f9e8bc93de", 0, "abc"),
    #         # ("5980a4f1cee8315a244de6ac5cc6a16209d8a416", 0, "pdr dev CaDiCal one solver ternary normal"),
    #         # ("667c81f92d9eaa4975fe7917dc04737c052ba6df", 0, "pdr dev CaDiCal one solver"),
    #         # ("5b92a37afc8e69cc7a7b03eeeab463b98b4eda13", 0, "pdr dev CaDiCal"),
    #         # ("0e937c4ba2b5502f82218012c249c21829a89777", 0, "pdr dev orig"),
    #         ("cae0b51e4d09c712c8d7e4b56abc011d3ac5af6e", 0, "pdr cav"),
    #         # ("330261d497beddbfc3762da299516160676ef854", 0, "pdr main"),
    #         ("f5cb4a711731f20c5efe632d39be06d020c51369", 0, "rIC3"),
    #         ("6ef92b19a1cec009f0976d5ce109bf5338a3214f", 0, "rIC3 1"),
    #         ("632bf084f9dfd7f8c9f336904d1a68343fcb4a9b", 0, "rIC3 2"),
    #         ("b3a370fd54a81cbbc029a27370f1ea216f3d9655", 0, "rIC3 3"),
    #         # ("e44626f1ee1578e90e5f7152842f268ab741a048", 0, "rIC3 3"),
    #         # ("db997d3ee575f1f6b3ec274f7fc150ebfcffb8f0", 0, "rIC3 4"),
    #         ("7b8599a50496c90dcdbe93b7620f223438452afe", 0, "abc")
    #         # ("719c60c65b3bd784582c3397157665c9a476a08a", 1, "pdrER old"),
    #         # ("ecd76b5d978a72040de4e0b8925f5c563c15f44b", 1, "pdrER new"),
    #     ]
    # )
    #
    # print("HWMCC 24", flush=True)
    # sleep(2)
    # compare(
    #     time_limit=3600,
    #     details=[
    #         # ("fffb059a2e59e7a925c7afdff72a80a08b602bb6", 0, "abc"),
    #         # ("e85dc1672beeecabd7812da8418178f529238c3b", 0, "pdr"),
    #         # ("7ccb88504b44810fada11e3db40838c8aca48d58", 0, "pdrER"),
    #         # ("6e3c5c5e2e6194ee885a121e1d2c32f9e8bc93de", 0, "abc"),
    #         ("0405346df4edadaea8a4287db562747f72bc926e", 0, "pdr"),
    #         ("e353e2aae9ff96f65e8e0f6c20df20c94d326c23", 0, "pdrER"),
    #         # ("719c60c65b3bd784582c3397157665c9a476a08a", 1, "pdrER old"),
    #         # ("ecd76b5d978a72040de4e0b8925f5c563c15f44b", 1, "pdrER new"),
    #     ]
    # )
    #
    # print("HWMCC 20", flush=True)
    # sleep(2)
    # compare(
    #     time_limit=3600,
    #     details=[
    #         # ("95b21554144ad8fe3988c4351126caef914d4be8", 0, "abc"),
    #         # ("9ab8036e131f1024379fa62e2e1d2922c5ca7f0f", 0, "pdr"),
    #         # ("612e27c52b7538e15768271b349bb72408cbd77b", 0, "pdrER"),
    #         # ("509a500d0330bf5d7d4a9972901b19c39a56175f", 0, "abc"),
    #         ("69bf4036316e55dc1628b703718ae5ec9b3c6f16", 0, "pdr"),
    #         ("1c4d2bd0fc878273e8dfa12f7095e9b76a25594c", 0, "pdrER"),
    #     ]
    # )
    #
    # print("HWMCC 19", flush=True)
    # sleep(2)
    # compare(
    #     time_limit=3600,
    #     details=[
    #         # ("e3e47eaa7958f047adf51b2816466794f7399db5", 0, "abc"),
    #         # ("fdb018406e3dd4ab26de578f335c3351754f87a5", 0, "pdr"),
    #         # ("640bcab2e8d5eb225c9444c0d30778277ae054a0", 0, "pdrER"),
    #         # ("d2e1115801ce7c2ea6448a9f60ef7e4d3191079f", 0, "abc"),
    #         ("ca2bcfbce49b97d57708bf3c33983b8cdcca6236", 0, "pdr"),
    #         ("cfc0cf5d77b4364c763a64790ba8788a5e90a53c", 0, "pdrER"),
    #     ]
    # )

    # print("HWMCC 17", flush=True)
    # sleep(2)
    # compare(
    #     time_limit=3600,
    #     details=[
    #         # ("65f8aedd0ad89b37fa5184cc7ed49a8c5c3f2a37", 0, "abc"),
    #         # ("8b2c91a62182e39e3eb2401e79160414be4a001e", 0, "pdr"),
    #         # ("14a89aae9d222b15ba12acc3500d012d34e36f64", 0, "pdrER"),
    #         ("1a22912ff5e1565a80f50f2d6ed196460f67df55", 0, "abc"),
    #         ("6ea09052f19474ff0c105ee3dd42ea619778414a", 0, "pdr"),
    #         ("6ea09052f19474ff0c105ee3dd42ea619778414a", 1, "pdrER"),
    #     ]
    # )


if __name__ == "__main__":
    main()
