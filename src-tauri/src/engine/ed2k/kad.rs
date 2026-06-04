// ed2k Kademlia (KAD) DHT 网络模块
// 实现 128 位 ID 空间、k-bucket 路由表、XOR 距离度量
// 支持节点查找、关键词搜索、源搜索

use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{debug, info, warn};

use super::proto;

/// KAD 节点 ID（128 位）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KadId(pub [u8; 16]);

impl KadId {
    /// 从字节数组创建
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        KadId(bytes)
    }

    /// 计算两个 ID 的 XOR 距离
    pub fn xor_distance(&self, other: &KadId) -> KadId {
        let mut result = [0u8; 16];
        for i in 0..16 {
            result[i] = self.0[i] ^ other.0[i];
        }
        KadId(result)
    }

    /// 获取距离的前导零位数（用于确定 k-bucket 索引）
    pub fn leading_zeros(&self) -> u32 {
        for (i, &byte) in self.0.iter().enumerate() {
            if byte != 0 {
                return (i as u32) * 8 + byte.leading_zeros();
            }
        }
        128
    }

    /// 判断是否比另一个 ID 更接近目标（完整 128 位比较）
    pub fn is_closer_than(&self, target: &KadId, other: &KadId) -> bool {
        let dist_self = self.xor_distance(target);
        let dist_other = other.xor_distance(target);
        // 逐字节比较，小的更近
        dist_self.0 < dist_other.0
    }
}

/// KAD 节点信息
#[derive(Debug, Clone)]
pub struct KadNode {
    /// 节点 ID
    pub id: KadId,
    /// 节点地址
    pub addr: SocketAddr,
    /// 最后通信时间
    pub last_seen: std::time::Instant,
    /// 延迟（毫秒）
    pub ping: u32,
    /// 是否已验证
    pub verified: bool,
}

/// k-bucket（存放相近节点）
const K_BUCKET_SIZE: usize = 10;

/// 搜索并发度
const ALPHA_SEARCH: usize = 3;

/// KAD 路由表
pub struct KadRoutingTable {
    /// 本节点 ID
    local_id: KadId,
    /// k-bucket 列表（128 个 bucket，每个对应一个距离范围）
    buckets: Vec<Vec<KadNode>>,
}

impl Clone for KadRoutingTable {
    fn clone(&self) -> Self {
        KadRoutingTable {
            local_id: self.local_id,
            buckets: self.buckets.iter().map(|bucket| {
                bucket.iter().map(|node| KadNode {
                    id: node.id,
                    addr: node.addr,
                    last_seen: std::time::Instant::now(), // 重置时间戳
                    ping: node.ping,
                    verified: node.verified,
                }).collect()
            }).collect(),
        }
    }
}

impl KadRoutingTable {
    /// 创建新的路由表
    pub fn new(local_id: KadId) -> Self {
        let mut buckets = Vec::with_capacity(128);
        for _ in 0..128 {
            buckets.push(Vec::new());
        }

        KadRoutingTable { local_id, buckets }
    }

    /// 添加或更新节点
    pub fn update_node(&mut self, node: KadNode) {
        let distance = self.local_id.xor_distance(&node.id);
        let bucket_idx = distance.leading_zeros() as usize;

        if bucket_idx >= 128 {
            return;
        }

        let bucket = &mut self.buckets[bucket_idx];

        // 检查是否已存在
        if let Some(existing) = bucket.iter_mut().find(|n| n.id == node.id) {
            existing.last_seen = std::time::Instant::now();
            existing.addr = node.addr;
            existing.ping = node.ping;
            return;
        }

        // 如果 bucket 未满，直接添加
        if bucket.len() < K_BUCKET_SIZE {
            bucket.push(node);
            return;
        }

        // Bucket 已满，替换最旧的未验证节点
        if let Some(idx) = bucket.iter().position(|n| !n.verified) {
            bucket[idx] = node;
        }
        // 否则忽略（最旧的节点应该还在响应）
    }

