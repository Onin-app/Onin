import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'Onin',
  description: '专为开发者和高效用户打造的键盘启动器',
  lang: 'zh-CN',
  base: '/Onin/',

  themeConfig: {
    logo: '/logo.png',
    siteTitle: 'Onin',

    nav: [
      { text: '用户指南', link: '/guide/' },
      { text: '插件开发', link: '/plugin-dev/' },
      { text: 'SDK API', link: '/sdk/' },
      {
        text: 'GitHub',
        link: 'https://github.com/b-yp/Onin',
      },
    ],

    sidebar: {
      '/guide/': [
        {
          text: '用户指南',
          items: [
            { text: '什么是 Onin', link: '/guide/' },
            { text: '下载与安装', link: '/guide/installation' },
            { text: '基础用法', link: '/guide/basic-usage' },
            { text: '内置扩展', link: '/guide/built-in' },
          ],
        },
      ],

      '/plugin-dev/': [
        {
          text: '插件开发',
          items: [
            { text: '核心概念', link: '/plugin-dev/' },
            { text: '5 分钟快速开始', link: '/plugin-dev/quickstart' },
            { text: 'manifest.json 详解', link: '/plugin-dev/manifest' },
            { text: '显示模式', link: '/plugin-dev/display-modes' },
            { text: '权限配置', link: '/plugin-dev/permissions' },
          ],
        },
      ],

      '/sdk/': [
        {
          text: 'SDK API 参考',
          items: [
            { text: '概览', link: '/sdk/' },
            { text: 'command', link: '/sdk/command' },
            { text: 'storage', link: '/sdk/storage' },
            { text: 'http', link: '/sdk/http' },
            { text: 'fs', link: '/sdk/fs' },
            { text: 'clipboard', link: '/sdk/clipboard' },
            { text: 'dialog', link: '/sdk/dialog' },
            { text: 'notification', link: '/sdk/notification' },
            { text: 'scheduler', link: '/sdk/scheduler' },
            { text: 'lifecycle', link: '/sdk/lifecycle' },
            { text: 'settings', link: '/sdk/settings' },
            { text: 'pluginWindow', link: '/sdk/window' },
            { text: 'ai', link: '/sdk/ai' },
          ],
        },
      ],
    },

    socialLinks: [{ icon: 'github', link: 'https://github.com/b-yp/Onin' }],

    editLink: {
      pattern: 'https://github.com/b-yp/Onin/edit/main/packages/docs/:path',
      text: '在 GitHub 上编辑此页',
    },

    footer: {
      message: '基于 MIT 协议发布',
      copyright: 'Copyright © 2024 Onin Team',
    },

    search: {
      provider: 'local',
    },

    docFooter: {
      prev: '上一页',
      next: '下一页',
    },

    outline: {
      label: '本页目录',
    },

    returnToTopLabel: '回到顶部',
    sidebarMenuLabel: '菜单',
    darkModeSwitchLabel: '主题',
    lightModeSwitchTitle: '切换到浅色模式',
    darkModeSwitchTitle: '切换到深色模式',
  },

  head: [['link', { rel: 'icon', href: '/logo.png' }]],
});
