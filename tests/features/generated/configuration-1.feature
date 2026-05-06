# Generated from docs/specs/configuration.md
# Run: python scripts/generate_features.py

Feature: Configuration

  Scenario: Running without a config file uses human-friendly defaults
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-10 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    Then the file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      """
      Kennzahl,2024
      # Betriebseinnahmen,
      Umsatzsteuerpflichtige Betriebseinnahmen,100.00
      Vereinnahmte Umsatzsteuer,19.00
      Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer,0.00
      Summe Betriebseinnahmen,119.00
      ,
      # Betriebsausgaben,
      ,
      Home-Office-Pauschale,1260.00
      An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer,0.00
      Summe Betriebskosten,0.00
      Summe Betriebsausgaben,1260.00
      ,
      # Ermittlung des Gewinns,
      Steuerpflichtiger Gewinn/Verlust,-1141.00
      ,
      # Zusätzliche Angaben bei Einzelunternehmen,
      Entnahmen,0.00
      Einlagen,0.00
      """

  Scenario: The default config can be generated
    When I run "hledger-elster init-config --output elster.toml"
    Then the file "elster.toml" should contain exactly:
      """
      [euer.home_office_pauschale]
      enabled = true
      default_days = "max"
      # Set per-year days when the default does not match your situation.
      # 2020-2022: 5 EUR/day, capped at 600 EUR.
      # 2023+: 6 EUR/day, capped at 1260 EUR.

      [euer.home_office_pauschale.days]
      # 2024 = 210
      """
