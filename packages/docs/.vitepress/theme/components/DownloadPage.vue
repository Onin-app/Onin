<template>
  <div class="download-container">
    <!-- 极光背景装饰 -->
    <div class="aurora-glow glow-1"></div>
    <div class="aurora-glow glow-2"></div>

    <div class="download-content">
      <!-- 头部：标题与引言 -->
      <header class="download-header">
        <h1 class="gradient-text">获取 Onin 键盘启动器</h1>
        <p class="subtitle">极速响应，无限扩展。让您在键盘上掌控一切。</p>
      </header>

      <!-- 下载源切换组件 (Pill Switcher) -->
      <div class="source-selector-wrapper">
        <span class="selector-label">下载通道：</span>
        <div class="source-selector">
          <button 
            :class="['source-btn', downloadSource === 'github' ? 'active' : '']" 
            @click="downloadSource = 'github'"
          >
            GitHub 官方
          </button>
          <button 
            :class="['source-btn', downloadSource === 'oss' ? 'active' : '']" 
            @click="downloadSource = 'oss'"
          >
            国内极速直连 🚀
          </button>
          <!-- 选中背景滑块 -->
          <div class="selector-slider" :style="sliderStyle"></div>
        </div>
      </div>

      <!-- 核心大推荐区 -->
      <section class="hero-download-section">
        <!-- 加载中 -->
        <div v-if="isLoading" class="hero-card loading-card">
          <div class="spinner"></div>
          <p>正在为您检测最佳版本号与系统架构...</p>
        </div>

        <!-- 识别成功推荐卡片 -->
        <div v-else class="hero-card fade-in">
          <div class="recommended-badge">智能推荐</div>
          <div class="system-icon-wrapper">
            <svg v-if="detectedSystem.os === 'windows'" class="system-svg" viewBox="0 0 88 88" fill="currentColor">
              <path d="M0 12.402l35.687-4.86.016 34.622-35.703.111v-29.873zm35.685 33.914l.019 34.727-35.704-4.896v-29.72l35.685-.111zm4.18-39.467l48.135-6.849v41.298h-48.135v-34.449zm48.135 39.467v41.297l-48.135-6.883-.008-34.303h48.143z"/>
            </svg>
            <svg v-else-if="detectedSystem.os === 'macos'" class="system-svg" viewBox="0 0 170 170" fill="currentColor">
              <path d="M150.37 130.25c-2.45 5.66-5.35 10.87-8.71 15.66-4.58 6.53-8.33 11.05-11.22 13.56-4.48 4.12-9.28 6.23-14.42 6.35-3.69 0-8.14-1.05-13.32-3.18-5.19-2.12-9.97-3.17-14.34-3.17-4.58 0-9.49 1.05-14.75 3.17-5.26 2.13-9.5 3.24-12.74 3.35-4.37.13-9.13-1.9-14.28-6.08-3.57-2.9-7.44-7.61-11.61-14.13-8.38-12.87-15.02-28.51-19.91-46.91-4.89-18.4-4.25-34.41 1.92-48.02 4.12-9.06 9.87-16.26 17.26-21.61 7.39-5.35 15.43-8.08 24.11-8.19 5.37-.11 11.28 1.56 17.74 5.03 6.46 3.46 10.84 5.19 13.14 5.19 1.8 0 6.07-1.62 12.84-4.86 6.76-3.24 12.63-4.75 17.59-4.53 14.19.67 25.13 6.08 32.8 16.25-12.31 7.5-18.28 17.92-17.9 31.28.38 10.3 4.27 18.87 11.68 25.72 7.41 6.85 16.2 10.59 26.37 11.22-2.3 6.76-4.8 13.06-7.51 18.91zM119.22 9c0 7.38-2.73 14.25-8.19 20.61-5.46 6.35-12.01 10.45-19.64 12.29-1.01-7.82 1.83-15.01 8.52-21.57C96.6 13.78 103.74 9.5 111.35 7.5c.56 1 .84 1.5 1.5 1.5.2 0 1.01 0 6.37 0z"/>
            </svg>
            <svg v-else class="system-svg" viewBox="0 0 24 24" fill="currentColor">
              <path d="M19.35 10.04C18.67 6.59 15.64 4 12 4 9.11 4 6.6 5.64 5.35 8.04 2.34 8.36 0 10.91 0 14c0 3.31 2.69 6 6 6h13c2.76 0 5-2.24 5-5 0-2.64-2.05-4.78-4.65-4.96zM19 18H6c-2.21 0-4-1.79-4-4 0-2.05 1.53-3.76 3.56-3.97l1.07-.11.5-.95C8.08 7.14 9.94 6 12 6c2.62 0 4.88 1.86 5.39 4.43l.3 1.5 1.53.11c1.56.1 2.78 1.41 2.78 2.96 0 1.65-1.35 3-3 3z"/>
            </svg>
          </div>

          <div class="recommend-info">
            <h2>
              适用于 {{ systemLabel }}
              <span v-if="detectedSystem.os === 'macos'" class="chip-badge">
                {{ detectedSystem.arch === 'arm64' ? 'Apple 芯片' : 'Intel 芯片' }}
              </span>
            </h2>
            <p class="filename-text">推荐安装包：<code>{{ recommendedFileName }}</code></p>
            <p class="version-tag">最新版本：<span class="ver">{{ latestVersion }}</span></p>
          </div>

          <!-- 主下载按钮 -->
          <a :href="recommendedDownloadUrl" class="btn btn-primary btn-hero" :download="recommendedFileName">
            <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
              <polyline points="7 10 12 15 17 10"></polyline>
              <line x1="12" y1="15" x2="12" y2="3"></line>
            </svg>
            立即下载 
          </a>

          <!-- OSS 极速下载提示 -->
          <div v-if="downloadSource === 'oss'" class="mirror-selector-wrapper fade-in">
            <div class="mirror-tip">
              🚀 <strong>已启用国内极速直连通道！</strong>
              <div class="mirror-security-note">本通道由国内骨干网对象存储（OSS）与百兆 CDN 专线提供，官方 100% 绿色安全直连。国内下载无需代理，百兆宽带一秒跑满！绝对安全，绝无任何安全红屏拦截隐忧。</div>
            </div>
          </div>
        </div>
      </section>

      <!-- 所有版本矩阵区 -->
      <section class="all-platforms-section">
        <h2 class="section-title">手动选择平台与架构</h2>
        <p class="section-subtitle">如果我们的自动检测有偏差，或者您想为其他电脑下载，请在下方手动选择：</p>
        
        <div class="cards-grid">
          <!-- Windows Card -->
          <div :class="['platform-card', detectedSystem.os === 'windows' ? 'highlight-border' : '']">
            <div class="card-os-header">
              <span class="os-name">Windows</span>
              <span v-if="detectedSystem.os === 'windows'" class="current-tag">当前系统</span>
            </div>
            <div class="card-body">
              <h3>Windows 64位 安装包</h3>
              <p class="arch-desc">适用于所有 Windows 10 (1903) 及以上的 64 位系统。</p>
              <span class="ext-tag">.exe</span>
            </div>
            <div class="card-action-buttons">
              <a :href="getDownloadUrlBySource('windows', 'x64', 'oss')" class="btn btn-platform-oss" :download="getFileName('windows', 'x64')">
                <svg class="btn-mini-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
                </svg>
                国内极速下载 🚀
              </a>
              <a :href="getDownloadUrlBySource('windows', 'x64', 'github')" class="btn btn-platform-github" :download="getFileName('windows', 'x64')">
                <svg class="btn-mini-icon" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
                </svg>
                GitHub 官方下载
              </a>
            </div>
          </div>

          <!-- Mac ARM Card -->
          <div :class="['platform-card', (detectedSystem.os === 'macos' && detectedSystem.arch === 'arm64') ? 'highlight-border' : '']">
            <div class="card-os-header">
              <span class="os-name">macOS (Apple 芯片)</span>
              <span v-if="detectedSystem.os === 'macos' && detectedSystem.arch === 'arm64'" class="current-tag">当前系统</span>
            </div>
            <div class="card-body">
              <h3>Mac (Apple Silicon)</h3>
              <p class="arch-desc">适用于 <strong>2020年及之后</strong> 生产的搭载 M1, M2, M3 等芯片的 Mac。</p>
              <span class="ext-tag">.dmg (ARM64)</span>
            </div>
            <div class="card-action-buttons">
              <a :href="getDownloadUrlBySource('macos', 'arm64', 'oss')" class="btn btn-platform-oss" :download="getFileName('macos', 'arm64')">
                <svg class="btn-mini-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
                </svg>
                国内极速下载 🚀
              </a>
              <a :href="getDownloadUrlBySource('macos', 'arm64', 'github')" class="btn btn-platform-github" :download="getFileName('macos', 'arm64')">
                <svg class="btn-mini-icon" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
                </svg>
                GitHub 官方下载
              </a>
            </div>
          </div>

          <!-- Mac Intel Card -->
          <div :class="['platform-card', (detectedSystem.os === 'macos' && detectedSystem.arch === 'x64') ? 'highlight-border' : '']">
            <div class="card-os-header">
              <span class="os-name">macOS (Intel 芯片)</span>
              <span v-if="detectedSystem.os === 'macos' && detectedSystem.arch === 'x64'" class="current-tag">当前系统</span>
            </div>
            <div class="card-body">
              <h3>Mac (Intel 芯片)</h3>
              <p class="arch-desc">适用于 <strong>2020年之前</strong> 生产的搭载 Intel 处理器的旧款 Mac。</p>
              <span class="ext-tag">.dmg (x64)</span>
            </div>
            <div class="card-action-buttons">
              <a :href="getDownloadUrlBySource('macos', 'x64', 'oss')" class="btn btn-platform-oss" :download="getFileName('macos', 'x64')">
                <svg class="btn-mini-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
                </svg>
                国内极速下载 🚀
              </a>
              <a :href="getDownloadUrlBySource('macos', 'x64', 'github')" class="btn btn-platform-github" :download="getFileName('macos', 'x64')">
                <svg class="btn-mini-icon" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
                </svg>
                GitHub 官方下载
              </a>
            </div>
          </div>
        </div>
      </section>

      <!-- 避坑与安装指南 (交互式) -->
      <section class="guides-section">
        <h2 class="section-title">🚀 常见安装问题与避坑指南</h2>
        
        <div class="accordion">
          <!-- Windows Guidance -->
          <div class="guide-item">
            <div class="guide-title">
              <span class="guide-icon">💻</span> Windows 安装与 SmartScreen 阻止
            </div>
            <div class="guide-content">
              <p>Windows 10/11 在下载和运行某些新发布的文件时，可能会弹出 <strong>「Microsoft Defender SmartScreen 阻止了未识别的应用」</strong> 的安全警告。</p>
              <div class="tip-box info">
                <strong>解决方法：</strong>
                <ol>
                  <li>在安全警告弹窗中，点击左侧小字 <strong>「更多信息」</strong>。</li>
                  <li>此时右下角会多出一个 <strong>「仍要运行」</strong> 按钮。</li>
                  <li>点击 <strong>「仍要运行」</strong> 即可继续完成 Onin 的快速安装！</li>
                </ol>
              </div>
            </div>
          </div>

          <!-- macOS Guidance -->
          <div class="guide-item">
            <div class="guide-title">
              <span class="guide-icon">🍎</span> macOS 提示「已损坏，无法打开」
            </div>
            <div class="guide-content">
              <p>由于 Onin 目前尚未完成昂贵的 Apple 官方公证（Notarization），在首次双击打开拖拽完成的软件时，系统可能会提示：<strong>「“Onin.app”已损坏，无法打开。您应该将它移到废纸篓。」</strong> 或提示来自未识别开发者。</p>
              <p class="bold">这纯属 Apple 的安全保护策略，请放心使用。您仅需执行一步终端命令即可完美绕过限制：</p>
              
              <div class="code-container">
                <span class="code-lang">终端命令</span>
                <div class="code-body">
                  <code>xattr -cr /Applications/Onin.app</code>
                  <button @click="copyCommand" class="copy-btn" :class="{ copied: isCopied }">
                    <span v-if="!isCopied">一键复制</span>
                    <span v-else class="copied-animation">已复制 √</span>
                  </button>
                </div>
              </div>

              <div class="tip-box success">
                <strong>后续步骤：</strong>
                <p>复制上面命令后，打开您 Mac 的「终端 (Terminal)」应用，粘贴命令并按下回车。再次双击打开 Onin 即可顺畅打开并正常运行！</p>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';

