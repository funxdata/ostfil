use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// 将 `path` 压缩写入 `save_path`（调用方给出完整目标路径，函数不再追加后缀）。
pub fn do_pack(path: &str, save_path: &str) -> Result<(), String> {
    let input_file = File::open(path).map_err(|e| format!("failed to open {}: {}", path, e))?;
    let output_file =
        File::create(save_path).map_err(|e| format!("failed to create {}: {}", save_path, e))?;

    let mut reader = BufReader::new(input_file);
    let mut encoder = GzEncoder::new(output_file, Compression::best());

    std::io::copy(&mut reader, &mut encoder).map_err(|e| format!("compression failed: {}", e))?;

    encoder
        .finish()
        .map_err(|e| format!("failed to finish compression: {}", e))?;

    Ok(())
}

/// 将 `path` 解压写入 `save_path`（调用方给出完整目标路径，函数不再拼接后缀）。
pub fn un_pack(path: &str, save_path: &str) -> Result<(), String> {
    let input_file = File::open(path).map_err(|e| format!("failed to open {}: {}", path, e))?;
    let output_file =
        File::create(save_path).map_err(|e| format!("failed to create {}: {}", save_path, e))?;

    let mut decoder = GzDecoder::new(input_file);
    let mut writer = BufWriter::new(output_file);

    std::io::copy(&mut decoder, &mut writer).map_err(|e| format!("decompression failed: {}", e))?;

    Ok(())
}
