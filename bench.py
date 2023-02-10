#!/usr/bin/env python

"""
Run benchmarks in different combinations.
"""

import os
import shutil
import subprocess
from dataclasses import dataclass
from typing import List, Tuple

RESULTS_DIR = "results"


def make_results_dir():
    """
    Clear and create the results dir.
    """
    shutil.rmtree(RESULTS_DIR)
    os.makedirs(RESULTS_DIR)


@dataclass
class Config:
    """
    Config for benchmark runs.
    """

    bin_name: str
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
        directory = f"{self.bin_name}_{self.sync_method}_{self.servers}_{restarts}"
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
    os.makedirs(out_dir)
    cmd = f"{config.bin_name} check-iterative {config.to_args()} | tee {out_file}"
    print("Running command:", cmd)
    subprocess.run(
        cmd,
        shell=True,
        check=True,
    )


def main():
    """
    Run the benchmarks.
    """
    make_results_dir()

    sync_methods = ["changes", "messages", "save-load"]
    for servers in [2, 3]:
        for sync_method in sync_methods:
            for restarts in [True, False]:
                # amc-counter
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=[],
                        extra_args=[],
                    )
                )
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["counter-type"],
                        extra_args=[],
                    )
                )
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["initial-change"],
                        extra_args=[],
                    )
                )
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["counter-type", "initial-change"],
                        extra_args=[],
                    )
                )

                # amc-moves
                run(
                    Config(
                        bin_name="amc-moves",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=[],
                        extra_args=[],
                    )
                )

                # amc-todo
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=[],
                        extra_args=[],
                    )
                )
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["random-ids"],
                        extra_args=[],
                    )
                )
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["initial-change"],
                        extra_args=[],
                    )
                )
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["initial-change", "random-ids"],
                        extra_args=[],
                    )
                )

                # amc-automerge
                for object_type in ["map", "list", "text"]:
                    for datatype in [
                        "bytes",
                        "string",
                        "int",
                        "uint",
                        "timestamp",
                        "boolean",
                        "null",
                    ]:
                        run(
                            Config(
                                bin_name="amc-automerge",
                                sync_method=sync_method,
                                servers=servers,
                                restarts=restarts,
                                extra_flags=[datatype],
                                extra_args=[("object-type", object_type)],
                            )
                        )


if __name__ == "__main__":
    main()
