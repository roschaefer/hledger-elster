# Generated from docs/specs/export-hygiene.md
# Run: python scripts/generate_features.py

Feature: Export hygiene

  Scenario: Stale files in the export directory are reported
    Given a file named "journal.journal" with content:
      """
      account assets:bank:checking  ; elster_account:private, elster_item:Girokonto
      account expenses:private:health-care:kv  ; elster_form:einkommensteuer, elster_deduction:nicht_abzugsfaehig, elster_item:Krankenversicherung, elster_section:Vorsorgeaufwand

      2024-06-01 Health care contribution
          expenses:private:health-care:kv   840.00 EUR
          assets:bank:checking             -840.00 EUR
      """
    And a file named "export/2024/steuererklaerung/stale.csv" with content:
      """
      stale
      """
    And a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = false
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
    Then stderr should contain:
      """
      Warning: untouched files remain in ELSTER export directory. Consider emptying the export directory and running the tool again.
      """
    And stderr should contain:
      """
      stale.csv
      """
