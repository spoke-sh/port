import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';

import styles from './index.module.css';

const narrativeTracks = [
  {
    title: 'Local First',
    body:
      'Start with the strongest Port feedback loop: local Linux Firecracker or macOS AVF, one operator vocabulary, and proof you can run yourself.',
    href: '/docs/start-here/local-first',
    cta: 'Read the local path',
  },
  {
    title: 'Path To Production',
    body:
      'Move from local proof to hosted control-plane operation without changing the core Port verbs. The site keeps the current boundaries explicit instead of implying platform magic.',
    href: '/docs/path-to-production/overview',
    cta: 'Read the production path',
  },
  {
    title: 'Host Guides',
    body:
      'Choose the right operator host for your team: Linux for the deepest runtime lane, macOS for AVF and control work, Windows through WSL or remote Linux.',
    href: '/docs/hosts/linux',
    cta: 'Browse host guides',
  },
];

const cloudTracks = [
  {
    title: 'AWS',
    body:
      'The clearest current cloud narrative: hosted control plane, registered AWS Linux node, standard Firecracker lane, and proof-backed external app hosting.',
    href: '/docs/path-to-production/aws',
  },
  {
    title: 'GCP',
    body:
      'The same hosted operating model with a GCP-aligned node and machine identity, useful when the provider decision is already set but the Port contract should stay stable.',
    href: '/docs/path-to-production/gcp',
  },
  {
    title: 'Azure',
    body:
      'An honest boundary track. Azure is modeled explicitly, but the current Firecracker MVP is not a shipped Azure lane yet. The docs explain what is true today and what must change.',
    href: '/docs/path-to-production/azure',
  },
];

const hostTracks = [
  {
    title: 'Linux',
    body: 'Primary Port runtime host for local Firecracker, local cluster work, hosted control-plane demos, and SSH-managed execution.',
    href: '/docs/hosts/linux',
  },
  {
    title: 'macOS',
    body: 'First-class local operator path through Apple Virtualization Framework, plus a strong control workstation for Linux-hosted environments.',
    href: '/docs/hosts/macos',
  },
  {
    title: 'Windows',
    body: 'A workstation story through WSL or a remote Linux host. The site keeps native-package limits explicit while still showing a workable operator path.',
    href: '/docs/hosts/windows',
  },
];

export default function Home(): JSX.Element {
  return (
    <Layout
      title="Port"
      description="User-facing Port docs for local and hosted microVM workflows.">
      <main className={styles.page}>
        <section className={styles.hero}>
          <div className={styles.heroBackdrop} />
          <div className={clsx('container', styles.heroInner)}>
            <div className={styles.heroPanel}>
              <p className={styles.eyebrow}>Agentic Compute Orchestration</p>
              <Heading as="h1" className={styles.title}>
                Port turns microVM infrastructure into one readable operator
                surface.
              </Heading>
              <p className={styles.subtitle}>
                Use the same `port` vocabulary across local bring-up, hosted
                control planes, and cloud-shaped rollout paths. Start locally,
                then follow the production narratives without rewriting the
                mental model.
              </p>
              <div className={styles.heroActions}>
                <Link className="button button--primary button--lg" to="/docs/intro">
                  Read the Docs
                </Link>
                <Link
                  className="button button--secondary button--lg"
                  to="/docs/path-to-production/overview">
                  Path To Production
                </Link>
              </div>
              <div className={styles.statRow}>
                <div className={styles.statCard}>
                  <span className={styles.statLabel}>Best Current Lane</span>
                  <strong>Linux local + hosted standard</strong>
                </div>
                <div className={styles.statCard}>
                  <span className={styles.statLabel}>Cloud Focus</span>
                  <strong>AWS, GCP, Azure boundaries</strong>
                </div>
                <div className={styles.statCard}>
                  <span className={styles.statLabel}>Host Coverage</span>
                  <strong>Linux, macOS, Windows</strong>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className={clsx('container', styles.section)}>
          <div className={styles.sectionHeader}>
            <p className={styles.kicker}>Narrative Tracks</p>
            <Heading as="h2" className={styles.sectionTitle}>
              Read Port from the operator path outward.
            </Heading>
            <p className={styles.sectionBody}>
              The site is organized around the decisions operators actually make:
              where to start, how to move toward production, and which host lane
              to trust.
            </p>
          </div>
          <div className={styles.cardGrid}>
            {narrativeTracks.map((track) => (
              <Link
                key={track.title}
                className={styles.card}
                to={track.href}>
                <Heading as="h3" className={styles.cardTitle}>
                  {track.title}
                </Heading>
                <p className={styles.cardBody}>{track.body}</p>
                <span className={styles.cardCta}>{track.cta}</span>
              </Link>
            ))}
          </div>
        </section>

        <section className={clsx('container', styles.section)}>
          <div className={styles.band}>
            <div className={styles.sectionHeader}>
              <p className={styles.kicker}>Major Cloud Paths</p>
              <Heading as="h2" className={styles.sectionTitle}>
                Make cloud decisions without blurring the current truth.
              </Heading>
              <p className={styles.sectionBody}>
                Port’s strongest public docs story is now the staged path from a
                local proof to a provider-shaped rollout. AWS and GCP have the
                clearest hosted narrative today. Azure stays explicit as a real
                design target with current runtime limits.
              </p>
            </div>
            <div className={styles.cloudGrid}>
              {cloudTracks.map((track) => (
                <Link
                  key={track.title}
                  className={styles.cloudCard}
                  to={track.href}>
                  <Heading as="h3" className={styles.cloudTitle}>
                    {track.title}
                  </Heading>
                  <p className={styles.cloudBody}>{track.body}</p>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={clsx('container', styles.section, styles.sectionTight)}>
          <div className={styles.sectionHeader}>
            <p className={styles.kicker}>Host Platforms</p>
            <Heading as="h2" className={styles.sectionTitle}>
              Pick the right workstation and runtime host deliberately.
            </Heading>
          </div>
          <div className={styles.hostGrid}>
            {hostTracks.map((track) => (
              <Link
                key={track.title}
                className={styles.hostCard}
                to={track.href}>
                <Heading as="h3" className={styles.hostTitle}>
                  {track.title}
                </Heading>
                <p className={styles.hostBody}>{track.body}</p>
              </Link>
            ))}
          </div>
        </section>
      </main>
    </Layout>
  );
}
