# Plan zur Steuerautomatisierung

## Ziel

Es soll ein wiederholbarer Ablauf entstehen, der:

1. Kontoauszüge aus CSV importiert,
2. Transaktionen normalisiert und klassifiziert,
3. steuerrelevante Summen berechnet,
4. ein nachvollziehbares Prüfungspaket mit in Git sichtbaren Änderungen erzeugt und
5. eine Tabelle für die Prüfung durch das Finanzamt exportiert.

Das System soll Nachvollziehbarkeit vor Komplexität priorisieren: Jede gemeldete Zahl muss sich auf die ursprünglichen CSV-Zeilen zurückführen lassen.

## Kernanforderungen

- Ein oder mehrere CSV-Formate von Banken oder Brokern importieren
- Ursprüngliche Dateien unverändert erhalten
- Felder in ein internes Standardschema normalisieren
- Kategorisierung und Steuerlogik unterstützen
- Einen vollständigen Prüfpfad von der Quellzeile bis zur Endausgabe erhalten
- Wichtige Zwischendaten in Textformaten halten, die lesbare Git-Diffs erzeugen
- Zwischenausgaben für manuelle Prüfung erzeugen
- Eine finale, gut lesbare Spreadsheet-Ausgabe in `.xlsx` erzeugen
- Alle in Ausgabedateien sichtbaren Bezeichnungen auf Deutsch halten

## Sprache der Ausgaben

Alle Bezeichnungen, die in exportierten Dateien sichtbar sind, müssen auf Deutsch sein, da diese Dateien potenziell dem Steuerprüfer zugänglich gemacht werden.

Das betrifft insbesondere:

- Spaltenüberschriften in CSV-Dateien
- Blattnamen in `.xlsx`-Dateien
- Dateinamen von Exporten
- sichtbare Abschnitts- oder Summenbezeichnungen in Exportdateien

Interne Implementierungsdetails dürfen technisch Englisch verwenden, aber jede nutzersichtbare Ausgabe muss Deutsch sein.

Beispiel:

- statt `standard_sums.csv` soll ein deutscher Dateiname wie `standardsummen.csv` verwendet werden

## Formatierung finanzieller Werte

Alle Felder in Exportdateien, die finanzielle Zahlenwerte enthalten, sollen als Währungswerte in `EUR` formatiert werden.

Das betrifft insbesondere:

- `.xlsx`-Dateien

Für die Umsetzung gilt:

- sichtbare Zahlenwerte in Tabellenkalkulationsdateien sollen als Währung formatiert sein, nicht nur als einfache Zahl
- das Währungsformat soll `EUR` ausdrücken
- diese Anforderung gilt für alle Summenspalten und später auch für weitere finanzielle Spalten

## Empfohlene Architektur

Verwendet wird eine kleine lokale Datenpipeline mit zwei klar getrennten Bereichen:

1. **Software unter `src/`**
   - Python-Code
   - automatisierte Tests
   - keine fachlichen Eingabedaten

2. **Falldaten unter `data/`**
   - originale CSV-Dateien
   - steuerliche Regeldateien für einen konkreten Fall oder ein konkretes Jahr
   - erzeugte Prüf- und Exportdateien

Die Trennung ist bewusst so gewählt, dass:

- `git add src` nur Softwareänderungen umfasst
- Eingabedaten und Regelwerke als fachliche Daten behandelt werden
- verschiedene Steuerjahre mit eigenen Regeln und Eingabedateien nebeneinander existieren können

## Beobachtetes Eingabeformat

Auf Basis des Beispiel-Exports der Bank hat das aktuelle CSV-Format folgende Eigenschaften:

- Trennzeichen: `;`
- Text-Quoting: doppelte Anführungszeichen
- Datumsformat: `YYYY-MM-DD`
- Beträge: deutsches Dezimalformat, zum Beispiel `-12,88`
- Sprache: deutsche Spaltennamen und deutsche Transaktionsbezeichnungen

Beobachtete Spalten:

- `Buchungsdatum`
- `Wertstellungsdatum`
- `Transaktionstyp`
- `Empfänger`
- `Betrag`
- `IBAN`
- `Verwendungszweck`
- `end_to_end_id`
- `Buchungsstatus`
- `Kategorie`
- `Persönliche Notiz`

Beobachtete Eigenschaften:

- Einige Felder können leer sein, insbesondere `IBAN`, `Verwendungszweck`, `end_to_end_id` und `Persönliche Notiz`
- Negative Beträge scheinen ausgehende Zahlungen darzustellen
- `Kategorie` enthält bereits nützliche Labels wie `Privat`, `Umsatzsteuer 19%` und `Umsatzsteuer-Vorauszahlung`
- `Buchungsstatus` sollte verwendet werden, um künftig nicht gebuchte oder vorläufige Buchungen auszufiltern

