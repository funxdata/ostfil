use ostfil::utils::pack::do_pack;
use ostfil::{post_delete_file, post_get_fil, post_storage};
use serde_json::{json, Value};
use std::ffi::{c_char, c_int, CString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

type Ffi = unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int;

/// 切换工作目录是进程级操作，含 CWD 的测试必须串行执行。
static CWD_LOCK: Mutex<()> = Mutex::new(());

/// 按 C ABI 约定调用一个 FFI 函数，返回解析后的 JSON。
fn call(f: Ffi, payload: &Value) -> Value {
    let input = CString::new(payload.to_string()).expect("payload 不含 NUL");
    let mut out = vec![0u8; 65536];

    let rc = unsafe { f(input.as_ptr(), out.as_mut_ptr() as *mut c_char) };
    assert_eq!(rc, 0, "FFI 返回值应为 0（缓冲区写入成功）");

    let end = out.iter().position(|&b| b == 0).expect("输出应以 NUL 结尾");
    serde_json::from_slice(&out[..end]).expect("输出应为合法 JSON")
}

/// 临时切换工作目录，离开作用域时还原。
///
/// `STORAGE_DIR` 是相对路径 `./storage`，测试必须切到临时目录，
/// 避免在仓库里生成存储目录。
struct CwdGuard(PathBuf);

impl CwdGuard {
    fn enter(dir: &Path) -> Self {
        let previous = std::env::current_dir().expect("读取当前工作目录");
        std::env::set_current_dir(dir).expect("切换工作目录");
        CwdGuard(previous)
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.0);
    }
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ostfil-ffi-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("创建临时目录");
    dir
}

#[test]
fn storage_get_delete_roundtrip() {
    let _lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = scratch("roundtrip");
    let guard = CwdGuard::enter(&root);

    let sample = root.join("sample.txt");
    fs::write(&sample, b"hello ostfil\n").expect("写入样例文件");

    let stored = call(post_storage, &json!({ "file_path": sample }));
    assert_eq!(stored["code"], 200, "storage: {}", stored);

    let data = &stored["data"];
    let uuid = data["uuid"].as_str().expect("uuid 应为字符串").to_string();
    assert_eq!(data["size"], 13);
    assert_eq!(data["extra"], "txt");
    assert_eq!(data["content_type"], "text/plain");
    assert_eq!(
        data["sha256"],
        "2dc4378f5479b2f6ac42eda016d41bdbc2b51638a550800592d18d9052fdcdf9"
    );
    assert_eq!(data["sampled"], false);
    assert_eq!(data["sample_bytes"], 0);

    let dir = root.join("storage").join(&uuid);
    assert!(dir.join("original").exists(), "应保留 original");
    assert!(dir.join("original.z").exists(), "压缩产物应为 original.z");
    assert!(dir.join("info.json").exists(), "应写入 info.json");

    // original 丢失时应能通过 original.z 还原
    fs::remove_file(dir.join("original")).expect("删除 original");
    let restored = call(post_get_fil, &json!({ "uuid": uuid }));
    assert_eq!(restored["code"], 200, "get(restore): {}", restored);
    assert!(dir.join("original").exists(), "应解压还原 original");

    let got = call(post_get_fil, &json!({ "uuid": uuid }));
    assert_eq!(got["code"], 200);
    assert_eq!(
        got["data"]["sha256"],
        "2dc4378f5479b2f6ac42eda016d41bdbc2b51638a550800592d18d9052fdcdf9"
    );

    let deleted = call(post_delete_file, &json!({ "uuid": uuid }));
    assert_eq!(deleted["code"], 200, "delete: {}", deleted);
    assert!(!dir.exists(), "删除后目录应不存在");

    let again = call(post_get_fil, &json!({ "uuid": uuid }));
    assert_eq!(again["code"], 400104);

    // 先还原工作目录，Windows 上无法删除正在被占用的当前目录
    drop(guard);
    fs::remove_dir_all(&root).expect("清理临时目录");
}

#[test]
fn rejects_unsafe_uuid() {
    for uuid in ["../../important", "D:/some/dir", "not-a-uuid", ""] {
        let deleted = call(post_delete_file, &json!({ "uuid": uuid }));
        assert_eq!(deleted["code"], 400105, "delete 应拒绝 uuid {:?}", uuid);
        let got = call(post_get_fil, &json!({ "uuid": uuid }));
        assert_eq!(got["code"], 400105, "get 应拒绝 uuid {:?}", uuid);
    }
}

#[test]
fn rejects_directory_as_file_path() {
    let out = call(post_storage, &json!({ "file_path": std::env::temp_dir() }));
    assert_eq!(out["code"], 400106, "{}", out);
}

#[test]
fn missing_file_returns_400103() {
    let out = call(post_storage, &json!({ "file_path": "/ostfil-not-exist/here.txt" }));
    assert_eq!(out["code"], 400103, "{}", out);
}

#[test]
fn restores_legacy_original_zz() {
    let _lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let root = scratch("legacy");
    let guard = CwdGuard::enter(&root);

    let uuid = "18eb8248-2f5f-490b-96d0-8f155e6f791b";
    let dir = root.join("storage").join(uuid);
    fs::create_dir_all(&dir).expect("创建旧布局目录");

    // 旧布局：只有 original.z.z，没有 original
    let payload = b"legacy data\n";
    let original = dir.join("original");
    fs::write(&original, payload).expect("写入原文件");
    do_pack(
        original.to_str().unwrap(),
        dir.join("original.z.z").to_str().unwrap(),
    )
    .expect("压缩");
    fs::remove_file(&original).expect("删除 original");

    // 旧布局 + 旧元数据：只有 original.z.z，info.json 里是已废弃的 md5 字段
    fs::write(
        dir.join("info.json"),
        json!({
            "uuid": uuid,
            "size": payload.len() as u64,
            "extra": "txt",
            "end_time": 0,
            "md5": "",
            "content_type": ""
        })
        .to_string(),
    )
    .expect("写入 info.json");

    let got = call(post_get_fil, &json!({ "uuid": uuid }));
    assert_eq!(got["code"], 200, "{}", got);
    assert_eq!(fs::read(&original).unwrap(), payload, "应从 original.z.z 还原");
    assert_eq!(got["data"]["content_type"], "text/plain", "应补齐 content_type");
    assert!(
        !got["data"]["sha256"].as_str().unwrap().is_empty(),
        "应补算 sha256"
    );

    drop(guard);
    fs::remove_dir_all(&root).expect("清理临时目录");
}
