use std::sync::Arc;

use alphanumeric::StripCharacters;
use simple_string_patterns::SimpleMatch;
use to_segments::ToSegments;

const DEFAULT_NUM_MIN: i8 = 0;
const DEFAULT_NUM_MAX: i8 = 1;

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
    rules: &TruthyRuleSet,
) -> Option<bool> {
    if !has_true_and_false_options(&rules.options()) {
        return None;
    }
    let txt = txt.trim();
    for opt in rules.options() {
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
    if rules.use_defaults  {
        if let Some(core_match) = is_truthy_in_range(txt, rules.empty_is_false, rules.min(), rules.max()) {
            return Some(core_match);
        }
    }
    if rules.use_standard  {
        if let Some(core_match) = is_truthy_standard(txt, rules.empty_is_false) {
            return Some(core_match);
        }
    }
    None
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
        rules: &TruthyRuleSet,
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
        rules: &TruthyRuleSet,
    ) -> Option<bool> {
        is_truthy_custom(self.as_ref(), rules)
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

    pub fn set_case_sensitive(mut self, value: bool) -> Self {
        if self.case_sensitive != value {
            self.case_sensitive = value;
        }
        self
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

/// Bundles custom truthy/falsy patterns with fallback behaviour.
/// Build once, reuse across many rows.
#[derive(Debug, Clone)]
pub struct TruthyRuleSet {
    opts: Vec<TruthyOption>,
    use_defaults: bool,
    use_standard: bool,
    empty_is_false: bool,
    min_max: Option<(i8, i8)>,
}

impl TruthyRuleSet {
    pub fn new() -> Self {
        Self {
            opts: vec![],
            use_defaults: false,
            use_standard: false,
            empty_is_false: false,
            min_max: None,
        }
    }

    /// Add simple true option with default case-insensitive exact matching.
    pub fn add_true(mut self, pattern: &str) -> Self {
        self.opts.push(TruthyOption::new_true(pattern));
        self
    }

    // Add simple true option with match mode and case sensitivity specified.
    pub fn add_true_option(mut self, pattern: &str, mode: MatchMode, case_sensitive: bool) -> Self {
        self.opts.push(TruthyOption::new_true(pattern).match_mode(mode).set_case_sensitive(case_sensitive));
        self
    }

    /// Add simple false option with default case-insensitive exact matching.
    pub fn add_false(mut self, pattern: &str) -> Self {
        self.opts.push(TruthyOption::new_false(pattern));
        self
    }

    /// Add simple false option with match mode and case sensitivity specified.
    pub fn add_false_option(mut self, pattern: &str, mode: MatchMode, case_sensitive: bool) -> Self {
        self.opts.push(TruthyOption::new_false(pattern).match_mode(mode).set_case_sensitive(case_sensitive));
        self
    }

    pub fn add(mut self, opt: TruthyOption) -> Self {
        self.opts.push(opt);
        self
    }

    /// Set empty strings to be treated as false.
    pub fn empty_is_false(mut self) -> Self {
        self.empty_is_false = true;
        self
    }

    /// Use default language-neutraltruthy/falsy options (0, 1, true, false) as a fallback if no custom options match.
    pub fn use_defaults(mut self) -> Self {
        self.use_defaults = true;
        self
    }

    /// Use common English-medium truthy/falsy options 'y', 'n', 'yes', 'no', 'ok', 'none', 'not' + a few emojis as a fallback if no custom options match.
    pub fn use_standard(mut self) -> Self {
        self.use_standard = true;
        self.use_defaults = true;
        self
    }


    /// Define a custom range of integers that are considered truthy/falsy.
    /// Numeric strings outside this range will be ignored.
    pub fn set_min_max(mut self, min: i8, max: i8) -> Self {
        if max > min && min <= 0 && max >= 1 {
            self.min_max = Some((min, max));
        }
        self
    }

    pub fn min(&self) -> i8 {
        if let Some((min, _max)) = self.min_max {
            min
        } else {
            DEFAULT_NUM_MIN
        }
    }

    pub fn max(&self) -> i8 {
        if let Some((_min, max)) = self.min_max {
            max
        } else {
            DEFAULT_NUM_MAX
        }
    }

    /// Parse a string using the custom truthy/falsy options, with optional fallbacks to standard or default options.
    pub fn parse(&self, txt: &str) -> Option<bool> {
        if has_true_and_false_options(&self.opts) {
            let result = is_truthy_custom(txt, &self);
            if result.is_some() {
                return result;
            }
        }
        if self.use_standard {
            is_truthy_standard(txt, self.empty_is_false)
        } else if self.use_defaults {
            is_truthy_core(txt, self.empty_is_false)
        } else {
            None
        }
    }

    /// Get all options in this ruleset, including true and false options.
    pub fn options(&self) -> &[TruthyOption] {
        &self.opts
    }
}

pub fn extract_truth_patterns(opts: &[TruthyOption], is_true: bool) -> Vec<String> {
    opts.iter()
        .filter(|o| o.is_true == is_true)
        .map(|o| o.pattern.to_string())
        .collect()
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

    // --- IsTruthy trait ---

    #[test]
    fn test_is_truthy_default() {
        // is_truthy() delegates to is_truthy_core with empty_is_false = false
        assert_eq!("true".is_truthy(), Some(true));
        assert_eq!("false".is_truthy(), Some(false));
        assert_eq!("1".is_truthy(), Some(true));
        assert_eq!("0".is_truthy(), Some(false));
        // -1 is outside the default 0..=1 range now: could mean "true" under naive
        // MySQL/PHP nonzero-is-true casting, or a "not found"/sentinel value from
        // elsewhere — ambiguous enough that the default should refuse to guess.
        assert_eq!("-1".is_truthy(), None);
        assert_eq!("".is_truthy(), None);
        assert_eq!("  ".is_truthy(), None);
        assert_eq!("hello".is_truthy(), None);
        assert_eq!("99".is_truthy(), None);
        assert_eq!(String::from("true").is_truthy(), Some(true));
    }

    #[test]
    fn test_is_truthy_core_empty_handling() {
        assert_eq!("".is_truthy_core(false), None);
        assert_eq!("".is_truthy_core(true), Some(false));
    }

    #[test]
    fn test_is_truthy_standard() {
        // standard is a superset of core
        assert_eq!("yes".is_truthy_standard(false), Some(true));
        assert_eq!("y".is_truthy_standard(false), Some(true));
        assert_eq!("Ok".is_truthy_standard(false), Some(true));
        assert_eq!("n".is_truthy_standard(false), Some(false));
        assert_eq!("no".is_truthy_standard(false), Some(false));
        assert_eq!("none".is_truthy_standard(false), Some(false));
        // core values still work
        assert_eq!("true".is_truthy_standard(false), Some(true));
        assert_eq!("1".is_truthy_standard(false), Some(true));
        assert_eq!("0".is_truthy_standard(false), Some(false));
        // unrecognised
        assert_eq!("maybe".is_truthy_standard(false), None);
    }

    #[test]
    fn test_is_truthy_custom() {
        // standard is a superset of core
        let rules = TruthyRuleSet::new()
            .add_true("oui")
            .add_false("non")
            .use_standard();
        let text_true_strings = ["oui", "yes", "y", "1", "true"];
        for txt in text_true_strings {
            assert_eq!(txt.is_truthy_custom(&rules), Some(true), "Failed true for input: {}", txt);
        }
        let text_false_strings = ["non", "no", "n", "0", "false"];
        for txt in text_false_strings {
            assert_eq!(txt.is_truthy_custom(&rules), Some(false), "Failed false for input: {}", txt);
        }
    }


    // --- TruthyRuleSet ---

    #[test]
    fn test_set_min_max_actually_applies() {
        // set_min_max's guard condition used to be `min > max && min <= 0 && max >= 1`,
        // which is a contradiction (min <= 0 && max >= 1 implies min < max) and so could
        // never be true for any input — the custom range silently never took effect.
        let rules = TruthyRuleSet::new()
            .add_true("ok")
            .add_false("fail")
            .use_defaults()
            .set_min_max(-3, 3);
        assert_eq!(rules.min(), -3);
        assert_eq!(rules.max(), 3);
        assert_eq!(rules.parse("3"), Some(true));
        assert_eq!(rules.parse("-3"), Some(false));
        // out of the custom range now, even though it was in the old hardcoded default
        assert_eq!(rules.parse("2"), Some(true));

        // an invalid range (min >= max, or outside the 0/1 straddle) leaves the default
        let unchanged = TruthyRuleSet::new().set_min_max(5, -3);
        assert_eq!(unchanged.min(), DEFAULT_NUM_MIN);
        assert_eq!(unchanged.max(), DEFAULT_NUM_MAX);
    }

    #[test]
    fn test_ruleset_minimal() {
        // one true + one false pattern is all you need
        let rules = TruthyRuleSet::new()
            .add_true("si")
            .add_false("no");

        assert_eq!(rules.parse("si"), Some(true));
        assert_eq!(rules.parse("no"), Some(false));
        // no fallback — unrecognised returns None
        assert_eq!(rules.parse("yes"), None);
        assert_eq!(rules.parse("1"), None);
        assert_eq!(rules.parse(""), None);
    }

    #[test]
    fn test_ruleset_use_standard() {
        // custom patterns + standard English + core numeric/boolean
        let rules = TruthyRuleSet::new()
            .add_true("si")
            .add_false("no")
            .use_standard();

        // custom patterns match first
        assert_eq!(rules.parse("si"), Some(true));
        assert_eq!(rules.parse("no"), Some(false));
        // standard English words
        assert_eq!(rules.parse("yes"), Some(true));
        assert_eq!(rules.parse("n"), Some(false));
        // core values via standard fallback
        assert_eq!(rules.parse("true"), Some(true));
        assert_eq!(rules.parse("0"), Some(false));
        // unrecognised
        assert_eq!(rules.parse("forse"), None);
    }

    #[test]
    fn test_ruleset_use_defaults_without_standard() {
        // custom + core (true/false/0/1) but NOT English words
        let rules = TruthyRuleSet::new()
            .add_true("ok")
            .add_false("fail")
            .use_defaults();

        assert_eq!(rules.parse("ok"), Some(true));
        assert_eq!(rules.parse("fail"), Some(false));
        assert_eq!(rules.parse("1"), Some(true));
        assert_eq!(rules.parse("true"), Some(true));
        // standard words are NOT recognised
        assert_eq!(rules.parse("yes"), None);
        assert_eq!(rules.parse("n"), None);
    }

    #[test]
    fn test_ruleset_empty_is_false() {
        let rules = TruthyRuleSet::new()
            .add_true("ok")
            .add_false("fail")
            .empty_is_false()
            .use_defaults();

        assert_eq!(rules.parse(""), Some(false));
        assert_eq!(rules.parse("  "), Some(false));
    }

    #[test]
    fn test_ruleset_with_match_modes() {
        let rules = TruthyRuleSet::new()
            .add_true_option("posi",MatchMode::StartsWith, false)
            .add_false_option("neg",MatchMode::StartsWith, false)
            .add_true_option("baar",MatchMode::EndsWith, false)
            .add_false_option("kötü",MatchMode::Contains, false)
            .add_true_option("Oui",MatchMode::Exact, true);

        assert_eq!(rules.parse("Positif"), Some(true));
        assert_eq!(rules.parse("negative"), Some(false));
        assert_eq!(rules.parse("foobaar"), Some(true));
        assert_eq!(rules.parse("baarx"), None);
        assert_eq!(rules.parse("çok kötü"), Some(false));
        assert_eq!(rules.parse("herkötüdür"), Some(false));
        assert_eq!(rules.parse("Oui"), Some(true));
        assert_eq!(rules.parse("oui"), None);
    }
}
