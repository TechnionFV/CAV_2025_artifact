"""
***************************************************************************************************
import
***************************************************************************************************
"""
import csv
import json
import math
from typing import Optional

import matplotlib.pyplot as plt

from utils import read_csv_files

"""
***************************************************************************************************
Constants
***************************************************************************************************
"""

global FILES
global TEST_TIME
global TARGET
DELTA = 2

dataFrame = dict[str, dict[str, any]]

"""
***************************************************************************************************
helper functions
***************************************************************************************************
"""


def time_to_runtime(time: float, timeout: float) -> float:
    if math.isinf(time):
        return timeout
    elif time >= (TEST_TIME + DELTA):
        return timeout
    else:
        float(time)
        return time


def time_to_is_solved(time: float) -> bool:
    if math.isinf(time):
        return False
    elif time >= (TEST_TIME + DELTA):
        return False
    elif TEST_TIME - time < 0.1:
        return False
    else:
        return True


def save_figure(name: str, close=True):
    should_save = True
    if should_save:
        plt.savefig(f"{TARGET}/{name}", metadata={'Date': None}, bbox_inches='tight')
    if close:
        plt.close()


def try_float(x: str, default=float('inf')) -> float:
    try:
        return float(x)
    except Exception:
        return default


def get_times_of_run(dfs: list[dataFrame], i: int) -> list[(str, float)]:
    f = FILES[i]
    df = dfs[i]
    times = []
    for key, value in df.items():
        time_string = value["Time/Error"] if "Time/Error" in value else value["TimeError"]
        time = try_float(time_string)
        if time > (TEST_TIME + DELTA):
            time = float('inf')
        times += [(key, time)]
    return times


def get_depths_of_run(dfs: list[dataFrame], i: int) -> list[(str, float)]:
    f = FILES[i]
    df = dfs[i]
    t = f["depth column"]
    depths = []
    for key, value in df.items():
        depth_string = value[t]
        depth = int(float(depth_string))
        depths += [(key, depth)]
    return depths


def get_invariant_sizes_of_run(dfs: list[dataFrame], i: int) -> Optional[list[(str, float)]]:
    f = FILES[i]
    df = dfs[i]
    t = f["invariant column"]
    try:
        return [(key, float(value[t])) for key, value in df.items()]
    except KeyError:
        print(f"Entry {i} does not have the column {t}")
        return None


def get_trace_size_at_each_depth_of_run(
        dfs: list[dataFrame], i: int
) -> Optional[list[(str, float)]]:
    f = FILES[i]
    df = dfs[i]
    t = f["trace column"]
    try:
        return [(key, value[t]) for key, value in df.items()]
    except KeyError:
        print(f"Entry {i} does not have the column {t}")
        return None


def get_po_size_at_each_depth_of_run(dfs: list[dataFrame], i: int) -> Optional[list[(str, float)]]:
    f = FILES[i]
    df = dfs[i]
    t = f["po column"]
    try:
        return [(key, value[t]) for key, value in df.items()]
    except KeyError:
        print(f"Entry {i} does not have the column {t}")
        return None


def get_results_of_run(dfs: list[dataFrame], i: int) -> list[(str, float)]:
    f = FILES[i]
    df = dfs[i]
    t = f["result column"]
    vals = []
    for key, value in df.items():
        r = value[t]
        vals += [(key, r)]
    return vals


