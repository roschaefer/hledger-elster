# Executable Specifications

These Markdown documents are the source of truth for end-to-end acceptance
tests. Each chapter has its own directory with a `README.md`, so GitHub renders
it when the directory is opened. The `scrut` blocks are executed by
[Scrut](https://github.com/facebookincubator/scrut), so documentation and
tested behavior cannot drift.

Every scenario shows its inputs next to the output they produce. Documents
source [`readme-setup.sh`](readme-setup.sh) in a hidden Scrut block, which
provides these helpers:

- `new_scenario` starts a scenario in an empty directory;
- `build_scenario FUNCTION` starts a scenario from the state a chapter's
  `setup.bash` builds (see below);
- `write_file PATH` writes a journal, config, or other file from a heredoc;
- `new_git_repository` initializes the scenario directory as a Git repository;
- `xlsx_sheet WORKBOOK SHEET` prints a workbook tab as `;`-separated CSV;
- `strip_section_markers CSV` drops the `"# "` section-header prefix that only
  the CSV export carries.

Scrut gives every document a fresh temporary working directory and checks the
exit code of every block, so a failing setup step fails its scenario.

When several scenarios start from the same state, their chapter keeps that
state in a `setup.bash` defining one function named after the folder
(`scenario_commit_evidence` in `commit-evidence/`). The chapter's `README.md`
builds it under "Shared setup", prints the files it wrote so the inputs stay
visible next to the output, runs the first scenario, and links to one document
per further scenario in the same folder. Each of those calls `build_scenario`
first.

The examples document the public journal tagging contract. Account directives
carry ELSTER metadata in hledger comments, and postings inherit the metadata from
their account hierarchy. The generated CSV files are intentionally asserted as
plain text because they are the auditable artifacts a user can inspect and import.

[`README.md`](../../README.md)'s `elster_*` tag tables are the authoritative
reference for that contract — the source of truth a journal author reads is
the table, not the source code. Whenever a tag is added, removed, or its
semantics change, README's tables and the specs here must be updated in the
same change; a test (`enrich::tests::readme_tag_tables_match_known_tags`)
fails the build if README drifts from the tags the code actually recognizes.

Run them with:

```sh
just test-scrut
```

Specifications:

- [Configuration](configuration/)
- [CSV/xlsx equivalence](csv-xlsx-equivalence/)
- [Traceability](traceability/)
- [Export hygiene](export-hygiene/)
- [Business expenses and income](form-section-item-tags/)
- [VAT payments and settlements](vat/)
- [VAT reverse charge](vat-reverse-charge/)
- [Business vs. private accounts](business-accounts/)
- [Health care and insurance](health-care/)
- [GWG and AfA](afa/)
- [Donations](donations/)
- [Git commit evidence](commit-evidence/)
- [Reconciliation library](reconciliation/)