// 企业级国内云对象存储 (OSS) 基准地址配置
// 您可在此处将其修改为您的阿里云 OSS、腾讯云 COS、华为云 OBS 或任意配置了 CDN 的云存储域名
const OSS_BASE_URL = 'https://onin-app.oss-cn-beijing.aliyuncs.com';

// 状态变量
const downloadSource = ref('github'); // 'github' | 'oss'
const latestVersion = ref('1.10.0'); // 默认本地稳定版 fallback
const isLoading = ref(true);
const isCopied = ref(false);
const detectedSystem = ref({ os: 'windows', arch: 'x64' });

// 下载通道选择滑块样式
const sliderStyle = computed(() => {
  return {
    transform: downloadSource.value === 'oss' ? 'translateX(100%)' : 'translateX(0%)',
  };
});

// 系统标签
const systemLabel = computed(() => {
  if (detectedSystem.value.os === 'windows') return 'Windows';
  if (detectedSystem.value.os === 'macos') return 'macOS';
  return '推荐系统';
});

// 计算推荐的下载文件名
const recommendedFileName = computed(() => {
  return getFileName(detectedSystem.value.os, detectedSystem.value.arch);
});

// 计算推荐的下载链接
const recommendedDownloadUrl = computed(() => {
  return getDownloadUrl(detectedSystem.value.os, detectedSystem.value.arch);
});

