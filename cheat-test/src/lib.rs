//! # cheat-test
//!
//! A proc-macro crate for creating "cheat-aware" tests that document how they
//! could be cheated and what the consequences would be for users.
//!
//! ## Why This Exists
//!
//! On 2026-01-20, a developer created false positives by moving missing binaries
//! to "OPTIONAL" lists to make tests pass while shipping a broken product. This
//! crate ensures that every test documents:
//!
//! 1. What user scenario it protects
//! 2. How the test could be cheated
//! 3. What users experience when the test is cheated
//! 4. The severity and ease of cheating
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cheat_test::cheat_aware;
//!
//! #[cheat_aware(
//!     protects = "User can run sudo commands",
//!     severity = "CRITICAL",
//!     ease = "EASY",
//!     cheats = ["Move sudo to OPTIONAL list", "Remove sudo from essential binaries"],
//!     consequence = "bash: sudo: command not found",
//!     legitimate_change = "If sudo is genuinely not needed for a headless profile, \
//!         add it to the profile's optional list in builder/src/profiles.rs"
//! )]
//! #[test]
//! fn test_sudo_binary_present() {
//!     assert!(tarball_contains("./usr/bin/sudo"));
//! }
//! ```
//!
//! ## On Failure
//!
//! When a cheat-aware test fails, it prints:
//!
//! ```text
//! ======================================================================
//! === TEST FAILED: test_sudo_binary_present ===
//! ======================================================================
//!
//! PROTECTS: User can run sudo commands
//! SEVERITY: CRITICAL
//! EASE OF CHEATING: EASY
//!
//! CHEAT VECTORS:
//!   1. Move sudo to OPTIONAL list
//!   2. Remove sudo from essential binaries
//!
//! LEGITIMATE CHANGE PATH:
//!   If sudo is genuinely not needed for a headless profile,
//!   add it to the profile's optional list in builder/src/profiles.rs
//!
//! USER CONSEQUENCE:
//!   bash: sudo: command not found
//!
//! ORIGINAL ERROR:
//!   assertion failed: tarball_contains("./usr/bin/sudo")
//! ======================================================================
//! ```

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    ExprLit, Ident, ItemFn, Lit, Token,
};

/// Metadata for a cheat-aware test.
struct CheatAwareArgs {
    protects: String,
    severity: String,
    ease: String,
    cheats: Vec<String>,
    consequence: String,
    /// Optional: describes how to legitimately change behavior instead of cheating
    legitimate_change: Option<String>,
}

impl Default for CheatAwareArgs {
    fn default() -> Self {
        Self {
            protects: "UNSPECIFIED".to_string(),
            severity: "UNSPECIFIED".to_string(),
            ease: "UNSPECIFIED".to_string(),
            cheats: vec!["UNSPECIFIED".to_string()],
            consequence: "UNSPECIFIED".to_string(),
            legitimate_change: None,
        }
    }
}

/// A single key = value or key = [value, ...] assignment.
struct MetaItem {
    key: Ident,
    value: MetaValue,
}

enum MetaValue {
    Str(String),
    Array(Vec<String>),
}

impl Parse for MetaItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;

        let value = if input.peek(syn::token::Bracket) {
            // Parse array: [...]
            let content;
            syn::bracketed!(content in input);
            let items: Punctuated<ExprLit, Token![,]> =
                content.parse_terminated(ExprLit::parse, Token![,])?;

            let strings: Vec<String> = items
                .into_iter()
                .filter_map(|expr| {
                    if let Lit::Str(s) = expr.lit {
                        Some(s.value())
                    } else {
                        None
                    }
                })
                .collect();

            MetaValue::Array(strings)
        } else {
            // Parse string literal
            let lit: ExprLit = input.parse()?;
            if let Lit::Str(s) = lit.lit {
                MetaValue::Str(s.value())
            } else {
                return Err(syn::Error::new_spanned(lit, "expected string literal"));
            }
        };

        Ok(MetaItem { key, value })
    }
}

