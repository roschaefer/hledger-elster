# Executable Specifications

These Markdown files are the source of truth for end-to-end acceptance tests.
Each fenced `gherkin` block is extracted into `tests/features/generated/` by
`python scripts/generate_features.py`; the committed generated files are checked
by pytest so documentation and executed features cannot drift.

The examples document the public journal tagging contract. Account directives
carry ELSTER metadata in hledger comments, and postings inherit the metadata from
their account hierarchy. The generated CSV files are intentionally asserted as
plain text because they are the auditable artifacts a user can inspect and import.

Use:

```sh
just generate-features
just acceptance
```

Specifications:

- [Business accounts](./business-accounts.md)
- [Donations](./donations.md)
- [Export hygiene](./export-hygiene.md)
- [Health care and insurance](./health-care.md)
- [VAT payments](./vat.md)
