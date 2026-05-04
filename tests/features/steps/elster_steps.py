from __future__ import annotations

import os
import shlex
import subprocess
from pathlib import Path

from behave import given, then, use_step_matcher, when


use_step_matcher("re")


@given(r'a file named "(?P<path>[^"]+)" with content:')
def write_file(context, path: str) -> None:
    target = _resolve_work_path(context, path)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(_doc_string(context), encoding="utf-8")


@when(r'I run "(?P<command>[^"]+)"')
def run_command(context, command: str) -> None:
    args = shlex.split(command)
    if args[:2] == ["hledger", "elster"]:
        args = [str(context.project_root / "hledger-elster"), *args[2:]]
    elif args[:1] == ["hledger-elster"]:
        args = [str(context.project_root / "hledger-elster"), *args[1:]]
    else:
        raise AssertionError(f"Unsupported command: {command}")

    env = os.environ.copy()
    env["PYTHONPATH"] = str(context.project_root / "src")
    context.last_result = subprocess.run(
        args,
        cwd=context.work_dir,
        env=env,
        text=True,
        capture_output=True,
    )
    assert context.last_result.returncode == 0, context.last_result.stdout + context.last_result.stderr


@then(r'the file "(?P<path>[^"]+)" should contain exactly:')
def file_should_contain_exactly(context, path: str) -> None:
    actual_path = _resolve_work_path(context, path)
    assert actual_path.exists(), f"Expected output file was not created: {path}"
    assert actual_path.read_text(encoding="utf-8") == _doc_string(context)


def _resolve_work_path(context, path: str) -> Path:
    relative_path = Path(path)
    if relative_path.is_absolute() or ".." in relative_path.parts:
        raise AssertionError(f"Unsafe scenario path: {path}")
    return context.work_dir / relative_path


def _doc_string(context) -> str:
    return context.text + "\n"