// 获取文件名
const getFileName = (os, arch) => {
  const ver = latestVersion.value.replace(/^v/, '');
  if (os === 'windows') {
    return `Onin_${ver}_x64-setup.exe`;
  }
  if (os === 'macos') {
    if (arch === 'arm64') {
      return `Onin_${ver}_aarch64.dmg`;
    }
    return `Onin_${ver}_x64.dmg`;
  }
  return '';
};

// 根据指定的下载源拼接下载链接
const getDownloadUrlBySource = (os, arch, source) => {
  const ver = latestVersion.value.replace(/^v/, '');
  const filename = getFileName(os, arch);
  if (!filename) return '#';
  
  if (source === 'oss') {
    return `${OSS_BASE_URL}/releases/v${ver}/${filename}`;
  }
  return `https://github.com/b-yp/Onin/releases/download/v${ver}/${filename}`;
};

// 拼接推荐的下载链接
const getDownloadUrl = (os, arch) => {
  return getDownloadUrlBySource(os, arch, downloadSource.value);
};

// 探测用户的系统与 CPU 架构
const detectUserEnvironment = () => {
  if (typeof window === 'undefined') return;

  const userAgent = window.navigator.userAgent.toLowerCase();
  const platform = window.navigator.platform.toLowerCase();
  
  let os = 'windows'; // 默认设为 windows
  let arch = 'x64';
  
  // 1. 探测 OS
  if (userAgent.indexOf('win') !== -1) {
    os = 'windows';
  } else if (userAgent.indexOf('mac') !== -1 || platform.indexOf('mac') !== -1) {
    os = 'macos';
    
    // 2. 探测 macOS 架构 (Apple M系列 vs Intel)
    let isArm = false;
    
    // 检查 CPU 核心数
    if (navigator.hardwareConcurrency && navigator.hardwareConcurrency >= 8) {
      // 现代 Apple 芯片通常拥有 >= 8 个核心
      isArm = true;
    }
    
    // 使用 WebGL 探测 GPU 渲染器
    try {
      const canvas = document.createElement('canvas');
      const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
      if (gl) {
        const debugInfo = gl.getExtension('WEBGL_debug_renderer_info');
        if (debugInfo) {
          const renderer = gl.getParameter(debugInfo.UNMASKED_RENDERER_VENDOR_ID || debugInfo.UNMASKED_RENDERER_STRING) || '';
          if (renderer.toLowerCase().includes('apple')) {
            isArm = true;
          }
        }
      }
    } catch (e) {
      console.warn('WebGL is not supported or failed to detect GPU', e);
    }
    
    // UA 包含 arm64 关键字
    if (userAgent.includes('arm64') || userAgent.includes('aarch64')) {
      isArm = true;
    }
    
    arch = isArm ? 'arm64' : 'x64';
  } else if (userAgent.indexOf('linux') !== -1) {
    os = 'linux';
  }
  
  detectedSystem.value = { os, arch };
};

