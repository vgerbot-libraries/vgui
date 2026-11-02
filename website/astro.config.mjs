import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://vgerbot-libraries.github.io',
  base: '/vgui',
  output: 'static',
  integrations: [
    starlight({
      title: {
        en: 'vgui',
        'zh-CN': 'vgui',
      },
      description: 'A declarative, reactive GUI framework for Rust, built on GPUI.',
      defaultLocale: 'root',
      locales: {
        root: { label: 'English', lang: 'en' },
        'zh-cn': { label: '简体中文', lang: 'zh-CN' },
      },
      social: [
        {
          icon: 'github',
          label: 'GitHub',
          href: 'https://github.com/vgerbot-libraries/vgui',
        },
      ],
      sidebar: [
        {
          label: 'Start Here',
          translations: { 'zh-CN': '从这里开始' },
          items: [
            {
              label: 'Introduction',
              translations: { 'zh-CN': '简介' },
              link: '/docs/',
            },
            {
              label: 'Installation',
              translations: { 'zh-CN': '安装' },
              link: '/docs/installation/',
            },
            {
              label: 'Quick Start',
              translations: { 'zh-CN': '快速开始' },
              link: '/docs/quick-start/',
            },
            {
              label: 'Your First Reactive View',
              translations: { 'zh-CN': '第一个响应式视图' },
              link: '/docs/first-reactive-view/',
            },
            {
              label: 'Native & Web',
              translations: { 'zh-CN': '原生与 Web' },
              link: '/docs/native-and-web/',
            },
            {
              label: 'Compatibility & Troubleshooting',
              translations: { 'zh-CN': '兼容性与故障排查' },
              link: '/docs/compatibility-troubleshooting/',
            },
          ],
        },
        {
          label: 'Components',
          translations: { 'zh-CN': '组件' },
          link: '/components/',
        },
      ],
      customCss: ['./src/styles/custom.css'],
      components: {
        Header: './src/components/Header.astro',
        Sidebar: './src/components/Sidebar.astro',
      },
      favicon: '/favicon.svg',
      pagination: false,
      lastUpdated: false,
    }),
  ],
});