def make_competition_graph(dfs: list[dataFrame]):
    functions: list[list[float]] = []
    virtual_best = None
    for i in range(len(dfs)):
        times = get_times_of_run(dfs, i)
        if virtual_best is None:
            virtual_best = [t for t in times]
        else:
            assert [n for n, _ in virtual_best] == [n for n, _ in times]
            virtual_best = [(n1, min(t1, t2)) for (n1, t1), (n2, t2) in zip(virtual_best, times)]
        times = [t for _, t in times if time_to_is_solved(time=t)]
        times = sorted(times)
        functions += [times]
    virtual_best = [t for _, t in virtual_best if time_to_is_solved(time=t)]
    virtual_best = sorted(virtual_best)
    functions += [virtual_best]

    for times, f, ls in zip(functions, [f["title"] for f in FILES] + ["VB"],
                            ['dashed', 'dashdot', 'solid'] * len(functions)):
        plt.plot(times, [i for i in range(1, len(times) + 1)], label=f, ls=ls)

    plt.xlabel("Time (sec)")
    plt.ylabel("# solved instances")
    # if True:
    #     plt.ylim(ymin=450)
    #     for y in [500, 550, 600]:
    #         plt.axhline(y=y, color='grey', ls=':')
    #     for x in range(500, 3600, 1000):
    #         plt.axvline(x=x, color='grey', ls='dotted')
    # if True:
    #     plt.ylim(ymin=130)
    #     for y in range(140, 191, 20):
    #         plt.axhline(y=y, color='green', ls=':')
    #     for x in range(500, 3600, 1000):
    #         plt.axvline(x=x, color='purple', ls='dotted')
    # plt.axhline(y=575, color='green', ls=':')
    # plt.axvline(x=2200, color='purple', ls='--')
    plt.legend()
    save_figure("competition_plot.svg", close=False)
    save_figure("competition_plot.png")


def make_scatter(title_x: str, title_y: str, x: list[float], y: list[float], file_name: str,
                 title: str):
    f, ax = plt.subplots(figsize=(7, 7))
    # ax.set_aspect('equal', adjustable='box')
    ax.scatter(x, y)
    plt.xlabel(title_x)
    plt.ylabel(title_y)
    # plt.xscale('log')
    # plt.yscale('log')
    # ax.xlim(0)
    # ax.ylim(0)
    plt.title(f'{title} for {title_x} and {title_y}')
    plt.axis('square')
    ax.plot(ax.get_xlim(), ax.get_ylim(), ls="--", c=".3")

    save_figure(f"{file_name}.svg", close=False)
    save_figure(f"{file_name}.png")


def make_time_scatter_plot(dfs: list[dataFrame], i: int, j: int):
    times_i = get_times_of_run(dfs=dfs, i=i)
    times_j = get_times_of_run(dfs=dfs, i=j)

    assert len(times_j) == len(times_i)
    for (x, tx), (y, ty) in zip(times_i, times_j):
        assert x == y
    times_i = [t for _, t in times_i]
    times_j = [t for _, t in times_j]

    for f in [times_i, times_j]:
        for k in range(len(f)):
            if math.isinf(f[k]):
                f[k] = TEST_TIME

    make_scatter(
        title_x=FILES[i]["title"], title_y=FILES[j]["title"],
        x=times_i, y=times_j,
        title="Runtime Comparison",
        file_name=f"time_scatter_plot_{i}_and_{j}"
    )


def make_invariant_size_scatter_plot(dfs: list[dataFrame], i: int, j: int):
    a = get_invariant_sizes_of_run(dfs=dfs, i=i)
    b = get_invariant_sizes_of_run(dfs=dfs, i=j)
    if a is None or b is None:
        return

    assert len(b) == len(a)
    for (x, tx), (y, ty) in zip(a, b):
        assert x == y

    x = []
    y = []
    max_lim = max([i for _, i in a if math.isfinite(i)])
    max_lim = max(max_lim, max([i for _, i in b if math.isfinite(i)]))
    for (n, inv_a), (_, inv_b) in zip(a, b):
        x.append(int(inv_a) if math.isfinite(inv_a) else max_lim)
        y.append(int(inv_b) if math.isfinite(inv_b) else max_lim)
        # if (math.isinf(inv_a) or math.isinf(inv_b)) and (
        #         not (math.isinf(inv_a) and math.isinf(inv_b))):
        #     print(x[-1], y[-1])
    make_scatter(
        title_x=FILES[i]["title"], title_y=FILES[j]["title"],
        x=x, y=y,
        title="Invariant Size",
        file_name=f"invariant_size_scatter_plot_{i}_and_{j}"
    )


