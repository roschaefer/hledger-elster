# Executable Specifications

These Markdown files are the source of truth for end-to-end acceptance tests.
Each fenced `gherkin` block is compiled into a Cucumber feature at build time by
`build.rs` (see `tests/cucumber.rs` for step definitions) -- nothing is
committed to git, so documentation and executed features cannot drift.

The examples document the public journal tagging contract. Account directives
carry ELSTER metadata in hledger comments, and postings inherit the metadata from
their account hierarchy. The generated CSV files are intentionally asserted as
plain text because they are the auditable artifacts a user can inspect and import.

Run them with:

```sh
cargo test --test cucumber
```

Specifications:

- [Configuration](./00-configuration.md)
- [Export hygiene](./01-export-hygiene.md)
- [Business expenses and income](./02-form-section-item-tags.md)
- [VAT payments and settlements](./03-vat.md)
- [VAT reverse charge](./04-vat-reverse-charge.md)
- [Business vs. private accounts](./05-business-accounts.md)
- [Health care and insurance](./06-health-care.md)
- [GWG and AfA](./07-afa.md)
- [Donations](./08-donations.md)
- [Command-line help](./09-cli-help.md)
