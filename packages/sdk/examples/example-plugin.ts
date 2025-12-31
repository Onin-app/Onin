import onin, { type PluginManifest } from '@onin/sdk';

// 示例：如何在插件中使用新的权限系统

async function fetchWeatherData() {
  try {
    // 使用 HTTP API 发起请求
    const response = await onin.http.get(
      'https://api.openweathermap.org/data/2.5/weather',
      {
        headers: {
          'Content-Type': 'application/json',
        },
        timeout: 10000,
      },
    );

    console.log('天气数据:', response.body);

    // 存储到本地
    await onin.storage.setItem('last-weather', JSON.stringify(response.body));

    // 显示通知
    await onin.notification.show({
      title: '天气更新',
      body: '天气数据已更新',
      sound: true,
    });
  } catch (error) {
    if (onin.http.errors.isPermissionDeniedError(error)) {
      console.error('权限不足:', error.message);
      console.error(
        '请检查 manifest.json 中的 permissions.http.allowUrls 配置',
      );
    } else if (onin.http.errors.isTimeoutError(error)) {
      console.error('请求超时:', error.message);
    } else {
      console.error('其他错误:', error.message);
    }
  }
}

// 注册命令处理器
onin.command.register('weather-current', async () => {
  await fetchWeatherData();
});

// 示例 manifest.json 配置
const exampleManifest: PluginManifest = {
  id: 'com.example.weather-plugin',
  name: '天气插件',
  version: '1.0.0',
  description: '获取天气信息的示例插件',
  entry: 'index.html',
  permissions: {
    http: {
      enable: true,
      allowUrls: [
        'https://api.openweathermap.org/*',
        'https://*.weather.com/*',
      ],
      timeout: 10000,
      maxRetries: 2,
    },
    storage: {
      enable: true,
      local: true,
      session: false,
      maxSize: '1MB',
    },
    notification: {
      enable: true,
      sound: true,
      badge: false,
    },
    command: {
      enable: true,
      allowCommands: ['weather-*'],
      maxExecutionTime: 5000,
    },
  },
  commands: [
    {
      id: 'weather-current',
      name: '获取当前天气',
      description: '获取当前位置的天气信息',
    },
  ],
};

console.log('示例 manifest 配置:', JSON.stringify(exampleManifest, null, 2));
