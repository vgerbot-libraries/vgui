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
          label: 'Docs',
          translations: { 'zh-CN': '文档' },
          link: '/docs/',
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
