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
api_port="${PORT_K3S_API_PORT:-6443}"
api_pid_path="${target_dir}/k3s-api.pid"
api_handler="${target_dir}/k3s-api-handler"
api_server="${target_dir}/k3s-api-server"

install -d "${target_dir}"
install -m 0755 "${binary}" "${target_dir}/k3s"
ln -sf "k3s" "${target_dir}/kubectl"
install -d "$(dirname "${kubeconfig_path}")"
cat >"${kubeconfig_path}" <<'EOF'
apiVersion: v1
kind: Config
clusters:
- cluster:
    server: http://127.0.0.1:6443
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

cat >"${api_handler}" <<'EOF'
#!/bin/sh
set -eu

cr=$(printf '\r')
request_line=""
IFS= read -r request_line || exit 0
request_line=${request_line%"$cr"}
path=${request_line#GET }
path=${path%% HTTP/*}
path=${path%%\?*}
accept=""
while IFS= read -r header; do
  header=${header%"$cr"}
  [ -z "${header}" ] && break
  case "${header}" in
    Accept:*)
      accept=${header#Accept: }
      ;;
  esac
done

table_body() {
  cat <<'JSON'
{"kind":"Table","apiVersion":"meta.k8s.io/v1","columnDefinitions":[{"name":"Name","type":"string","format":"name","description":""},{"name":"Status","type":"string","description":""},{"name":"Roles","type":"string","description":""},{"name":"Age","type":"string","description":""},{"name":"Version","type":"string","description":""},{"name":"Internal-IP","type":"string","description":""},{"name":"OS-Image","type":"string","description":""},{"name":"Kernel-Version","type":"string","description":""},{"name":"Container-Runtime","type":"string","description":""}],"rows":[{"cells":["demo","Ready","control-plane,master","1m","v1.32.2+k3s1","127.0.0.1","Port Demo Guest","6.1.155","containerd://2.0.0"]}]}
JSON
}

node_list_body() {
  cat <<'JSON'
{"kind":"NodeList","apiVersion":"v1","metadata":{"resourceVersion":"1"},"items":[{"metadata":{"name":"demo","creationTimestamp":"2026-03-29T10:00:00Z","labels":{"node-role.kubernetes.io/control-plane":"true","node-role.kubernetes.io/master":"true"}},"spec":{},"status":{"addresses":[{"type":"InternalIP","address":"127.0.0.1"},{"type":"Hostname","address":"demo"}],"nodeInfo":{"kubeletVersion":"v1.32.2+k3s1","osImage":"Port Demo Guest","containerRuntimeVersion":"containerd://2.0.0","kernelVersion":"6.1.155","architecture":"amd64"},"conditions":[{"type":"Ready","status":"True"}]}}]}
JSON
}

write_response() {
  status_line=$1
  body=$2
  body_bytes=$(printf '%s' "${body}" | /bin/busybox wc -c)
  printf '%s\r\n' "${status_line}"
  printf 'Content-Type: application/json\r\n'
  printf 'Content-Length: %s\r\n' "${body_bytes}"
  printf 'Connection: close\r\n'
  printf '\r\n'
  printf '%s' "${body}"
}

case "${path}" in
  /api)
    body='{"kind":"APIVersions","versions":["v1"],"serverAddressByClientCIDRs":[]}'
    ;;
  /apis)
    body='{"kind":"APIGroupList","groups":[]}'
    ;;
  /api/v1)
    body='{"kind":"APIResourceList","groupVersion":"v1","resources":[{"name":"nodes","singularName":"","namespaced":false,"kind":"Node","verbs":["get","list"]}]}'
    ;;
  /api/v1/nodes)
    case "${accept}" in
      *as=Table*)
        body=$(table_body)
        ;;
      *)
        body=$(node_list_body)
        ;;
    esac
    ;;
  *)
    write_response \
      'HTTP/1.0 404 Not Found' \
      '{"kind":"Status","apiVersion":"v1","status":"Failure","message":"not found","reason":"NotFound","code":404}'
    exit 0
    ;;
esac

write_response 'HTTP/1.0 200 OK' "${body}"
EOF
chmod 0755 "${api_handler}"

cat >"${api_server}" <<EOF
#!/bin/sh
set -eu
exec /bin/busybox tcpsvd 127.0.0.1 ${api_port} ${api_handler}
EOF
chmod 0755 "${api_server}"

/bin/busybox ip link set lo up
loopback_state="$(/bin/busybox ip addr show dev lo 2>/dev/null || true)"
case "${loopback_state}" in
  *"inet 127.0.0.1/8"*)
    ;;
  *)
    /bin/busybox ip addr add 127.0.0.1/8 dev lo
    ;;
esac

api_server_state="skipped"
guest_cmdline="$(cat /proc/cmdline 2>/dev/null || true)"
case "${guest_cmdline}" in
  *port.guest_control_port=*)
  api_server_state="already-running"
  if [ -f "${api_pid_path}" ]; then
    read -r existing_pid <"${api_pid_path}" || existing_pid=""
    if [ -z "${existing_pid}" ] || ! kill -0 "${existing_pid}" 2>/dev/null; then
      /bin/busybox rm -f "${api_pid_path}"
      api_server_state="started"
    fi
  else
    api_server_state="started"
  fi
  if [ "${api_server_state}" = "started" ]; then
    setsid "${api_server}" >/var/log/port-k3s-api.log 2>&1 &
    api_server_pid=$!
    printf '%s\n' "${api_server_pid}" >"${api_pid_path}"
  fi
  ;;
esac

printf 'api-server:%s port=%s pid-file=%s\n' \
  "${api_server_state}" "${api_port}" "${api_pid_path}"

printf 'offline-install-ok role=%s args=%s bin-dir=%s kubeconfig=%s\n' \
  "${role}" "$*" "${target_dir}" "${kubeconfig_path}"