// 获取最新 GitHub Release 版本
const fetchLatestVersion = async () => {
  try {
    const res = await fetch('https://api.github.com/repos/b-yp/Onin/releases/latest');
    if (res.ok) {
      const data = await res.json();
      if (data && data.tag_name) {
        latestVersion.value = data.tag_name;
      }
    }
  } catch (err) {
    console.warn('无法获取 GitHub 官方最新版本号，已使用 fallback 版本: ' + latestVersion.value);
  } finally {
    isLoading.value = false;
  }
};

// 复制终端代码
const copyCommand = () => {
  const commandText = 'xattr -cr /Applications/Onin.app';
  if (navigator.clipboard) {
    navigator.clipboard.writeText(commandText).then(() => {
      isCopied.value = true;
      setTimeout(() => {
        isCopied.value = false;
      }, 2000);
    });
  } else {
    // Fallback 复制方案
    const textArea = document.createElement('textarea');
    textArea.value = commandText;
    textArea.style.position = 'fixed';
    document.body.appendChild(textArea);
    textArea.focus();
    textArea.select();
    try {
      document.execCommand('copy');
      isCopied.value = true;
      setTimeout(() => {
        isCopied.value = false;
      }, 2000);
    } catch (err) {
      console.error('复制失败', err);
    }
    document.body.removeChild(textArea);
  }
};

