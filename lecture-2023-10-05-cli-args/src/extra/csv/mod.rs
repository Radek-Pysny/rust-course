//! This module publishes function `_csv` that reads a csv from stdin and then it prints a table on
//! stdout.
//!
//! It is using standard csv crate, that takes care of troubles with different field count.
//!
//! Input:
//!
//! > a,bcd,e
//! > 1,23,456
//! > 69,,
//!
//! Output:
//!
//! > |a |bcd|e  |
//! > ------------
//! > |1 |23 |456|
//! > |69|   |   |
//!

use std::error::Error;
use std::io;
use std::fmt;
use std::fmt::Formatter;
use csv;


/// Just inner struct that is used for holding of CSV data read from stdin and then formatted to
/// stdout thanks to implementation of Display trait.
struct Csv {
    /// `header` contains all the fields of header.
    header: Vec<String>,

    /// `lines` contains a complete content of CSV file except its header line.
    lines: Vec<Vec<String>>,

    /// `widths` is a helper vector with maximum size of each column of the internally held CSV.
    /// It is calculated during process of reading the input data from stdin via static method
    /// `from_csv_reader`.
    widths: Vec<usize>,
}


impl Csv {
    /// from_csv_reader construct Csv struct by parsing the CSV content via the given CSV reader
    fn from_csv_reader(
        reader: &mut csv::Reader<io::Stdin>,
    ) -> std::result::Result<Self, Box<dyn Error>> {
        let mut result = Csv{
            header: Vec::new(),
            lines: Vec::new(),
            widths: Vec::new(),
        };

        for field in reader.headers()? {
            result.header.push(field.to_string());
            result.widths.push(field.len());
        }

        for input_line in reader.records() {
            let input_line = input_line?;
            let mut output_line: Vec<String> = Vec::new();
            for (index, field) in input_line.iter().enumerate() {
                output_line.push(field.to_string());
                result.widths[index] = std::cmp::max(result.widths[index], field.len())
            }
            result.lines.push(output_line);
        }

        Ok(result)
    }
}


impl fmt::Display for Csv {
    /// Thanks to implementation of Display trait, .to_string() is can be used.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "|")?;
        for (index, field) in self.header.iter().enumerate() {
            let mut output = field.to_string();
            output.push_str(" ".repeat(self.widths[index] - field.len()).as_str());
            write!(f, "{}|", output)?;
        }
        write!(f, "\n")?;

        write!(f, "{}\n", "-".repeat(self.widths.iter().sum::<usize>() + self.header.len() + 1))?;

        for line in &self.lines {
            write!(f, "|")?;
            for (index, field) in line.iter().enumerate() {
                let mut output = field.to_string();
                output.push_str(" ".repeat(self.widths[index] - field.len()).as_str());
                write!(f, "{}|", output)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}


pub fn _csv() -> Result<String, Box<dyn Error>> {
    let mut csv_reader = csv::Reader::from_reader(io::stdin());
    let content = Csv::from_csv_reader(&mut csv_reader)?;
    Ok(content.to_string())
}
