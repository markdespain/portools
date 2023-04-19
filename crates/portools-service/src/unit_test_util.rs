#[cfg(test)]
pub mod resource {
    use actix_web::web::Bytes;
    use std::path::PathBuf;
    use test_util::resource;

    pub fn load_bytes(resource: &str) -> Bytes {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resource/test/csv_digester/");
        path.push(resource);
        resource::load_bytes(path.to_str().unwrap())
    }
}