onMounted(() => {
  detectUserEnvironment();
  fetchLatestVersion();
});
</script>

<style scoped>
/* 全局变量和字体定义 */
@import url('https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap');

.download-container {
  font-family: 'Outfit', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  position: relative;
  min-height: 80vh;
  padding: 3rem 1.5rem;
  overflow: hidden;
  color: var(--vp-c-text-1);
}

/* 极光背景虚化光晕 - 极其炫酷 */
.aurora-glow {
  position: absolute;
  border-radius: 50%;
  filter: blur(100px);
  opacity: 0.15;
  z-index: 0;
  pointer-events: none;
}

.glow-1 {
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, hsl(265, 85%, 60%) 0%, transparent 70%);
  top: -100px;
  right: -50px;
}

.glow-2 {
  width: 500px;
  height: 500px;
  background: radial-gradient(circle, hsl(200, 85%, 55%) 0%, transparent 70%);
  bottom: 0px;
  left: -150px;
}

/* 暗色模式下让背景更加瞩目 */
.dark .aurora-glow {
  opacity: 0.35;
}

.download-content {
  position: relative;
  z-index: 1;
  max-width: 900px;
  margin: 0 auto;
}

/* 头部样式 */
.download-header {
  text-align: center;
  margin-bottom: 3.5rem;
}

.download-header h1 {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.03em;
  margin-bottom: 1rem;
}

.gradient-text {
  background: linear-gradient(135deg, hsl(265, 85%, 60%), hsl(200, 85%, 55%));
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  color: transparent;
  display: inline-block;
  line-height: 1.3;
  padding-bottom: 0.1em;
}

.subtitle {
  font-size: 1.25rem;
  color: var(--vp-c-text-2);
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.6;
}

/* 下载源切换栏 (Pill Switcher) */
.source-selector-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 2.5rem;
  gap: 0.75rem;
}

.selector-label {
  font-size: 0.95rem;
  font-weight: 500;
  color: var(--vp-c-text-2);
}

