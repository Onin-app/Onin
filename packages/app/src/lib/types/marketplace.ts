// 插件市场类型定义（直接使用后端格式）

export interface MarketplacePlugin {
  id: string;
  folder: string;
  name: string;
  description: string;
  author: string;
  repository: string;
  icon: string;
  category: string;
  keywords: string[];
  addedAt: number;
  license?: string;
  downloads: number;
  stars: number;
  // 下载相关字段（仅在详情接口返回）
  downloadUrl?: string;
  version?: string;
  size?: number;
  checksum?: string;
  releaseDate?: string;
  releaseNotes?: string;
  readme?: string;
}

export interface FetchPluginsParams {
  page?: number;
  limit?: number;
  category?: string;
  keyword?: string;
  sort?: "downloads" | "rating" | "updated" | "name";
}

// 后端 API 返回格式
export interface PluginListResponse {
  data: MarketplacePlugin[];
  meta: {
    total: number;
    page: number;
    limit: number;
    totalPages: number;
  };
}