def make_trace_size_scatter_plot(dfs: list[dataFrame], i: int, j: int):
    a = get_trace_size_at_each_depth_of_run(dfs=dfs, i=i)
    b = get_trace_size_at_each_depth_of_run(dfs=dfs, i=j)
    if a is None or b is None:
        return

    assert len(b) == len(a)
    for (x, tx), (y, ty) in zip(a, b):
        assert x == y

    aa = []
    bb = []
    change = []
    for (n, arr_1), (_, arr_2) in zip(a, b):
        arr_1 = arr_1.split("_")
        arr_2 = arr_2.split("_")
        min_len = min(len(arr_1), len(arr_2))
        aa.append((n, float(arr_1[min_len - 1])))
        bb.append((n, float(arr_2[min_len - 1])))
        if math.isinf(aa[-1][1]) or math.isinf(bb[-1][1]):
            continue
        change.append(bb[-1][1] if aa[-1][1] == 0 else bb[-1][1] / aa[-1][1])
        # sum_2 +=
        # count += 1
    a = aa
    b = bb
    change.sort()
    ttt = FILES[i]["title"]
    # print(f"Avg. Trace Size at last common depth ({ttt}) = {change[len(change) // 2]}")
    x = [t for _, t in a]
    y = [t for _, t in b]
    make_scatter(
        title_x=FILES[i]["title"], title_y=FILES[j]["title"],
        x=x, y=y,
        title="Trace Size at Last Common Depth",
        file_name=f"last_common_depth_trace_size_scatter_plot_{i}_and_{j}"
    )


def make_proof_obligation_size_scatter_plot(dfs: list[dataFrame], i: int, j: int):
    a = get_po_size_at_each_depth_of_run(dfs=dfs, i=i)
    b = get_po_size_at_each_depth_of_run(dfs=dfs, i=j)
    if a is None or b is None:
        return

    assert len(b) == len(a)
    for (x, tx), (y, ty) in zip(a, b):
        assert x == y

    aa = []
    bb = []
    for (n, arr_1), (_, arr_2) in zip(a, b):
        arr_1 = arr_1.split("_")
        arr_2 = arr_2.split("_")
        min_len = min(len(arr_1), len(arr_2))
        aa.append((n, float(arr_1[min_len - 1])))
        bb.append((n, float(arr_2[min_len - 1])))
    a = aa
    b = bb

    x = [t for _, t in a]
    y = [t for _, t in b]
    make_scatter(
        title_x=FILES[i]["title"], title_y=FILES[j]["title"],
        x=x, y=y,
        title="Total Proof Obligations at Last Common Depth",
        file_name=f"last_common_depth_po_size_scatter_plot_{i}_and_{j}"
    )


def make_scatter_plots(dfs: list[dataFrame]):
    for i in range(len(dfs)):
        for j in range(i + 1, len(dfs)):
            make_time_scatter_plot(dfs=dfs, i=i, j=j)
            make_trace_size_scatter_plot(dfs=dfs, i=i, j=j)
            make_invariant_size_scatter_plot(dfs=dfs, i=i, j=j)
            make_proof_obligation_size_scatter_plot(dfs=dfs, i=i, j=j)
    return


def make_ternary_plot(dfs: list[dataFrame]):
    if len(dfs) < 3:
        return
    title_0 = FILES[0]["title"]
    title_1 = FILES[1]["title"]
    title_2 = FILES[2]["title"]

    times_0 = get_times_of_run(dfs=dfs, i=0)
    times_1 = get_times_of_run(dfs=dfs, i=1)
    times_2 = get_times_of_run(dfs=dfs, i=2)
    assert len(times_0) == len(times_1)
    assert len(times_0) == len(times_2)
    for (x, tx), (y, ty), (z, tz) in zip(times_0, times_1, times_2):
        assert x == y
        assert x == z

    times_0 = [t for _, t in times_0]
    times_1 = [t for _, t in times_1]
    times_2 = [t for _, t in times_2]

    for f in [times_0, times_1, times_2]:
        for k in range(len(f)):
            if math.isinf(f[k]):
                f[k] = TEST_TIME

    plt.close(fig="all")
    fig = plt.figure(figsize=(12, 12))
    ax = fig.add_subplot(projection='3d')
    color = [((t0 / TEST_TIME), (t1 / TEST_TIME), (t2 / TEST_TIME),) for
             t0, t1, t2 in zip(times_0, times_1, times_2)]
    ax.scatter(times_0, times_1, times_2, c=color)
    ax.set_xlabel(title_0)
    ax.set_ylabel(title_1)
    ax.set_zlabel(title_2)
    save_figure("ternary_plot.svg")
    return


