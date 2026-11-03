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
          label: 'Core Concepts',
          translations: { 'zh-CN': '核心概念' },
          items: [
            {
              label: 'Architecture',
              translations: { 'zh-CN': '架构' },
              link: '/docs/architecture/',
            },
            {
              label: 'The view! Macro',
              translations: { 'zh-CN': 'view! 宏' },
              link: '/docs/view-macro/',
            },
            {
              label: 'Reactivity',
              translations: { 'zh-CN': '响应式状态' },
              link: '/docs/reactivity/',
            },
            {
              label: 'Control Flow',
              translations: { 'zh-CN': '控制流' },
              link: '/docs/control-flow/',
            },
            {
              label: 'Custom Components',
              translations: { 'zh-CN': '自定义组件' },
              link: '/docs/custom-components/',
            },
          ],
        },
        {
          label: 'Styling',
          translations: { 'zh-CN': '样式' },
          items: [
            {
              label: 'Overview',
              translations: { 'zh-CN': '概述' },
              link: '/docs/overview/',
            },
            {
              label: 'Tailwind & Variants',
              translations: { 'zh-CN': 'Tailwind 与 Variants' },
              link: '/docs/tailwind-and-variants/',
            },
            {
              label: 'Theming & Animation',
              translations: { 'zh-CN': '主题与动画' },
              link: '/docs/theming-and-animation/',
            },
          ],
        },
        {
          label: 'Elements & State',
          translations: { 'zh-CN': '元素与状态' },
          items: [
            {
              label: 'Built-in Elements & Events',
              translations: { 'zh-CN': '内置元素与事件' },
              link: '/docs/built-in-elements-and-events/',
            },
            {
              label: 'Inputs & Forms',
              translations: { 'zh-CN': '输入与表单' },
              link: '/docs/inputs-and-forms/',
            },
            {
              label: 'Refs & Context',
              translations: { 'zh-CN': 'Refs 与 Context' },
              link: '/docs/refs-and-context/',
            },
          ],
        },
        {
          label: 'Examples',
          translations: { 'zh-CN': '示例' },
          items: [
            {
              label: 'Counter',
              translations: { 'zh-CN': '计数器' },
              link: '/docs/examples/counter/',
            },
            {
              label: 'Store & Lists',
              translations: { 'zh-CN': 'Store 与列表' },
              link: '/docs/examples/store-and-lists/',
            },
            {
              label: 'Events & Inputs',
              translations: { 'zh-CN': '事件与输入' },
              link: '/docs/examples/events-and-inputs/',
            },
          ],
        },
        {
          label: 'Components',
          translations: { 'zh-CN': '组件' },
          items: [
            {
              label: 'Overview',
              translations: { 'zh-CN': '组件概览' },
              link: '/components/',
            },
            {
              label: 'Actions & Form Controls',
              translations: { 'zh-CN': '操作与表单控件' },
              link: '/components/actions-and-form-controls/',
            },
            {
              label: 'Data Display',
              translations: { 'zh-CN': '数据展示' },
              link: '/components/data-display/',
            },
            {
              label: 'Navigation & Layout',
              translations: { 'zh-CN': '导航与布局' },
              link: '/components/navigation-and-layout/',
            },
            {
              label: 'Overlays & Feedback',
              translations: { 'zh-CN': '浮层与反馈' },
              link: '/components/overlays-and-feedback/',
            },
          ],
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