## Gewählter Ansatz

Es wird ein einzelner Ablauf verwendet, der auf versionierten CSV-Dateien basiert:

- Python für Import und Berechnungen
- Versionierte CSV-Dateien für Rohdaten und Prüfdaten
- Spreadsheet-Export nach `.xlsx`

Dieser Ansatz ist gewählt, weil:

- Git-Diffs für die relevanten Daten lesbar bleiben
- der Ablauf prüfbar bleibt, ohne Binärdatenbanken speziell inspizieren zu müssen
- `.xlsx` das Standardformat für Microsoft-Office-Tabellen ist und direkt zum Prüfungsablauf passt

## Vorgeschlagene Projektstruktur

```text
taxes/
  PLAN.md
  src/
    calculate/
      generate_tax_review.py
      tax_review/
        adjustments.py
        cli.py
        details.py
        io.py
        models.py
        rules.py
        summaries.py
        utils.py
        writers.py
        tests/
  data/
    2025/
        bankexporte/
          transactions_2025-01-01_2025-12-31.csv
          20-03-2026_Umsatzliste_Girokonto_DE69120300001032981670.csv
        normalisiert/
          transactions_2025-01-01_2025-12-31.normalized.csv
          20-03-2026_Umsatzliste_Girokonto_DE69120300001032981670.normalized.csv
        outputs/
          Steuerauswertung/
            geschaeftskonto.csv
            privatkonto.csv
            einnahmenueberschussrechnung.csv
            umsatzsteuer.csv
            einkommensteuer.csv
            details_geschaeftskonto/
            details_privatkonto/
          Steuerauswertung.xlsx
```

## Bedeutung der früher vorgeschlagenen Verzeichnisse

Die frühere Struktur hatte folgende Idee:

- `src/ingest/`: Import der Rohdateien
- `src/normalize/`: Überführung in ein Standardschema
- `src/classify/`: steuerliche und fachliche Zuordnung
- `src/calculate/`: Summenbildung
- `src/export/`: Ausgabeformate
- `data/normalized/`: persistierte Zwischenschicht
- `data/build/`: temporäre Build-Artefakte
- `data/exports/`: finale Exporte getrennt von Prüfdateien

Das war als klassische ETL-Pipeline gedacht. Inzwischen ist das für dieses Projekt zu fein aufgeteilt und wird praktisch nicht genutzt:

- die leeren Stufenverzeichnisse unter `src/` sind obsolet
- `data/normalized/`, `data/build/` und `data/exports/` werden aktuell nicht verwendet
- the tax semantics are now stored directly as `tax_` tags in `ledger/accounts.journal`

Deshalb wird das Projekt auf eine kompaktere Struktur umgestellt:

- Software und Tests zusammen unter `src/`
- Rohdaten je Jahr unter `data/<jahr>/bankexporte`
- normalisierte CSVs je Jahr unter `data/<jahr>/normalisiert`
- erzeugte Ergebnisse je Jahr unter `data/<jahr>/outputs`

## Kanonisches Transaktionsschema

Jede Quellzeile wird in einen standardisierten Transaktionsdatensatz mit stabiler Spaltenreihenfolge überführt:

- `booking_date`
- `value_date`
- `transaction_type`
- `counterparty`
- `amount`
- `currency`
- `iban`
- `reference`
- `end_to_end_id`
- `booking_status`
- `source_category`
- `tax_category`
- `tax_subcategory`
- `personal_note`
- `direction`
- `account_id`
- `tax_year`
- `source_file`
- `source_row`

Hinweise:

- `amount` wird in ein maschinenfreundliches Dezimalformat wie `-12.88` normalisiert
- `currency` kann für dieses Bankformat standardmäßig auf `EUR` gesetzt werden, solange keine Fremdwährungen auftauchen
- `direction` wird aus dem Vorzeichen von `amount` abgeleitet, zum Beispiel `outgoing` bei negativen Werten
- `source_category` bewahrt den ursprünglichen Wert aus `Kategorie`, auch wenn die spätere Steuerklassifikation davon abweicht
- `tax_category` enthält die steuerliche Hauptklassifikation im eigenen System
- `tax_subcategory` enthält optionale Unterkategorien für individuelle Auswertungen

## Erforderliche Summen

Die Spreadsheet-Ausgabe soll diese Summen berechnen und mit genau diesen Bezeichnungen anzeigen:

- `Einnahme(Brutto)`
- `Einnahme(Netto)`
- `Vereinnahmte Umsatzsteuer`
- `Betriebsausgaben(Netto)`
- `Bezahlte Vorsteuer`
- `UStVA`
- `Auszahlung`
- `Einkommensteuervorauszahlung`

