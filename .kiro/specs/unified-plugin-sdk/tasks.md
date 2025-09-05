# 实现计划

- [ ] 1. 设置项目结构和构建配置
  - 创建 types、core、adapters 和 utils 目录结构
  - 配置 Vite 库模式构建和正确的外部依赖
  - 设置 TypeScript 库开发配置
  - _需求: 5.1, 5.3_

- [ ] 2. 实现核心类型定义
  - 创建 PluginEnvironment、ApiResponse、NotificationOptions 的 TypeScript 接口
  - 定义包含所有必需方法的 PluginSDKFunctions 接口
  - 创建错误处理类型和常量
  - _需求: 2.1, 2.2, 2.3, 2.4_

- [ ] 3. 实现环境检测功能
  - 创建 detectEnvironment 函数来识别 headless 和 UI 环境
  - 添加检测 Deno.core.ops 的逻辑用于 headless 环境
  - 添加检测 window.__TAURI__ 的逻辑用于 UI 环境
  - 处理未知环境情况并提供适当的错误处理
  - _需求: 1.1, 1.2, 1.3, 4.1_

- [ ] 4. 创建参数验证工具
  - 实现 validateNotificationOptions 函数并进行适当的类型检查
  - 创建通用验证管道函数
  - 为验证失败添加全面的错误消息
  - 编写验证辅助函数以便将来扩展
  - _需求: 4.3, 2.3, 2.4_

- [ ] 5. 实现 headless 环境适配器
  - 创建返回 PluginSDKFunctions 的 createHeadlessAdapter 函数
  - 使用 Deno.core.ops.op_invoke 实现 showNotification 方法
  - 添加适当的错误处理和响应格式化
  - 确保与现有 op_invoke 后端实现的兼容性
  - _需求: 1.2, 3.2, 4.2, 1.4, 1.5_

- [ ] 6. 实现 UI 环境适配器
  - 创建返回 PluginSDKFunctions 的 createUIAdapter 函数
  - 使用 @tauri-apps/api/core invoke 实现 showNotification 方法
  - 添加适当的错误处理和响应格式化
  - 确保与现有 Tauri 命令后端的兼容性
  - _需求: 1.3, 3.3, 4.2, 1.4, 1.5_

- [ ] 7. 创建主 SDK 工厂函数
  - 实现带环境检测的 createPluginSDK 函数
  - 为不支持的环境添加适当的错误处理
  - 创建便捷的 sdk 导出以供直接使用
  - 确保基于检测到的环境进行适当的适配器选择
  - _需求: 1.1, 4.1, 5.3_

- [ ] 8. 创建主入口点和导出
  - 设置包含所有公共 API 导出的 src/index.ts
  - 导出所有类型、函数和常量
  - 确保正确的 TypeScript 声明生成
  - 组织导出以获得最佳的开发者体验
  - _需求: 2.1, 2.2, 5.3_

- [ ] 9. 设置综合测试套件
  - 配置 Vitest 进行单元测试
  - 为 headless 和 UI 测试创建模拟环境
  - 编写环境检测功能的测试
  - 使用各种输入场景测试参数验证
  - _需求: 4.1, 4.3, 4.4_

- [ ] 10. 测试适配器实现
  - 使用模拟的 Deno.core.ops 为 headless 适配器编写单元测试
  - 使用模拟的 Tauri API 为 UI 适配器编写单元测试
  - 测试两个适配器的错误处理场景
  - 验证适配器间响应格式的一致性
  - _需求: 1.2, 1.3, 1.4, 1.5, 4.2_

- [ ] 11. 测试主 SDK 集成
  - 为 createPluginSDK 函数编写集成测试
  - 测试环境检测和适配器选择
  - 验证不支持环境的错误处理
  - 测试便捷 sdk 导出功能
  - _需求: 1.1, 4.1, 5.4_

- [ ] 12. 配置构建和分发
  - 设置 Vite 库构建配置
  - 为 @tauri-apps/api/core 配置适当的外部依赖
  - 生成 TypeScript 声明文件
  - 设置包含适当导出和类型的 package.json
  - _需求: 5.3, 2.1_