use garde::Validate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, JsonSchema, Validate, PartialEq, Eq)]
pub struct AccountIdentifierInput {
    #[garde(custom(validate_label))]
    #[schemars(
        length(min = 1, max = 80),
        pattern(r"^[^\u0000-\u001f\u007f-\u009f\u2028\u2029]*[^\s\u0000-\u001f\u007f-\u009f\u2028\u2029][^\u0000-\u001f\u007f-\u009f\u2028\u2029]*$"),
        extend("x-validation" = serde_json::json!({
            "required": "Enter an identifier",
            "blank": "Enter an identifier",
            "pattern": "Identifier must be on one line without control characters",
            "maxLength": "Identifier must be 80 characters or fewer"
        }))
    )]
    pub label: String,
    #[garde(custom(validate_value))]
    #[schemars(
        length(max = 250),
        pattern(r"^[^\u0000-\u001f\u007f-\u009f\u2028\u2029]*$"),
        extend("x-validation" = serde_json::json!({
            "type": "Value must be text",
            "pattern": "Value must be on one line without control characters",
            "maxLength": "Value must be 250 characters or fewer"
        }))
    )]
    pub value: String,
}

fn validate_label(value: &str, _ctx: &()) -> garde::Result {
    validate_text(
        value,
        80,
        Some("Enter an identifier"),
        "Identifier must be on one line without control characters",
        "Identifier must be 80 characters or fewer",
    )
}

fn validate_value(value: &str, _ctx: &()) -> garde::Result {
    validate_text(
        value,
        250,
        None,
        "Value must be on one line without control characters",
        "Value must be 250 characters or fewer",
    )
}

fn validate_text(
    value: &str,
    max: usize,
    required: Option<&str>,
    invalid: &str,
    too_long: &str,
) -> garde::Result {
    if let Some(required) = required.filter(|_| value.trim().is_empty()) {
        return Err(garde::Error::new(required));
    }
    if value
        .chars()
        .any(|ch| ch.is_control() || matches!(ch, '\u{2028}' | '\u{2029}'))
    {
        return Err(garde::Error::new(invalid));
    }
    if value.chars().count() > max {
        return Err(garde::Error::new(too_long));
    }
    Ok(())
}

/// Search-only normalization, shared by stored values and queries for every label.
/// Keep other punctuation literal (for example slashes in provider references).
pub fn canonicalize_account_identifier(value: &str) -> String {
    value
        .chars()
        .filter(|ch| !ch.is_whitespace() && !matches!(ch, '-' | '.' | '\u{2010}'..='\u{2014}'))
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_distinguishes_missing_text_from_invalid_characters() {
        ["", " ", "\u{00a0}\u{3000}", "\t", "\r\n"]
            .into_iter()
            .for_each(|value| {
                assert_eq!(
                    validate_label(value, &()).unwrap_err().to_string(),
                    "Enter an identifier"
                );
            });
        [
            "first\nsecond",
            "number\r",
            "number\t",
            "\0number",
            "number\u{2028}",
        ]
        .into_iter()
        .for_each(|value| {
            assert_eq!(
                validate_label(value, &()).unwrap_err().to_string(),
                "Identifier must be on one line without control characters"
            );
            assert_eq!(
                validate_value(value, &()).unwrap_err().to_string(),
                "Value must be on one line without control characters"
            );
        });
        assert_eq!(
            validate_label(&"名".repeat(81), &())
                .unwrap_err()
                .to_string(),
            "Identifier must be 80 characters or fewer"
        );
    }

    #[test]
    fn values_allow_blank_text_but_still_reject_controls_and_excess_length() {
        ["", " ", "\u{00a0}\u{3000}"].into_iter().for_each(|value| {
            assert!(validate_value(value, &()).is_ok());
        });
        ["\t", "\n", "\r\n", "\0", "\u{2028}"]
            .into_iter()
            .for_each(|value| {
                assert_eq!(
                    validate_value(value, &()).unwrap_err().to_string(),
                    "Value must be on one line without control characters"
                );
            });
        assert!(validate_value(&"名".repeat(250), &()).is_ok());
        assert_eq!(
            validate_value(&"名".repeat(251), &())
                .unwrap_err()
                .to_string(),
            "Value must be 250 characters or fewer"
        );
    }

    #[test]
    fn canonicalization_is_shared_unicode_aware_and_preserves_significant_characters() {
        assert_eq!(
            canonicalize_account_identifier(" 00-12\u{00a0}34.56 "),
            "00123456"
        );
        assert_eq!(
            canonicalize_account_identifier("ÄBC—００１/_%"),
            "äbc００１/_%"
        );
        assert_eq!(canonicalize_account_identifier(" - .\t"), "");
    }
}