Diese Summen sollen verfügbar sein für:

- das gesamte Jahr
- jedes Quartal
- jeden Monat

## Steuerkonstanten

Für die ersten Standardregeln wird ein expliziter Umsatzsteuersatz als Konstante geführt:

- `UST_SATZ_STANDARD = 0.19`

Diese Konstante soll zentral definiert werden, damit:

- Berechnungsregeln nicht mit hart codierten Prozentwerten arbeiten
- spätere Erweiterungen für weitere Steuersätze möglich bleiben
- Änderungen nachvollziehbar an einer Stelle erfolgen

Zusätzlich gilt für die Kategorielogik:

- Kategorien mit Präfix `Umsatzsteuer ` können unterschiedliche Sätze enthalten, zum Beispiel `19%`, `7%` oder `0%`
- der konkrete Steuersatz soll deshalb nach Möglichkeit aus der Kategorie selbst ausgelesen werden

## Ableitungslogik der Summen

Die Summen im System entstehen in zwei getrennten Schichten:

1. Standardsummen, die direkt aus `Betrag`, Zahlungsrichtung und `Kategorie` abgeleitet werden
2. Individuelle steuerliche Zuordnungen, die zusätzlich auf `Empfänger`, `Verwendungszweck` oder ähnlichen Merkmalen beruhen

Diese Trennung ist wichtig, damit die Grundlogik einfach, reproduzierbar und prüfbar bleibt, während individuelle Auswertungen trotzdem möglich sind.

## Standardableitung aus Betrag und Kategorie

Die bisher genannten Standardsummen sollen primär aus diesen Informationen berechnet werden:

- normalisierter Betrag
- Eingangs- oder Ausgangsrichtung der Zahlung
- ursprüngliche Kategorie aus der Bankdatei

Das bedeutet:

- die Kategorie aus der CSV bleibt zunächst die primäre fachliche Grundlage
- Regeln zur Summenbildung sollen explizit dokumentieren, welche `source_category` zu welcher Summe beiträgt
- frei formulierte Textfelder sollen für die Standardsummen nur dann verwendet werden, wenn `Kategorie` fachlich nicht ausreicht

### Erste konkrete Standardregel: `Einnahme(Brutto)`

Die Standardsumme `Einnahme(Brutto)` wird zunächst wie folgt definiert:

- `amount` ist positiv
- `source_category` beginnt mit `Umsatzsteuer `

Wichtig:

- Das Leerzeichen am Ende von `Umsatzsteuer ` ist fachlich relevant
- Dadurch werden Kategorien wie `Umsatzsteuer-Vorauszahlung` bewusst nicht erfasst

Die Regel ist damit als Präfix-Regel zu verstehen:

- gültig: `Umsatzsteuer 19%`
- nicht gültig: `Umsatzsteuer-Vorauszahlung`

Diese Regel gehört zur Standardsummen-Logik und soll ohne freie Textanalyse auskommen.

### Zweite konkrete Standardregel: `Einnahme(Netto)`

Die Standardsumme `Einnahme(Netto)` basiert auf derselben Grundmenge wie `Einnahme(Brutto)`:

- `amount` ist positiv
- `source_category` beginnt mit `Umsatzsteuer `

Die Berechnung erfolgt jedoch netto, also ohne enthaltene Umsatzsteuer.

Formel:

- `Einnahme(Netto) = Einnahme(Brutto) / (1 + UST_SATZ_STANDARD)`

Für den aktuellen Stand bedeutet das:

- `Einnahme(Netto) = Einnahme(Brutto) / 1.19`

Wichtig:

- Auch diese Regel darf `Umsatzsteuer-Vorauszahlung` nicht erfassen
- Die gemeinsame Filterlogik mit `Einnahme(Brutto)` sollte technisch an einer Stelle definiert werden
- Rundungsregeln müssen später fachlich festgelegt werden, damit Monats-, Quartals- und Jahressummen konsistent bleiben

### Dritte konkrete Standardregel: `Vereinnahmte Umsatzsteuer`

Die Standardsumme `Vereinnahmte Umsatzsteuer` wird als Differenz aus Brutto und Netto definiert.

Formel:

- `Vereinnahmte Umsatzsteuer = Einnahme(Brutto) - Einnahme(Netto)`

Wichtig:

- Die zugrunde liegende Filtermenge ist dieselbe wie bei `Einnahme(Brutto)` und `Einnahme(Netto)`
- Die Regel soll keine eigene abweichende Auswahl von Transaktionen einführen
- Rundung und Aggregation müssen konsistent mit den Regeln für Brutto und Netto erfolgen

### Vierte konkrete Standardregel: `Betriebsausgaben(Netto)`