def print_recap(dfs: list[dataFrame]):
    times = [get_times_of_run(dfs=dfs, i=i) for i in range(len(dfs))]
    depths = [get_depths_of_run(dfs=dfs, i=i) for i in range(len(dfs))]
    results = [get_results_of_run(dfs=dfs, i=i) for i in range(len(dfs))]
    for vals in [times, depths, results]:
        for f in vals:
            assert len(f) == len(times[0])
            for (x, tx), (y, ty) in zip(f, times[0]):
                assert x == y

    interesting_cases = {}
    models_no_one_solved = []
    wins = [[] for _ in dfs]
    wins_sat = [[] for _ in dfs]
    wins_un_sat = [[] for _ in dfs]
    unique_wins = [{} for _ in dfs]

    time_sum = [0 for _ in dfs]
    virtual_best_time_sum = 0

    depth_sum = [0 for _ in dfs]
    virtual_best_depth_sum = 0

    virtual_best_sat = 0
    virtual_best_un_sat = 0
    for i in range(len(times[0])):
        ts = [x[i] for x in times]
        ds = [x[i][1] for x in depths]
        rs = [x[i][1] for x in results]

        assert not ("SAT" in rs and "UN_SAT" in rs)
        is_sat = "SAT" in rs
        is_un_sat = "UN_SAT" in rs
        assert not (is_sat and is_un_sat)

        model_name = ts[0][0]
        solved_by = []

        real_ts = [time_to_runtime(time=t[1], timeout=TEST_TIME) for t in ts]
        virtual_best_time_sum += min(real_ts)
        for k, runtime in enumerate(real_ts):
            time_sum[k] += runtime

        virtual_best_depth_sum += min(ds)
        for k, depth in enumerate(ds):
            depth_sum[k] += depth

        for j, (e, x) in enumerate(ts):
            if time_to_is_solved(time=x):
                if is_sat:
                    wins_sat[j] += [e]
                if is_un_sat:
                    wins_un_sat[j] += [e]
                wins[j] += [e]
                solved_by += [(j, x)]

        if 0 < len(solved_by) < len(ts):
            # some solver did not solve this
            model_name = ts[0][0]
            interesting_cases[model_name] = solved_by
        if len(solved_by) == 1:
            solver = solved_by[0][0]
            time = solved_by[0][1]
            unique_wins[solver][model_name] = time
        if len(solved_by) > 0:
            assert is_sat or is_un_sat
            if is_sat:
                virtual_best_sat += 1
            else:
                virtual_best_un_sat += 1
        else:
            assert len(solved_by) == 0
            models_no_one_solved += [model_name]

    # print data in tables

    col_labels = [f'{f["title"]} ({len(wins[i])}, {len(unique_wins[i])})' for i, f in
                  enumerate(FILES)]
    index = [k for k in interesting_cases]
    data = []
    for key, value in interesting_cases.items():
        row = []
        for (i, f) in enumerate(FILES):
            is_solved = i in [j for j, _ in value]
            if is_solved:
                row += [str([t for j, t in value if j == i][0])]
            else:
                row += [""]
        assert len(row) == len(col_labels)
        data += [row]

    with open(f'{TARGET}/interesting_models_breakdown.csv', 'w', newline='') as csvfile:
        w = csv.writer(csvfile)
        w.writerow(['model'] + col_labels)
        for x, l in zip(index, data):
            w.writerow([x] + l)

    res_dict = {
        "Recap Time": TEST_TIME
    }
    for i in range(len(dfs)):
        invariants = get_invariant_sizes_of_run(dfs=dfs, i=i)
        invariants = [x for _, x in invariants if math.isfinite(x)]
        assert all(math.isfinite(x) for x in invariants)
        # assert len(invariants) > 0
        ff = FILES[i]
        title = ff["title"]
        res_dict[f"{title} results:"] = {
            f"Solved": len(wins[i]),
            f"Solved (SAT)": len(wins_sat[i]),
            f"Solved (UN-SAT)": len(wins_un_sat[i]),
            "Unique Wins": unique_wins[i],
            "Total Time": time_sum[i],
            "Average Time": time_sum[i] / len(times[0]),
            "Total Depth": depth_sum[i],
            "Average Depth": depth_sum[i] / len(times[0]),
            # "Average Invariant Size": sum(invariants) / len(invariants),
            # "Average Trace Size": sum(invariants) / len(invariants),
            "Latex tabular": f'& {title} & {len(wins[i])} & {len(wins_sat[i])} & {len(wins_un_sat[i])} & {len(unique_wins[i])} & {"{:.1f}".format(time_sum[i] / len(times[0]))} & {"{:.1f}".format(depth_sum[i] / len(times[0]))} \\'
        }

    res_dict["Virtual Best"] = {
        f"Solved": virtual_best_sat + virtual_best_un_sat,
        f"Solved (SAT)": virtual_best_sat,
        f"Solved (UN-SAT)": virtual_best_un_sat,
        "Total Time": virtual_best_time_sum,
        "Average Time": virtual_best_time_sum / len(times[0]),
        "Total Depth": virtual_best_depth_sum,
        "Average Depth": virtual_best_depth_sum / len(times[0]),
        "Latex tabular": f'& VB & {virtual_best_sat + virtual_best_un_sat} & {virtual_best_sat} & {virtual_best_un_sat} & & {"{:.1f}".format(virtual_best_time_sum / len(times[0]))} & {"{:.1f}".format(virtual_best_depth_sum / len(times[0]))} \\'
    }

    res_dict["No Solvers"] = models_no_one_solved
    res_dict["Interesting Cases"] = [str(k).replace(".out.txt", "") for k in interesting_cases]
    res_dict["Interesting Cases One Line"] = " ".join(res_dict["Interesting Cases"])

    with open(f'{TARGET}/recap.json', 'w') as f:
        f.write(json.dumps(res_dict, sort_keys=True, indent=4))


