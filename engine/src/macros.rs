// #![warn(rustdoc::invalid_rust_codeblocks)]
// #![warn(rustdoc::invalid_rust_codeblocks)]
#![doc = "Macros"]
#![doc = "# `identifier!(...)`"]
#![doc = "This macro generates an identifier struct that can be used in the Trifonius engine."]
#![doc = "A generated identifier consists of a named tuple-struct,"]
#![doc = "which implicitly implements a number of useful traits like"]
#![doc = "[`Clone`],"]
#![doc = "[`Debug`][core::fmt::Debug],"]
#![doc = "[`Deref`][std::ops::Deref],"]
#![doc = "[`Deserialize`][serde::Deserialize],"]
#![doc = "[`Display`][std::fmt::Display],"]
#![doc = "[`Eq`],"]
#![doc = "[`FromStr`][core::str::FromStr],"]
#![doc = "[`Hash`][core::hash::Hash],"]
#![doc = "[`PartialEq`],"]
#![doc = "[`Ord`],"]
#![doc = "[`PartialOrd`],"]
#![doc = "[`Serialize`][serde::Serialize] and"]
#![doc = "[`TryFrom`]."]
#![doc = "It also provides validation by means of an `is_valid(id: &str)` method and"]
#![doc = "a `new(id: &str)` factory function that panics on any invalid identifier."]
#![doc = ""]
#![doc = "Note that this macro requires the `lazy_static` dependency."]
#![doc = ""]
#![doc = "## Examples"]
#![doc = "Create a `ProcessorId` identifier component"]
#![doc = "```rust"]
#![doc = "use trifonius_engine::identifier;"]
#![doc = ""]
#![doc = "identifier!("]
#![doc = "  \"processor\",              // path of the crate"]
#![doc = "  ProcessorId,              // name of the generated identifier struct"]
#![doc = "  \"processor identifier\",   // text used in generated comments"]
#![doc = "  \"^[a-z][a-z0-9]{0,19}$\",  // regular expression to validate the identifier"]
#![doc = "  \"validprocessorid\",       // example of valid identifier"]
#![doc = "  \"invalid_processor_id\",   // example of invalid identifier"]
#![doc = "  /// Doc comment           // multiline doc comment for the generated code"]
#![doc = ");"]
#![doc = "```"]
#![doc = "This will yield something like:"]
#![doc = "```rust,ignore"]
#![doc = "/// Doc comment"]
#![doc = "#[derive(Clone, Debug, serde::Deserialize, Eq, Hash, PartialEq, Ord,"]
#![doc = "           PartialOrd, serde::Serialize)]"]
#![doc = "pub struct ProcessorId(String);"]
#![doc = "impl ProcessorId {"]
#![doc = "  /// Create new `ProcessorId`"]
#![doc = "  pub fn new(id: &str) -> Self { ... }"]
#![doc = "  /// Validate processor identifier"]
#![doc = "  pub fn is_valid(id: &str) -> bool { ... }"]
#![doc = "  /// Returns the regular expression"]
#![doc = "  pub fn regex() -> &'static regex::Regex { ... }"]
#![doc = "}"]
#![doc = "impl TryFrom<&str> for ProcessorId { ... }"]
#![doc = "impl TryFrom<String> for ProcessorId { ... }"]
#![doc = "impl Display for ProcessorId { ... }"]
#![doc = "impl std::ops::Deref for ProcessorId { ... }"]
#![doc = "```"]