Als `Betriebsausgaben(Netto)` zählen alle Ausgaben, für die Vorsteuer bezahlt wurde und die deshalb einer Kategorie mit Präfix `Umsatzsteuer ` zugeordnet sind.

Filterregel:

- `amount` ist negativ
- `source_category` beginnt mit `Umsatzsteuer `

Beispiele für passende Kategorien:

- `Umsatzsteuer 19%`
- `Umsatzsteuer 7%`
- `Umsatzsteuer 0%`

Wichtig:

- Auch hier ist das Leerzeichen in `Umsatzsteuer ` fachlich relevant
- Kategorien wie `Umsatzsteuer-Vorauszahlung` dürfen nicht erfasst werden
- Der konkrete Steuersatz soll aus der Kategorie abgeleitet werden, statt immer pauschal `UST_SATZ_STANDARD` zu verwenden

Berechnungsprinzip:

- `Betriebsausgaben(Netto) = Betrag(Absolutwert) / (1 + individueller_ust_satz)`

Beispiele:

- bei `Umsatzsteuer 19%`: Division durch `1.19`
- bei `Umsatzsteuer 7%`: Division durch `1.07`
- bei `Umsatzsteuer 0%`: Division durch `1.00`

Offen für die Implementierung:

- der Umsatzsteuersatz soll robust aus `source_category` geparst werden
- falls kein Steuersatz extrahiert werden kann, soll der Datensatz markiert werden statt stillschweigend falsch berechnet zu werden

### Fünfte konkrete Standardregel: `Bezahlte Vorsteuer`

Die Standardsumme `Bezahlte Vorsteuer` wird als Differenz zwischen Bruttoausgabe und Nettoausgabe definiert.

Die zugrunde liegende Filtermenge ist dieselbe wie bei `Betriebsausgaben(Netto)`:

- `amount` ist negativ
- `source_category` beginnt mit `Umsatzsteuer `

Formel:

- `Bezahlte Vorsteuer = Betriebsausgaben(Brutto) - Betriebsausgaben(Netto)`

Dabei gilt:

- `Betriebsausgaben(Brutto)` entspricht dem Absolutwert der erfassten Ausgaben
- `Betriebsausgaben(Netto)` wird über den aus der Kategorie gelesenen Steuersatz berechnet

Wichtig:

- Die Regel darf keine zusätzliche oder abweichende Transaktionsauswahl einführen
- Rundung und Aggregation müssen konsistent mit `Betriebsausgaben(Netto)` erfolgen

### Sechste konkrete Standardregel: `UStVA`

Die Standardsumme `UStVA` wird über die Kategorie `Umsatzsteuer-Vorauszahlung` erkannt.

Filterregel:

- `source_category` ist genau `Umsatzsteuer-Vorauszahlung`

Interpretation:

- diese Kategorie ist fachlich von Kategorien mit Präfix `Umsatzsteuer ` zu unterscheiden
- insbesondere darf `Umsatzsteuer-Vorauszahlung` nicht in die Regeln für Einnahmen oder Betriebsausgaben mit Umsatzsteuersatz fallen

Berechnungsprinzip:

- `UStVA` wird aus dem Betrag der entsprechend markierten Transaktionen aggregiert

Vorzeichenlogik:

- `UStVA` wird mit Vorzeichen dargestellt
- negative Werte entsprechen einer Zahlung
- positive Werte sind theoretisch möglich und würden eine Erstattung oder Gegenbuchung darstellen

### Siebte konkrete Standardregel: `Auszahlung`

Die Standardsumme `Auszahlung` wird über die exakte Kategorie `Privat` erkannt.

Filterregel:

- `source_category` ist genau `Privat`

Berechnungsprinzip:

- `Auszahlung` wird aus dem Betrag der entsprechend markierten Transaktionen aggregiert

Vorzeichenlogik:

- `Auszahlung` wird mit Vorzeichen dargestellt
- im aktuellen Datenbild sind typischerweise negative Werte zu erwarten
- positive Werte bleiben theoretisch möglich und sollen nicht künstlich umgerechnet werden

Wichtig:

- die Kategorie `Privat` ist fachlich getrennt von den Umsatzsteuer-Kategorien zu behandeln
- diese Transaktionen dürfen nicht in umsatzsteuerbezogene Standardsummen einfließen

### Achte konkrete Standardregel: `Einkommensteuervorauszahlung`

Die Standardsumme `Einkommensteuervorauszahlung` wird über die exakte Kategorie `Einkommensteuer-Vorauszahlung` erkannt.

Filterregel:

- `source_category` ist genau `Einkommensteuer-Vorauszahlung`

Berechnungsprinzip:

- `Einkommensteuervorauszahlung` wird aus dem Betrag der entsprechend markierten Transaktionen aggregiert

Vorzeichenlogik:

