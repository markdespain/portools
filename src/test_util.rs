#[cfg(test)]
pub mod assertion {
    use crate::model::Lot;
    use crate::validate::Invalid;
    use std::fmt::Debug;

    pub fn assert_ok<T, E>(expected: &T, result: &Result<T, E>)
    where
        T: Debug + PartialEq,
        E: Debug,
    {
        match result {
            Err(e) => {
                panic!("expected Ok but got Err: {:?}", e)
            }
            Ok(actual) => {
                assert_eq!(expected, actual);
            }
        }
    }

    pub fn assert_err<T, E>(expected_err: E, actual: Result<T, E>)
    where T : Debug, E: Debug + PartialEq,
    {
        match actual {
            Err(actual_err) => {
                assert_eq!(expected_err, actual_err)
            }
            Ok(actual_lot) => {
                panic!("expected Err but got Ok: {:?}", actual_lot);
            }
        }
    }
}
