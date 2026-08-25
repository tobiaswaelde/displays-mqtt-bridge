import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'Displays MQTT Bridge',
  description: 'DDC/CI display control through MQTT',
  base: '/displays-mqtt-bridge/',
  cleanUrls: true,
  lastUpdated: true,
  head: [
    [
      'link',
      { rel: 'icon', type: 'image/svg+xml', href: '/displays-mqtt-bridge/favicon.svg' },
    ],
  ],
  themeConfig: {
    logo: '/logo.svg',
    nav: [
      { text: 'Guide', link: '/' },
      { text: 'Configuration', link: '/configuration' },
      { text: 'MQTT', link: '/mqtt' },
      { text: 'Hardware', link: '/hardware' },
      { text: 'Deployment', link: '/deployment' },
    ],
    sidebar: [
      { text: 'Overview', link: '/' },
      { text: 'Getting started', link: '/getting-started' },
      { text: 'Configuration', link: '/configuration' },
      { text: 'MQTT contract', link: '/mqtt' },
      { text: 'Hardware requirements', link: '/hardware' },
      { text: 'Docker deployment', link: '/deployment' },
      { text: 'Troubleshooting', link: '/troubleshooting' },
    ],
    editLink: {
      pattern: 'https://github.com/tobiaswaelde/displays-mqtt-bridge/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },
    socialLinks: [{ icon: 'github', link: 'https://github.com/tobiaswaelde/displays-mqtt-bridge' }],
  },
});