- `Einkommensteuervorauszahlung` wird mit Vorzeichen dargestellt
- im aktuellen Datenbild sind typischerweise negative Werte zu erwarten
- positive Werte bleiben theoretisch möglich und sollen nicht künstlich umgerechnet werden

Wichtig:

- diese Kategorie ist fachlich getrennt von `Privat`, `Umsatzsteuer-Vorauszahlung` und allen Kategorien mit Präfix `Umsatzsteuer `
- diese Transaktionen dürfen daher nicht in andere Standardsummen einfließen

## Individuelle Regeln nach Empfänger

Zusätzlich soll das System individuelle Regeln unterstützen, die sich auf `Empfänger`, `Verwendungszweck` oder weitere Felder beziehen.

Diese Regeln sind nötig für steuerliche Auswertungen, die nicht allein aus `Betrag` und `Kategorie` hervorgehen, zum Beispiel:

- Kauf eines Laptops
- Serverkosten
- Kosten für Codex
- Kosten für andere KI-Dienste oder Agenten

Für diese Regeln gilt:

- sie sind individuell pro Steuerfall
- sie sollen konfigurierbar statt fest im Code verdrahtet sein
- sie sollen in versionierten Regeldateien liegen
- sie sollen nachvollziehbar dokumentieren, warum ein Empfänger oder Verwendungszweck einer bestimmten Auswertung zugeordnet wurde

Mögliche Regelkriterien:

- exakter `Empfänger`
- normalisierter `Empfänger`
- Teilstring im `Verwendungszweck`
- Kombination aus `Empfänger`, `source_category` und weiteren Merkmalen

Ergebnisse solcher Regeln können sein:

- zusätzliche steuerliche Hauptkategorien
- Unterkategorien für Betriebsausgaben
- spezielle Auswertungen für einzelne Kostenarten

## Individuelle Betriebskostenkategorien

Zusätzlich zu den Standardsummen soll es individuelle Kategorien für Betriebskosten geben.

Fachliche Regeln:

- jede Transaktion, die zu `Betriebsausgaben(Netto)` gehört, muss genau einer Betriebskostenkategorie zugeordnet werden
- es darf keine Betriebsausgabe ohne Betriebskostenkategorie geben
- eine Betriebsausgabe darf nicht gleichzeitig mehreren Betriebskostenkategorien zugeordnet sein
- die verfügbaren Betriebskostenkategorien können pro Jahr unterschiedlich sein

Beispiele für Betriebskostenkategorien:

- `Serverkosten`
- `Backups`
- `KI-Agenten`

Für jede Betriebskostenkategorie soll es eigene Summen geben:

- für das Gesamtjahr
- für jedes Quartal
- für jeden Monat

Konsistenzanforderung:

- die Summe aller Betriebskostenkategorien eines Zeitraums muss exakt der Summe von `Betriebsausgaben(Netto)` für denselben Zeitraum entsprechen

Technische Umsetzung:

- die Regeln für Betriebskostenkategorien liegen in einer eigenen versionierten Regeldatei
- die Zuordnung kann über `Empfänger` und `Verwendungszweck` erfolgen
- bei fehlender oder mehrdeutiger Zuordnung soll der Lauf fehlschlagen statt stillschweigend falsche Ergebnisse zu erzeugen

Prüfausgaben:

- Detailansichten enthalten zusätzlich eine Spalte `Betriebskostenkategorie`
- es gibt eine eigene Übersicht `Betriebskostenkategorien` mit Summen pro Zeitraum und Kategorie

## Konkrete Regelstruktur für die Implementierung

Die Standardsummen sollen nicht nur textlich beschrieben werden, sondern in einer klaren Regelstruktur abbildbar sein.

Eine Regel soll mindestens diese Bestandteile haben:

- `name`: Name der Summe
- `scope`: Gültigkeitsbereich, zum Beispiel `standard_sum`
- `filter`: fachliche Auswahl von Transaktionen
- `formula`: Berechnungslogik
- `sign_policy`: Behandlung des Vorzeichens
- `exclusions`: explizite Ausschlüsse
- `notes`: fachliche Hinweise

Eine mögliche Zielstruktur in YAML könnte so aussehen:

```yaml
name: Einnahme(Brutto)
scope: standard_sum
filter:
  amount_sign: positive
  source_category_prefix: "Umsatzsteuer "
formula:
  type: sum_amount
sign_policy: keep
exclusions:
  - source_category_exact: "Umsatzsteuer-Vorauszahlung"
notes:
  - "Leerzeichen im Präfix ist fachlich relevant"
```

## Regelentwurf für die Standardsummen

Die folgenden Regelentwürfe beschreiben die acht Standardsummen in einer direkt umsetzbaren Form.

### Regel: `Einnahme(Brutto)`

