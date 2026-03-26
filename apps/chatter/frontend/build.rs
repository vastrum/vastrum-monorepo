use std::fs;
use std::path::Path;

fn main() {
    let path = Path::new("generated/tailwind.css");
    if !path.exists() {
        fs::create_dir_all("generated").unwrap();
        fs::write(path, "/* placeholder - run `npm run build:css` to generate */").unwrap();
    }
}
