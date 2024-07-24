#[macro_export]
macro_rules! identifier {
  ($name:ident, $label:literal, $regex:literal) => {
    #[doc = concat!("# `", stringify!($name), "`")]
    #[doc = ""]
    #[doc = concat!("A `", stringify!($name), "` is used to create, represent, validate and display a ", $label, ".")]
    #[doc = ""]
    #[doc = "## Validation rules"]
    #[doc = ""]
    #[doc = concat!("A `", stringify!($name), "` needs to match the regular expression `", $regex, "`.")]
    #[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    pub struct $name(String);

    impl $name {
      #[doc = concat!("# Create new `", stringify!($name), "`")]
      #[doc = concat!("This static factory function is used to create a new `", stringify!($name), "` instance from the provided string slice, representing a ", $label, ".")]
      #[doc = concat!("The `id` argument will be validated against the regular expression `", $regex, "`.")]
      #[doc = "If validation fails, the function will panic."]
      #[doc = concat!("If you need a safe factory function, use one of the [`", stringify!($name), "::try_from()`] methods, which will return a `Result<", stringify!($name), ", String>`.")]
      #[doc = "## Examples"]
      #[doc = concat!("Create a `", stringify!($name), "` instance from a string slice:")]
      #[doc = "```"]
      #[doc = concat!("let id = ", stringify!($name), "::new(\"abcdefgh\");")]
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
      #[doc = concat!("If `is_valid(id)` returns `true`, [`", stringify!($name), "::new(id)`] will never panic and can be used safely.")]
      #[doc = "## Examples"]
      #[doc = concat!("Validate a ", $label, " string slice:")]
      #[doc = "```"]
      #[doc = concat!("assert!(", stringify!($name), "::is_valid(\"abcdefgh\"));")]
      #[doc = concat!("assert!(!", stringify!($name), "::is_valid(\"12345678\"));")]
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
      #[doc = "```"]
      #[doc = concat!("let regex: &'static Regex = ", stringify!($name), "::regex();")]
      #[doc = concat!("assert!(regex.is_match(\"abcdefgh\"));")]
      #[doc = concat!("assert!(!regex.is_match(\"12345678\"));")]
      #[doc = "```"]
      #[doc = "## Returns"]
      #[doc = concat!("The regular expression used by this `", stringify!($name), "`.")]
      pub fn regex() -> &'static regex::Regex {
        lazy_static! {
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

    impl Display for $name {
      fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }

    #[doc(hidden)]
    impl Deref for $name {
      type Target = str;

      fn deref(&self) -> &Self::Target {
        &self.0.as_ref()
      }
    }
  };
}
