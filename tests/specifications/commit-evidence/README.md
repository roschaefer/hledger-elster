# Git Commit Evidence

<!--
```scrut {fail_fast: true}
$ source "$TESTDIR/../readme-setup.sh"
```
-->

The commit evidence export is an explicit step after the repository state for a
tax declaration has been committed. It writes a PDF that can be uploaded to
ELSTER as a supporting document without becoming part of the committed state it
identifies.

I added this because of an actual tax audit (`Steuerprüfung`). One complaint was
that my LibreOffice spreadsheets were modifiable after the fact. Under the
[GoBD](https://ao.bundesfinanzministerium.de/ao/2023/Anhaenge/BMF-Schreiben-und-gleichlautende-Laendererlasse/Anhang-64/inhalt.html),
that is a problem because it weakens the two goals `Unveränderbarkeit` and
`Nachprüfbarkeit`: the tax authority must be able to verify which records
existed for a filing, and those records must not be silently mutable after
submission.

This subcommand does not make the repository immutable by itself. It produces an
ELSTER-uploadable PDF that records the clean Git commit hash for the repository
state used to prepare the declaration.

Your side of the workflow is:

1. Commit the repository state that was used for the tax declaration.
2. Export the evidence PDF to a path outside the repository.
3. Back up your Git repository, including that commit object and its history, to
   a safe location.
4. Keep the original chain of commits available. Do not rebase, squash, amend, or
   otherwise rewrite the history that contains the committed tax declaration
   state after you have produced and submitted the evidence PDF.
5. Upload the evidence PDF to ELSTER as a `Beleg`.
6. Make sure the `Beleg` is actually included with the filing. In Mein ELSTER,
   uploading a document under `Meine Belege` only stores it in your account; it
   must be linked to the submitted income tax declaration so the tax office can
   access it with that declaration. If you need to send the document separately,
   use ELSTER's `Belegnachreichung` form instead.

If the repository cannot later be produced, or if the original commit history is
rewritten, the commit hash in the PDF is only a string and no longer a useful way
to verify the underlying files.

### A clean repository can export copyable commit evidence

```scrut
$ new_scenario
```

```scrut
$ new_git_repository
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> EOF
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ git add . && git commit --quiet --message "scenario state"
```

```scrut
$ hledger-elster export-commit-evidence --output ../commit-evidence.pdf
```

```scrut
$ test -f ../commit-evidence.pdf
```

```scrut
$ grep --quiet --fixed-strings "$(git rev-parse HEAD)" ../commit-evidence.pdf
```

```scrut
$ git status --porcelain
```

### Commit evidence cannot be written into the repository

```scrut
$ new_scenario
```

```scrut
$ new_git_repository
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> EOF
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ git add . && git commit --quiet --message "scenario state"
```

```scrut
$ hledger-elster export-commit-evidence --output commit-evidence.pdf
Error: commit evidence output must be outside the Git repository: */scenario/work/commit-evidence.pdf (glob)
[1]
```

```scrut
$ test ! -e commit-evidence.pdf
```

```scrut
$ git status --porcelain
```

### A dirty repository cannot export commit evidence

```scrut
$ new_scenario
```

```scrut
$ new_git_repository
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> EOF
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ git add . && git commit --quiet --message "scenario state"
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> 
> 2024-01-10 Client invoice
>     income:business       -119.00 EUR
>     assets:bank:business   119.00 EUR
> EOF
```

```scrut
$ hledger-elster export-commit-evidence --output ../commit-evidence.pdf
Error: working tree has uncommitted changes; commit or stash them before exporting commit evidence
[1]
```

```scrut
$ test ! -e ../commit-evidence.pdf
```

### Existing commit evidence is not silently overwritten

```scrut
$ new_scenario
```

```scrut
$ new_git_repository
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> EOF
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ git add . && git commit --quiet --message "scenario state"
```

```scrut
$ write_file ../commit-evidence.pdf <<'EOF'
> previous evidence
> EOF
```

```scrut
$ hledger-elster export-commit-evidence --output ../commit-evidence.pdf
Error: commit evidence file already exists: */scenario/work/../commit-evidence.pdf (use --force to overwrite) (glob)
[1]
```

### --force overwrites existing commit evidence

```scrut
$ new_scenario
```

```scrut
$ new_git_repository
```

```scrut
$ write_file journal.journal <<'EOF'
> account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
> account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
> EOF
```

```scrut
$ write_file elster.toml <<'EOF'
> [euer.home_office_pauschale]
> enabled = false
> EOF
```

```scrut
$ git add . && git commit --quiet --message "scenario state"
```

```scrut
$ write_file ../commit-evidence.pdf <<'EOF'
> previous evidence
> EOF
```

```scrut
$ hledger-elster export-commit-evidence --output ../commit-evidence.pdf --force
```

```scrut
$ grep --quiet --fixed-strings "$(git rev-parse HEAD)" ../commit-evidence.pdf
```

## Third-Party Timestamp Evidence

For users who also want an external timestamp signal that accounting records
were created or updated within the GoBD ten-day expectation, GitHub can add
useful third-party evidence. One practical workflow is to sign every Git commit,
encrypt private journal data with `git-crypt`, and push the repository to
GitHub regularly. The `git-crypt` encryption keeps GitHub from reading your
private journal files while still allowing GitHub to store and verify the commit
objects. GitHub stores a verification date for signed commits, so the hosted
repository can help corroborate when a signed commit, and therefore the
transactions contained in that commit, were known to an external service.

This only works if you preserve the original commit hashes. Do not rebase,
squash, amend, or otherwise rewrite the relevant history after pushing it. A
history rewrite creates different commit hashes; if you push those rewritten
commits again, GitHub verifies different commits at a different date, and the
original verification date no longer supports the evidence PDF's commit hash.
This is still not a substitute for preserving the repository and keys, but it
strengthens the timestamp story beyond a local commit hash.
