// 浏览器扩展后台服务 (Service Worker)
// 监听下载请求、右键菜单、与下载器通信

// 下载器本地 API 地址
const DOWNLOADER_API = "http://127.0.0.1:6800/jsonrpc";

// ==================== 下载拦截 ====================

// 缓存拦截开关状态，避免异步 storage 查询导致 suggest() 超时
let interceptEnabled = false;
chrome.storage.local.get(["interceptEnabled"], (result) => {
  interceptEnabled = !!result.interceptEnabled;
});
chrome.storage.onChanged.addListener((changes) => {
  if (changes.interceptEnabled) {
    interceptEnabled = !!changes.interceptEnabled.newValue;
  }
});

// 监听浏览器下载请求，转发到下载器
chrome.downloads.onDeterminingFilename.addListener((downloadItem, suggest) => {
  if (interceptEnabled) {
    // 取消浏览器下载，转发到下载器
    chrome.downloads.cancel(downloadItem.id);
    sendToDownloader(downloadItem.url, downloadItem.filename);
    // 即使取消也要调用 suggest，否则 Chrome 会挂起
    suggest({ filename: downloadItem.filename });
  } else {
    suggest({ filename: downloadItem.filename });
  }
});

// ==================== 右键菜单 ====================

// 创建右键菜单
chrome.runtime.onInstalled.addListener(() => {
  chrome.contextMenus.create({
    id: "download-with-downloader",
    title: "使用全协议下载器下载",
    contexts: ["link", "video", "audio"]
  });

  chrome.contextMenus.create({
    id: "download-video",
    title: "下载此视频",
    contexts: ["video"]
  });

  chrome.contextMenus.create({
    id: "download-all-links",
    title: "批量下载页面链接",
    contexts: ["page"]
  });
});

// 处理右键菜单点击
chrome.contextMenus.onClicked.addListener((info, tab) => {
  switch (info.menuItemId) {
    case "download-with-downloader":
      sendToDownloader(info.linkUrl || info.srcUrl, "");
      break;
    case "download-video":
      sendToDownloader(info.srcUrl, "");
      break;
    case "download-all-links":
      // 注入脚本获取所有链接
      chrome.scripting.executeScript({
        target: { tabId: tab.id },
        files: ["content.js"]
      });
      break;
  }
});

// ==================== 消息处理 ====================

// 处理来自 content script 和 popup 的消息
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  switch (message.type) {
    case "DOWNLOAD_URL":
      sendToDownloader(message.url, message.filename || "");
      sendResponse({ success: true });
      break;

    case "DOWNLOAD_VIDEO":
      sendToDownloader(message.url, message.filename || "");
      sendResponse({ success: true });
      break;

    case "BATCH_DOWNLOAD":
      message.urls.forEach(url => {
        sendToDownloader(url, "");
      });
      sendResponse({ success: true, count: message.urls.length });
      break;

    case "CHECK_CONNECTION":
      checkDownloaderConnection().then(sendResponse);
      return true; // 异步响应

    default:
      sendResponse({ error: "未知消息类型" });
  }
});

// ==================== 与下载器通信 ====================

// 发送链接到下载器
async function sendToDownloader(url, filename) {
  if (!url) return;

  try {
    const response = await fetch(DOWNLOADER_API, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "downloader.addTask",
        params: [[url], { dir: "", fileName: filename }],
        id: Date.now()
      })
    });

    const result = await response.json();
    if (result.result) {
      showNotification("下载已添加", `任务 ${filename || url} 已添加到下载器`);
    } else if (result.error) {
      showNotification("下载失败", result.error.message);
    }
  } catch (e) {
    showNotification("连接失败", "无法连接到下载器，请确认下载器已启动");
  }
}

// 检查下载器连接状态
async function checkDownloaderConnection() {
  try {
    const response = await fetch(DOWNLOADER_API, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        method: "downloader.getVersion",
        params: [],
        id: 1
      })
    });
    const result = await response.json();
    return { connected: true, version: result.result?.version };
  } catch {
    return { connected: false };
  }
}

// 显示通知
function showNotification(title, message) {
  chrome.notifications.create({
    type: "basic",
    iconUrl: "icons/icon128.png",
    title: title,
    message: message
  });
}