.source-selector {
  position: relative;
  display: flex;
  background: var(--vp-c-bg-soft);
  padding: 0.25rem;
  border-radius: 50px;
  border: 1px solid var(--vp-c-divider);
  box-shadow: inset 0 2px 4px rgba(0,0,0,0.03);
  width: 320px;
}

.source-btn {
  flex: 1;
  background: transparent;
  border: none;
  padding: 0.5rem 1rem;
  font-family: inherit;
  font-size: 0.9rem;
  font-weight: 600;
  color: var(--vp-c-text-2);
  cursor: pointer;
  z-index: 2;
  border-radius: 50px;
  transition: color 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.source-btn.active {
  color: #ffffff;
}

/* 亮色模式下选中文字为黑色/暗色以保证对比度，如果不是 active */
.source-btn.active {
  color: #fff !important;
}

.source-selector-wrapper button:not(.active):hover {
  color: var(--vp-c-text-1);
}

.selector-slider {
  position: absolute;
  top: 0.25rem;
  left: 0.25rem;
  bottom: 0.25rem;
  width: calc(50% - 0.25rem);
  background: linear-gradient(135deg, hsl(265, 80%, 55%), hsl(200, 80%, 50%));
  border-radius: 50px;
  z-index: 1;
  transition: transform 0.35s cubic-bezier(0.25, 0.8, 0.25, 1);
  box-shadow: 0 4px 10px rgba(138, 43, 226, 0.25);
}

/* 核心推荐卡片区 */
.hero-download-section {
  margin-bottom: 4rem;
}

.hero-card {
  position: relative;
  background: rgba(255, 255, 255, 0.4);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.4);
  border-radius: 24px;
  padding: 3rem;
  text-align: center;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.05);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.dark .hero-card {
  background: rgba(30, 30, 35, 0.45);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.25);
}

.hero-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 25px 50px rgba(138, 43, 226, 0.1);
  border-color: rgba(138, 43, 226, 0.2);
}

.dark .hero-card:hover {
  box-shadow: 0 25px 50px rgba(138, 43, 226, 0.2);
  border-color: rgba(138, 43, 226, 0.3);
}

.recommended-badge {
  position: absolute;
  top: 1.25rem;
  right: 1.5rem;
  background: linear-gradient(135deg, hsl(265, 80%, 55%), hsl(200, 80%, 50%));
  color: #fff;
  font-size: 0.8rem;
  font-weight: 600;
  padding: 0.35rem 0.85rem;
  border-radius: 50px;
  letter-spacing: 0.05em;
  box-shadow: 0 2px 6px rgba(138, 43, 226, 0.2);
}

.system-icon-wrapper {
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-brand-1);
  width: 90px;
  height: 90px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 0.5rem;
  box-shadow: 0 8px 16px rgba(0, 0, 0, 0.03);
}

.system-svg {
  width: 48px;
  height: 48px;
  fill: currentColor;
}

.recommend-info h2 {
  font-size: 1.8rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.chip-badge {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  font-size: 0.85rem;
  font-weight: 600;
  padding: 0.2rem 0.6rem;
  border-radius: 6px;
}

.filename-text {
  color: var(--vp-c-text-2);
  font-size: 0.95rem;
  margin-bottom: 0.25rem;
}

.filename-text code {
  background: var(--vp-c-bg-alt);
  padding: 0.2rem 0.5rem;
  border-radius: 4px;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.85rem;
}

.version-tag {
  font-size: 0.9rem;
  color: var(--vp-c-text-3);
}

.version-tag .ver {
  font-weight: 600;
  color: var(--vp-c-text-1);
}

/* 极具动感的流光大按钮 */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  font-weight: 600;
  text-decoration: none !important;
  border-radius: 12px;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  cursor: pointer;
}

.btn-primary {
  color: #ffffff !important;
  background: linear-gradient(135deg, hsl(265, 80%, 55%), hsl(200, 80%, 50%));
  border: none;
  box-shadow: 0 8px 24px rgba(138, 43, 226, 0.3);
}

.btn-hero {
  width: 260px;
  height: 58px;
  font-size: 1.15rem;
  border-radius: 50px;
}

