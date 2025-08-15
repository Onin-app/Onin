export interface PluginInfo {
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  loaded: boolean;
  status: 'active' | 'inactive' | 'error' | 'loading';
  permissions?: string[];
  path?: string;
  main?: string;
  engines?: Record<string, string>;
  keywords?: string[];
  repository?: string;
}

export interface PluginManifest {
  name: string;
  version: string;
  description: string;
  author: string;
  main: string;
  permissions: string[];
  engines: {
    baize: string;
  };
  keywords?: string[];
  repository?: string;
}

export interface PluginError {
  plugin: string;
  message: string;
  type: 'load_failed' | 'invalid_manifest' | 'permission_denied' | 'version_incompatible';
}