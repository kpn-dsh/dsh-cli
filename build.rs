fn main() {
  println!("cargo:rustc-env=BUILD_DATE={}", chrono::offset::Utc::now());
}
