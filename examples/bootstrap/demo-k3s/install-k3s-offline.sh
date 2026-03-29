#!/bin/sh
set -eu

role="${1:-server}"
if [ "$#" -gt 0 ]; then
  shift
fi

stage_root=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
binary="${stage_root}/k3s"
target_dir="${PORT_K3S_BIN_DIR:-${stage_root}/bin}"
kubeconfig_path="${PORT_K3S_KUBECONFIG_PATH:-etc/rancher/k3s/k3s.yaml}"

install -d "${target_dir}"
install -m 0755 "${binary}" "${target_dir}/k3s"
ln -sf "k3s" "${target_dir}/kubectl"
install -d "$(dirname "${kubeconfig_path}")"
cat >"${kubeconfig_path}" <<'EOF'
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: https://127.0.0.1:6443
    certificate-authority-data: ZGVtby1jYQ==
  name: demo
contexts:
- context:
    cluster: demo
    user: demo
  name: demo
current-context: demo
users:
- name: demo
  user:
    token: demo-token
EOF

printf 'offline-install-ok role=%s args=%s bin-dir=%s kubeconfig=%s\n' \
  "${role}" "$*" "${target_dir}" "${kubeconfig_path}"
