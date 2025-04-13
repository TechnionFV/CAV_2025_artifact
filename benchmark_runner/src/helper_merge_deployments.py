import csv
import sys

from utils import get_path_to_repo


def merge_deployments(i: int, m: list[str]):
    seen_in_previous_sets = set()
    result_rows = []
    total_rows = 0
    field_names = None
    rp = get_path_to_repo()
    for w in m:
        helper_seen = set()
        with open(f'{rp}/work_dirs/{w}/results/deployment_{i}.csv', newline='') as csvfile:
            reader = csv.DictReader(csvfile)
            assert field_names is None or field_names == reader.fieldnames
            field_names = reader.fieldnames

            for row in reader:
                total_rows += 1
                model_name = row["model"]
                r = model_name.split("_", 1)
                name = r[1]
                if name in seen_in_previous_sets:
                    continue
                helper_seen.add(name)
                result_rows.append(row)
        seen_in_previous_sets.update(helper_seen)
    print(f"filtered rows = {len(result_rows)}, original rows = {total_rows}")

    with open(f'{rp}/work_dirs/merge_result/results/deployment_{i}.csv', 'w',
              newline='') as csvfile:
        writer = csv.DictWriter(csvfile, fieldnames=field_names)
        writer.writeheader()
        for r in result_rows:
            writer.writerow(r)

    return None


def process_arguments() -> list[str]:
    if len(sys.argv) < 2:
        raise RuntimeError("Usage: python script.py 2025_03_13_07_12_33 2025_03_13_07_12_36 ...")
    arguments = sys.argv[1:]
    print("Merging Work Dirs:", arguments)
    return arguments


def main():
    m = process_arguments()
    cases = 2
    for i in range(0, cases):
        merge_deployments(i=i, m=m)


if __name__ == "__main__":
    main()
