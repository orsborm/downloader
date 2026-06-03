// ed2k hash 计算模块
// ed2k 使用 MD4 算法，按 9.28MB 分块计算 hash
// 单块文件：直接 MD4(data)
// 多块文件：先计算每块 MD4，再对所有块 hash 拼接后做 MD4

use md4::Digest;

/// ed2k 分块大小：9.28 MB（9728000 字节）
pub const ED2K_CHUNK_SIZE: usize = 9_728_000;

/// 计算 ed2k hash
///
/// # 算法
/// - 单块文件（≤ 9.28MB）：直接对整个文件计算 MD4
/// - 多块文件：将文件按 9.28MB 分块，每块计算 MD4，
///   然后将所有块的 MD4 结果拼接后再做一次 MD4
pub fn ed2k_hash(data: &[u8]) -> [u8; 16] {
    if data.len() <= ED2K_CHUNK_SIZE {
        return md4_hash(data);
    }

    let mut chunk_hashes = Vec::new();
    for chunk in data.chunks(ED2K_CHUNK_SIZE) {
        let hash = md4_hash(chunk);
        chunk_hashes.extend_from_slice(&hash);
    }

    md4_hash(&chunk_hashes)
}

/// MD4 hash 计算（使用 md4 crate）
fn md4_hash(data: &[u8]) -> [u8; 16] {
    let mut hasher = md4::Md4::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&result);
    out
}

/// AICH 子块大小：180 KB
pub const AICH_BLOCK_SIZE: usize = 180 * 1024;

/// 计算 AICH hash（高级智能损坏处理）
///
/// 将文件按 180KB 子块分块，计算每个子块的 SHA1，
/// 然后构建 Merkle Tree，返回所有叶子节点的 hash
pub fn aich_hash(data: &[u8]) -> Vec<[u8; 20]> {
    if data.is_empty() {
        return Vec::new();
    }

    // 计算所有叶子节点的 SHA1
    let mut leaves: Vec<[u8; 20]> = data
        .chunks(AICH_BLOCK_SIZE)
        .map(sha1_hash)
        .collect();

    // 构建 Merkle Tree（自底向上）
    while leaves.len() > 1 {
        let mut next_level = Vec::new();
        for pair in leaves.chunks(2) {
            if pair.len() == 2 {
                // 合并两个子节点的 hash
                let mut combined = Vec::with_capacity(40);
                combined.extend_from_slice(&pair[0]);
                combined.extend_from_slice(&pair[1]);
                next_level.push(sha1_hash(&combined));
            } else {
                // 奇数节点直接提升
                next_level.push(pair[0]);
            }
        }
        leaves = next_level;
    }

    // 返回根 hash 作为单元素向量
    leaves
}

/// SHA1 hash 计算
fn sha1_hash(data: &[u8]) -> [u8; 20] {
    use sha1::Digest;
    let mut hasher = sha1::Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut out = [0u8; 20];
    out.copy_from_slice(&result);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ed2k_hash_empty() {
        let hash = ed2k_hash(b"");
        assert_eq!(hash.len(), 16);
        // MD4 of empty string: 31d6cfe0d16ae931b73c59d7e0c089c0
        let expected: [u8; 16] = [
            0x31, 0xd6, 0xcf, 0xe0, 0xd1, 0x6a, 0xe9, 0x31,
            0xb7, 0x3c, 0x59, 0xd7, 0xe0, 0xc0, 0x89, 0xc0,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_ed2k_hash_small_file() {
        let data = vec![0u8; 1024];
        let hash = ed2k_hash(&data);
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_ed2k_hash_chunk_boundary() {
        let data = vec![0u8; ED2K_CHUNK_SIZE];
        let hash = ed2k_hash(&data);
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_md4_known_vector() {
        // MD4("abc") = a448017aaf21d8525fc10ae87aa6729d
        let hash = md4_hash(b"abc");
        let expected: [u8; 16] = [
            0xa4, 0x48, 0x01, 0x7a, 0xaf, 0x21, 0xd8, 0x52,
            0x5f, 0xc1, 0x0a, 0xe8, 0x7a, 0xa6, 0x72, 0x9d,
        ];
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_ed2k_multi_chunk() {
        // Create data slightly larger than one chunk
        let data = vec![0xABu8; ED2K_CHUNK_SIZE + 100];
        let hash = ed2k_hash(&data);
        assert_eq!(hash.len(), 16);
        // Multi-chunk: should be MD4(MD4(chunk1) || MD4(chunk2))
        // Verify it's different from single-chunk hash
        let single_hash = md4_hash(&data);
        assert_ne!(hash, single_hash);
    }

    #[test]
    fn test_aich_empty() {
        let result = aich_hash(b"");
        assert!(result.is_empty());
    }

    #[test]
    fn test_aich_single_block() {
        let data = vec![0u8; 1024]; // 小于 180KB
        let result = aich_hash(&data);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 20);
    }

    #[test]
    fn test_aich_two_blocks() {
        let data = vec![0xABu8; AICH_BLOCK_SIZE + 100]; // 跨越两个子块
        let result = aich_hash(&data);
        assert_eq!(result.len(), 1); // 根节点
        assert_eq!(result[0].len(), 20);
    }

    #[test]
    fn test_aich_deterministic() {
        let data = vec![0x42u8; AICH_BLOCK_SIZE * 3 + 500];
        let hash1 = aich_hash(&data);
        let hash2 = aich_hash(&data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_aich_different_data_different_hash() {
        let data1 = vec![0u8; AICH_BLOCK_SIZE * 2];
        let data2 = vec![1u8; AICH_BLOCK_SIZE * 2];
        let hash1 = aich_hash(&data1);
        let hash2 = aich_hash(&data2);
        assert_ne!(hash1[0], hash2[0]);
    }
}
