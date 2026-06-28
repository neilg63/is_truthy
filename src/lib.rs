use std::sync::Arc;

use alphanumeric::StripCharacters;
use simple_string_patterns::SimpleMatch;
use to_segments::ToSegments;

const DEFAULT_NUM_MIN: i8 = -2;
const DEFAULT_NUM_MAX: i8 = 2;

/// Language-agnostic truthy/falsy string matcher.
/// Only strings representing low integers (-2 to 2), as well as case-insensitive "true" and "false"
/// This calls the more general `is_truthy_in_range` function with default min/max integer values.
pub fn is_truthy_core(txt: &str, empty_is_false: bool) -> Option<bool> {
    is_truthy_in_range(txt, empty_is_false, DEFAULT_NUM_MIN, DEFAULT_NUM_MAX)
}

pub fn is_truthy_in_range(txt: &str, empty_is_false: bool, min: i8, max: i8) -> Option<bool> {
    let test_str = txt.trim().to_lowercase();
    match test_str.as_str() {
        "" => {
            if empty_is_false {
                Some(false)
            } else {
                None
            }
        }
        "false" => Some(false),
        "true" => Some(true),
        _ => {
            if let Some(num) = test_str.to_first_number::<i8>() {
                if num >= min && num <= max {
                    Some(num > 0)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

/*
    Standard truthy/falsy string matcher, interpreting generic
    English or international truthy/falsy markers.
    This calls the more general `is_truthy_core` function, and adds additional common true/false strings.
*/
pub fn is_truthy_standard(txt: &str, empty_is_false: bool) -> Option<bool> {
    if let Some(core_match) = is_truthy_core(txt, empty_is_false) {
        Some(core_match)
    } else {
        match txt.trim().to_lowercase().as_str() {
            "no" | "not" | "none" | "n" | "✗" | "✕" | "☒" | "❌" => Some(false),
            "ok" | "okay" | "y" | "yes" | "✓" | "☑" | "✔" | "✅" => Some(true),
            _ => None,
        }
    }
}

// Check if the provided list of TruthyOptions contains at least
// one true and one false option.
pub fn has_true_and_false_options(opts: &[TruthyOption]) -> bool {
    opts.iter().any(|o| o.is_true) && opts.iter().any(|o| !o.is_true)
}

pub fn is_truthy_custom(
    txt: &str,
    opts: &[TruthyOption],
    use_defaults: bool,
    empty_is_false: bool,
) -> Option<bool> {
    if !has_true_and_false_options(opts) {
        return None;
    }
    let txt = txt.trim();
    for opt in opts {
        let pattern = opt.pattern();
        let matched = if opt.case_sensitive {
            match opt.match_mode {
                MatchMode::Exact => txt == pattern,
                MatchMode::StartsWith => txt.starts_with(pattern),
                MatchMode::EndsWith => txt.ends_with(pattern),
                MatchMode::Contains => txt.contains(pattern),
            }
        } else {
            match opt.match_mode {
                MatchMode::Exact => txt.equals_ci_alphanum(pattern),
                MatchMode::StartsWith => txt.starts_with_ci_alphanum(pattern),
                MatchMode::EndsWith => txt.ends_with_ci_alphanum(pattern),
                MatchMode::Contains => txt.contains_ci_alphanum(pattern),
            }
        };
        if matched {
            return Some(opt.is_true);
        }
    }
    if use_defaults {
        is_truthy_core(txt, empty_is_false)
    } else {
        None
    }
}

pub trait IsTruthy {
    /// Default implementation of is_truthy()
    /// uses the core truthy/falsy matcher and assumes that empty strings are not false.
    fn is_truthy(&self) -> Option<bool> {
        self.is_truthy_core(false)
    }
    /// Default implementation of is_truthy_in_range()
    /// only accepting low integers (-2 to 2) as valid truthy/falsy values.
    fn is_truthy_core(&self, empty_is_false: bool) -> Option<bool>;
    /// Lets you define a custom range of integers that are considered truthy/falsy.
    fn is_truthy_in_range(&self, empty_is_false: bool, min: i8, max: i8) -> Option<bool>;
    /// Default implementation of is_truthy_standard()
    /// uses the standard truthy/falsy matcher.
    /// with common English and international truthy/falsy markers.
    fn is_truthy_standard(&self, empty_is_false: bool) -> Option<bool>;
    fn is_truthy_custom(
        &self,
        opts: &[TruthyOption],
        use_defaults: bool,
        empty_is_false: bool,
    ) -> Option<bool>;
}

impl<T: AsRef<str>> IsTruthy for T {
    fn is_truthy_core(&self, empty_is_false: bool) -> Option<bool> {
        is_truthy_core(self.as_ref(), empty_is_false)
    }
    fn is_truthy_in_range(&self, empty_is_false: bool, min: i8, max: i8) -> Option<bool> {
        is_truthy_in_range(self.as_ref(), empty_is_false, min, max)
    }
    fn is_truthy_standard(&self, empty_is_false: bool) -> Option<bool> {
        is_truthy_standard(self.as_ref(), empty_is_false)
    }
    fn is_truthy_custom(
        &self,
        opts: &[TruthyOption],
        use_defaults: bool,
        empty_is_false: bool,
    ) -> Option<bool> {
        is_truthy_custom(self.as_ref(), opts, use_defaults, empty_is_false)
    }
}

/// How a TruthyOption pattern is matched against input text
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MatchMode {
    #[default]
    Exact,
    StartsWith,
    EndsWith,
    Contains,
}

impl MatchMode {
    pub fn from_flag(starts_with: bool) -> Self {
        if starts_with {
            MatchMode::StartsWith
        } else {
            MatchMode::Exact
        }
    }
}

/// Flexible boolean pattern matcher.
/// Defaults to case-insensitive exact matching.
#[derive(Debug, Clone)]
pub struct TruthyOption {
    pub is_true: bool,
    pub pattern: Arc<str>,
    pub case_sensitive: bool,
    pub match_mode: MatchMode,
}

impl TruthyOption {
    pub fn new_true(pattern: &str) -> Self {
        Self {
            is_true: true,
            pattern: Arc::from(pattern),
            case_sensitive: false,
            match_mode: MatchMode::default(),
        }
    }

    pub fn new_false(pattern: &str) -> Self {
        Self {
            is_true: false,
            pattern: Arc::from(pattern),
            case_sensitive: false,
            match_mode: MatchMode::default(),
        }
    }

    pub fn match_mode(mut self, mode: MatchMode) -> Self {
        self.match_mode = mode;
        self
    }

    pub fn case_sensitive(mut self) -> Self {
        self.case_sensitive = true;
        self
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

pub fn extract_truth_patterns(opts: &[TruthyOption], is_true: bool) -> Vec<String> {
    opts.iter()
        .filter(|o| o.is_true == is_true)
        .map(|o| o.pattern.to_string())
        .collect()
}

/// Convert a custom string setting into a full set of TruthyOptions.
/// e.g. "truthy:ok,good|failed,bad" translates into two true options (ok, good)
/// and two false options (failed, bad).
/// case_sensitive and match_mode are applied globally.
pub fn split_truthy_custom_option_str(
    custom_str: &str,
    case_sensitive: bool,
    match_mode: MatchMode,
) -> Vec<TruthyOption> {
    if let (Some(head), Some(tail)) = custom_str.to_head_tail(":") {
        if let (Some(first), second) = tail.to_head_tail(",") {
            if !first.is_empty() && head.starts_with_ci_alphanum("tr") {
                return to_truth_options(first, second.unwrap_or(""), case_sensitive, match_mode);
            }
        }
    }
    vec![]
}

/// Split a comma-separated string of true and false options into a list of TruthyOptions.
/// Alternative true or false options may be split by | (pipe) characters.
pub fn to_truth_options(
    true_str: &str,
    false_str: &str,
    case_sensitive: bool,
    match_mode: MatchMode,
) -> Vec<TruthyOption> {
    let build = |is_true: bool, pattern: &str| -> TruthyOption {
        let opt = if is_true {
            TruthyOption::new_true(pattern)
        } else {
            TruthyOption::new_false(pattern)
        }
        .match_mode(match_mode);
        if case_sensitive {
            opt.case_sensitive()
        } else {
            opt
        }
    };
    let mut matchers: Vec<TruthyOption> = vec![];
    for match_str in true_str.to_segments("|") {
        matchers.push(build(true, &match_str));
    }
    for match_str in false_str.to_segments("|") {
        matchers.push(build(false, &match_str));
    }
    matchers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truthy_core() {
        assert_eq!("1".is_truthy_core(false), Some(true));
        assert_eq!("0".is_truthy_core(false), Some(false));
        assert_eq!("false".is_truthy_core(false), Some(false));
        assert_eq!("true".is_truthy_core(false), Some(true));
        assert_eq!("-1".is_truthy_core(false), Some(false));
        assert_eq!("99".is_truthy_core(false), None);
        assert_eq!("".is_truthy_core(false), None);
        assert_eq!("".is_truthy_core(true), Some(false));
    }

    #[test]
    fn test_truthy_standard() {
        assert_eq!("n".is_truthy_standard(false), Some(false));
        assert_eq!("Ok".is_truthy_standard(false), Some(true));
        assert_eq!("yes".is_truthy_standard(false), Some(true));
        assert_eq!("false".is_truthy_standard(false), Some(false));
        assert_eq!("maybe".is_truthy_standard(false), None);
    }

    #[test]
    fn test_builder_pattern() {
        let opts = vec![
            TruthyOption::new_true("posi").match_mode(MatchMode::StartsWith),
            TruthyOption::new_false("neg").match_mode(MatchMode::StartsWith),
            TruthyOption::new_true("baar").match_mode(MatchMode::EndsWith),
            TruthyOption::new_false("kötü").match_mode(MatchMode::Contains),
            TruthyOption::new_true("Oui").case_sensitive(),
        ];

        // starts_with matches (case-insensitive)
        assert_eq!("Positif".is_truthy_custom(&opts, false, false), Some(true));
        assert_eq!(
            "negative".is_truthy_custom(&opts, false, false),
            Some(false)
        );

        // ends_with match
        assert_eq!("foobaar".is_truthy_custom(&opts, false, false), Some(true));
        assert_eq!("baar".is_truthy_custom(&opts, false, false), Some(true));
        assert_eq!("baarx".is_truthy_custom(&opts, false, false), None);

        // contains match
        assert_eq!(
            "çok kötü".is_truthy_custom(&opts, false, false),
            Some(false)
        );
        assert_eq!(
            "herkötüdür".is_truthy_custom(&opts, false, false),
            Some(false)
        );

        // case-sensitive exact: must match case
        assert_eq!("Oui".is_truthy_custom(&opts, false, false), Some(true));
        assert_eq!("oui".is_truthy_custom(&opts, false, false), None);

        // no match returns None
        assert_eq!("maybe".is_truthy_custom(&opts, false, false), None);
    }

    #[test]
    fn test_truthy_custom_from_str() {
        let custom_setting_str = "truthy:si|vero,no|falso";
        let custom_flags =
            split_truthy_custom_option_str(custom_setting_str, false, MatchMode::Exact);

        assert_eq!("yes".is_truthy_custom(&custom_flags, true, false), None);
        assert_eq!("false".is_truthy_custom(&custom_flags, false, false), None);
        assert_eq!("si".is_truthy_custom(&custom_flags, true, true), Some(true));
    }
}