"""
***************************************************************************************************
API
***************************************************************************************************
"""


def make_plots(work_dir: str, time_limit: int, deployment_names: list[str]):
    global TEST_TIME
    global FILES
    global TARGET
    TEST_TIME = time_limit
    FILES = [
        {
            "path": f"{work_dir}/results/deployment_{i}.csv",
            "title": f"{deployment_names[i]}",
            "key column": "model",
            "depth column": "Depth",
            "result column": "Result",
            "trace column": "TraceSizes",
            "po column": "POSizes",
            "invariant column": "InvariantSize"
        } for i in range(len(deployment_names))
    ]
    TARGET = f"{work_dir}/results/graphs"
    # To get consistent SVG graphs (SVGs don't change if the data does not change)
    # uuid.UUID(int=1234567890123456798, version=4)
    import matplotlib as mpl
    mpl.rcParams['svg.hashsalt'] = "1234567890123456789"
    dfs = read_csv_files([x['path'] for x in FILES])
    make_competition_graph(dfs)
    make_scatter_plots(dfs)
    make_ternary_plot(dfs)
    print_recap(dfs)
    print("DONE")


def make_generic_dot_plot(csv_1: str, csv_2: str, column_name: str):
    dfs = read_csv_files([csv_1, csv_2])
    values = [[(key, float(value[column_name])) for key, value in df.items()] for df in dfs]
    dots_i = values[0]
    dots_j = values[1]

    assert len(dots_j) == len(dots_i)
    for (x, tx), (y, ty) in zip(dots_i, dots_j):
        assert x == y

    dots_i = [t for _, t in dots_i]
    dots_j = [t for _, t in dots_j]

    make_scatter(
        title_x="csv_1", title_y="csv_2",
        x=dots_i, y=dots_j,
        title="Temp Dot File",
        file_name=f"temp"
    )


def main():
    pass
    # make_plots(work_dir="merge_result", time_limit=3600, deployment_names=["pdr", "pdrER"])
    # make_generic_dot_plot(
    #     csv_1="/home/andrew/Desktop/formal_verification/my_repos/benchmark-hwmc/work_dirs/merge_result/results/deployment_0.csv",
    #     csv_2="/home/andrew/Desktop/formal_verification/my_repos/benchmark-hwmc/work_dirs/merge_result/results/deployment_1.csv",
    #     column_name=""
    # )
    # make_plots(work_dir="2025_01_25_17_44_35", time_limit=3600, deployment_names=["pdr", "pdrER"])
    # make_plots(work_dir="merge_result", time_limit=3600, deployment_names=["pdr", "pdrER"])
    # make_plots(work_dir="2025_01_25_17_44_35", time_limit=3600, deployment_names=["pdr", "pdrER"])


if __name__ == "__main__":
    main()
