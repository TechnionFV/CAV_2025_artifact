import csv

import utils
from graph_maker import make_plots
from main import make_work_dir, get_aig_files_in_test

SRC_WORK_DIR = "2025_01_23_05_39_52"
DEPLOYMENTS = ["rfv PDR 17-24", "rfv PDR ER 17-24", "abc PDR 17-24"]
TIME_LIMIT = 3600


def main():
    repo_path = utils.get_path_to_repo()
    original_aig_files = get_aig_files_in_test(suite="hwmcc_17_19_20_24_fold_fraigy_orchestrate",
                                               tests=["aig"], repo_path=repo_path)
    original_aig_files.sort()
    i_original_aig_files = [(x, y) for x, y in enumerate(original_aig_files)]

    utils.change_directory(repo_path)
    deployments_csvs = []
    for i in range(0, len(DEPLOYMENTS)):
        with open(f"{repo_path}/work_dirs/{SRC_WORK_DIR}/results/deployment_{i}.csv") as f:
            r = csv.DictReader(f)
            deployments_csvs.append((r.fieldnames, [row for row in r]))

    work_dir = make_work_dir(repo_path=repo_path)
    print(f"workdir: {work_dir}")

    skip = 0

    for dataset in ["17", "19", "24"]:
        dataset = f"hwmcc{dataset}"
        result_path = f"{repo_path}/work_dirs/{work_dir}/{dataset}"
        print(f"making {dataset} in {result_path}")
        utils.run_cmd(f"mkdir {result_path}")
        utils.run_cmd(f"mkdir {result_path}/results")
        utils.run_cmd(f"mkdir {result_path}/results/graphs")

        aig_files = get_aig_files_in_test(suite=dataset, tests=["aig"], repo_path=repo_path)
        aig_files.sort()

        total_matches = []
        for f in aig_files:
            n = f
            n = n.split("hwmcc19/aig")[-1]
            n = n.split(dataset)[-1]
            n = n.split("2024")[-1]
            n = n.split("2020")[-1]
            n = n.split("2019")[-1]

            n = n[1:]
            matches = [x for x in i_original_aig_files if n in x[1]]
            if len(matches) != 1:
                print(f"SKIPPING {f}")
                skip += 1
                continue
            assert len(matches) == 1
            match = matches[0]
            total_matches.append(match)

        if dataset == "17":
            assert len(total_matches) == 300
        elif dataset == "19":
            assert len(total_matches) == 317
        elif dataset == "20":
            assert len(total_matches) == 324
        elif dataset == "24":
            assert len(total_matches) == 318

        for d, (fieldnames, csv_reader) in enumerate(deployments_csvs):
            with open(f"{result_path}/results/deployment_{d}.csv", 'w') as f:
                writer = csv.DictWriter(f, fieldnames=fieldnames)

                writer.writeheader()
                for row in csv_reader:
                    j = row["model"].split("_")[0]
                    if j in [f"{i}" for i, _ in total_matches]:
                        # print(row, )
                        writer.writerow(row)
        make_plots(work_dir=result_path, time_limit=TIME_LIMIT, deployment_names=DEPLOYMENTS)

    print("skip", skip)


if __name__ == "__main__":
    raise RuntimeError(
        "This script is not well tested, comment this line if you want to use it anyways")
    main()
