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
server_log_path="${PORT_K3S_LOG_PATH:-var/log/port-k3s.log}"
server_pid_path="${target_dir}/k3s-server.pid"
server_ready_path="${target_dir}/k3s-server.ready"
node_name="${PORT_K3S_NODE_NAME:-demo}"
node_ip="${PORT_K3S_NODE_IP:-10.66.0.1}"
node_ip_cidr="${PORT_K3S_NODE_IP_CIDR:-${node_ip}/24}"
cluster_iface="${PORT_K3S_CLUSTER_IFACE:-port0}"
wait_seconds="${PORT_K3S_WAIT_SECONDS:-90}"

make_absolute() {
  case "$1" in
    /*) printf '%s\n' "$1" ;;
    *) printf '/%s\n' "$1" ;;
  esac
}

target_dir="$(make_absolute "${target_dir}")"
kubeconfig_path="$(make_absolute "${kubeconfig_path}")"
server_log_path="$(make_absolute "${server_log_path}")"

install -d \
  "${target_dir}" \
  "$(dirname "${kubeconfig_path}")" \
  "$(dirname "${server_log_path}")" \
  /etc/rancher/k3s \
  /opt/cni/bin \
  /run \
  /sys/fs/cgroup \
  /var/lib/cni \
  /var/lib/kubelet \
  /var/lib/rancher/k3s/agent/images \
  /var/lib/rancher/k3s \
  /var/log
install -m 0755 "${binary}" "${target_dir}/k3s"
ln -sf "k3s" "${target_dir}/kubectl"

ensure_cgroup2() {
  if /bin/busybox grep -q ' /sys/fs/cgroup cgroup2 ' /proc/mounts 2>/dev/null; then
    return
  fi
  /bin/busybox mount -t cgroup2 cgroup2 /sys/fs/cgroup 2>/dev/null || true
}

ensure_loopback() {
  /bin/busybox ip link set lo up
  if ! /bin/busybox ip addr show dev lo 2>/dev/null | /bin/busybox grep -q 'inet 127.0.0.1/8'; then
    /bin/busybox ip addr add 127.0.0.1/8 dev lo
  fi
}

load_kube_modules() {
  for module in \
    bridge \
    br_netfilter \
    dummy \
    nf_conntrack \
    nf_conntrack_netlink \
    nf_nat \
    nf_tables \
    nft_chain_nat \
    nft_compat \
    nft_nat \
    overlay \
    ipt_REJECT \
    veth \
    x_tables \
    xt_MASQUERADE \
    xt_addrtype \
    xt_comment \
    xt_conntrack \
    xt_mark \
    xt_nat \
    xt_tcpudp
  do
    /usr/bin/modprobe "${module}" 2>/dev/null || true
  done
}

ensure_cluster_iface() {
  if ! /bin/busybox ip link show dev "${cluster_iface}" >/dev/null 2>&1; then
    /bin/busybox ip link add "${cluster_iface}" type dummy
  fi
  if ! /bin/busybox ip addr show dev "${cluster_iface}" 2>/dev/null | /bin/busybox grep -q "inet ${node_ip_cidr}"; then
    /bin/busybox ip addr add "${node_ip_cidr}" dev "${cluster_iface}" 2>/dev/null || true
  fi
  /bin/busybox ip link set "${cluster_iface}" up
}

ensure_hostname() {
  printf '%s\n' "${node_name}" >/proc/sys/kernel/hostname 2>/dev/null || true
}

ensure_hosts_file() {
  if [ -f /etc/hosts ] && /bin/busybox grep -q 'localhost' /etc/hosts 2>/dev/null; then
    return
  fi
  cat >/etc/hosts <<'EOF'
127.0.0.1 localhost
::1 localhost
EOF
}

configure_kube_sysctls() {
  printf '1\n' >/proc/sys/net/ipv4/ip_forward 2>/dev/null || true
  if [ -w /proc/sys/net/bridge/bridge-nf-call-iptables ]; then
    printf '1\n' >/proc/sys/net/bridge/bridge-nf-call-iptables 2>/dev/null || true
  fi
  if [ -w /proc/sys/net/bridge/bridge-nf-call-ip6tables ]; then
    printf '1\n' >/proc/sys/net/bridge/bridge-nf-call-ip6tables 2>/dev/null || true
  fi
}

kubectl_node_ready() {
  "${target_dir}/k3s" kubectl get nodes -o wide --request-timeout=5s >/dev/null 2>&1
}

server_state="started"
if kubectl_node_ready; then
  server_state="already-running"
else
  /bin/busybox rm -f "${server_pid_path}" "${server_ready_path}"
fi

case "${role}" in
  server)
    ensure_cgroup2
    ensure_loopback
    load_kube_modules
    ensure_cluster_iface
    ensure_hostname
    ensure_hosts_file
    configure_kube_sysctls
    if [ "${server_state}" = "started" ]; then
      setsid "${target_dir}/k3s" \
        server \
        --bind-address 0.0.0.0 \
        --https-listen-port 6443 \
        --advertise-address "${node_ip}" \
        --tls-san 127.0.0.1 \
        --tls-san "${node_ip}" \
        --write-kubeconfig "${kubeconfig_path}" \
        --write-kubeconfig-mode 0644 \
        --data-dir /var/lib/rancher/k3s \
        --node-name "${node_name}" \
        --node-ip "${node_ip}" \
        --node-external-ip "${node_ip}" \
        --flannel-iface "${cluster_iface}" \
        --flannel-backend host-gw \
        --disable servicelb \
        "$@" >"${server_log_path}" 2>&1 &
      server_pid=$!
      printf '%s\n' "${server_pid}" >"${server_pid_path}"
    fi
    ready=0
    attempts=0
    while [ "${attempts}" -lt "${wait_seconds}" ]; do
      if kubectl_node_ready; then
        /bin/busybox touch "${server_ready_path}"
        ready=1
        break
      fi
      sleep 1
      attempts=$((attempts + 1))
    done
    if [ "${ready}" -ne 1 ]; then
      cat "${server_log_path}" >&2 || true
      echo "timed out waiting for local K3s readiness after ${wait_seconds}s" >&2
      exit 1
    fi
    ;;
  *)
    echo "unsupported-role:${role}" >&2
    exit 1
    ;;
esac

printf 'k3s-server:%s pid-file=%s log=%s\n' \
  "${server_state}" "${server_pid_path}" "${server_log_path}"

printf 'offline-install-ok role=%s args=%s bin-dir=%s kubeconfig=%s\n' \
  "${role}" "$*" "${target_dir}" "${kubeconfig_path}"
