use std::{env, path::PathBuf};
use walkdir::WalkDir;

fn compile_version(tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_root = manifest_dir
        .join("meta")
        .join("braiins")
        .join("proto")
        .join(tag)
        .join("proto");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?).join(tag);
    std::fs::create_dir_all(&out_dir)?;

    let protos: Vec<PathBuf> = WalkDir::new(&proto_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension() == Some("proto".as_ref()))
        .inspect(|p| println!("{}", p.path().display()))
        .map(|e| e.path().to_owned())
        .collect();

    tonic_prost_build::configure()
        .out_dir(&out_dir)
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(&protos, &[proto_root])?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let versions = std::fs::read_dir("meta/braiins/proto")?
        .filter_map(Result::ok)
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.file_name().into_string().unwrap())
        .collect::<Vec<_>>();

    for v in versions {
        compile_version(&v)?;
    }
    Ok(())
}