#[macro_export]
macro_rules! identifier {
  (
    $crate_name:literal,
    $name:ident,
    $label:literal,
    $regex:literal,
    $valid_id:literal,
    $invalid_id:literal,
    $(#[$doc:meta])*
  ) => {
    #[doc = concat!("# `", stringify!($name), "`")]
    #[doc = ""]
    $(#[$doc])*
    #[doc = ""]
    #[doc = "## Validation rules"]
    #[doc = ""]
    #[doc = concat!("A `", stringify!($name), "` needs to match the regular expression `", $regex, "`.")]
    #[derive(Clone, Debug, serde::Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize)]
    #[serde(try_from = "String")]
    pub struct $name(String);

    impl $name {
      #[doc = concat!("# Create new `", stringify!($name), "`")]
      #[doc = concat!("This static factory function is used to create a new `", stringify!($name), "` instance from the provided string slice, representing a ", $label, ".")]
      #[doc = concat!("The `id` argument will be validated against the regular expression `", $regex, "`.")]
      #[doc = "If validation fails, the function will panic."]
      #[doc = concat!("If you need a safe factory function, use one of the [`", stringify!($name), "::try_from()`] methods, which will return a `Result<", stringify!($name), ", String>`.")]
      #[doc = "## Examples"]
      #[doc = concat!("Create a `", stringify!($name), "` instance from a string slice:")]
      #[doc = "```rust"]
      #[doc = concat!("use ", $crate_name, "::", stringify!($name), ";")]
      #[doc = ""]
      #[doc = concat!("let id = ", stringify!($name), "::new(\"", $valid_id, "\");")]
      #[doc = concat!("println!(\"", $label, " is {}\", id);")]
      #[doc = "```"]
      #[doc = "## Parameters"]
      #[doc = concat!("* `id` - the ", $label, ".")]
      #[doc = "## Returns"]
      #[doc = concat!("* The created `", stringify!($name), "`, when validation was successful.")]
      #[doc = "* The function panics when the validation was not successful."]
      pub fn new(id: &str) -> Self {
        if Self::is_valid(id) {
          Self(id.to_string())
        } else {
          panic!("invalid {} '{}'", $label, id)
        }
      }

      #[doc = concat!("# Validate ", $label)]
      #[doc = concat!("This static function is used to check whether the provided string slice is a valid ", $label, ".")]
      #[doc = concat!("The `id` argument will be validated against the regular expression `", $regex, "`.")]
      #[doc = "When `id` is valid, the function will return `true`, else it will return `false`."]
      #[doc = concat!("If `is_valid(id)` returns `true`, [`", stringify!($name), "::new()`] will never panic and can be used safely.")]
      #[doc = "## Examples"]
      #[doc = concat!("Validate a ", $label, " string slice:")]
      #[doc = "```rust"]
      #[doc = concat!("use ", $crate_name, "::", stringify!($name), ";")]
      #[doc = ""]
      #[doc = concat!("assert!(", stringify!($name), "::is_valid(\"", $valid_id, "\"));")]
      #[doc = concat!("assert!(!", stringify!($name), "::is_valid(\"", $invalid_id, "\"));")]
      #[doc = "```"]
      #[doc = "## Parameters"]
      #[doc = concat!("* `id` - the ", $label, ".")]
      #[doc = "## Returns"]
      #[doc = concat!("* `true` - when `id` matches the regular expression `", $regex, "`,")]
      #[doc = "* `false` - otherwise."]
      pub fn is_valid(id: &str) -> bool {
        Self::regex().is_match(id)
      }

      #[doc = "# Returns the regular expression"]
      #[doc = concat!("This static function will return a reference to the regular expression used by this `", stringify!($name), "` to validate ", $label, " strings.")]
      #[doc = concat!("The textual representation of the regular expression is `", $regex, "`.")]
      #[doc = "## Examples"]
      #[doc = concat!("Validate a ", $label, " using the regular expression:")]
      #[doc = "```rust"]
      #[doc = "use regex::Regex;"]
      #[doc = concat!("use ", $crate_name, "::", stringify!($name), ";")]
      #[doc = ""]
      #[doc = concat!("let regex: &'static Regex = ", stringify!($name), "::regex();")]
      #[doc = concat!("assert!(regex.is_match(\"", $valid_id, "\"));")]
      #[doc = concat!("assert!(!regex.is_match(\"",  $invalid_id, "\"));")]
      #[doc = "```"]
      #[doc = "## Returns"]
      #[doc = concat!("The regular expression used by this `", stringify!($name), "`.")]
      pub fn regex() -> &'static regex::Regex {
        lazy_static::lazy_static! {
          static ref ID_REGEX: regex::Regex = regex::Regex::new($regex).unwrap();
        }
        &ID_REGEX
      }
    }

    impl TryFrom<&str> for $name {
      type Error = String;

      fn try_from(id: &str) -> Result<Self, Self::Error> {
        if Self::is_valid(id) {
          Ok(Self(id.to_string()))
        } else {
          Err(format!("invalid {} '{}'", $label, id))
        }
      }
    }

    impl TryFrom<String> for $name {
      type Error = String;

      fn try_from(id: String) -> Result<Self, Self::Error> {
        if Self::is_valid(id.as_str()) {
          Ok(Self(id))
        } else {
          Err(format!("invalid {} '{}'", $label, id))
        }
      }
    }

    impl core::str::FromStr for $name {
      type Err = String;

      fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }

    #[doc(hidden)]
    impl std::ops::Deref for $name {
      type Target = str;

      fn deref(&self) -> &Self::Target {
        &self.0.as_ref()
      }
    }
  };
}
