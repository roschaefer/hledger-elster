# Generated from docs/specs/configuration.md
# Run: python scripts/generate_features.py

Feature: Home-Office-Pauschale configuration

  Scenario: A custom config changes the Home-Office-Pauschale
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_item:Betriebseinnahmen

      2024-01-10 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR
      """
    And a file named "elster.toml" with content:
      """
      [euer.home_office_pauschale]
      enabled = true
      default_days = "max"

      [euer.home_office_pauschale.days]
      2024 = 10
      """
    When I run "hledger elster -f journal.journal --config elster.toml -o export"
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
      Home-Office-Pauschale,60.00
      An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer,0.00
      Summe Betriebskosten,0.00
      Summe Betriebsausgaben,60.00
      ,
      # Ermittlung des Gewinns,
      Steuerpflichtiger Gewinn/Verlust,59.00
      ,
      # Zusätzliche Angaben bei Einzelunternehmen,
      Entnahmen,0.00
      Einlagen,0.00
      """
