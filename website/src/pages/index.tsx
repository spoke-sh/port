import clsx from 'clsx';
import Link from '@docusaurus/Link';
import Layout from '@theme/Layout';

import styles from './index.module.css';

const signalItems = [
  {
    eyebrow: 'One Operator Surface',
    title: 'Keep local and hosted verbs aligned',
    body:
      'Port keeps the same CLI vocabulary across local machines, hosted control planes, and provider-shaped rollout paths instead of forcing a new operating surface at each step.',
    href: '/docs/intro',
    cta: 'Read Port, Explained',
  },
  {
    eyebrow: 'Proof Before Drift',
    title: 'Stay explicit about what works today',
    body:
      'The public narrative keeps the current Firecracker, AVF, AWS, GCP, and Azure boundaries visible so operators can choose a path from real support instead of a vague promise.',
    href: '/docs/path-to-production/overview',
    cta: 'Read the production path',
  },
  {
    eyebrow: 'Host-Aware Guidance',
    title: 'Match the docs to the host you actually have',
    body:
      'Linux, macOS, and Windows each get an explicit Port story. The site shows which host is the runtime truth, which one is the workstation, and how the contract changes.',
    href: '/docs/hosts/linux',
    cta: 'Browse host guides',
  },
];

const laneItems = [
  {
    eyebrow: 'Local First',
    title: 'Prove the runtime where you control the box',
    body:
      'The strongest starting path is still a Linux-backed local proof with explicit cluster, machine, and guest lifecycle. macOS stays first-class through AVF with the same operator verbs.',
    href: '/docs/start-here/local-first',
    cta: 'Follow the local narrative',
  },
  {
    eyebrow: 'Hosted Control Plane',
    title: 'Move to cloud-shaped operation without changing the CLI',
    body:
      'Port’s hosted path keeps `port machine`, `port guest`, and `port cluster` readable while shifting placement, routing, and node ownership into the hosted control-plane model.',
    href: '/docs/path-to-production/overview',
    cta: 'Read the hosted path',
  },
  {
    eyebrow: 'Provider Boundaries',
    title: 'Choose AWS, GCP, or Azure with honest constraints',
    body:
      'AWS and GCP currently have the clearest rollout narrative. Azure remains explicit as a design target with a visible runtime boundary instead of a paper feature.',
    href: '/docs/path-to-production/aws',
    cta: 'See provider tracks',
  },
];

const providerItems = [
  {
    eyebrow: 'AWS',
    title: 'The clearest provider-backed production story',
    body:
      'Hosted standard and hosted PVM paths stay explicit, with AWS-specific node preparation and readiness captured in the same Port operator contract.',
    href: '/docs/path-to-production/aws',
  },
  {
    eyebrow: 'GCP',
    title: 'A provider-aware hosted lane without rewriting the model',
    body:
      'GCP keeps the same hosted control-plane and node vocabulary, making it a good fit when the provider choice is set but the Port workflow should stay stable.',
    href: '/docs/path-to-production/gcp',
  },
  {
    eyebrow: 'Azure',
    title: 'A real track with the current limits left visible',
    body:
      'Azure is modeled in the docs and config surface, but the Firecracker MVP lane is not a shipped Azure path yet. The site keeps that distinction obvious.',
    href: '/docs/path-to-production/azure',
  },
];

const hostItems = [
  {
    eyebrow: 'Linux',
    title: 'Primary runtime host',
    body:
      'Use Linux for the deepest Port story today: local Firecracker, local clusters, hosted control-plane demos, and the strongest path toward hosted provider rollout.',
    href: '/docs/hosts/linux',
  },
  {
    eyebrow: 'macOS',
    title: 'First-class AVF workstation and local lane',
    body:
      'macOS remains a real Port environment through Apple Virtualization Framework, while still acting as a strong control workstation for Linux-hosted execution.',
    href: '/docs/hosts/macos',
  },
  {
    eyebrow: 'Windows',
    title: 'A workstation path through Linux-backed runtime',
    body:
      'Windows stays part of the operator story through WSL or a remote Linux host. The docs keep the runtime boundary honest without forcing a second mental model.',
    href: '/docs/hosts/windows',
  },
];

const operatorLoop = [
  {
    label: 'Verify The Host',
    command: 'port doctor',
    href: '/docs/start-here/install-port',
  },
  {
    label: 'Prove The Local Path',
    command: 'port cluster up',
    href: '/docs/start-here/local-first',
  },
  {
    label: 'Launch A Machine',
    command: 'port machine launch',
    href: '/docs/path-to-production/overview',
  },
  {
    label: 'Inspect Runtime State',
    command: 'port machine status',
    href: '/docs/hosts/linux',
  },
];

