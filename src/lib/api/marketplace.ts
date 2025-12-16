// 插件市场 API 客户端

import type {
  FetchPluginsParams,
  PluginListResponse,
  MarketplacePlugin,
} from '$lib/types/marketplace';

// API 配置
const API_BASE_URL = import.meta.env.VITE_MARKETPLACE_API_URL || 'https://api.baize.app';
const API_KEY = import.meta.env.VITE_MARKETPLACE_API_KEY || '';

// 通用请求函数
async function request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const url = `${API_BASE_URL}${endpoint}`;

  const headers = new Headers(options.headers);
  if (API_KEY) {
    headers.set('X-API-Key', API_KEY);
  }
  headers.set('Content-Type', 'application/json');

  const response = await fetch(url, {
    ...options,
    headers,
  });

  if (!response.ok) {
    throw new Error(`API request failed: ${response.statusText}`);
  }

  return response.json();
}

// 获取插件列表
export async function fetchPlugins(params: FetchPluginsParams = {}): Promise<PluginListResponse> {
  const queryParams = new URLSearchParams();

  if (params.page) queryParams.set('page', params.page.toString());
  if (params.limit) queryParams.set('limit', params.limit.toString());
  if (params.category) queryParams.set('category', params.category);
  if (params.keyword) queryParams.set('keyword', params.keyword);

  const query = queryParams.toString();
  const endpoint = `/api/v1/plugins${query ? `?${query}` : ''}`;

  return request<PluginListResponse>(endpoint);
}

// 获取插件详情
export async function fetchPluginDetail(pluginId: string): Promise<MarketplacePlugin> {
  const response = await request<{ data: MarketplacePlugin }>(`/api/v1/plugins/${pluginId}`);
  console.log("[API] Plugin detail response:", {
    pluginId,
    hasReleaseNotes: !!response.data.releaseNotes,
    releaseNotesLength: response.data.releaseNotes?.length,
    data: response.data
  });
  return response.data;
}

// 下载并安装插件
export async function downloadAndInstallPlugin(
  downloadUrl: string,
  pluginId: string,
  iconUrl?: string
): Promise<void> {
  const { invoke } = await import('@tauri-apps/api/core');

  await invoke('download_and_install_plugin', {
    downloadUrl,
    pluginId,
    iconUrl,
  });
}
