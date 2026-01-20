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
//!     consequence = "bash: sudo: command not found"
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
//! === TEST FAILED: test_sudo_binary_present ===
//!
//! PROTECTS: User can run sudo commands
//! SEVERITY: CRITICAL
//! EASE OF CHEATING: EASY
//!
//! CHEAT VECTORS:
//!   1. Move sudo to OPTIONAL list
//!   2. Remove sudo from essential binaries
//!
//! USER CONSEQUENCE:
//!   bash: sudo: command not found
//!
//! ORIGINAL ERROR:
//!   assertion failed: tarball_contains("./usr/bin/sudo")
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
}

impl Default for CheatAwareArgs {
    fn default() -> Self {
        Self {
            protects: "UNSPECIFIED".to_string(),
            severity: "UNSPECIFIED".to_string(),
            ease: "UNSPECIFIED".to_string(),
            cheats: vec!["UNSPECIFIED".to_string()],
            consequence: "UNSPECIFIED".to_string(),
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
                (key, _) => {
                    return Err(syn::Error::new_spanned(
                        item.key,
                        format!(
                            "unknown key '{}'. Valid keys: protects, severity, ease, cheats, consequence",
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
///
/// # Example
///
/// ```rust,ignore
/// #[cheat_aware(
///     protects = "User can log in",
///     severity = "CRITICAL",
///     ease = "EASY",
///     cheats = ["Skip PAM config check", "Accept any password"],
///     consequence = "Authentication failure"
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