    /// 查找最接近目标的 K 个节点
    pub fn find_closest(&self, target: &KadId, count: usize) -> Vec<KadNode> {
        let mut all_nodes: Vec<(KadId, &KadNode)> = Vec::new();

        for bucket in &self.buckets {
            for node in bucket {
                all_nodes.push((node.id.xor_distance(target), node));
            }
        }

        // 按距离排序
        all_nodes.sort_by(|a, b| {
            let dist_a = a.0.leading_zeros();
            let dist_b = b.0.leading_zeros();
            dist_b.cmp(&dist_a)
        });

        all_nodes
            .into_iter()
            .take(count)
            .map(|(_, node)| node.clone())
            .collect()
    }

    /// 获取节点总数
    pub fn total_nodes(&self) -> usize {
        self.buckets.iter().map(|b| b.len()).sum()
    }

    /// 清理过期节点（超过 5 分钟未通信）
    pub fn cleanup(&mut self) {
        let cutoff = std::time::Duration::from_secs(300);
        for bucket in &mut self.buckets {
            bucket.retain(|node| node.last_seen.elapsed() < cutoff);
        }
    }
}

/// KAD 状态信息
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KadStatus {
    /// 是否正在运行
    pub running: bool,
    /// 路由表节点数
    pub node_count: usize,
    /// 监听端口
    pub listen_port: u16,
    /// 是否已完成引导尝试
    pub bootstrap_done: bool,
}

/// KAD 操作类型
#[derive(Debug, Clone, Copy)]
pub enum KadOperation {
    /// 引导请求
    BootstrapReq = 0x01,
    /// 引导响应
    BootstrapRes = 0x09,
    /// 关键词搜索请求
    SearchKeyReq = 0x33,
    /// 关键词搜索响应
    SearchKeyRes = 0x3B,
    /// 源搜索请求
    SearchSourceReq = 0x42,
    /// 源搜索响应
    SearchSourceRes = 0x4A,
    /// 发布请求
    PublishKeyReq = 0x23,
}

/// KAD 引擎
pub struct KadEngine {
    /// 路由表
    routing_table: KadRoutingTable,
    /// 本节点 ID
    local_id: KadId,
    /// 是否已启动
    running: bool,
    /// 是否已完成引导尝试（无论是否收到响应）
    bootstrap_attempted: bool,
    /// 持久 UDP 套接字（启动后绑定，所有 KAD 通信共用）
    socket: Option<Arc<UdpSocket>>,
    /// 监听端口（启动时绑定）
    listen_port: u16,
}

impl Clone for KadEngine {
    fn clone(&self) -> Self {
        KadEngine {
            routing_table: self.routing_table.clone(),
            local_id: self.local_id,
            running: self.running,
            bootstrap_attempted: self.bootstrap_attempted,
            socket: None, // 克隆时不复制 socket
            listen_port: self.listen_port,
        }
    }
}

impl KadEngine {
    /// 创建新的 KAD 引擎
    pub fn new(local_id: KadId, listen_port: u16) -> Self {
        info!("KAD 引擎初始化: 节点 ID = {:02x?}, 端口 = {}", local_id.0, listen_port);
        KadEngine {
            routing_table: KadRoutingTable::new(local_id),
            local_id,
            running: false,
            bootstrap_attempted: false,
            socket: None,
            listen_port,
        }
    }

    /// 启动 KAD 网络：绑定 UDP 端口 + 引导
    pub async fn start(&mut self, bootstrap_addrs: &[SocketAddr]) -> Result<(), anyhow::Error> {
        if bootstrap_addrs.is_empty() {
            anyhow::bail!("无可用引导节点");
        }

        // 绑定持久 UDP 套接字
        let addr = format!("0.0.0.0:{}", self.listen_port);
        let socket = Arc::new(UdpSocket::bind(&addr).await?);
        info!("KAD UDP 套接字绑定: {}", addr);
        self.socket = Some(socket.clone());
        self.running = true;

        // 向所有引导节点发送请求
        for bootstrap_addr in bootstrap_addrs {
            if let Err(e) = self.send_bootstrap_req(&socket, *bootstrap_addr).await {
                debug!("KAD 引导请求失败 {}: {}", bootstrap_addr, e);
            }
        }

        // 等待响应（最多 5 秒）
        let mut buf = [0u8; 2048];
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(
                std::time::Duration::from_millis(500),
                socket.recv_from(&mut buf),
            )
            .await
            {
                Ok(Ok((len, from))) => {
                    debug!("收到 KAD 响应: {} 字节, 来自 {}", len, from);
                    self.parse_bootstrap_response(&buf[..len]);
                    // 收到第一个响应后继续收集更多节点
                }
                _ => continue,
            }
        }

