use std::fs::File;
use std::io::Read;
use std::path::Path;

/// 计算文件 MD5
pub fn compute_md5(path: &Path) -> Result<String, String> {
    let mut file = File::open(path)
        .map_err(|e| format!("failed to open {}: {}", path.display(), e))?;

    let mut context = md5::Context::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

        if n == 0 {
            break;
        }

        context.consume(&buffer[..n]);
    }

    let digest = context.finalize();

    Ok(format!("{:x}", digest))
}
