#!/usr/bin/env python

"""
Plot the benchmark results.
"""

import os
import re
from dataclasses import dataclass
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
    duration_ms: int


def to_latex_table(results: List[Results]) -> str:
    """Convert list of results to a latex table."""
    lines = [f"{r.states} & {r.unique} & {r.depth} & {r.duration_ms}" for r in results]
    return "\n".join(lines)

def is_float(s:str) -> bool:
    try:
        float(s)
        return True
    except ValueError:
        return False

def main():
    results_dir = "results"

    results = []
    for resdir in os.listdir(results_dir):
        with open(os.path.join(results_dir, resdir, "out"), "r", encoding="utf-8") as resfile:
            for line in resfile.readlines():
                line = line.strip()
                if not line.startswith("Done"):
                    continue
                parts = [re.sub("[,mns]+", "", x) for x in re.split(r"[ =]+", line) if x]
                nums = [float(x) for x in parts if is_float(x)]
                assert len(nums) == 4
                states = nums[0]
                unique = nums[1]
                depth = nums[2]
                duration_ms = nums[3]
                results.append((resdir, states, unique, depth, duration_ms))

    df = pd.DataFrame(results, columns = ["run_cmd", "states", "unique", "depth", "duration_ms"])
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

    a = sns.relplot(df, x="depth", y="duration_ms", hue="run_cmd", kind="line", marker='o')
    a.set(xlabel="Depth")
    a.set(ylabel="Duration (ms)")
    plt.show()

if __name__ == "__main__":
    main()
