#!/usr/bin/env python

"""
Run benchmarks in different combinations.
"""

import os
import subprocess
from dataclasses import dataclass
from typing import List, Tuple

from loguru import logger

RESULTS_DIR = "results"


def make_results_dir():
    """
    Clear and create the results dir.
    """
    if not os.path.exists(RESULTS_DIR):
        os.makedirs(RESULTS_DIR)


@dataclass
class Config:
    """
    Config for benchmark runs.
    """

    bin_name: str
    search_type: str
    sync_method: str
    servers: int
    restarts: bool
    extra_flags: List[str]
    extra_args: List[Tuple[str, str]]

    def dir(self) -> str:
        """
        Build the dir name for this config.
        """
        restarts = "restarts" if self.restarts else "norestarts"
        directory = f"{self.bin_name}_{self.search_type}_{self.sync_method}_{self.servers}_{restarts}"
        if self.extra_flags:
            directory += "_"
            directory += "_".join(self.extra_flags)
        if self.extra_args:
            directory += "_"
            directory += "_".join([f"{a[0]}={a[1]}" for a in self.extra_args])
        return directory

    def to_args(self) -> str:
        """
        Convert config to args for running.
        """
        args = f"--sync-method={self.sync_method} --servers={self.servers}"
        if self.extra_flags:
            args += " "
            args += " ".join([f"--{n}" for n in self.extra_flags])
        if self.extra_args:
            args += " "
            args += " ".join([f"--{a[0]}={a[1]}" for a in self.extra_args])
        return args


def run(config: Config):
    """
    Run a single config.
    """
    out_dir = os.path.join(RESULTS_DIR, config.dir())
    out_file = os.path.join(out_dir, "out")
    if os.path.exists(out_dir):
        logger.info("Skipping {}", out_dir)
        return
    os.makedirs(out_dir)
    cmd = (
        f"{config.bin_name} check-{config.search_type} {config.to_args()} > {out_file}"
    )
    logger.info("Running command: {}", cmd)
    timeout_s = 60 * 60 * 24  # 1 day
    try:
        subprocess.run(
            cmd,
            shell=True,
            check=True,
            timeout=timeout_s,
        )
    except subprocess.TimeoutExpired:
        logger.warning("Timed out after {}s", timeout_s)


def main():
    """
    Run the benchmarks.
    """
    make_results_dir()

    amc_check_flags = [
        "in-sync-check",
        "historical-check",
        "error-free-check",
        "save-load-check",
    ]

    configs = [
        # amc-counter
        Config(
            bin_name="amc-counter",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=[],
            extra_args=[],
        ),
        Config(
            bin_name="amc-counter",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=["counter-type"],
            extra_args=[],
        ),
        Config(
            bin_name="amc-counter",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=["initial-change"],
            extra_args=[],
        ),
        Config(
            bin_name="amc-counter",
            search_type="dfs",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=["counter-type", "initial-change"],
            extra_args=[],
        ),

        # amc-moves
        Config(
            bin_name="amc-moves",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=[],
            extra_args=[],
        ),

        # amc-todo
        Config(
            bin_name="amc-todo",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=[],
            extra_args=[],
        ),
        Config(
            bin_name="amc-todo",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=["random-ids"],
            extra_args=[],
        ),
        Config(
            bin_name="amc-todo",
            search_type="iterative",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=["initial-change"],
            extra_args=[],
        ),
        Config(
            bin_name="amc-todo",
            search_type="dfs",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=["initial-change", "random-ids"],
            extra_args=[],
        ),

        # amc-automerge
        Config(
            bin_name="amc-automerge",
            search_type="dfs",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=amc_check_flags + ["string", "put", "delete"],
            extra_args=[("object-type", "map"), ("keys", "foo,bar")],
        ),
        Config(
            bin_name="amc-automerge",
            search_type="dfs",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=amc_check_flags + ["string", "insert", "delete"],
            extra_args=[("object-type", "list"), ("indices", "0,1")],
        ),
        Config(
            bin_name="amc-automerge",
            search_type="dfs",
            sync_method="changes",
            servers=2,
            restarts=False,
            extra_flags=amc_check_flags + ["string", "insert", "delete"],
            extra_args=[("object-type", "text"), ("indices", "0,1")],
        ),
    ]
    for config in configs:
        run(config)


if __name__ == "__main__":
    main()
