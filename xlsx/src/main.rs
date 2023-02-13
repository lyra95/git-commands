use anyhow::Result;
use clap::Parser;
use xlsxwriter::{FormatAlignment, Workbook};

fn main() -> Result<()> {
    let args = Args::parse();
    let text = parse_text(&args.text, &args.line_separator, &args.column_separator);
    write(&text, &args.output_path, &args.sheet_name)
}

fn parse_text<'a>(text: &'a str, line_sep: &'a str, column_sep: &'a str) -> Vec<Vec<&'a str>> {
    let lines = text.split(line_sep);

    let mut result = vec![];
    for line in lines {
        let cols = line.split(column_sep);
        result.push(cols.into_iter().collect());
    }
    result
}

fn write(text: &Vec<Vec<&str>>, path: &str, name: &str) -> Result<()> {
    let workbook = Workbook::new(path)?;
    let mut sheet = workbook.add_worksheet(Some(name))?;
    let format = workbook.add_format().set_align(FormatAlignment::Left);
    let mut row = 0;
    let mut col = 0;

    for line in text {
        for column in line {
            if let Some(num) = parse_number(*column) {
                sheet.write_number(row, col, num, Some(&format))?;
            } else if let Some(b) = parse_boolean(*column) {
                sheet.write_boolean(row, col, b, Some(&format))?;
            } else if let Some(formula) = parse_formula(*column) {
                sheet.write_formula(row, col, formula, Some(&format))?;
            } else if let Some(url) = parse_url(*column) {
                sheet.write_url(row, col, url, Some(&format))?;
            } else {
                sheet.write_string(row, col, *column, Some(&format))?;
            }

            col += 1;
        }

        row += 1;
        col = 0;
    }

    workbook.close()?;
    Ok(())
}

fn parse_number(text: &str) -> Option<f64> {
    str::parse(text).map_or_else(|_| None, |x| Some(x))
}

fn parse_formula(text: &str) -> Option<&str> {
    if text.starts_with('=') {
        return Some(text);
    }
    None
}

fn parse_boolean(text: &str) -> Option<bool> {
    if text == "TRUE" {
        return Some(true)
    } else if text == "FALSE" {
        return Some(false)
    }

    None
}

fn parse_url(text: &str) -> Option<&str> {
    if text.starts_with("http") {
        return Some(text);
    }
    None
}

#[derive(Parser)]
#[command(author, version, about = "generate xlsx from txt file", long_about = None)]
struct Args {
    #[arg()]
    text: String,
    #[arg(short, long, default_value(","), help = "column separator")]
    column_separator: String,
    #[arg(short, long, default_value("\n"), help = "line separator")]
    line_separator: String,
    #[arg(short, long, default_value("./result.xlsx"))]
    output_path: String,
    #[arg(short, long, default_value("sheet1"))]
    sheet_name: String,
}
