# Generated from docs/specs/vat.md
# Run: python scripts/generate_features.py

Feature: VAT payments

  Scenario: VAT advance reversals reduce the amount already paid
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:taxes:umsatzsteuer:vorauszahlung  ; elster_role:vat_advance, elster_item:USt-Vorauszahlung
      account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024

      2024-02-15 VAT advance
          expenses:taxes:umsatzsteuer:vorauszahlung:2024  100.00 EUR
          assets:bank:business                           -100.00 EUR

      2024-03-01 VAT advance reversal
          expenses:taxes:umsatzsteuer:vorauszahlung:2024  -40.00 EUR
          assets:bank:business                             40.00 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    Then the file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      """
      Zeitraum,Einnahme (Netto),Vereinnahmte Umsatzsteuer,Abziehbare Vorsteuerbeträge,Vorauszahlungssoll,Bereits Entrichtet
      2024-01,0.00,0.00,0.00,0.00,
      2024-02,0.00,0.00,0.00,0.00,
      2024-03,0.00,0.00,0.00,0.00,
      2024-04,0.00,0.00,0.00,0.00,
      2024-05,0.00,0.00,0.00,0.00,
      2024-06,0.00,0.00,0.00,0.00,
      2024-07,0.00,0.00,0.00,0.00,
      2024-08,0.00,0.00,0.00,0.00,
      2024-09,0.00,0.00,0.00,0.00,
      2024-10,0.00,0.00,0.00,0.00,
      2024-11,0.00,0.00,0.00,0.00,
      2024-12,0.00,0.00,0.00,0.00,
      2024 Q1,0.00,0.00,0.00,0.00,
      2024 Q2,0.00,0.00,0.00,0.00,
      2024 Q3,0.00,0.00,0.00,0.00,
      2024 Q4,0.00,0.00,0.00,0.00,
      2024,0.00,0.00,0.00,0.00,60.00
      """

  Scenario: Booking year and VAT period are evaluated independently
    Given a file named "journal.journal" with content:
      """
      account assets:bank:business  ; elster_account:business, elster_item:Geschäftskonto
      account expenses:taxes:umsatzsteuer  ; elster_role:vat_payment
      account expenses:taxes:umsatzsteuer:vorauszahlung  ; elster_role:vat_advance
      account expenses:taxes:umsatzsteuer:vorauszahlung:2024  ; elster_period:2024
      account expenses:taxes:umsatzsteuer:vorauszahlung:2025  ; elster_period:2025

      2024-02-15 VAT advance 2024
          expenses:taxes:umsatzsteuer:vorauszahlung:2024  100.00 EUR
          assets:bank:business                           -100.00 EUR

      2024-03-01 VAT advance reversal 2024
          expenses:taxes:umsatzsteuer:vorauszahlung:2024  -40.00 EUR
          assets:bank:business                             40.00 EUR

      2024-08-22 VAT final refund for 2023
          expenses:taxes:umsatzsteuer  -25.00 EUR
          assets:bank:business          25.00 EUR

      2025-01-10 Late VAT advance for 2024
          expenses:taxes:umsatzsteuer:vorauszahlung:2024   30.00 EUR
          assets:bank:business                             -30.00 EUR

      2025-02-14 VAT advance 2025
          expenses:taxes:umsatzsteuer:vorauszahlung:2025  200.00 EUR
          assets:bank:business                           -200.00 EUR

      2025-03-02 VAT advance reversal 2025
          expenses:taxes:umsatzsteuer:vorauszahlung:2025  -50.00 EUR
          assets:bank:business                             50.00 EUR

      2025-09-01 VAT final payment 2025
          expenses:taxes:umsatzsteuer   80.00 EUR
          assets:bank:business         -80.00 EUR
      """
    When I run "hledger elster -f journal.journal -o export"
    Then the file "export/2024/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      """
      Kennzahl,2024
      # Betriebseinnahmen,
      Umsatzsteuerpflichtige Betriebseinnahmen,0.00
      Vereinnahmte Umsatzsteuer,0.00
      Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer,25.00
      Summe Betriebseinnahmen,25.00
      ,
      # Betriebsausgaben,
      ,
      Home-Office-Pauschale,1260.00
      An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer,100.00
      Summe Betriebskosten,0.00
      Summe Betriebsausgaben,1360.00
      ,
      # Ermittlung des Gewinns,
      Steuerpflichtiger Gewinn/Verlust,-1360.00
      ,
      # Zusätzliche Angaben bei Einzelunternehmen,
      Entnahmen,0.00
      Einlagen,0.00
      """
    And the file "export/2025/steuererklaerung/einnahmen-ueberschuss-rechnung.csv" should contain exactly:
      """
      Kennzahl,2025
      # Betriebseinnahmen,
      Umsatzsteuerpflichtige Betriebseinnahmen,0.00
      Vereinnahmte Umsatzsteuer,0.00
      Vom Finanzamt erstattete und ggf. verrechnete Umsatzsteuer,0.00
      Summe Betriebseinnahmen,0.00
      ,
      # Betriebsausgaben,
      ,
      Home-Office-Pauschale,1260.00
      An das Finanzamt gezahlte und ggf. verrechnete Umsatzsteuer,310.00
      Summe Betriebskosten,0.00
      Summe Betriebsausgaben,1570.00
      ,
      # Ermittlung des Gewinns,
      Steuerpflichtiger Gewinn/Verlust,-1570.00
      ,
      # Zusätzliche Angaben bei Einzelunternehmen,
      Entnahmen,0.00
      Einlagen,0.00
      """
    And the file "export/2024/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      """
      Zeitraum,Einnahme (Netto),Vereinnahmte Umsatzsteuer,Abziehbare Vorsteuerbeträge,Vorauszahlungssoll,Bereits Entrichtet
      2024-01,0.00,0.00,0.00,0.00,
      2024-02,0.00,0.00,0.00,0.00,
      2024-03,0.00,0.00,0.00,0.00,
      2024-04,0.00,0.00,0.00,0.00,
      2024-05,0.00,0.00,0.00,0.00,
      2024-06,0.00,0.00,0.00,0.00,
      2024-07,0.00,0.00,0.00,0.00,
      2024-08,0.00,0.00,0.00,0.00,
      2024-09,0.00,0.00,0.00,0.00,
      2024-10,0.00,0.00,0.00,0.00,
      2024-11,0.00,0.00,0.00,0.00,
      2024-12,0.00,0.00,0.00,0.00,
      2024 Q1,0.00,0.00,0.00,0.00,
      2024 Q2,0.00,0.00,0.00,0.00,
      2024 Q3,0.00,0.00,0.00,0.00,
      2024 Q4,0.00,0.00,0.00,0.00,
      2024,0.00,0.00,0.00,0.00,90.00
      """
    And the file "export/2025/steuererklaerung/umsatzsteuer.csv" should contain exactly:
      """
      Zeitraum,Einnahme (Netto),Vereinnahmte Umsatzsteuer,Abziehbare Vorsteuerbeträge,Vorauszahlungssoll,Bereits Entrichtet
      2025-01,0.00,0.00,0.00,0.00,
      2025-02,0.00,0.00,0.00,0.00,
      2025-03,0.00,0.00,0.00,0.00,
      2025-04,0.00,0.00,0.00,0.00,
      2025-05,0.00,0.00,0.00,0.00,
      2025-06,0.00,0.00,0.00,0.00,
      2025-07,0.00,0.00,0.00,0.00,
      2025-08,0.00,0.00,0.00,0.00,
      2025-09,0.00,0.00,0.00,0.00,
      2025-10,0.00,0.00,0.00,0.00,
      2025-11,0.00,0.00,0.00,0.00,
      2025-12,0.00,0.00,0.00,0.00,
      2025 Q1,0.00,0.00,0.00,0.00,
      2025 Q2,0.00,0.00,0.00,0.00,
      2025 Q3,0.00,0.00,0.00,0.00,
      2025 Q4,0.00,0.00,0.00,0.00,
      2025,0.00,0.00,0.00,0.00,150.00
      """