export default function Home(): JSX.Element {
  return (
    <Layout
      title="Port"
      description="Port is the public operator docs site for local and hosted microVM workflows.">
      <main className={styles.page}>
        <section className={styles.hero}>
          <div className="container">
            <div className={styles.heroGrid}>
              <div className={styles.heroCopy}>
                <p className={styles.eyebrow}>Agentic Compute Orchestration</p>
                <h1>Operate local and hosted microVM workflows with one readable CLI surface.</h1>
                <p className={styles.lede}>
                  Port keeps the operator contract legible across local proof,
                  hosted control planes, and provider-shaped rollout paths. The
                  docs show what is true today, what host to trust, and how to
                  move toward production without switching mental models.
                </p>
                <div className={styles.actions}>
                  <Link className={styles.primaryAction} to="/docs/intro">
                    Read The Docs
                  </Link>
                  <Link className={styles.secondaryAction} to="/docs/start-here/local-first">
                    Start Local
                  </Link>
                </div>
                <ul className={styles.heroPoints}>
                  <li>Keep the same `port` verbs across local and hosted lanes.</li>
                  <li>Read explicit AWS, GCP, Azure, Linux, macOS, and Windows boundaries.</li>
                  <li>Use the docs to prove the runtime path before expanding scope.</li>
                </ul>
              </div>
              <div className={styles.scenePanel}>
                <div className={styles.sceneFrame}>
                  <div className={styles.sceneChrome} aria-hidden="true">
                    <span />
                    <span />
                    <span />
                  </div>
                  <p className={styles.sceneLabel}>Typical Operator Loop</p>
                  <ol className={styles.sceneSteps}>
                    {operatorLoop.map((item) => (
                      <li key={item.command}>
                        <Link className={styles.sceneStepLink} to={item.href}>
                          <span>{item.label}</span>
                          <code>{item.command}</code>
                        </Link>
                      </li>
                    ))}
                  </ol>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Why Port</p>
              <h2>Port behaves like one operator model, not a stack of disconnected setup guides.</h2>
              <p>
                The public docs stay structured around how operators actually
                work: verify the host, prove the local path, understand the
                current runtime boundary, and then pick the hosted lane that
                matches the target environment.
              </p>
            </div>
            <div className={styles.cardGrid}>
              {signalItems.map((item) => (
                <Link key={item.title} className={styles.card} to={item.href}>
                  <p className={styles.cardEyebrow}>{item.eyebrow}</p>
                  <h3 className={styles.cardTitle}>{item.title}</h3>
                  <p className={styles.cardBody}>{item.body}</p>
                  <span className={styles.cardCta}>{item.cta}</span>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.sectionAlt}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Operating Lanes</p>
              <h2>Start small, keep the truth visible, and scale out without changing the surface.</h2>
              <p>
                Port’s strongest docs story is the progression from local proof
                to hosted control-plane ownership. Each lane keeps the same
                operator vocabulary while making the infrastructure boundary more
                explicit instead of more hidden.
              </p>
            </div>
            <div className={styles.cardGrid}>
              {laneItems.map((item) => (
                <Link key={item.title} className={styles.card} to={item.href}>
                  <p className={styles.cardEyebrow}>{item.eyebrow}</p>
                  <h3 className={styles.cardTitle}>{item.title}</h3>
                  <p className={styles.cardBody}>{item.body}</p>
                  <span className={styles.cardCta}>{item.cta}</span>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.section}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Provider Tracks</p>
              <h2>Pick the rollout path that matches the provider decision you already have.</h2>
              <p>
                The provider guides use the same Port contract, but they stay
                honest about what is shipped, what is partial, and what is still
                an explicit planning boundary.
              </p>
            </div>
            <div className={styles.cardGrid}>
              {providerItems.map((item) => (
                <Link key={item.title} className={styles.card} to={item.href}>
                  <p className={styles.cardEyebrow}>{item.eyebrow}</p>
                  <h3 className={styles.cardTitle}>{item.title}</h3>
                  <p className={styles.cardBody}>{item.body}</p>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.sectionAlt}>
          <div className="container">
            <div className={styles.sectionHeader}>
              <p className={styles.sectionEyebrow}>Host Guides</p>
              <h2>Choose the right workstation and runtime host deliberately.</h2>
              <p>
                The host guides explain what each platform can do in Port today,
                where the runtime truth lives, and when a workstation host is
                different from the execution host.
              </p>
            </div>
            <div className={styles.cardGrid}>
              {hostItems.map((item) => (
                <Link key={item.title} className={styles.card} to={item.href}>
                  <p className={styles.cardEyebrow}>{item.eyebrow}</p>
                  <h3 className={styles.cardTitle}>{item.title}</h3>
                  <p className={styles.cardBody}>{item.body}</p>
                </Link>
              ))}
            </div>
          </div>
        </section>

        <section className={styles.ctaBand}>
          <div className="container">
            <div className={styles.ctaCard}>
              <div>
                <p className={styles.sectionEyebrow}>Start Here</p>
                <h2>Read the narrative, verify the host, and prove the first Port lane.</h2>
              </div>
              <div className={clsx(styles.actions, styles.ctaActions)}>
                <Link className={styles.primaryAction} to="/docs/intro">
                  Open The Docs
                </Link>
                <Link className={styles.secondaryAction} to="/docs/start-here/install-port">
                  Install Port
                </Link>
              </div>
            </div>
          </div>
        </section>
      </main>
    </Layout>
  );
}
