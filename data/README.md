# Ticker Sniffer Data Directory

This directory contains the `company_symbol_list.csv` file, which is used for managing company symbols, names, and their alternate names.

## `company_symbol_list.csv`

Link: [company_symbol_list.csv](company_symbol_list.csv)

### Purpose
This file provides a mapping of ticker symbols to their full company names and alternate names.

### Columns
- **Symbol**: The ticker symbol for the company.
- **Company Name**: The full name of the company.
- **Alternate Names**: A list of alternative names for the company.

### Format for "Alternate Names"
The **Alternate Names** column must follow these rules:
1. **Simple CSV Format**: Alternate names are separated by commas.
   - Example: `Alternate Name 1, Alternate Name 2, Alternate Name 3`
2. **Avoid Embedded Commas**: If an alternate name contains a comma, omit the comma and do not use delimiters (e.g., quotes).
   - Example:
     - Incorrect: `"Name, with comma"`
     - Correct: `Name with comma`

This ensures compatibility with standard CSV parsers and prevents errors due to embedded commas or inconsistent delimiters.

### Example Entries

```csv
"Symbol","Company Name","Alternate Names"
"A","Agilent Technologies, Inc.","Agilent, Agilent Tech, Agilent Technologies"
"AA","Alcoa Corporation","Alcoa, Alcoa Corp, Alcoa Inc"
"AAA","AXS First Priority CLO Bond ETF","AXS Bond ETF, First Priority CLO ETF"
```

### Notes
- Ensure alternate names are concise and meaningful.
- Verify that all entries are accurate and free of duplicates or formatting issues.
