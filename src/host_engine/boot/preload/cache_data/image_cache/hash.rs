//! 图片缓存变更检测
// TODO: 迁移至 storage::CacheStore

use std::fs;
use std::io;
use std::path::Path;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 读取文件内容并生成稳定 hash，用于判断图片内容是否变化。
pub fn hash_file(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!("{:016x}", fnv1a64(&bytes)))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
