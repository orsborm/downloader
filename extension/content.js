// 浏览器扩展内容脚本
// 检测页面中的视频资源和可下载链接

(function() {
  'use strict';

  // ==================== 视频资源检测 ====================

  // 检测 <video> 和 <audio> 标签
  function detectMediaElements() {
    const media = [];

    // 检测 <video> 标签
    document.querySelectorAll('video').forEach(video => {
      if (video.src) {
        media.push({ url: video.src, type: 'video', format: getFormat(video.src) });
      }
      // 检测 <source> 子标签
      video.querySelectorAll('source').forEach(source => {
        if (source.src) {
          media.push({ url: source.src, type: 'video', format: getFormat(source.src) });
        }
      });
    });

    // 检测 <audio> 标签
    document.querySelectorAll('audio').forEach(audio => {
      if (audio.src) {
        media.push({ url: audio.src, type: 'audio', format: getFormat(audio.src) });
      }
    });

    return media;
  }

  // 检测网络请求中的流媒体资源
  function detectStreamingResources() {
    const resources = [];

    // 检测 m3u8 播放列表
    const scripts = document.querySelectorAll('script');
    scripts.forEach(script => {
      const content = script.textContent;
      // 匹配 m3u8 URL
      const m3u8Match = content.match(/https?:\/\/[^\s"']+\.m3u8[^\s"']*/g);
      if (m3u8Match) {
        m3u8Match.forEach(url => {
          resources.push({ url, type: 'hls', format: 'm3u8' });
        });
      }

      // 匹配 mpd URL
      const mpdMatch = content.match(/https?:\/\/[^\s"']+\.mpd[^\s"']*/g);
      if (mpdMatch) {
        mpdMatch.forEach(url => {
          resources.push({ url, type: 'dash', format: 'mpd' });
        });
      }
    });

    return resources;
  }

  // 检测页面中的可下载链接
  function detectDownloadLinks() {
    const links = [];

    document.querySelectorAll('a[href]').forEach(a => {
      const href = a.href;
      if (isDownloadableUrl(href)) {
        links.push({
          url: href,
          filename: getFilename(href),
          text: a.textContent.trim()
        });
      }
    });

    return links;
  }

  // 判断是否为可下载 URL
  function isDownloadableUrl(url) {
    const downloadableExtensions = [
      '.zip', '.rar', '.7z', '.tar', '.gz', '.bz2', '.xz',
      '.exe', '.msi', '.dmg', '.deb', '.rpm',
      '.iso', '.img',
      '.mp4', '.mkv', '.avi', '.mov', '.wmv', '.flv',
      '.mp3', '.wav', '.flac', '.aac', '.ogg',
      '.pdf', '.doc', '.docx', '.xls', '.xlsx', '.ppt', '.pptx',
      '.apk', '.ipa',
      '.torrent'
    ];

    const lower = url.toLowerCase();
    return downloadableExtensions.some(ext => lower.includes(ext)) ||
           lower.startsWith('magnet:') ||
           lower.startsWith('ed2k://');
  }

  // 从 URL 提取文件名
  function getFilename(url) {
    try {
      const pathname = new URL(url).pathname;
      const parts = pathname.split('/');
      return decodeURIComponent(parts[parts.length - 1]) || '';
    } catch {
      return '';
    }
  }

  // 获取文件格式
  function getFormat(url) {
    const filename = getFilename(url);
    const ext = filename.split('.').pop().toLowerCase();
    return ext;
  }

  // ==================== 批量链接获取 ====================

  // 获取页面中所有可下载链接
  function getAllDownloadableLinks() {
    const links = detectDownloadLinks();
    const media = detectMediaElements();
    const streaming = detectStreamingResources();

    // 合并并去重
    const allUrls = new Set();
    const result = [];

    [...links, ...media.map(m => ({ url: m.url, filename: getFilename(m.url), text: m.type })),
     ...streaming.map(s => ({ url: s.url, filename: getFilename(s.url), text: s.type }))].forEach(item => {
      if (!allUrls.has(item.url)) {
        allUrls.add(item.url);
        result.push(item);
      }
    });

    return result;
  }

  // ==================== 注入下载按钮 ====================

  // 在视频元素旁注入下载按钮
  function injectDownloadButtons() {
    document.querySelectorAll('video').forEach(video => {
      if (video.dataset.downloadButtonAdded) return;

      const button = document.createElement('div');
      button.innerHTML = '⬇';
      button.title = '使用下载器下载此视频';
      button.style.cssText = `
        position: absolute;
        top: 8px;
        right: 8px;
        width: 32px;
        height: 32px;
        background: rgba(0,0,0,0.7);
        color: white;
        border-radius: 4px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        font-size: 16px;
        z-index: 999999;
        transition: background 0.2s;
      `;

      button.addEventListener('click', (e) => {
        e.stopPropagation();
        if (video.src) {
          chrome.runtime.sendMessage({
            type: 'DOWNLOAD_VIDEO',
            url: video.src,
            filename: getFilename(video.src)
          });
        }
      });

      // 确保父元素有定位
      const parent = video.parentElement;
      if (parent) {
        parent.style.position = 'relative';
        parent.appendChild(button);
        video.dataset.downloadButtonAdded = 'true';
      }
    });
  }

  // ==================== 消息监听 ====================

  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    switch (message.type) {
      case 'GET_ALL_LINKS':
        sendResponse(getAllDownloadableLinks());
        break;

      case 'GET_MEDIA':
        sendResponse([...detectMediaElements(), ...detectStreamingResources()]);
        break;
    }
  });

  // ==================== 初始化 ====================

  // 页面加载完成后检测资源
  if (document.readyState === 'complete') {
    injectDownloadButtons();
  } else {
    window.addEventListener('load', injectDownloadButtons);
  }

})();
