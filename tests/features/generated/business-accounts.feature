# Generated from docs/specs/business-accounts.md
# Run: python scripts/generate_features.py

Feature: Business accounts

  Scenario: Business account postings classify income, owner draws, and owner contributions
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_label:Geschäftskonto
      account assets:bank:private   ; elster_account:private, elster_label:Girokonto
      account transfers:clearing
      account income:business       ; elster_form:einnahmenueberschussrechnung, elster_vat_rate:0.19, elster_label:Betriebseinnahmen

      2024-01-15 Client invoice
          income:business       -119.00 EUR
          assets:bank:business   119.00 EUR

      2024-09-01 Owner draw
          liabilities:owner       50.00 EUR
          assets:bank:business   -50.00 EUR

      2024-09-02 Owner contribution
          liabilities:owner      -40.00 EUR
          assets:bank:business    40.00 EUR

      2024-09-03 Internal transfer
          transfers:clearing      75.00 EUR
          assets:bank:business   -75.00 EUR
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
      Entnahmen,125.00
      Einlagen,40.00
      """
