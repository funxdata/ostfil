use ostfil::utils::checksum::{
    digest_file, verify_file, FULL_HASH_LIMIT, SAMPLE_CHUNK_BYTES,
};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ostfil-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("创建临时目录");
    dir
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[test]
fn hashes_small_file_completely() {
    let dir = scratch("digest-small");
    let path = dir.join("a.txt");
    fs::write(&path, b"hello ostfil\n").expect("写入文件");

    let checksum = digest_file(&path).expect("计算摘要");
    assert_eq!(
        checksum.sha256,
        "2dc4378f5479b2f6ac42eda016d41bdbc2b51638a550800592d18d9052fdcdf9"
    );
    assert_eq!(checksum.size, 13);
    assert!(!checksum.sampled);
    assert_eq!(checksum.sample_bytes, 0);

    verify_file(&path, &checksum).expect("校验应通过");
    fs::remove_dir_all(&dir).expect("清理临时目录");
}

#[test]
fn hashes_exactly_limit_completely() {
    let dir = scratch("digest-limit");
    let path = dir.join("limit.bin");
    File::create(&path)
        .expect("创建文件")
        .set_len(FULL_HASH_LIMIT)
        .expect("设置大小");

    let checksum = digest_file(&path).expect("计算摘要");
    assert!(!checksum.sampled, "等于上限时不采样");
    assert_eq!(checksum.size, FULL_HASH_LIMIT);
    assert_eq!(checksum.sample_bytes, 0);

    fs::remove_dir_all(&dir).expect("清理临时目录");
}

#[test]
fn samples_large_file_by_size_head_and_tail() {
    let dir = scratch("digest-large");
    let path = dir.join("large.bin");
    let size = FULL_HASH_LIMIT + SAMPLE_CHUNK_BYTES + 1;
    let chunk = SAMPLE_CHUNK_BYTES as usize;

    let mut file = File::create(&path).expect("创建文件");
    file.set_len(size).expect("设置大小");
    file.seek(SeekFrom::Start(0)).expect("定位头部");
    file.write_all(&vec![0xAAu8; chunk]).expect("写头部");
    file.seek(SeekFrom::Start(size - SAMPLE_CHUNK_BYTES))
        .expect("定位尾部");
    file.write_all(&vec![0x5Au8; chunk]).expect("写尾部");
    drop(file);

    let checksum = digest_file(&path).expect("计算摘要");
    assert!(checksum.sampled);
    assert_eq!(checksum.sample_bytes, SAMPLE_CHUNK_BYTES);
    assert_eq!(checksum.size, size);

    // 与 dytes 的规则一致：size(u64 LE) + 首 4 MiB + 尾 4 MiB
    let content = fs::read(&path).expect("读取文件");
    let mut hasher = Sha256::new();
    hasher.update(size.to_le_bytes());
    hasher.update(&content[..chunk]);
    hasher.update(&content[content.len() - chunk..]);
    assert_eq!(checksum.sha256, hex(&hasher.finalize()));

    verify_file(&path, &checksum).expect("校验应通过");
    fs::remove_dir_all(&dir).expect("清理临时目录");
}

#[test]
fn verify_file_detects_change() {
    let dir = scratch("digest-verify");
    let path = dir.join("b.txt");
    fs::write(&path, b"hello ostfil\n").expect("写入文件");
    let checksum = digest_file(&path).expect("计算摘要");

    fs::write(&path, b"hello ostfil!\n").expect("改写文件");
    assert!(verify_file(&path, &checksum).is_err(), "内容变化应校验失败");

    fs::remove_dir_all(&dir).expect("清理临时目录");
}
