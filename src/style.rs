use clap::builder::styling::{AnsiColor, Color, Style};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DshColor {
  /// Displayed in the default terminal color
  #[serde(rename = "normal")]
  Normal,
  #[serde(rename = "black")]
  Black,
  #[serde(rename = "blue")]
  Blue,
  #[serde(rename = "cyan")]
  Cyan,
  #[serde(rename = "green")]
  Green,
  #[serde(rename = "magenta")]
  Magenta,
  #[serde(rename = "red")]
  Red,
  #[serde(rename = "yellow")]
  Yellow,
  #[serde(rename = "white")]
  White,
}

impl Default for DshColor {
  fn default() -> Self {
    Self::Normal
  }
}

impl Display for DshColor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshColor::Normal => write!(f, "normal"),
      DshColor::Black => write!(f, "black"),
      DshColor::Blue => write!(f, "blue"),
      DshColor::Cyan => write!(f, "cyan"),
      DshColor::Green => write!(f, "green"),
      DshColor::Magenta => write!(f, "magenta"),
      DshColor::Red => write!(f, "red"),
      DshColor::White => write!(f, "white"),
      DshColor::Yellow => write!(f, "yellow"),
    }
  }
}

impl TryFrom<&str> for DshColor {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "normal" => Ok(Self::Normal),
      "black" => Ok(Self::Black),
      "blue" => Ok(Self::Blue),
      "cyan" => Ok(Self::Cyan),
      "green" => Ok(Self::Green),
      "magenta" => Ok(Self::Magenta),
      "red" => Ok(Self::Red),
      "white" => Ok(Self::White),
      "yellow" => Ok(Self::Yellow),
      _ => Err(format!("invalid matching color '{}'", value)),
    }
  }
}

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DshStyle {
  /// Default terminal font
  #[serde(rename = "normal")]
  Normal,
  #[serde(rename = "bold")]
  Bold,
  #[serde(rename = "dim")]
  Dim,
  #[serde(rename = "italic")]
  Italic,
  #[serde(rename = "underline")]
  Underline,
  #[serde(rename = "reverse")]
  Reverse,
}

impl Default for DshStyle {
  fn default() -> Self {
    Self::Normal
  }
}

impl Display for DshStyle {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshStyle::Normal => write!(f, "normal"),
      DshStyle::Bold => write!(f, "bold"),
      DshStyle::Dim => write!(f, "dim"),
      DshStyle::Italic => write!(f, "italic"),
      DshStyle::Underline => write!(f, "underline"),
      DshStyle::Reverse => write!(f, "reverse"),
    }
  }
}

impl TryFrom<&str> for DshStyle {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "normal" => Ok(Self::Normal),
      "bold" => Ok(Self::Bold),
      "dim" => Ok(Self::Dim),
      "italic" => Ok(Self::Italic),
      "underline" => Ok(Self::Underline),
      "reverse" => Ok(Self::Reverse),
      _ => Err(format!("invalid matching style '{}'", value)),
    }
  }
}

pub(crate) fn style_from(style: &DshStyle, color: &DshColor) -> Style {
  let style = match style {
    DshStyle::Normal => Style::new(),
    DshStyle::Bold => Style::new().bold(),
    DshStyle::Dim => Style::new().dimmed(),
    DshStyle::Italic => Style::new().italic(),
    DshStyle::Underline => Style::new().underline(),
    DshStyle::Reverse => Style::new().invert(),
  };
  match color {
    DshColor::Normal => style,
    DshColor::Black => style.fg_color(Some(Color::Ansi(AnsiColor::Black))),
    DshColor::Blue => style.fg_color(Some(Color::Ansi(AnsiColor::Blue))),
    DshColor::Cyan => style.fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
    DshColor::Green => style.fg_color(Some(Color::Ansi(AnsiColor::Green))),
    DshColor::Magenta => style.fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
    DshColor::Red => style.fg_color(Some(Color::Ansi(AnsiColor::Red))),
    DshColor::White => style.fg_color(Some(Color::Ansi(AnsiColor::White))),
    DshColor::Yellow => style.fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
  }
}

pub(crate) fn apply_default_error_style<T: Display>(text: T) -> String {
  let error_style = style_from(&DshStyle::Bold, &DshColor::Red);
  format!("{error_style}{text}{error_style:#}")
}

pub(crate) fn apply_default_warning_style<T: Display>(text: T) -> String {
  let warning_style = style_from(&DshStyle::Bold, &DshColor::Blue);
  format!("{warning_style}{text}{warning_style:#}")
}
