# Verification

Verification compares current generated tax results with values from previous manual tax
calculations and prior ELSTER submissions.

## Purpose

- catch regressions in tax logic
- preserve known-good values
- surface previously unnoticed mistakes in prior manual calculations
- track open questions where the cause of a deviation is not yet understood

## Approach: pytest

Verification is implemented as a pytest test suite under `tests/`.

Tests are grouped by ELSTER form and section, mirroring the structure of the submitted
forms. This makes it straightforward to locate the relevant test when reviewing a
specific line in a prior submission.

Running `pytest -v tests/` produces:

- all failing tests at the bottom — these are either software bugs or unresolved open questions
- a `CONFIRMED DIFFERENCES` section listing accepted deviations from prior filings
- a `NEEDS REVIEW` section listing deviations whose cause is still under investigation

## The four verification states

### expected

The computed value matches the expected value exactly.

```python
def test_hetzner(self, dataset):
    assert euer.deductible_net(dataset, "expenses:business:hosting:hetzner") == Decimal("197.96")
```

A mismatch is a software bug. The test fails immediately.

### approximate

The computed value matches within a known tolerance, typically due to rounding across
multiple postings.

```python
def test_uberspace(self, dataset):
    assert euer.deductible_net(dataset, "expenses:business:hosting:uberspace") == pytest.approx(75.63, abs=Decimal("0.01"))
```

A mismatch beyond the tolerance is a software bug. The test fails immediately.

### confirmed

The computed value differs from a previously filed value, but the difference has been
investigated and accepted. The prior filing is understood to have been incorrect.

The test is marked with `@pytest.mark.confirmed`. Both `previously=` and `reason=` are
required. `previously` records the value from the prior manual filing so it can be
cross-referenced without looking at git history. The assertion pins the current accepted
value.

```python
@pytest.mark.confirmed(previously=Decimal("4.94"), reason="One invoice from Privatkonto was not captured")
def test_domains(self, dataset):
    assert euer.deductible_net(dataset, "expenses:business:hosting:domains") == Decimal("9.88")
```

The test passes and appears in the `CONFIRMED DIFFERENCES` summary. If the computed
value later drifts from `9.88`, the test fails as a software regression.

### under_review

The computed value differs from a previously filed value. The cause is not yet clear —
it could be a software issue or an external one (for example, an insurer charging more
than the policy states).

The test is marked with `@pytest.mark.needs_review`. Both `previously=` and `reason=`
are required. The assertion pins the currently observed deviation value.

```python
@pytest.mark.needs_review(previously=Decimal("343.50"), reason="Insurance sent 398.20 but policy states 343.50 — waiting for clarification")
def test_hansemerkur(self, dataset):
    assert est.deductible_amount(dataset, "expenses:insurance:travel:hansemerkur") == Decimal("398.20")
```

The test does not fail the build. It appears in the `NEEDS REVIEW` summary. If the
computed value later drifts from the written-down deviation value (`398.20`), the test
fails hard — a further unexplained change on top of an already-open question is always
a blocking failure.

Once the review is resolved, the test either moves to `confirmed` or is fixed by
correcting the software.

## Drift protection

For both `confirmed` and `under_review`, the assertion always pins the test to a specific
value. Any subsequent drift in the computed output will cause a hard failure, regardless
of the review state. This prevents silent compounding of deviations.

## Agent policy

Agents must not edit `verification.yml` files or test expected values without explicit
approval for the specific change in the current conversation.

If a generated value suggests a test update, the agent must show the proposed diff and
ask before applying it.