```yaml
name: Einnahme(Brutto)
scope: standard_sum
filter:
  amount_sign: positive
  source_category_prefix: "Umsatzsteuer "
formula:
  type: sum_amount
sign_policy: keep
exclusions:
  - source_category_exact: "Umsatzsteuer-Vorauszahlung"
notes:
  - "Leerzeichen im Präfix ist fachlich relevant"
```

### Regel: `Einnahme(Netto)`

```yaml
name: Einnahme(Netto)
scope: standard_sum
filter:
  amount_sign: positive
  source_category_prefix: "Umsatzsteuer "
formula:
  type: sum_amount_divided_by_vat_rate
  vat_rate_source: category_prefix_suffix_percent
  fallback_vat_rate: 0.19
sign_policy: keep
exclusions:
  - source_category_exact: "Umsatzsteuer-Vorauszahlung"
notes:
  - "Wenn kein Steuersatz aus der Kategorie gelesen werden kann, muss der Datensatz markiert werden"
```

### Regel: `Vereinnahmte Umsatzsteuer`

```yaml
name: Vereinnahmte Umsatzsteuer
scope: standard_sum
filter:
  amount_sign: positive
  source_category_prefix: "Umsatzsteuer "
formula:
  type: difference_of_rules
  minuend: "Einnahme(Brutto)"
  subtrahend: "Einnahme(Netto)"
sign_policy: keep
exclusions:
  - source_category_exact: "Umsatzsteuer-Vorauszahlung"
notes:
  - "Verwendet dieselbe Filtermenge wie Einnahme(Brutto) und Einnahme(Netto)"
```

### Regel: `Betriebsausgaben(Netto)`

```yaml
name: Betriebsausgaben(Netto)
scope: standard_sum
filter:
  amount_sign: negative
  source_category_prefix: "Umsatzsteuer "
formula:
  type: sum_absolute_amount_divided_by_vat_rate
  vat_rate_source: category_prefix_suffix_percent
sign_policy: absolute
exclusions:
  - source_category_exact: "Umsatzsteuer-Vorauszahlung"
notes:
  - "Unterstützt z. B. 19 %, 7 % und 0 %"
  - "Wenn kein Steuersatz aus der Kategorie gelesen werden kann, muss der Datensatz markiert werden"
```

### Regel: `Bezahlte Vorsteuer`

```yaml
name: Bezahlte Vorsteuer
scope: standard_sum
filter:
  amount_sign: negative
  source_category_prefix: "Umsatzsteuer "
formula:
  type: difference_of_absolute_amount_and_rule
  absolute_amount_basis:
    amount_sign: negative
    source_category_prefix: "Umsatzsteuer "
  subtrahend: "Betriebsausgaben(Netto)"
sign_policy: absolute
exclusions:
  - source_category_exact: "Umsatzsteuer-Vorauszahlung"
notes:
  - "Die Bruttobasis ist hier eine interne Hilfsgröße und keine eigene Ausgabespalte"
```

### Regel: `UStVA`

```yaml
name: UStVA
scope: standard_sum
filter:
  source_category_exact: "Umsatzsteuer-Vorauszahlung"
formula:
  type: sum_amount
sign_policy: keep
exclusions: []
notes:
  - "Negative Werte bedeuten Zahlung"
  - "Positive Werte bedeuten Erstattung oder Gegenbuchung"
```

### Regel: `Auszahlung`

```yaml
name: Auszahlung
scope: standard_sum
filter:
  source_category_exact: "Privat"
formula:
  type: sum_amount
sign_policy: keep
exclusions: []
notes:
  - "Typischerweise negativ, aber positive Werte bleiben möglich"
```

### Regel: `Einkommensteuervorauszahlung`

```yaml
name: Einkommensteuervorauszahlung
scope: standard_sum
filter:
  source_category_exact: "Einkommensteuer-Vorauszahlung"
formula:
  type: sum_amount
sign_policy: keep
exclusions: []
notes:
  - "Typischerweise negativ, aber positive Werte bleiben möglich"
```

## Feldzuordnung Quelle -> Kanonisches Schema

Die aktuellen CSV-Felder der Bank werden wie folgt abgebildet:

- `Buchungsdatum` -> `booking_date`
- `Wertstellungsdatum` -> `value_date`
- `Transaktionstyp` -> `transaction_type`
- `Empfänger` -> `counterparty`
- `Betrag` -> `amount`
- `IBAN` -> `iban`
- `Verwendungszweck` -> `reference`
- `end_to_end_id` -> `end_to_end_id`
- `Buchungsstatus` -> `booking_status`
- `Kategorie` -> `source_category`
- `Persönliche Notiz` -> `personal_note`

Abgeleitete Felder:

