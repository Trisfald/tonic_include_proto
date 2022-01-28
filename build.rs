fn main() {
    tonic_build::configure()
        .compile(
            &["proto/xy.proto".to_owned(), "proto/xz.proto".to_owned()],
            &["proto".to_owned()],
        )
        .unwrap();
}
