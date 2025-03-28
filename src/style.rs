use clap::builder::styling::{AnsiColor, Color, Style};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DshColor {
  /// Matches will be displayed in the normal color
  #[serde(rename = "normal")]
  Normal,
  /// Matches will be displayed in red
  #[serde(rename = "red")]
  Red,
  /// Matches will be displayed in green
  #[serde(rename = "green")]
  Green,
  /// Matches will be displayed in blue
  #[serde(rename = "blue")]
  Blue,
}

impl Display for DshColor {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshColor::Normal => write!(f, "normal"),
      DshColor::Red => write!(f, "red"),
      DshColor::Green => write!(f, "green"),
      DshColor::Blue => write!(f, "blue"),
    }
  }
}

impl TryFrom<&str> for DshColor {
  type Error = String;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "normal" => Ok(Self::Normal),
      "red" => Ok(Self::Red),
      "green" => Ok(Self::Green),
      "blue" => Ok(Self::Blue),
      _ => Err(format!("invalid matching color '{}'", value)),
    }
  }
}

#[derive(clap::ValueEnum, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum DshStyle {
  /// Matches will be displayed in normal font
  #[serde(rename = "normal")]
  Normal,
  /// Matches will be displayed bold
  #[serde(rename = "bold")]
  Bold,
  /// Matches will be displayed dimmed
  #[serde(rename = "dim")]
  Dim,
  /// Matches will be displayed in italics
  #[serde(rename = "italic")]
  Italic,
  /// Matches will be displayed underlined
  #[serde(rename = "underlined")]
  Underlined,
  /// Matches will be displayed reversed
  #[serde(rename = "reverse")]
  Reverse,
}

impl Display for DshStyle {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DshStyle::Normal => write!(f, "normal"),
      DshStyle::Bold => write!(f, "bold"),
      DshStyle::Dim => write!(f, "dim"),
      DshStyle::Italic => write!(f, "italic"),
      DshStyle::Underlined => write!(f, "underlined"),
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
      "underlined" => Ok(Self::Underlined),
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
    DshStyle::Underlined => Style::new().underline(),
    DshStyle::Reverse => Style::new().invert(),
  };
  match color {
    DshColor::Normal => style,
    DshColor::Red => style.fg_color(Some(Color::Ansi(AnsiColor::Red))),
    DshColor::Green => style.fg_color(Some(Color::Ansi(AnsiColor::Green))),
    DshColor::Blue => style.fg_color(Some(Color::Ansi(AnsiColor::Blue))),
  }
}

pub(crate) fn wrap_style<T: AsRef<str>>(style: Style, string: T) -> String {
  format!("{style}{}{style:#}", string.as_ref())
}
