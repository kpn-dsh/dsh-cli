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
      #[doc = concat!("# Create a new `", stringify!($name), "`")]
      #[doc = ""]
      #[doc = concat!("This static method is used to create a new `", stringify!($name), "` instance from the provided `&str`, representing a ", $label, ".")]
      #[doc = concat!("The `id` argument will be validated against the regular expression `", $regex, "`. When the validation fails, the function will panic.")]
      #[doc = concat!("When you need a safe constructor, use one of the [`", stringify!($name), "::try_from()`] methods.")]
      #[doc = ""]
      #[doc = "## Parameters"]
      #[doc = concat!("* `id` - the ", $label, ".")]
      #[doc = ""]
      #[doc = "## Returns"]
      #[doc = concat!("* The created `", stringify!($name), "`, when validation was successful.")]
      #[doc = concat!("* The function panics when the validation was not successful.")]
      pub fn new(id: &str) -> Self {
        if Self::is_valid(id) {
          Self(id.to_string())
        } else {
          panic!("invalid {} '{}'", $label, id)
        }
      }

      #[doc = concat!("# Validates a ", $label)]
      #[doc = ""]
      #[doc = concat!("This static method is used to check whether the provided `&str` is a valid ", $label, ".")]
      #[doc = concat!("The `id` argument will be validated against the regular expression `", $regex, "`. When the validation is successful, the function will return `true`, else it returns `false`.")]
      #[doc = concat!("If `is_valid()` returns `true`, the `", stringify!($name), "::new()` function will never panic and can be safely used.")]
      #[doc = ""]
      #[doc = "## Parameters"]
      #[doc = concat!("* `id` - the ", $label, ".")]
      #[doc = ""]
      #[doc = "## Returns"]
      #[doc = concat!("* `true` - when `id` matches the regular expression `", $regex, "`,")]
      #[doc = concat!("* `false` - otherwise.")]
      pub fn is_valid(id: &str) -> bool {
        Self::regex().is_match(id)
      }

      #[doc = "# Returns the regular expression"]
      #[doc = ""]
      #[doc = concat!("This static method will return a reference to the regular expression used by this `", stringify!($name), "` to validate a ", $label, " string.")]
      #[doc = concat!("The textual representation of the regular expression is `", $regex, "`.")]
      #[doc = ""]
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