.btn-primary:hover {
  transform: scale(1.03);
  box-shadow: 0 12px 30px rgba(138, 43, 226, 0.45);
  background: linear-gradient(135deg, hsl(265, 85%, 60%), hsl(200, 85%, 55%));
}

.btn-primary:active {
  transform: scale(0.98);
}

.btn-icon {
  width: 22px;
  height: 22px;
}

.mirror-selector-wrapper {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
  width: 100%;
  max-width: 500px;
  margin-top: 0.5rem;
}

.mirror-tip {
  font-size: 0.85rem;
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
  padding: 0.6rem 1.2rem;
  border-radius: 12px;
  font-weight: 500;
  line-height: 1.5;
  width: 100%;
}

.mirror-security-note {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  opacity: 0.85;
  line-height: 1.4;
  border-top: 1px dashed rgba(var(--vp-c-brand-1-rgb, 138, 43, 226), 0.2);
  padding-top: 0.4rem;
}

.mirror-dropdown-container {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  padding: 0.4rem 0.75rem;
  border-radius: 50px;
  width: 100%;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.03);
}

.mirror-select-icon {
  font-size: 1rem;
}

.mirror-select {
  flex: 1;
  background: transparent;
  border: none;
  font-family: inherit;
  font-size: 0.85rem;
  font-weight: 600;
  color: var(--vp-c-text-1);
  cursor: pointer;
  outline: none;
  padding-right: 1.5rem;
}

.retest-btn {
  background: var(--vp-c-bg-alt);
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-2);
  font-family: inherit;
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.3rem 0.75rem;
  border-radius: 50px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.retest-btn:hover:not(:disabled) {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
}

.mini-spinner {
  width: 12px;
  height: 12px;
  border: 2px solid var(--vp-c-divider);
  border-top-color: var(--vp-c-brand-1);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

/* 全部版本矩阵区 */
.all-platforms-section {
  margin-bottom: 4rem;
}

.section-title {
  font-size: 1.75rem;
  font-weight: 700;
  text-align: center;
  margin-bottom: 0.5rem;
}

.section-subtitle {
  font-size: 1rem;
  color: var(--vp-c-text-2);
  text-align: center;
  margin-bottom: 2.5rem;
}

.cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 1.5rem;
}

.platform-card {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 16px;
  padding: 1.75rem;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  min-height: 330px;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.platform-card:hover {
  transform: translateY(-4px);
  border-color: var(--vp-c-brand-1);
  box-shadow: 0 10px 20px rgba(0, 0, 0, 0.05);
}

.highlight-border {
  border: 2px solid var(--vp-c-brand-1) !important;
  box-shadow: 0 8px 24px rgba(138, 43, 226, 0.08);
}

.card-action-buttons {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  width: 100%;
  margin-top: 1rem;
}

.btn-mini-icon {
  width: 16px;
  height: 16px;
}

.btn-platform-oss {
  width: 100%;
  height: 42px;
  color: #ffffff !important;
  background: linear-gradient(135deg, hsl(265, 80%, 55%), hsl(200, 80%, 50%));
  border: none;
  font-size: 0.9rem;
  box-shadow: 0 4px 12px rgba(138, 43, 226, 0.15);
}

.btn-platform-oss:hover {
  transform: scale(1.02);
  box-shadow: 0 6px 16px rgba(138, 43, 226, 0.25);
  background: linear-gradient(135deg, hsl(265, 85%, 60%), hsl(200, 85%, 55%));
}

.btn-platform-github {
  width: 100%;
  height: 42px;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-1) !important;
  border: 1px solid var(--vp-c-divider);
  font-size: 0.9rem;
}

.btn-platform-github:hover {
  background: var(--vp-c-bg-mute);
  color: var(--vp-c-brand-1) !important;
  border-color: var(--vp-c-brand-1);
  transform: scale(1.02);
}

.card-os-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 1.25rem;
}

.os-name {
  font-size: 0.85rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--vp-c-text-2);
}

.current-tag {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
  font-size: 0.75rem;
  font-weight: 600;
  padding: 0.15rem 0.5rem;
  border-radius: 50px;
}

