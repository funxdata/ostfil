use ostfil::utils::pack::{do_pack, un_pack};
use std::fs;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ostfil-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("创建临时目录");
    dir
}

#[test]
fn pack_and_unpack_roundtrip() {
    let dir = scratch("pack");
    let src = dir.join("a.txt");
    let packed = dir.join("a.z");
    let restored = dir.join("a.out");

    fs::write(&src, b"hello ostfil\n").expect("写入源文件");

    do_pack(src.to_str().unwrap(), packed.to_str().unwrap()).expect("压缩");
    assert!(packed.exists(), "压缩产物应写入调用方给出的路径");
    assert!(!dir.join("a.z.z").exists(), "不应再追加 .z 后缀");

    un_pack(packed.to_str().unwrap(), restored.to_str().unwrap()).expect("解压");
    assert_eq!(fs::read(&restored).unwrap(), b"hello ostfil\n");

    fs::remove_dir_all(&dir).expect("清理临时目录");
}