- `currency` -> konstanter Wert `EUR`
- `direction` -> aus `amount` abgeleitet
- `tax_year` -> aus `booking_date` abgeleitet
- `source_file` -> Import-Metadaten
- `source_row` -> Import-Metadaten

Zusätzliche Klassifikationsfelder:

- `tax_category` -> aus Regeln auf Basis von `source_category`, `Empfänger`, `Verwendungszweck` und weiteren Merkmalen
- `tax_subcategory` -> aus feineren individuellen Regeln

## Importregeln für dieses Bankformat

Der Importer für dieses CSV-Format soll:

- CSV mit Semikolon-Trennung und doppelten Anführungszeichen parsen
- deutsche Dezimalzahlen in normalisierte Dezimalwerte umwandeln
- leere Felder als leere Werte übernehmen statt Platzhalter zu erfinden
- die ursprüngliche Kategorie in `source_category` erhalten
- unbekannte Spaltenlayouts ablehnen oder markieren
- eine deterministische Ausgabe erzeugen, sortiert nach `booking_date` und danach `source_row`

## Umsetzungsphasen

### Phase 1: Beispielimport

- 2 bis 3 repräsentative CSV-Dateien sammeln
- die verwendeten Spalten je Institut identifizieren
- pro CSV-Format einen Importer schreiben
- alle Eingaben in das einheitliche Schema normalisieren
- sicherstellen, dass die normalisierte Ausgabe stabil sortiert ist und lesbare Git-Diffs erzeugt

Ergebnis:

- ein Befehl, der Rohdaten in eine normalisierte Tabelle umwandelt
- eine deterministische CSV-Ausgabe

### Phase 2: Klassifikation

- steuerrelevante Kategorien definieren
- deterministische Regeln anhand von Beschreibung, Konto, Gegenpartei oder Tags ergänzen
- zwischen Standardsummen-Logik und individuellen Empfängerregeln unterscheiden
- manuelle Overrides für Sonderfälle ermöglichen

Ergebnis:

- ein klassifizierter Transaktionsdatensatz plus Ausnahmeliste

### Phase 3: Berechnungslogik

- die Steuerberechnungen für das Jahr definieren
- die erforderlichen Summen berechnen: `Einnahme(Brutto)`, `Einnahme(Netto)`, `Vereinnahmte Umsatzsteuer`, `Betriebsausgaben(Netto)`, `Bezahlte Vorsteuer`, `UStVA`, `Auszahlung`, `Einkommensteuervorauszahlung`
- die Standardsummen primär aus `amount`, Zahlungsrichtung und `source_category` ableiten
- die Regel für `Einnahme(Brutto)` fest anwenden: positiver `amount` und Präfix `Umsatzsteuer `
- die Regel für `Einnahme(Netto)` fest anwenden: dieselbe Filtermenge wie `Einnahme(Brutto)`, anschließend Division durch `1 + UST_SATZ_STANDARD`
- die Regel für `Vereinnahmte Umsatzsteuer` fest anwenden: `Einnahme(Brutto) - Einnahme(Netto)`
- die Regel für `Betriebsausgaben(Netto)` fest anwenden: negativer `amount`, Präfix `Umsatzsteuer `, Nettoermittlung über den aus der Kategorie gelesenen Steuersatz
- die Regel für `Bezahlte Vorsteuer` fest anwenden: `Betriebsausgaben(Brutto) - Betriebsausgaben(Netto)`
- die Regel für `UStVA` fest anwenden: Kategorie exakt `Umsatzsteuer-Vorauszahlung`
- die Regel für `Auszahlung` fest anwenden: Kategorie exakt `Privat`
- die Regel für `Einkommensteuervorauszahlung` fest anwenden: Kategorie exakt `Einkommensteuer-Vorauszahlung`
- zusätzliche individuelle Auswertungen aus `tax_category` und `tax_subcategory` ableiten
- Aggregationen pro Monat, Quartal und Gesamtjahr erzeugen
- alle Rechenschritte transparent und reproduzierbar halten
- erzeugte CSV-Zusammenfassungen als primäre Prüfartefakte verwenden

Ergebnis:

- eine maschinell erzeugte Steuerzusammenfassung mit Rückverweisen auf die Quelltransaktionen

### Phase 4: Prüfausgaben

- normalisierte Transaktionen als CSV exportieren
- Summen und Kategorien als CSV exportieren
- Ausnahmen und manuelle Overrides als CSV exportieren
- eine Prüf-Arbeitsmappe erzeugen mit:
  - Jahresübersichten
  - Quartalsübersichten
  - Monatsübersichten
  - Blättern für auffällige Transaktionen
  - Detailblättern mit Quellbezug, wo nötig

Ergebnis:

- ein gut lesbares Prüfungspaket

### Phase 5: Finaler Spreadsheet-Export

