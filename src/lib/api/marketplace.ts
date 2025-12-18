// 插件市场 API 客户端

import type {
  FetchPluginsParams,
  PluginListResponse,
  MarketplacePlugin,
} from '$lib/types/marketplace';
import type { AppConfig } from '$lib/type';
import { toast } from 'svelte-sonner';

// API 配置
const API_KEY = import.meta.env.VITE_MARKETPLACE_API_KEY || '';

// 获取 API Base URL
async function getApiBaseUrl(): Promise<string | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const config = await invoke<AppConfig>('get_app_config');

    if (!config.marketplace_api_url) {
      toast.warning('请先在设置中配置插件市场 API 地址');
      return null;
    }

    return config.marketplace_api_url;
  } catch (error) {
    console.error('Failed to get marketplace API URL from config:', error);
    toast.error('获取配置失败，请检查设置');
    return null;
  }
}

// 通用请求函数
async function request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
  const API_BASE_URL = await getApiBaseUrl();

  if (!API_BASE_URL) {
    throw new Error('Marketplace API URL not configured');
  }

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
