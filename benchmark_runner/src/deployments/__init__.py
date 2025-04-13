import itertools

from .core.abc import abc_runs
from .core.rIC3 import ric3_runs
from .core.rfv import rfv_runs

deployment_profiles = list(itertools.chain(abc_runs, rfv_runs, ric3_runs))
