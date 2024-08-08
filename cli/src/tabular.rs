use std::fmt::Display;

/// Converts a sequence of sequences into a formatted table
///
/// Converts a `Vec<Vec<String>>` into a nicely formatted table.
/// Formatting is achieved by padding the values with spaces.
/// For each row, a prefix (`start`) and postfix(`end`) can be specified.
/// The items in a row are seperated by `separator`.
///
/// ## Parameters
/// * `rows`      - list of lists of items
/// * `start`     - prefix that will be put in front of every row
/// * `separator` - the separator between values on a row
/// * `end`       - postfix that will be put at the end of every row
///
/// ## Returns
/// * formatted table as a `Vec<String>`
pub(crate) fn make_tabular(rows: Vec<Vec<String>>, start: &str, separator: &str, end: &str) -> Vec<String> {
  let column_widths = column_widths(&rows);
  rows
    .into_iter()
    .map(|row| {
      row
        .into_iter()
        .zip(&column_widths)
        .map(|(cell, width)| format!("{}{}", cell, " ".repeat(*width - cell.len())))
        .collect()
    })
    .collect::<Vec<Vec<String>>>()
    .into_iter()
    .map(|row| row.join(separator))
    .collect::<Vec<String>>()
    .iter()
    .map(|row| format!("{}{}{}", start, row, end))
    .collect()
}

/// Converts a sequence of sequences into a formatted table
///
/// Converts a `Vec<Vec<T: Display>>` into a nicely formatted table.
/// Formatting is achieved by padding the values with spaces.
/// For each row, a prefix (`start`) and postfix(`end`) can be specified.
/// The items in a row are seperated by `separator`.
///
/// ## Parameters
/// * `rows`      - list of lists of items
/// * `start`     - prefix that will be put in front of every row
/// * `separator` - the separated between values on a row
/// * `end`       - postfix that will be put at the end of every row
///
/// ## Returns
/// * formatted table as a `Vec<String>`
pub(crate) fn _make_tabular_display<T>(rows: Vec<Vec<T>>, start: &str, separator: &str, end: &str) -> Vec<String>
where
  T: Display,
{
  make_tabular(from_display(rows), start, separator, end)
}

/// Converts a sequence of sequences into a default formatted table
///
/// Converts a `Vec<Vec<T: Display>>` into a nicely formatted table using default settings.
/// Formatting is achieved by padding the values with spaces.
/// No prefixes and postfixes will be used, and the items in a row will be seperated by two spaces.
///
/// ## Parameters
/// * `rows` - list of lists of items
///
/// ## Returns
/// * formatted table as a `Vec<String>`
pub(crate) fn _make_tabular_default<T>(rows: Vec<Vec<T>>) -> Vec<String>
where
  T: Display,
{
  _make_tabular_display(rows, "", "  ", "")
}

/// Converts a sequence of sequences into a default formatted table
///
/// Converts a `Vec<Vec<T: Display>>` into a nicely formatted table using default settings.
/// Formatting is achieved by padding the values with spaces.
/// No prefixes and postfixes will be used, and the items in a row will be seperated by two spaces.
///
/// ## Parameters
/// * `rows` - list of lists of items
///
/// ## Returns
/// * formatted table as a `Vec<String>`
pub(crate) fn make_tabular_with_headers<H, T>(headers: Vec<H>, rows: Vec<Vec<T>>) -> Vec<String>
where
  H: Display,
  T: Display,
{
  let mut table = vec![headers.iter().map(|h| h.to_string()).collect()];
  table.append(&mut from_display(rows));
  make_tabular(table, "", "  ", "")
}

fn from_display<T>(rows: Vec<Vec<T>>) -> Vec<Vec<String>>
where
  T: Display,
{
  rows
    .into_iter()
    .map(|row| row.into_iter().map(|cell| cell.to_string()).collect())
    .collect::<Vec<Vec<String>>>()
}

fn column_widths(string_rows: &[Vec<String>]) -> Vec<usize> {
  transpose(string_rows.iter().map(|row| row.iter().map(|cell| cell.len()).collect()).collect())
    .iter()
    .map(|col| col.iter().max().cloned().unwrap_or_default())
    .collect()
}

fn transpose<T>(v: Vec<Vec<T>>) -> Vec<Vec<T>>
where
  T: Clone,
{
  assert!(!v.is_empty());
  (0..v[0].len()).map(|i| v.iter().map(|inner| inner[i].clone()).collect::<Vec<T>>()).collect()
}

#[test]
fn test_make_tabular() {
  let v = vec![
    vec!["1".to_string(), "22".to_string(), "333".to_string()],
    vec!["4444".to_string(), "55555".to_string(), "666666".to_string()],
    vec!["7777777".to_string(), "88888888".to_string(), "999999999".to_string()],
  ];
  let table = make_tabular(v, "[", "|", "]");
  for line in &table {
    println!("{}", line)
  }
  assert_eq!(
    table,
    vec!["[1      |22      |333      ]", "[4444   |55555   |666666   ]", "[7777777|88888888|999999999]"]
  );
}
