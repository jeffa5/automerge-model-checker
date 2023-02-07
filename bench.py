#!/usr/bin/env python

"""
Run benchmarks in different combinations.
"""

import os
import shutil
import subprocess
from dataclasses import dataclass
from typing import List

RESULTS_DIR = "results"


def build():
    """
    Build the binaries.
    """
    subprocess.run(["cargo", "build", "--release"], check=True)


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

    def dir(self) -> str:
        """
        Build the dir name for this config.
        """
        restarts = "restarts" if self.restarts else "norestarts"
        directory = f"{self.bin_name}_{self.sync_method}_{self.servers}_{restarts}"
        if self.extra_flags:
            directory += "_"
            directory += "_".join(self.extra_flags)
        return directory

    def to_args(self) -> str:
        """
        Convert config to args for running.
        """
        args = f"--sync-method={self.sync_method} --servers={self.servers}"
        if self.extra_flags:
            args += " "
            args += " ".join([f"--{n}" for n in self.extra_flags])
        return args


def run(config: Config):
    """
    Run a single config.
    """
    out_dir = os.path.join(RESULTS_DIR, config.dir())
    out_file = os.path.join(out_dir, "out")
    os.makedirs(out_dir)
    cmd = (
        f"cargo run -q --release --bin {config.bin_name} -- "
        + f"check-iterative {config.to_args()} | tee {out_file}"
    )
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
    build()
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
                    )
                )
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["counter-type"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["initial-change"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-counter",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["counter-type", "initial-change"],
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
                    )
                )
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["random-ids"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["initial-change"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-todo",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["initial-change", "random-ids"],
                    )
                )

                # amc-automerge
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["bytes"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["string"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["int"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["uint"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["timestamp"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["boolean"],
                    )
                )
                run(
                    Config(
                        bin_name="amc-automerge",
                        sync_method=sync_method,
                        servers=servers,
                        restarts=restarts,
                        extra_flags=["null"],
                    )
                )


if __name__ == "__main__":
    main()
