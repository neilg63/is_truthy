[![mirror](https://img.shields.io/badge/mirror-github-blue)](https://github.com/neilg63/is-truthy)
[![crates.io](https://img.shields.io/crates/v/is-truthy.svg)](https://crates.io/crates/is-truthy)
[![docs.rs](https://docs.rs/is-truthy/badge.svg)](https://docs.rs/is-truthy)

# is-truthy

Flexible boolean parsing from string values. Real-world CSVs, spreadsheets and database columns rarely store booleans as clean `true`/`false` — they use `1`/`0`, `yes`/`no`, `y`/`n`, checkmarks or entirely custom, language-specific markers. This crate covers all three levels: a language-neutral core, a common-English "standard" superset, and a fully custom rule set for anything else, built once and reused across many rows.

## Quick start

```rust
use is_truthy::IsTruthy;

assert_eq!("true".is_truthy(), Some(true));
assert_eq!("1".is_truthy(), Some(true));
assert_eq!("0".is_truthy(), Some(false));
assert_eq!("hello".is_truthy(), None); // not boolean-like at all
```

`is_truthy()` (and the other `IsTruthy` methods below) are extension methods available on any `T: AsRef<str>` — `&str`, `String`, etc. — via the `IsTruthy` trait.

## The three levels

### 1. Core — language-neutral, numeric-or-literal only

`is_truthy_core` (via `.is_truthy()` / `.is_truthy_core(empty_is_false)`) only recognises the literals `"true"`/`"false"` and the exact integers `0`/`1` by default — nothing else numeric is accepted:

```rust
use is_truthy::IsTruthy;

assert_eq!("true".is_truthy_core(false), Some(true));
assert_eq!("false".is_truthy_core(false), Some(false));
assert_eq!("1".is_truthy_core(false), Some(true));
assert_eq!("0".is_truthy_core(false), Some(false));
assert_eq!("-1".is_truthy_core(false), None); // outside the default 0..=1 range
assert_eq!("2".is_truthy_core(false), None);  // ditto
assert_eq!("99".is_truthy_core(false), None);
assert_eq!("hello".is_truthy_core(false), None);
```

The default is deliberately this tight rather than tolerant, because a value outside `0`/`1` is genuinely ambiguous rather than safely ignorable. Databases commonly store boolean state in a `tinyint`/8-bit signed integer column, and MariaDB/MySQL's own boolean-context truthiness treats *any* nonzero value — including negative ones — as true. But applications don't always agree: PHP casts `-1` to `true` the same way, yet has also used `-1` elsewhere (historically, in some APIs) as a "not found"/sentinel value with nothing to do with booleans. Given that ambiguity, guessing is worse than returning `None` and making the caller decide. If you know your own data's convention — e.g. that stray `-1`/`2` values nearby really do mean boolean drift from manual edits or a buggy bulk update, not a sentinel — widen the range explicitly with `is_truthy_in_range` or `TruthyRuleSet::set_min_max` below.

Empty (or whitespace-only) input returns `None` by default — meaning "not a boolean-like value at all" rather than `false`. Pass `empty_is_false: true` if your data treats a blank cell as false rather than missing:

```rust
use is_truthy::IsTruthy;

assert_eq!("".is_truthy_core(false), None);
assert_eq!("".is_truthy_core(true), Some(false));
assert_eq!("  ".is_truthy_core(true), Some(false)); // whitespace is trimmed first
```

Need a wider numeric range than the default 0..=1? Use `is_truthy_in_range`/`.is_truthy_in_range(empty_is_false, min, max)` directly, or see `TruthyRuleSet::set_min_max` below for the reusable-ruleset equivalent.

### 2. Standard — adds common English words and symbols

`is_truthy_standard` (via `.is_truthy_standard(empty_is_false)`) is a superset of core, adding common English markers and a handful of symbols:

```rust
use is_truthy::IsTruthy;

// falsy: "no", "not", "none", "n", "✗", "✕", "☒", "❌"
assert_eq!("no".is_truthy_standard(false), Some(false));
assert_eq!("n".is_truthy_standard(false), Some(false));
assert_eq!("none".is_truthy_standard(false), Some(false));

// truthy: "ok", "okay", "y", "yes", "✓", "☑", "✔", "✅"
assert_eq!("yes".is_truthy_standard(false), Some(true));
assert_eq!("y".is_truthy_standard(false), Some(true));
assert_eq!("Ok".is_truthy_standard(false), Some(true)); // case-insensitive

// core values still work underneath
assert_eq!("true".is_truthy_standard(false), Some(true));
assert_eq!("0".is_truthy_standard(false), Some(false));

// unrecognised
assert_eq!("maybe".is_truthy_standard(false), None);
```

### 3. Custom — your own rules, with optional fallback

For anything language-specific, domain-specific, or driven by end-user configuration, build a `TruthyRuleSet` once and reuse it across every row of a batch:

```rust
use is_truthy::{IsTruthy, TruthyRuleSet};

let rules = TruthyRuleSet::new()
    .add_true("si")
    .add_false("no")
    .use_standard();

// your custom patterns match first
assert_eq!(rules.parse("si"), Some(true));
assert_eq!(rules.parse("no"), Some(false));
// then it falls through to the standard English words...
assert_eq!(rules.parse("yes"), Some(true));
assert_eq!(rules.parse("n"), Some(false));
// ...and the core numeric/literal values
assert_eq!(rules.parse("true"), Some(true));
assert_eq!(rules.parse("0"), Some(false));
// still unrecognised
assert_eq!(rules.parse("forse"), None);
```

`.is_truthy_custom(&rules)` is the equivalent extension method if you'd rather call it on the string directly: `"si".is_truthy_custom(&rules)`.

#### Fallback behaviour is opt-in and layered

A bare `TruthyRuleSet` with only custom patterns has **no fallback** — anything not explicitly listed returns `None`:

```rust
use is_truthy::TruthyRuleSet;

let rules = TruthyRuleSet::new().add_true("si").add_false("no");

assert_eq!(rules.parse("si"), Some(true));
assert_eq!(rules.parse("no"), Some(false));
assert_eq!(rules.parse("yes"), None); // no fallback configured
assert_eq!(rules.parse("1"), None);
assert_eq!(rules.parse(""), None);
```

`.use_defaults()` adds the core numeric/literal fallback (`true`/`false`/`0`/`1`) without the standard English words:

```rust
use is_truthy::TruthyRuleSet;

let rules = TruthyRuleSet::new()
    .add_true("ok")
    .add_false("fail")
    .use_defaults();

assert_eq!(rules.parse("ok"), Some(true));
assert_eq!(rules.parse("fail"), Some(false));
assert_eq!(rules.parse("1"), Some(true));
assert_eq!(rules.parse("true"), Some(true));
// standard English words are NOT recognised without use_standard()
assert_eq!(rules.parse("yes"), None);
assert_eq!(rules.parse("n"), None);
```

`.use_standard()` implies `.use_defaults()` too, so it gives you all three levels: custom patterns, then standard English/symbols, then core.

`.empty_is_false()` treats blank (or whitespace-only) input as `false` rather than `None`, consistently across whichever fallback level you've enabled:

```rust
use is_truthy::TruthyRuleSet;

let rules = TruthyRuleSet::new()
    .add_true("ok")
    .add_false("fail")
    .empty_is_false()
    .use_defaults();

assert_eq!(rules.parse(""), Some(false));
assert_eq!(rules.parse("  "), Some(false));
```

`.set_min_max(min, max)` overrides the core numeric range (default 0..=1) for this rule set — useful if you know your own data's convention makes a wider range safe (see the note on why the default is deliberately tight, above). `min` must be `<= 0`, `max` must be `>= 1`, and `min < max` — an invalid range is silently ignored and the default is kept:

```rust
use is_truthy::TruthyRuleSet;

let rules = TruthyRuleSet::new()
    .add_true("ok")
    .add_false("fail")
    .use_defaults()
    .set_min_max(-3, 3);

assert_eq!(rules.min(), -3);
assert_eq!(rules.max(), 3);
assert_eq!(rules.parse("3"), Some(true));
assert_eq!(rules.parse("-3"), Some(false));
```

### Match modes and case sensitivity

Each custom option can match a pattern as an exact whole-string match (the default), or as a prefix, suffix, or substring, and independently as case-sensitive or case-insensitive (the default). Case-insensitive matching also ignores punctuation and spaces (it's built on `simple-string-patterns`'s alphanumeric-aware comparisons), so e.g. an `Exact`, case-insensitive `"ok"` option also matches `"o.k!"`.

```rust
use is_truthy::{MatchMode, TruthyRuleSet};

let rules = TruthyRuleSet::new()
    .add_true_option("posi", MatchMode::StartsWith, false)
    .add_false_option("neg", MatchMode::StartsWith, false)
    .add_true_option("baar", MatchMode::EndsWith, false)
    .add_false_option("kötü", MatchMode::Contains, false)
    .add_true_option("Oui", MatchMode::Exact, true); // case-sensitive

assert_eq!(rules.parse("Positif"), Some(true));   // starts_with "posi", case-insensitive
assert_eq!(rules.parse("negative"), Some(false)); // starts_with "neg"
assert_eq!(rules.parse("foobaar"), Some(true));   // ends_with "baar"
assert_eq!(rules.parse("baarx"), None);           // doesn't end with "baar"
assert_eq!(rules.parse("çok kötü"), Some(false)); // contains "kötü"
assert_eq!(rules.parse("herkötüdür"), Some(false));
assert_eq!(rules.parse("Oui"), Some(true));       // exact, case-sensitive
assert_eq!(rules.parse("oui"), None);             // wrong case, and case-sensitive
```

| `MatchMode` | Meaning |
| ----------- | ------- |
| `Exact` (default) | The whole (trimmed) string must match the pattern |
| `StartsWith` | The string must start with the pattern |
| `EndsWith` | The string must end with the pattern |
| `Contains` | The pattern may appear anywhere in the string |

### Building options directly

`add_true`/`add_false`/`add_true_option`/`add_false_option` cover the common cases. For full control, build a `TruthyOption` directly and add it with `.add(opt)`:

```rust
use is_truthy::{MatchMode, TruthyOption, TruthyRuleSet};

let opt = TruthyOption::new_true("ready").match_mode(MatchMode::Contains).case_sensitive();
let rules = TruthyRuleSet::new().add(opt).add_false("not-ready");
```

## Traits and functions

| Name | Description |
| ---- | ----------- |
| `IsTruthy` | Extension trait adding `is_truthy`, `is_truthy_core`, `is_truthy_in_range`, `is_truthy_standard` and `is_truthy_custom` to any `T: AsRef<str>` |
| `is_truthy_core` / `is_truthy_in_range` | Free-function core matcher: `"true"`/`"false"` literals plus a configurable low-integer range (default 0..=1) |
| `is_truthy_standard` | Free-function standard matcher: core, plus common English words and check/cross symbols |
| `is_truthy_custom` | Free-function custom matcher taking a `&TruthyRuleSet` |
| `TruthyRuleSet` | Builder for a reusable set of custom true/false patterns with optional standard/core fallback — build once, `.parse()` many times |
| `TruthyOption` | A single pattern to match, with its truth value, `MatchMode` and case sensitivity |
| `MatchMode` | `Exact` / `StartsWith` / `EndsWith` / `Contains` — how a `TruthyOption`'s pattern is matched |
| `has_true_and_false_options` | Returns `true` only if a slice of `TruthyOption`s has at least one true *and* one false option — a rule set with only one side never matches anything, by design |

## A note on scope

This crate only classifies a single string value. Parsing a UI- or config-driven mini-language into a `TruthyRuleSet` (e.g. a settings string like `"truthy:ok,good|failed,bad"`) is deliberately left to the application that owns that format, rather than built into this crate.
