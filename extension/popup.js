// 浏览器扩展弹出窗口脚本
// 检测页面资源、手动添加链接、批量下载

document.addEventListener('DOMContentLoaded', () => {
  const statusEl = document.getElementById('status');
  const resourcesEl = document.getElementById('resources');
  const manualUrl = document.getElementById('manualUrl');
  const addManualBtn = document.getElementById('addManual');
  const batchBtn = document.getElementById('batchDownload');
  const interceptToggle = document.getElementById('interceptToggle');

  // ==================== 连接状态检测 ====================

  chrome.runtime.sendMessage({ type: 'CHECK_CONNECTION' }, (response) => {
    if (response?.connected) {
      statusEl.textContent = `已连接 v${response.version}`;
      statusEl.className = 'status connected';
    } else {
      statusEl.textContent = '未连接';
      statusEl.className = 'status disconnected';
    }
  });

  // ==================== 页面资源检测 ====================

  // 获取当前标签页的资源
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs[0]) {
      chrome.tabs.sendMessage(tabs[0].id, { type: 'GET_ALL_LINKS' }, (links) => {
        if (chrome.runtime.lastError || !links) {
          resourcesEl.innerHTML = '<div class="empty">无法检测页面资源</div>';
          return;
        }

        if (links.length === 0) {
          resourcesEl.innerHTML = '<div class="empty">未检测到可下载资源</div>';
          return;
        }

        resourcesEl.innerHTML = links.slice(0, 20).map(link => `
          <div class="link-item" data-url="${escapeHtml(link.url)}">
            <span class="type ${getTypeClass(link.url)}">${getTypeLabel(link.url)}</span>
            <span class="url" title="${escapeHtml(link.url)}">${escapeHtml(link.filename || link.text || link.url)}</span>
          </div>
        `).join('');

        // 点击下载
        resourcesEl.querySelectorAll('.link-item').forEach(item => {
          item.addEventListener('click', () => {
            const url = item.dataset.url;
            chrome.runtime.sendMessage({ type: 'DOWNLOAD_URL', url, filename: '' });
            item.style.background = '#d1fae5';
          });
        });
      });
    }
  });

  // ==================== 手动添加 ====================

  addManualBtn.addEventListener('click', () => {
    const url = manualUrl.value.trim();
    if (url) {
      chrome.runtime.sendMessage({ type: 'DOWNLOAD_URL', url, filename: '' }, () => {
        manualUrl.value = '';
        addManualBtn.textContent = '已添加 ✓';
        setTimeout(() => { addManualBtn.textContent = '添加到下载器'; }, 1500);
      });
    }
  });

  manualUrl.addEventListener('keydown', (e) => {
    if (e.key === 'Enter') addManualBtn.click();
  });

  // ==================== 批量下载 ====================

  batchBtn.addEventListener('click', () => {
    chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
      if (tabs[0]) {
        chrome.tabs.sendMessage(tabs[0].id, { type: 'GET_ALL_LINKS' }, (links) => {
          if (links && links.length > 0) {
            const urls = links.map(l => l.url);
            chrome.runtime.sendMessage({ type: 'BATCH_DOWNLOAD', urls }, (response) => {
              batchBtn.textContent = `已添加 ${response.count} 个任务`;
              setTimeout(() => { batchBtn.textContent = '批量下载页面所有链接'; }, 2000);
            });
          }
        });
      }
    });
  });

  // ==================== 拦截开关 ====================

  chrome.storage.local.get(['interceptEnabled'], (result) => {
    interceptToggle.checked = result.interceptEnabled || false;
  });

  interceptToggle.addEventListener('change', () => {
    chrome.storage.local.set({ interceptEnabled: interceptToggle.checked });
  });

  // ==================== 工具函数 ====================

  function getTypeClass(url) {
    const lower = url.toLowerCase();
    if (lower.includes('.m3u8')) return 'type-hls';
    if (lower.match(/\.(mp4|mkv|avi|mov|webm|mp3|wav|flac)/)) return 'type-video';
    return 'type-file';
  }

  function getTypeLabel(url) {
    const lower = url.toLowerCase();
    if (lower.includes('.m3u8')) return 'HLS';
    if (lower.match(/\.(mp4|mkv|avi|mov|webm)/)) return '视频';
    if (lower.match(/\.(mp3|wav|flac|aac)/)) return '音频';
    if (lower.startsWith('magnet:')) return '磁力链';
    if (lower.startsWith('ed2k://')) return 'ed2k';
    return '文件';
  }

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }
});
