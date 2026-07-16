# Executable Specifications

These Markdown files are the source of truth for end-to-end acceptance tests.
Each fenced `gherkin` block is compiled into a Cucumber feature at build time by
`build.rs` (see `tests/cucumber.rs` for step definitions) -- nothing is
committed to git, so documentation and executed features cannot drift.

The examples document the public journal tagging contract. Account directives
carry ELSTER metadata in hledger comments, and postings inherit the metadata from
their account hierarchy. The generated CSV files are intentionally asserted as
plain text because they are the auditable artifacts a user can inspect and import.

[`README.md`](../README.md)'s `elster_*` tag tables are the authoritative
reference for that contract — the source of truth a journal author reads is
the table, not the source code. Whenever a tag is added, removed, or its
semantics change, README's tables and the specs here must be updated in the
same change; a test (`enrich::tests::readme_tag_tables_match_known_tags`)
fails the build if README drifts from the tags the code actually recognizes.

Run them with:

```sh
cargo test --test cucumber
```

Specifications:

- [Configuration](./00-configuration.md)
- [CSV/xlsx equivalence](./01-csv-xlsx-equivalence.md)
- [Traceability](./02-traceability.md)
- [Export hygiene](./03-export-hygiene.md)
- [Business expenses and income](./04-form-section-item-tags.md)
- [VAT payments and settlements](./05-vat.md)
- [VAT reverse charge](./06-vat-reverse-charge.md)
- [Business vs. private accounts](./07-business-accounts.md)
- [Health care and insurance](./08-health-care.md)
- [GWG and AfA](./09-afa.md)
- [Donations](./10-donations.md)
