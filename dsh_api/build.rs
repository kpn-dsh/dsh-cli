use progenitor::GenerationSettings;

fn main() {
  let src = "openapi_spec/openapi_1_8_0_updated.json";
  println!("cargo:rerun-if-changed={}", src);
  let file = std::fs::File::open(src).unwrap();
  let spec = serde_json::from_reader(file).unwrap();
  let mut generator_settings = GenerationSettings::default();
  generator_settings.with_derive("PartialEq");
  let mut generator = progenitor::Generator::new(&generator_settings);
  let tokens = generator.generate_tokens(&spec).unwrap();
  let ast = syn::parse2(tokens).unwrap();
  let content = prettyplease::unparse(&ast);
  let mut out_file = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
  out_file.push("codegen.rs");
  // println!("cargo:warning= out_file: {:?}", &out_file);
  std::fs::write(out_file, content).unwrap();
}
