use ham_manager::Manifest;

#[test]
pub fn manifest_manager() {
    // Manifest file
    let filename = format!(
        "{}/examples/1_project/ham.yml",
        std::env::current_dir().unwrap().display()
    );

    // Manifest instance
    let manifest = Manifest::from_file(filename.as_str());

    // It was found
    assert_eq!(true, manifest.is_ok());

    let manifest = manifest.unwrap();

    // Expect the same version
    assert_eq!("1.0.0", manifest.version.unwrap());
}
