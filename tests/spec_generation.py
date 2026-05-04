from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parents[1]
SPEC_SOURCE_DIR = PROJECT_ROOT / "docs" / "specs"
GENERATED_FEATURE_DIR = PROJECT_ROOT / "tests" / "features" / "generated"
GHERKIN_FENCE_RE = re.compile(r"```(?:gherkin|feature)\s*\n(?P<content>.*?)\n```", re.DOTALL)


@dataclass(frozen=True)
class GeneratedFeature:
    path: Path
    content: str


def generated_features(
    source_dir: Path = SPEC_SOURCE_DIR,
    output_dir: Path = GENERATED_FEATURE_DIR,
) -> list[GeneratedFeature]:
    features: list[GeneratedFeature] = []
    for source_path in sorted(source_dir.glob("*.md")):
        matches = list(GHERKIN_FENCE_RE.finditer(source_path.read_text(encoding="utf-8")))
        for index, match in enumerate(matches, start=1):
            name = source_path.stem if len(matches) == 1 else f"{source_path.stem}-{index}"
            content = _feature_content(source_path, match.group("content"))
            features.append(GeneratedFeature(path=output_dir / f"{name}.feature", content=content))
    return features


def write_generated_features(
    source_dir: Path = SPEC_SOURCE_DIR,
    output_dir: Path = GENERATED_FEATURE_DIR,
) -> list[GeneratedFeature]:
    features = generated_features(source_dir=source_dir, output_dir=output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    for stale_path in output_dir.glob("*.feature"):
        if stale_path not in {feature.path for feature in features}:
            stale_path.unlink()
    for feature in features:
        feature.path.write_text(feature.content, encoding="utf-8")
    return features


def _feature_content(source_path: Path, gherkin: str) -> str:
    relative_source = source_path.relative_to(PROJECT_ROOT)
    return f"# Generated from {relative_source}\n# Run: python scripts/generate_features.py\n\n{gherkin.rstrip()}\n"