        self.bootstrap_attempted = true;

        let total = self.routing_table.total_nodes();
        if total > 0 {
            info!("KAD 引导完成，路由表中共 {} 个节点", total);
        } else {
            info!("KAD 引导完成，未收到响应（ed2k 服务器可能不支持 KAD UDP）");
        }

        Ok(())
    }

    /// 发送引导请求
    async fn send_bootstrap_req(&self, socket: &UdpSocket, addr: SocketAddr) -> Result<(), anyhow::Error> {
        // KADEMLIA2_BOOTSTRAP_REQ: [0xE4][4B len][0x01][16B node_id]
        let mut packet = Vec::with_capacity(22);
        packet.push(0xE4u8);
        let payload_len: u32 = 17;
        packet.extend_from_slice(&payload_len.to_le_bytes());
        packet.push(KadOperation::BootstrapReq as u8);
        packet.extend_from_slice(&self.local_id.0);
        socket.send_to(&packet, addr).await?;
        debug!("KAD 引导请求已发送到 {}", addr);
        Ok(())
    }

    /// 解析引导响应，提取节点列表并添加到路由表
    fn parse_bootstrap_response(&mut self, data: &[u8]) {
        // KADEMLIA2_BOOTSTRAP_RES 格式:
        // [0xE4] [4字节长度] [0x09] [16字节发送者ID] [1字节TCP端口类型] [2字节TCP端口]
        // 后跟联系人列表: [1字节数量] [每个联系人: 16字节ID + 4字节IP + 2字节UDP端口 + 2字节TCP端口 + 1字节版本]
        if data.len() < 22 {
            return;
        }

        // 验证 KAD 协议标识和操作码
        if data[0] != 0xE4 {
            debug!("KAD 响应: 无效协议标识 0x{:02X}", data[0]);
            return;
        }
        if data[5] != KadOperation::BootstrapRes as u8 {
            debug!("KAD 响应: 非引导响应操作码 0x{:02X}", data[5]);
            return;
        }

        // 跳过协议头(1) + 长度(4) + 操作码(1) + 发送者ID(16) = 22字节
        let mut offset = 22;
        if offset + 1 > data.len() {
            return;
        }

        let contact_count = data[offset] as usize;
        offset += 1;

        for _ in 0..contact_count {
            if offset + 25 > data.len() {
                break;
            }

            let mut id_bytes = [0u8; 16];
            id_bytes.copy_from_slice(&data[offset..offset + 16]);
            offset += 16;

            let ip = std::net::Ipv4Addr::new(data[offset], data[offset + 1], data[offset + 2], data[offset + 3]);
            offset += 4;

            let port = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;

            // 跳过 TCP 端口 (2 bytes) + 版本 (1 byte)
            offset += 3;

            let addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
            let node = KadNode {
                id: KadId::from_bytes(id_bytes),
                addr,
                last_seen: std::time::Instant::now(),
                ping: 0,
                verified: true,
            };

            info!("KAD 引导: 发现节点 {:02x?}@{}", &id_bytes[..4], addr);
            self.routing_table.update_node(node);
        }

        info!(
            "KAD 引导完成，路由表中共 {} 个节点",
            self.routing_table.total_nodes()
        );
    }

    /// 执行迭代式节点查找（Kademlia 协议核心）
    ///
    /// α=3 并发查询，最多迭代 10 轮，直到没有更近的节点
    pub async fn find_node(&mut self, target: &KadId) -> Vec<KadNode> {
        const ALPHA: usize = 3;
        const MAX_ITERATIONS: usize = 10;

        let mut visited: HashSet<SocketAddr> = HashSet::new();
        let mut best_distance = KadId([0xFFu8; 16]);

        // 初始：从路由表获取最近的 α 个节点
        let mut candidates = self.routing_table.find_closest(target, ALPHA);

        for iteration in 0..MAX_ITERATIONS {
            if candidates.is_empty() {
                break;
            }

            // 过滤已访问的节点
            candidates.retain(|n| !visited.contains(&n.addr));
            if candidates.is_empty() {
                break;
            }

            let mut new_nodes = Vec::new();

            // 向候选节点发送 FIND_NODE 请求
            for node in candidates.iter().take(ALPHA) {
                visited.insert(node.addr);

                match self.send_find_node_req(node, target).await {
                    Ok(nodes) => {
                        for n in nodes {
                            let dist = n.id.xor_distance(target);
                            // 完整 128 位比较：dist < best_distance 表示更近
                            if dist.0 < best_distance.0 {
                                best_distance = dist;
                                new_nodes.push(n.clone());
                            }
                            self.routing_table.update_node(n);
                        }
                    }
                    Err(e) => {
                        debug!("FIND_NODE 请求失败 {}: {}", node.addr, e);
                    }
                }
            }

            if new_nodes.is_empty() {
                debug!("KAD 迭代查找在第 {} 轮收敛", iteration);
                break;
            }

            candidates = new_nodes;
            debug!(
                "KAD 迭代查找第 {} 轮: 发现 {} 个新候选节点",
                iteration + 1,
                candidates.len()
            );
        }

        self.routing_table.find_closest(target, K_BUCKET_SIZE)
    }

    /// 向单个节点发送 FIND_NODE 请求并解析响应
    async fn send_find_node_req(&self, node: &KadNode, target: &KadId) -> Result<Vec<KadNode>, anyhow::Error> {
        let socket = match &self.socket {
            Some(s) => s.clone(),
            None => anyhow::bail!("KAD 未启动"),
        };

        // KADEMLIA2_REQ (0x02): [0xE4][4B len][0x02][16B target_id][16B sender_id]
        let mut packet = Vec::with_capacity(38);
        packet.push(0xE4u8);
        let payload_len: u32 = 33;
        packet.extend_from_slice(&payload_len.to_le_bytes());
        packet.push(0x02u8);
        packet.extend_from_slice(&target.0);
        packet.extend_from_slice(&self.local_id.0);

        socket.send_to(&packet, node.addr).await?;

        let mut buf = [0u8; 2048];
        match tokio::time::timeout(
            std::time::Duration::from_secs(3),
            socket.recv_from(&mut buf),
        )
        .await
        {
            Ok(Ok((len, _))) => Ok(self.parse_contact_list(&buf[..len])),
            _ => Ok(Vec::new()),
        }
    }

    /// 解析联系人列表（通用：从 KAD 响应中提取节点列表）
    fn parse_contact_list(&self, data: &[u8]) -> Vec<KadNode> {
        let mut nodes = Vec::new();
        // 跳过协议头: [0xE4] [4字节长度] [1字节操作码] [16字节发送者ID]
        let mut offset = 22usize;
        if offset + 1 > data.len() {
            return nodes;
        }

        let count = data[offset] as usize;
        offset += 1;

        for _ in 0..count {
            if offset + 25 > data.len() {
                break;
            }

            let mut id_bytes = [0u8; 16];
            id_bytes.copy_from_slice(&data[offset..offset + 16]);
            offset += 16;

            let ip = std::net::Ipv4Addr::new(data[offset], data[offset + 1], data[offset + 2], data[offset + 3]);
            offset += 4;

            let port = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;

            // 跳过 TCP 端口 (2) + 版本 (1)
            offset += 3;

            let addr = SocketAddr::new(std::net::IpAddr::V4(ip), port);
            nodes.push(KadNode {
                id: KadId::from_bytes(id_bytes),
                addr,
                last_seen: std::time::Instant::now(),
                ping: 0,
                verified: false,
            });
        }

        nodes
    }

    /// 搜索关键词（KADEMLIA2_SEARCH_KEY_REQ）
    ///
    /// 先用 find_node 定位负责该关键词的节点，然后发送搜索请求
    pub async fn search_keyword(&mut self, keyword: &str) -> Vec<String> {
        info!("KAD 关键词搜索: {}", keyword);

        // 计算关键词的 KAD ID（MD4 hash）
        let target = self.keyword_to_kad_id(keyword);

        // 先查找最近的节点
        let closest = self.find_node(&target).await;
        let mut results = Vec::new();

        for node in closest.iter().take(ALPHA_SEARCH) {
            match self.send_search_key_req(node, keyword).await {
                Ok(found) => {
                    results.extend(found);
                }
                Err(e) => {
                    debug!("关键词搜索请求失败 {}: {}", node.addr, e);
                }
            }
        }

        info!("KAD 关键词搜索完成: '{}' 找到 {} 个结果", keyword, results.len());
        results
    }

    /// 搜索文件源（KADEMLIA2_SEARCH_SOURCE_REQ）
    ///
    /// 查找拥有指定文件的源节点
    pub async fn search_source(&mut self, file_hash: &[u8; 16]) -> Vec<SocketAddr> {
        info!("KAD 源搜索: {:02x?}", file_hash);

        let target = KadId::from_bytes(*file_hash);
        let closest = self.find_node(&target).await;
        let mut sources = Vec::new();

        for node in closest.iter().take(ALPHA_SEARCH) {
            match self.send_search_source_req(node, file_hash).await {
                Ok(addrs) => {
                    sources.extend(addrs);
                }
                Err(e) => {
                    debug!("源搜索请求失败 {}: {}", node.addr, e);
                }
            }
        }

        info!("KAD 源搜索完成: 找到 {} 个源", sources.len());
        sources
    }

    /// 将关键词转换为 KAD ID（使用 MD4 hash）
    fn keyword_to_kad_id(&self, keyword: &str) -> KadId {
        use md4::Digest;
        let mut hasher = md4::Md4::new();
        hasher.update(keyword.as_bytes());
        let result = hasher.finalize();
        let mut id = [0u8; 16];
        id.copy_from_slice(&result);
        KadId::from_bytes(id)
    }

    /// 发送关键词搜索请求
    async fn send_search_key_req(&self, node: &KadNode, keyword: &str) -> Result<Vec<String>, anyhow::Error> {
        let socket = match &self.socket {
            Some(s) => s.clone(),
            None => anyhow::bail!("KAD 未启动"),
        };

        let keyword_bytes = keyword.as_bytes();
        let mut packet = Vec::with_capacity(40 + keyword_bytes.len());
        packet.push(0xE4u8);
        let payload_len = 17 + 2 + keyword_bytes.len() as u32; // opcode + id + len + keyword
        packet.extend_from_slice(&payload_len.to_le_bytes());
        packet.push(KadOperation::SearchKeyReq as u8);
        packet.extend_from_slice(&self.local_id.0);
        packet.extend_from_slice(&(keyword_bytes.len() as u16).to_le_bytes());
        packet.extend_from_slice(keyword_bytes);

        socket.send_to(&packet, node.addr).await?;

        let mut buf = [0u8; 4096];
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            socket.recv_from(&mut buf),
        )
        .await
        {
            Ok(Ok((len, _))) => {
                // 解析搜索结果：提取文件名字符串
                Ok(self.parse_search_results(&buf[..len]))
            }
            _ => Ok(Vec::new()),
        }
    }

    /// 发送源搜索请求
    async fn send_search_source_req(&self, node: &KadNode, file_hash: &[u8; 16]) -> Result<Vec<SocketAddr>, anyhow::Error> {
        let socket = match &self.socket {
            Some(s) => s.clone(),
            None => anyhow::bail!("KAD 未启动"),
        };

        let mut packet = Vec::with_capacity(40);
        packet.push(0xE4u8);
        let payload_len: u32 = 33; // opcode(1) + id(16) + hash(16)
        packet.extend_from_slice(&payload_len.to_le_bytes());
        packet.push(KadOperation::SearchSourceReq as u8);
        packet.extend_from_slice(&self.local_id.0);
        packet.extend_from_slice(file_hash);

        socket.send_to(&packet, node.addr).await?;

        let mut buf = [0u8; 2048];
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            socket.recv_from(&mut buf),
        )
        .await
        {
            Ok(Ok((len, _))) => Ok(self.parse_source_results(&buf[..len])),
            _ => Ok(Vec::new()),
        }
    }

    /// 解析搜索结果中的文件名
    fn parse_search_results(&self, data: &[u8]) -> Vec<String> {
        let mut results = Vec::new();
        // 跳过头部: [0xE4][4B len][1B opcode][16B sender_id]
        let mut offset = 22usize;
        if offset + 2 > data.len() {
            return results;
        }

        let count = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2;

        for _ in 0..count {
            // 每个结果: [16B hash][4B IP][2B port][tags...]
            if offset + 22 > data.len() {
                break;
            }
            offset += 16 + 4 + 2; // 跳过 hash + IP + port

            // 解析 tags 中的文件名
            if offset + 2 > data.len() {
                break;
            }
            let tag_count = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;

            for _ in 0..tag_count {
                if offset + 3 > data.len() {
                    break;
                }
                let tag_type = data[offset];
                offset += 1;
                let name_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
                offset += 2;

                if offset + name_len > data.len() {
                    break;
                }

                // FT_FILENAME = 0x01
                if tag_type == 0x01 {
                    if let Ok(name) = std::str::from_utf8(&data[offset..offset + name_len]) {
                        results.push(name.to_string());
                    }
                }
                offset += name_len;
            }
        }

        results
    }

    /// 解析源搜索结果中的地址
    fn parse_source_results(&self, data: &[u8]) -> Vec<SocketAddr> {
        let mut sources = Vec::new();
        // 跳过头部: [0xE4][4B len][1B opcode][16B sender_id][16B file_hash]
        let mut offset = 38usize;
        if offset + 1 > data.len() {
            return sources;
        }

        let count = data[offset] as usize;
        offset += 1;

        for _ in 0..count {
            // 每个源: [4B IP][2B UDP port][2B TCP port] = 8 bytes
            if offset + 8 > data.len() {
                break;
            }

            let ip = std::net::Ipv4Addr::new(data[offset], data[offset + 1], data[offset + 2], data[offset + 3]);
            offset += 4;

            let port = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2; // UDP port
            offset += 2; // TCP port (跳过)

            sources.push(SocketAddr::new(std::net::IpAddr::V4(ip), port));
        }

        sources
    }

    /// 定期维护：清理过期节点 + 刷新最旧的 bucket
    pub async fn maintain(&mut self) {
        if !self.running {
            return;
        }

        // 清理过期节点
        self.routing_table.cleanup();

        // 刷新最旧的 bucket（查找随机 ID 以保持 bucket 活跃）
        if let Some(socket) = &self.socket {
            // 生成随机目标 ID 进行查找，刷新路由表
            let mut rand_id = [0u8; 16];
            for b in &mut rand_id {
                *b = rand_byte();
            }
            let target = KadId::from_bytes(rand_id);
            let closest = self.routing_table.find_closest(&target, ALPHA_SEARCH);
            for node in closest {
                let _ = self.send_find_node_req(&node, &target).await;
            }
        }

        let total = self.routing_table.total_nodes();
        debug!("KAD 维护完成，路由表: {} 个节点", total);
    }

    /// 获取 KAD 状态
    pub fn status(&self) -> KadStatus {
        KadStatus {
            running: self.running,
            node_count: self.routing_table.total_nodes(),
            listen_port: self.listen_port,
            bootstrap_done: self.bootstrap_attempted,
        }
    }

    /// 停止 KAD 网络
    pub fn stop(&mut self) {
        self.running = false;
        self.socket = None;
        info!("KAD 网络已停止");
    }
}

/// 辅助函数：生成随机字节
fn rand_byte() -> u8 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (nanos & 0xFF) as u8
}
