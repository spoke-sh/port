import type {SidebarsConfig} from '@docusaurus/plugin-content-docs';

const sidebars: SidebarsConfig = {
  docsSidebar: [
    'intro',
    {
      type: 'category',
      label: 'Start Here',
      items: ['start-here/install-port', 'start-here/local-first'],
    },
    {
      type: 'category',
      label: 'Path To Production',
      items: [
        'path-to-production/overview',
        'path-to-production/aws',
        'path-to-production/gcp',
        'path-to-production/azure',
      ],
    },
    {
      type: 'category',
      label: 'Host Guides',
      items: ['hosts/linux', 'hosts/macos', 'hosts/windows'],
    },
    {
      type: 'category',
      label: 'Reference',
      items: ['reference/foundational-docs'],
    },
  ],
};

export default sidebars;
