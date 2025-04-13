"""
***************************************************************************************************
import
***************************************************************************************************
"""

import argparse

import utils
from deployments import deployment_profiles

"""
***************************************************************************************************
import
***************************************************************************************************
"""


def parse_arguments():
    parser = argparse.ArgumentParser(
        prog='Benchmark HardWare Model Checkers',
        description=utils.WELCOME_MSG,
        epilog='For more features open a issue the github repo.',
        formatter_class=argparse.RawTextHelpFormatter
    )

    # pick one of 2 configurations
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument(
        '--local',
        help='Run the race on a local machine with the number of threads specified.',
        action='store_true',
        default=False,
    )
    group.add_argument(
        '--slurm',
        help='Send batch jobs to slurm cluster.',
        action='store_true',
        default=False,
    )

    # pick parameters for each test
    parser.add_argument(
        '-t',
        '--timeout',
        help='The timeout in seconds to set for each algorithm to run on each test.',
        type=int,
        required=True,
    )
    k = "\n".join([f"{i} -> {x.name}" for i, x in enumerate(deployment_profiles)])
    parser.add_argument(
        '-d',
        help=f'Index of deployment profile to run, add yours in `src/deployments/__init__.py`:\n{k}',
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
        '--tests',
        nargs='+',
        help='The specific tests you would like to run, only name the test i.e "mul7". '
             'The test must exist in aig_inputs, to select all test simply provide "aig"',
        default="aig"
    )
    parser.add_argument(
        '-c',
        '--threads',
        help='Number of threads to use at the same time when using local mode',
        type=int,
        default=1,
    )
    parser.add_argument(
        '-s',
        '--saved',
        help='Indicate that the deployment has not changed. This will lead to skipping the fetching and compilation of the git source.',
        action='store_true',
        default=False,
    )
    parser.add_argument(
        '-a',
        '--analyze-only',
        help='Do not deploy any runs and just analyze the results in the provided subdirectory of /work_dirs',
        type=str,
        default='',
    )
    parser.add_argument(
        '-p',
        '--partition',
        help='The partition to use for the slurm cluster (only used when using slurm)',
        type=str,
        default="development",
    )
    args = parser.parse_args()
    return args
