#[cfg(test)]
pub mod assertion {
    use std::fmt::Debug;

    pub fn assert_ok_eq<T, E>(expected: &T, result: &Result<T, E>)
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

    pub fn assert_is_err<T, E>(result: Result<T, E>) -> E
    where
        T: Debug,
        E: Debug + PartialEq,
    {
        match result {
            Err(err_value) => err_value,
            Ok(ok_value) => {
                panic!("expected Err but got Ok: {:?}", ok_value);
            }
        }
    }

    pub fn assert_err_eq<T, E>(expected_err: E, result: Result<T, E>)
    where
        T: Debug,
        E: Debug + PartialEq,
    {
        let actual_err = assert_is_err(result);
        assert_eq!(expected_err, actual_err);
    }
}
