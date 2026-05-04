from __future__ import annotations

import shutil
import tempfile
from pathlib import Path


def before_scenario(context, scenario) -> None:
    context.project_root = Path(__file__).resolve().parents[2]
    context.work_dir = Path(tempfile.mkdtemp(prefix="hledger-elster-behave-"))
    context.last_result = None


def after_scenario(context, scenario) -> None:
    shutil.rmtree(context.work_dir)