- ein endgültiges Layout der Arbeitsmappe für die Prüfung durch das Finanzamt festlegen
- `.xlsx` als primäres Endformat erzeugen

Ergebnis:

- ein reproduzierbarer Spreadsheet-Export

## Spreadsheet-Ausgabe

Es wird eine Arbeitsmappe mit mehreren Tabellenblättern erzeugt:

- Primäre Ausgabe: `.xlsx`

Die Arbeitsmappe soll enthalten:

- eine Jahresübersicht für das gesamte Steuerjahr
- eine Quartalsübersicht für Q1 bis Q4
- eine Monatsübersicht für Januar bis Dezember
- die erforderlichen Summenspalten: `Einnahme(Brutto)`, `Einnahme(Netto)`, `Vereinnahmte Umsatzsteuer`, `Betriebsausgaben(Netto)`, `Bezahlte Vorsteuer`, `UStVA`, `Auszahlung`, `Einkommensteuervorauszahlung`
- zusätzliche Aufschlüsselungen nach Kategorien innerhalb dieser Zeiträume, wenn relevant

Diese Ausgabe funktioniert direkt in Spreadsheet-GUIs und liegt näher an den zugrunde liegenden Tabellendaten als ein Textdokument.

## Daten- und Prüfstrategie

Damit die Ausgabe in einer Prüfung belastbar ist:

- Roh-CSV-Dateien niemals verändern
- für jede normalisierte Transaktion Quelldatei und Quellzeile speichern
- normalisierte Daten und Prüfdaten als Textdateien unter Versionskontrolle halten
- Exporte deterministisch halten, insbesondere durch stabile Sortierung und feste Spaltenreihenfolge
- Transformationsprotokolle erhalten
- Regeldefinitionen in Dateien versionieren
- Prüf-CSV-Dateien zusammen mit dem finalen Spreadsheet exportieren
- Annahmen und bekannte Einschränkungen dokumentieren

## Git-freundliche Speicherstrategie

Git-versionierte Textdateien sind die maßgeblichen Datenquellen:

- `data/raw/*.csv`: originale Kontoauszüge
- `data/normalized/*.csv`: kanonisch normalisierte Transaktionen
- `data/review/*.csv`: klassifizierte Transaktionen, Ausnahmen, Overrides und Summen
- `config/**/*.yml` oder `config/**/*.json`: Regeln und Zuordnungen

Erzeugte Artefakte außerhalb des Kern-Prüfpfads:

- `data/build/*.sqlite`: optionale abgeleitete Datenbank, in der Regel per `.gitignore` ausgeschlossen
- `data/exports/*.xlsx`: erzeugte finale Tabellen

So bleibt der Revisionsverlauf lesbar, ohne auf lokale Datenbankfunktionen verzichten zu müssen.

## Wichtige fachliche Entscheidungen

Diese Punkte sollten früh festgelegt werden:

1. Welche Steuerjurisdiktion und welche Steuerjahre sind im Umfang?
2. Welche Transaktionstypen sind relevant?
   Beispiele: Gehalt, Dividenden, Kapitalerträge, Gebühren, Zinsen, Transfers, Erstattungen
3. Soll die Klassifikation vollständig automatisch laufen oder regelbasiert mit manueller Prüfung?
4. Wie soll die Arbeitsmappe für GUI-Prüfung und Finanzamt strukturiert sein?
5. Wird nur ein Jahresbericht benötigt oder zusätzlich kontobezogene Anhänge?
6. Welche erzeugten Dateien sollen committet werden und welche bleiben reine Build-Artefakte?

## Minimal Viable First Version

Die erste nützliche Version soll nur Folgendes können:

- ein CSV-Format importieren
- Transaktionen normalisieren
- 80 % der Zeilen automatisch klassifizieren
- eine Jahreszusammenfassung als CSV exportieren
- Monats- und Quartalszusammenfassungen als CSV exportieren
- alle maßgeblichen Zwischendaten als Git-versionierte CSV halten
- eine `.xlsx`-Arbeitsmappe mit Jahres-, Quartals- und Monatsübersichten sowie Blättern für auffällige Transaktionen erzeugen

Das reicht aus, um den Ablauf zu validieren, bevor mehrere Institute und Sonderfälle ergänzt werden.

## Nächste Schritte

1. Den gewählten Implementierungspfad festhalten:
   Python + CSV + Spreadsheet-Export
   Optionale Erweiterung: SQLite nur als abgeleiteter Cache
2. 2 bis 3 echte Beispiel-CSV-Dateien sammeln und deren Struktur prüfen
3. Das kanonische Transaktionsschema final festziehen
4. Die relevanten Steuerkategorien und Berechnungsregeln definieren
5. Zuerst den Importer bauen, danach den Spreadsheet-Export
