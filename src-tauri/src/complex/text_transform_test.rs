use crate::complex::text_transform::transform;
use crate::data::search::Replacement;
use crate::result::AppError;

fn rule(origin: &str, target: &str) -> Replacement {
    Replacement {
        origin: origin.to_owned(),
        target: target.to_owned(),
    }
}

#[test]
fn rules_use_original_text_and_reject_overlap() {
    assert_eq!(
        transform("abc abc", &[rule("abc", "def"), rule("def", "ghi")]),
        Ok("def def".to_owned())
    );

    assert_eq!(
        transform("abc", &[rule("ab", "x"), rule("bc", "y")]),
        Err(AppError::InvalidInput)
    );

    assert_eq!(
        transform("词词", &[rule("词", "\u{2003}")]),
        Ok(String::new())
    );

    assert_eq!(
        transform("a", &[rule("a", "b"), rule("a", "c")]),
        Err(AppError::InvalidInput)
    );

    assert_eq!(
        transform("a", &[rule("", "b")]),
        Err(AppError::InvalidInput)
    );
}
