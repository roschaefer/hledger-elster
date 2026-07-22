# Git Commit Evidence

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

```gherkin
Feature: Git commit evidence

  Background:
    Given a git repository
    And a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen
      """
    And a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """
    And I commit all files

  Scenario: A clean repository can export copyable commit evidence
    When I run "hledger-elster export-commit-evidence --output ../commit-evidence.pdf"
    Then the file outside the repository "commit-evidence.pdf" should exist
    And the PDF file outside the repository "commit-evidence.pdf" should contain the current git commit hash
    And the git working tree should be clean

  Scenario: Commit evidence cannot be written into the repository
    When I run "hledger-elster export-commit-evidence --output commit-evidence.pdf" and it fails
    Then stderr should contain:
      """
      commit evidence output must be outside the Git repository
      """
    And the file "commit-evidence.pdf" should not exist
    And the git working tree should be clean

  Scenario: A dirty repository cannot export commit evidence
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat:contains_vat, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-10 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    When I run "hledger-elster export-commit-evidence --output ../commit-evidence.pdf" and it fails
    Then stderr should contain:
      """
      working tree has uncommitted changes
      """
    And the file outside the repository "commit-evidence.pdf" should not exist
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
