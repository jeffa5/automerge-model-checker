#!/usr/bin/env python

"""
Plot the benchmark results.
"""

import os
import re
from dataclasses import dataclass
from pprint import pprint
import matplotlib.pyplot as plt
from typing import List
import pandas as pd
import seaborn as sns


@dataclass
class Results:
    """Class for storing results."""

    states: int
    unique: int
    depth: int
    duration_s: int


def to_latex_table(results: List[Results]) -> str:
    """Convert list of results to a latex table."""
    lines = [f"{r.states} & {r.unique} & {r.depth} & {r.duration_s}" for r in results]
    return "\n".join(lines)


def main():
    results_dir = "results"

    results = []
    for resdir in os.listdir(results_dir):
        with open(os.path.join(results_dir, resdir), "r", encoding="utf-8") as resfile:
            for line in resfile.readlines():
                line = line.strip()
                if not line.startswith("Done"):
                    continue
                parts = [re.sub("[,ns]+", "", x) for x in re.split(r"[ =]+", line) if x]
                nums = [int(x) for x in parts if x.isdigit()]
                assert len(nums) == 4
                states = nums[0]
                unique = nums[1]
                depth = nums[2]
                duration_s = nums[3]
                results.append((resdir, states, unique, depth, duration_s))

    df = pd.DataFrame(results, columns = ["run_cmd", "states", "unique", "depth", "duration_s"])
    df.set_index("run_cmd", inplace=True)
    print(df)

    a = sns.relplot(df, x="states", y="unique", hue="run_cmd", kind="line", marker='o')
    a.set(xscale="log")
    a.set(yscale="log")
    a.set(xlabel="Total states")
    a.set(ylabel="Unique states")
    plt.show()

    a = sns.relplot(df, x="depth", y="states", hue="run_cmd", kind="line", marker='o')
    a.set(yscale="log")
    a.set(xlabel="Depth")
    a.set(ylabel="Total states")
    plt.show()

    a = sns.relplot(df, x="depth", y="duration_s", hue="run_cmd", kind="line", marker='o')
    a.set(xlabel="Depth")
    a.set(ylabel="Duration (s)")
    plt.show()


if __name__ == "__main__":
    main()
