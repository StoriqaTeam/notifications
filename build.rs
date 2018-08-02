use std::env;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

fn main() {
    copy_templates(Path::new("./templates")).unwrap();
}

fn copy_templates(templates_dir: &Path) -> io::Result<()> {
    if templates_dir.is_dir() {
        let out_dir = env::var("OUT_DIR").unwrap();
        let dest_dir = Path::new(&out_dir).join("templates");
        match fs::create_dir(dest_dir.clone()) {
            Ok(_) => (),
            Err(e) => match e.kind() {
                ErrorKind::AlreadyExists => return Ok(()), // templates already exists, do not rewrite
                _ => return Err(e),
            },
        };
        for entry in fs::read_dir(templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().clone().unwrap();
            let dest_path = dest_dir.join(name);
            fs::copy(path.clone(), dest_path)?;
        }
    } else {
        panic!("no such directory {:?}", templates_dir);
    }
    Ok(())
}
