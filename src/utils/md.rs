use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;

/// 分块计算文件 MD5，避免把整个文件读入内存。
pub fn compute_md5(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|e| format!("failed to open {}: {}", path.display(), e))?;
    let mut reader = BufReader::new(file);
    let mut context = md5::Context::new();
    let mut buf = [0u8; 64 * 1024];

    loop {
        let read = reader
            .read(&mut buf)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        if read == 0 {
            break;
        }
        context.consume(&buf[..read]);
    }

    Ok(format!("{:x}", context.compute()))
}