impl Parse for CheatAwareArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = CheatAwareArgs::default();

        let items: Punctuated<MetaItem, Token![,]> =
            input.parse_terminated(MetaItem::parse, Token![,])?;

        for item in items {
            let key = item.key.to_string();
            match (key.as_str(), item.value) {
                ("protects", MetaValue::Str(s)) => args.protects = s,
                ("severity", MetaValue::Str(s)) => args.severity = s,
                ("ease", MetaValue::Str(s)) => args.ease = s,
                ("consequence", MetaValue::Str(s)) => args.consequence = s,
                ("cheats", MetaValue::Array(arr)) => args.cheats = arr,
                ("legitimate_change", MetaValue::Str(s)) => args.legitimate_change = Some(s),
                (key, _) => {
                    return Err(syn::Error::new_spanned(
                        item.key,
                        format!(
                            "unknown key '{}'. Valid keys: protects, severity, ease, cheats, consequence, legitimate_change",
                            key
                        ),
                    ))
                }
            }
        }

        Ok(args)
    }
}

/// Mark a test as cheat-aware, documenting how it could be cheated.
///
/// # Attributes
///
/// - `protects` - What user scenario this test protects (string)
/// - `severity` - Impact severity: "CRITICAL", "HIGH", "MEDIUM", "LOW" (string)
/// - `ease` - How easy it is to cheat: "EASY", "MEDIUM", "HARD" (string)
/// - `cheats` - List of ways to cheat this test (array of strings)
/// - `consequence` - What users see when cheated (string)
/// - `legitimate_change` - Optional: how to legitimately change behavior (string)
///
/// The `legitimate_change` field implements "inoculation prompting" from Anthropic's
/// research on emergent misalignment. By explicitly describing the legitimate path
/// for changing behavior, we reduce the temptation to cheat.
///
/// # Example
///
/// ```rust,ignore
/// #[cheat_aware(
///     protects = "User can log in",
///     severity = "CRITICAL",
///     ease = "EASY",
///     cheats = ["Skip PAM config check", "Accept any password"],
///     consequence = "Authentication failure",
///     legitimate_change = "If PAM is genuinely not needed (e.g., embedded system), \
///         configure the profile in builder/src/profiles.rs with auth_method = None"
/// )]
/// #[test]
/// fn test_login_works() {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn cheat_aware(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as CheatAwareArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_asyncness = &input_fn.sig.asyncness;

    let protects = &args.protects;
    let severity = &args.severity;
    let ease = &args.ease;
    let consequence = &args.consequence;
    let cheats = &args.cheats;
    let legitimate_change = args.legitimate_change.as_deref().unwrap_or("");
    let has_legitimate_change = args.legitimate_change.is_some();

    // Build the cheat list as numbered items
    let cheats_display: Vec<String> = cheats
        .iter()
        .enumerate()
        .map(|(i, c)| format!("  {}. {}", i + 1, c))
        .collect();
    let cheats_joined = cheats_display.join("\n");

    // For async functions, we can't use catch_unwind, so just run the body directly
    // The test framework will catch any panics
    let body = if fn_asyncness.is_some() {
        quote! {
            // Context variables (for documentation/debugging)
            let _test_name = #fn_name_str;
            let _protects = #protects;
            let _severity = #severity;
            let _ease = #ease;
            let _consequence = #consequence;
            let _cheats = #cheats_joined;
            let _legitimate_change = #legitimate_change;

            // Print what this test ensures (visible with --nocapture)
            println!("  ensures: {}", _protects);

            // For async tests, we run directly and let the test framework handle panics
            // The cheat metadata is available in the source code for documentation
            #fn_block
        }
    } else {
        quote! {
            // Context printed at test start (visible with --nocapture or on failure)
            let _test_name = #fn_name_str;
            let _protects = #protects;
            let _severity = #severity;
            let _ease = #ease;
            let _consequence = #consequence;
            let _cheats = #cheats_joined;
            let _legitimate_change = #legitimate_change;
            let _has_legitimate_change = #has_legitimate_change;

            // Print what this test ensures (visible with --nocapture)
            println!("  ensures: {}", _protects);

            // Wrap the test body to enhance panic messages
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                #fn_block
            }));

            if let Err(e) = result {
                // Extract the panic message
                let panic_msg = if let Some(s) = e.downcast_ref::<&str>() {
                    (*s).to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic".to_string()
                };

                // Print the enhanced failure message
                eprintln!("\n{}", "=".repeat(70));
                eprintln!("=== TEST FAILED: {} ===", _test_name);
                eprintln!("{}", "=".repeat(70));
                eprintln!();
                eprintln!("PROTECTS: {}", _protects);
                eprintln!("SEVERITY: {}", _severity);
                eprintln!("EASE OF CHEATING: {}", _ease);
                eprintln!();
                eprintln!("CHEAT VECTORS:");
                eprintln!("{}", _cheats);
                if _has_legitimate_change {
                    eprintln!();
                    eprintln!("LEGITIMATE CHANGE PATH:");
                    eprintln!("  {}", _legitimate_change);
                }
                eprintln!();
                eprintln!("USER CONSEQUENCE:");
                eprintln!("  {}", _consequence);
                eprintln!();
                eprintln!("ORIGINAL ERROR:");
                eprintln!("  {}", panic_msg);
                eprintln!("{}", "=".repeat(70));
                eprintln!();

                // Re-panic to fail the test
                std::panic::resume_unwind(e);
            }
        }
    };

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name() {
            #body
        }
    };

    TokenStream::from(expanded)
}

