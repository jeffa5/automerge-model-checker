#!/usr/bin/env python

"""
Plot the benchmark results.
"""

import os
import re
import shutil
from dataclasses import dataclass
from typing import List

import matplotlib.pyplot as plt
import pandas as pd
import seaborn as sns


@dataclass
class Results:
    """Class for storing results."""

    states: int
    unique: int
    depth: int
    duration_ms: int


def to_latex_table(results: List[Results]) -> str:
    """Convert list of results to a latex table."""
    lines = [f"{r.states} & {r.unique} & {r.depth} & {r.duration_ms}" for r in results]
    return "\n".join(lines)


def is_float(s: str) -> bool:
    try:
        float(s)
        return True
    except ValueError:
        return False


def main():
    results_dir = "results"

    results = []
    for resdir in os.listdir(results_dir):
        with open(
            os.path.join(results_dir, resdir, "out"), "r", encoding="utf-8"
        ) as resfile:
            for line in resfile.readlines():
                line = line.strip()
                if not line.startswith("Done"):
                    continue
                parts = [
                    re.sub("[,mns]+", "", x) for x in re.split(r"[ =]+", line) if x
                ]
                nums = [float(x) for x in parts if is_float(x)]
                assert len(nums) == 4
                states = int(nums[0])
                unique = int(nums[1])
                depth = int(nums[2])
                duration_ms = nums[3]
                results.append((resdir, states, unique, depth, duration_ms))

    df = pd.DataFrame(
        results, columns=["run_cmd", "states", "unique", "depth", "duration_ms"]
    )
    print(df)
    print(df.dtypes)

    shutil.rmtree("plots", ignore_errors=True)
    os.makedirs("plots")

    with open("plots/data.csv", "w") as datafile:
        datafile.write(df.to_csv())

    # counter headline results
    for cmd in [
        # automerge
        "amc-automerge_dfs_changes_2_norestarts_in-sync-check_historical-check_error-free-check_string_put_delete_object-type=map_keys=foo,bar",
        "amc-automerge_dfs_changes_2_norestarts_in-sync-check_historical-check_error-free-check_string_insert_delete_object-type=list_indices=0,1",
        # counter
        "amc-counter_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check",
        "amc-counter_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check_counter-type",
        "amc-counter_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check_initial-change",
        "amc-counter_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check_counter-type_initial-change",
        # moves
        "amc-moves_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check",
        # todo
        "amc-todo_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check",
        "amc-todo_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check_random-ids",
        "amc-todo_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check_initial-change",
        "amc-todo_iterative_changes_2_norestarts_in-sync-check_historical-check_error-free-check_initial-change_random-ids",
    ]:
        results = df[df["run_cmd"] == cmd]
        results = results.iloc[-1]
        print(results.tolist())

    a = sns.relplot(df, x="states", y="unique", hue="run_cmd", kind="line", marker="o")
    a.set(xscale="log")
    a.set(yscale="log")
    a.set(xlabel="Total states")
    a.set(ylabel="Unique states")
    plt.savefig("plots/total-vs-unique.pdf")

    a = sns.relplot(df, x="depth", y="states", hue="run_cmd", kind="line", marker="o")
    a.set(yscale="log")
    a.set(xlabel="Depth")
    a.set(ylabel="Total states")
    plt.savefig("plots/depth-vs-total.pdf")

    a = sns.relplot(
        df, x="depth", y="duration_ms", hue="run_cmd", kind="line", marker="o"
    )
    a.set(xlabel="Depth")
    a.set(ylabel="Duration (ms)")
    plt.savefig("plots/depth-vs-duration.pdf")


if __name__ == "__main__":
    main()
