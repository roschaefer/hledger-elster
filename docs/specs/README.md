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

- [Configuration](./configuration.md)
- [Export hygiene](./export-hygiene.md)
- [Business expenses and income](./form-section-item-tags.md)
- [VAT payments and settlements](./vat.md)
- [Business vs. private accounts](./business-accounts.md)
- [Health care and insurance](./health-care.md)
- [GWG and AfA](./afa.md)
- [Donations](./donations.md)
