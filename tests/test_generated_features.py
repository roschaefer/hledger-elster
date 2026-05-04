from __future__ import annotations

from tests.spec_generation import GENERATED_FEATURE_DIR, generated_features


def test_generated_features_match_markdown_sources() -> None:
    expected = generated_features()
    expected_by_path = {feature.path: feature.content for feature in expected}
    actual_paths = set(GENERATED_FEATURE_DIR.glob("*.feature"))

    assert actual_paths == set(expected_by_path), (
        "Generated behave features are out of date. "
        "Run `python scripts/generate_features.py` and commit the result."
    )
    for path, content in expected_by_path.items():
        assert path.read_text(encoding="utf-8") == content, (
            f"{path.relative_to(GENERATED_FEATURE_DIR.parents[1])} is out of date. "
            "Run `python scripts/generate_features.py`."
        )
