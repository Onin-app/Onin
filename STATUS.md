## 目录结构树

```
.
├── .gitignore # Git 忽略文件，用于排除不需要版本控制的文件和目录
├── package.json # Node.js 项目配置文件，定义了项目依赖和脚本
├── pnpm-lock.yaml # pnpm 包管理器的锁定文件，确保依赖版本一致
├── README.md # 项目说明文件，介绍了项目目标、技术栈和进度
├── STATUS.md # (本文) 项目文件结构和状态的动态文档
├── svelte.config.js # SvelteKit 配置文件，用于配置构建和适配器
├── tailwind.config.ts # Tailwind CSS 配置文件，用于定制样式和主题
├── tsconfig.json # TypeScript 配置文件，用于定义编译选项
├── vite.config.ts # Vite 配置文件，用于开发服务器和构建设置
├── .vscode/ # VSCode 编辑器特定配置
│   ├── extensions.json # 推荐的 VSCode 扩展
│   └── settings.json # VSCode 编辑器设置，如保存时自动格式化
├── src/ # Svelte 前端应用源码
│   ├── app.html # SvelteKit 应用的 HTML 入口模板
│   ├── index.css # 全局 CSS 样式文件，引入了 Tailwind CSS
│   ├── main.ts # 前端应用的主入口点，用于初始化图标库等
│   ├── lib/ # SvelteKit 的库目录，存放可重用模块
│   │   ├── type.ts # 定义了应用中使用的 TypeScript 类型，如 AppInfo 和 Theme
│   │   ├── components/ # Svelte UI 组件
│   │   │   ├── Icon.svelte # 一个通用的 SVG 图标组件
│   │   │   └── settings/ # 设置页面相关的组件
│   │   │       ├── GeneralSettings.svelte # “通用设置”页面的 UI 组件
│   │   │       ├── SetItem.svelte # 设置页面中每个设置项的通用布局组件
│   │   │       └── StartupSettings.svelte # “启动设置”页面的 UI 组件，用于管理自定义启动项
│   │   ├── stores/ # Svelte 的状态管理 stores
│   │   │   └── escapeHandler.ts # 一个全局 store，用于管理 ESC 键的处理函数
│   │   └── utils/ # 通用工具函数
│   │       ├── fuzzyMatch.ts # 实现应用列表的模糊搜索功能
│   │       └── theme.ts # 管理应用的主题（亮/暗/跟随系统）
│   └── routes/ # SvelteKit 的基于文件系统的路由
│       ├── +layout.svelte # 全局布局组件，处理全局事件监听
│       ├── +layout.ts # 全局布局的加载器，配置为客户端渲染
│       ├── +page.svelte # 主页面（应用搜索界面）的 Svelte 组件
│       └── settings/ # 设置页面的路由目录
│           └── +page.svelte # 设置页面的 Svelte 组件，包含通用设置和启动设置等标签页
├── src-tauri/ # Tauri 后端 Rust 应用源码
│   ├── .gitignore # Tauri 项目的 Git 忽略文件
│   ├── build.rs # Rust 构建脚本，由 tauri-build 使用
│   ├── Cargo.lock # Cargo 的锁定文件，确保 Rust 依赖版本一致
│   ├── Cargo.toml # Cargo 配置文件，定义了 Rust 依赖和项目元数据
│   ├── tauri.conf.json # Tauri 应用的核心配置文件
│   ├── src/ # Rust 后端源码
│   │   ├── app_cache_manager.rs # 管理已安装应用列表的缓存和后台刷新
│   │   ├── lib.rs # Rust 库的根文件，用于组织和初始化后端逻辑
│   │   ├── main.rs # Rust 应用的入口点
│   │   ├── shortcut_manager.rs # 管理全局快捷键的注册和处理
│   │   ├── startup_apps_manager.rs # 管理用户自定义的启动项列表
│   │   ├── tray_manager.rs # 管理系统托盘图标和菜单
│   │   ├── window_manager.rs # 管理主窗口的事件和行为（如显示/隐藏）
│   │   └── installed_apps/ # 用于发现和管理系统上已安装应用的模块
│   │       ├── exe_to_icon.rs # (Windows) 从 .exe 文件中提取图标
│   │       ├── linux.rs # (Linux) 获取已安装应用的逻辑
│   │       ├── macos.rs # (macOS) 获取已安装应用的逻辑
│   │       ├── mod.rs # `installed_apps` 模块的根文件，定义了公共接口
│   │       └── windows.rs # (Windows) 获取已安装应用的逻辑
└── static/ # 存放静态资源文件
    ├── favicon.png # 网站图标
    ├── ff_logo_dark.svg # 暗黑模式下的 Logo
    ├── ff_logo_light.svg # 明亮模式下的 Logo
    ├── svelte.svg # Svelte Logo
    ├── tauri.svg # Tauri Logo
    └── vite.svg # Vite Logo
```

## 注意项

- 后期要做数据同步
  - 支持 webdav
  - 支持 s3
