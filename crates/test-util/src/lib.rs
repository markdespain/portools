pub mod assertion {
    use std::fmt::Debug;

    pub fn assert_is_ok<T, E>(result: &Result<T, E>) -> &T
    where
        T: Debug,
        E: Debug,
    {
        match result {
            Err(e) => {
                panic!("expesdfdscted Ok but got Err: {:?}", e)
            }
            Ok(actual) => actual,
        }
    }

    pub fn assert_ok_eq<T, E>(expected: &T, result: &Result<T, E>)
    where
        T: Debug + PartialEq,
        E: Debug,
    {
        let actual = assert_is_ok(result);
        assert_eq!(expected, actual);
    }

    pub fn assert_result_vec_eq_fn<T, E, F>(expected: &Vec<T>, result: &Result<Vec<T>, E>, eq: F)
    where
        T: Debug + PartialEq,
        E: Debug,
        F: Fn(&T, &T) -> bool,
    {
        let actual = assert_is_ok(result);
        assert_vec_eq_fn(expected, actual, eq);
    }

    pub fn assert_vec_eq_fn<T, F>(expected: &Vec<T>, actual: &Vec<T>, eq: F)
    where
        T: Debug + PartialEq,
        F: Fn(&T, &T) -> bool,
    {
        assert_eq!(
            expected.len(),
            actual.len(),
            "Vectors should have the same length"
        );
        for (i, item) in actual.iter().enumerate() {
            let expected = &expected[i];
            assert!(
                eq(expected, item),
                "item at index {i} should equal expected\nexpected: {:?},\n  actual: {:?}",
                expected,
                item
            )
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

pub mod resource {
    use actix_web::web::Bytes;
    use std::fs;

    pub fn load_bytes(path: &str) -> Bytes {
        match fs::read(path) {
            Err(e) => {
                panic!("failed to load path {path}: {e}")
            }
            Ok(bytes) => Bytes::from(bytes),
        }
    }
}