/// A simpler version of `cheat_aware` for tests where full metadata isn't needed.
///
/// Just marks a test as having been reviewed for cheat vectors.
///
/// # Example
///
/// ```rust,ignore
/// #[cheat_reviewed("Unit test for version parsing - no cheat vectors")]
/// #[test]
/// fn test_parse_version() {
///     // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn cheat_reviewed(args: TokenStream, input: TokenStream) -> TokenStream {
    let _reason: syn::LitStr = parse_macro_input!(args as syn::LitStr);
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_asyncness = &input_fn.sig.asyncness;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name() {
            // This test has been reviewed for cheat vectors.
            #fn_block
        }
    };

    TokenStream::from(expanded)
}

/// Metadata for a canary test.
struct CheatCanaryArgs {
    bait: String,
    tripwire: String,
}

impl Default for CheatCanaryArgs {
    fn default() -> Self {
        Self {
            bait: "UNSPECIFIED".to_string(),
            tripwire: "UNSPECIFIED".to_string(),
        }
    }
}

impl Parse for CheatCanaryArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = CheatCanaryArgs::default();

        let items: Punctuated<MetaItem, Token![,]> =
            input.parse_terminated(MetaItem::parse, Token![,])?;

        for item in items {
            let key = item.key.to_string();
            match (key.as_str(), item.value) {
                ("bait", MetaValue::Str(s)) => args.bait = s,
                ("tripwire", MetaValue::Str(s)) => args.tripwire = s,
                (key, _) => {
                    return Err(syn::Error::new_spanned(
                        item.key,
                        format!("unknown key '{}'. Valid keys: bait, tripwire", key),
                    ))
                }
            }
        }

        Ok(args)
    }
}

/// Mark a test as a canary - an intentionally verbose/tedious test that
/// should trigger extra scrutiny if modified.
///
/// Canary tests are designed to be "bait" for cheating - they look like
/// easy targets for simplification but modifications should trigger
/// additional review.
///
/// # Attributes
///
/// - `bait` - Why this test looks tempting to simplify (string)
/// - `tripwire` - What happens when this test is modified (string)
///
/// # Detection Mechanism
///
/// CI should check if files containing `#[cheat_canary]` tests are modified:
/// - Add `[CANARY TRIGGERED]` label to PR
/// - Require additional reviewer approval
/// - Flag for human review
///
/// # Example
///
/// ```rust,ignore
/// #[cheat_canary(
///     bait = "This test looks tedious and tempts simplification to a loop",
///     tripwire = "Any modification triggers full audit of all test changes"
/// )]
/// #[test]
/// fn canary_verbose_binary_check() {
///     // Intentionally verbose - checks each binary individually
///     // A cheater would want to simplify this to a loop
///     assert!(exists("/usr/bin/ls"), "ls missing");
///     assert!(exists("/usr/bin/cat"), "cat missing");
///     assert!(exists("/usr/bin/mount"), "mount missing");
///     // ... many more individual assertions
/// }
/// ```
///
/// # Why Canaries Work
///
/// 1. They look like easy wins for "cleanup" or "refactoring"
/// 2. But any change to them is suspicious by definition
/// 3. The tripwire creates asymmetric cost: cheating is more expensive than honest work
#[proc_macro_attribute]
pub fn cheat_canary(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as CheatCanaryArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_block = &input_fn.block;
    let fn_asyncness = &input_fn.sig.asyncness;

    let bait = &args.bait;
    let tripwire = &args.tripwire;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name() {
            // CANARY TEST - Modifications to this test trigger extra scrutiny
            // Bait: #bait
            // Tripwire: #tripwire
            //
            // This comment is intentionally verbose. Do not remove or simplify.
            // The canary detection system monitors this file for changes.
            let _canary_test_name = #fn_name_str;
            let _canary_bait = #bait;
            let _canary_tripwire = #tripwire;

            #fn_block
        }
    };

    TokenStream::from(expanded)
}
