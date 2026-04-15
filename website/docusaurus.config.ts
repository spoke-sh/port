import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';

const siteUrl = process.env.DOCS_SITE_URL ?? 'https://port.spoke.sh';
const baseUrl = process.env.DOCS_BASE_URL ?? '/';
const repoUrl = 'https://github.com/spoke-sh/port';

const config: Config = {
  title: 'Port',
  tagline: 'One operator vocabulary for local and hosted microVM compute',
  favicon: 'img/favicon.svg',
  future: {
    v4: true,
  },
  url: siteUrl,
  baseUrl,
  organizationName: 'spoke-sh',
  projectName: 'port',
  onBrokenLinks: 'throw',
  markdown: {
    hooks: {
      onBrokenMarkdownLinks: 'throw',
    },
  },
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },
  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          routeBasePath: 'docs',
          editUrl: `${repoUrl}/tree/main/website/`,
          showLastUpdateAuthor: false,
          showLastUpdateTime: true,
        },
        blog: false,
        theme: {
          customCss: './src/css/custom.css',
        },
      } satisfies Preset.Options,
    ],
  ],
  themeConfig: {
    image: 'img/port-social-card.svg',
    metadata: [
      {
        property: 'og:site_name',
        content: 'Port',
      },
    ],
    colorMode: {
      defaultMode: 'light',
      disableSwitch: true,
      respectPrefersColorScheme: false,
    },
    navbar: {
      title: 'Port',
      items: [
        {
          type: 'doc',
          docId: 'intro',
          label: 'Docs',
          position: 'left',
        },
        {
          to: '/docs/start-here/local-first',
          label: 'Start Here',
          position: 'left',
        },
        {
          to: '/docs/path-to-production/overview',
          label: 'Path To Production',
          position: 'left',
        },
        {
          to: '/docs/hosts/linux',
          label: 'Hosts',
          position: 'left',
        },
        {
          href: 'https://www.spoke.sh',
          label: 'Spoke',
          position: 'right',
        },
        {
          href: repoUrl,
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Start Here',
          items: [
            {
              label: 'Port, Explained',
              to: '/docs/intro',
            },
            {
              label: 'Install Port',
              to: '/docs/start-here/install-port',
            },
            {
              label: 'Local Narrative',
              to: '/docs/start-here/local-first',
            },
          ],
        },
        {
          title: 'Production Paths',
          items: [
            {
              label: 'Overview',
              to: '/docs/path-to-production/overview',
            },
            {
              label: 'AWS',
              to: '/docs/path-to-production/aws',
            },
            {
              label: 'GCP',
              to: '/docs/path-to-production/gcp',
            },
            {
              label: 'Azure',
              to: '/docs/path-to-production/azure',
            },
          ],
        },
        {
          title: 'Project',
          items: [
            {
              label: 'GitHub',
              href: repoUrl,
            },
            {
              label: 'Foundational Docs',
              to: '/docs/reference/foundational-docs',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Port contributors.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