.card-body h3 {
  font-size: 1.25rem;
  font-weight: 700;
  margin-bottom: 0.5rem;
}

.arch-desc {
  font-size: 0.9rem;
  color: var(--vp-c-text-2);
  line-height: 1.5;
  margin-bottom: 1rem;
}

.ext-tag {
  display: inline-block;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-3);
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  padding: 0.15rem 0.4rem;
  border-radius: 4px;
  margin-bottom: 1.5rem;
}

.btn-secondary {
  width: 100%;
  height: 44px;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-1) !important;
  border: 1px solid var(--vp-c-divider);
}

.btn-secondary:hover {
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1) !important;
  border-color: var(--vp-c-brand-1);
}

/* 避坑与指南区 */
.guides-section {
  background: var(--vp-c-bg-soft);
  border: 1px solid var(--vp-c-divider);
  border-radius: 20px;
  padding: 2.5rem;
}

.guides-section .section-title {
  margin-bottom: 2rem;
  text-align: left;
}

.accordion {
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.guide-item {
  border-bottom: 1px solid var(--vp-c-divider);
  padding-bottom: 1.5rem;
}

.guide-item:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.guide-title {
  font-size: 1.2rem;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 1rem;
  color: var(--vp-c-text-1);
}

.guide-content p {
  font-size: 0.95rem;
  color: var(--vp-c-text-2);
  line-height: 1.6;
  margin-bottom: 0.75rem;
}

.bold {
  font-weight: 600;
}

/* 终端命令容器一键复制 */
.code-container {
  margin: 1.25rem 0;
  border-radius: 12px;
  overflow: hidden;
  border: 1px solid var(--vp-c-divider);
  background: #1e1e24; /* 强制使用好看的深色终端背景 */
}

.code-lang {
  display: block;
  background: #141418;
  color: #8e8e9c;
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.75rem;
  padding: 0.4rem 1rem;
  border-bottom: 1px solid #2d2d38;
}

.code-body {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.85rem 1.25rem;
  gap: 1rem;
}

.code-body code {
  color: #64e5ad; /* 终端翠绿 */
  font-family: 'JetBrains Mono', monospace;
  font-size: 0.9rem;
  overflow-x: auto;
  white-space: nowrap;
}

.copy-btn {
  flex-shrink: 0;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.15);
  color: #fff;
  font-family: inherit;
  font-size: 0.8rem;
  font-weight: 600;
  padding: 0.4rem 0.85rem;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.copy-btn:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.3);
}

.copy-btn.copied {
  background: #10b981;
  border-color: #10b981;
  box-shadow: 0 0 10px rgba(16, 185, 129, 0.3);
}

/* 提示盒子 */
.tip-box {
  border-radius: 10px;
  padding: 1.25rem;
  margin-top: 1rem;
  font-size: 0.95rem;
  line-height: 1.6;
}

.tip-box.info {
  background: var(--vp-c-brand-soft);
  border-left: 4px solid var(--vp-c-brand-1);
}

.tip-box.success {
  background: rgba(16, 185, 129, 0.08);
  border-left: 4px solid #10b981;
}

.tip-box ol {
  margin: 0.5rem 0 0 1.25rem;
}

.tip-box li {
  margin-bottom: 0.4rem;
}

/* Spinner 加载状态 */
.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid var(--vp-c-divider);
  border-top-color: var(--vp-c-brand-1);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 1rem;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 简单的进入动画 */
.fade-in {
  animation: fadeIn 0.6s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 移动端适配 */
@media (max-width: 640px) {
  .download-header h1 {
    font-size: 2.25rem;
  }
  .hero-card {
    padding: 2rem 1.5rem;
  }
  .cards-grid {
    grid-template-columns: 1fr;
  }
  .source-selector {
    width: 100%;
  }
  .guides-section {
    padding: 1.5rem;
  }
  .code-body {
    flex-direction: column;
    align-items: flex-start;
    gap: 0.75rem;
  }
  .copy-btn {
    width: 100%;
  }
}
</style>
