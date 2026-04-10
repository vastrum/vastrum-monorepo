#!/usr/bin/env bash
set -euo pipefail

# Vastrum network update script
# Stops all nodes, wipes DB, installs new binary, restarts.
# All nodes are stopped before any restart to prevent stale state leaking.
#
# Usage:
#   ./update-network.sh \
#     --ips debian@203.0.113.1,debian@203.0.113.2,root@203.0.113.3 \
#     --version v1.1.0

# --- Logging ---

log() { echo "[$(date +%H:%M:%S)] $*"; }
err() { echo "[$(date +%H:%M:%S)] ERROR: $*" >&2; }
warn() { echo "[$(date +%H:%M:%S)] WARN: $*" >&2; }

# --- Defaults ---

SSH_KEY=""
IPS=()
SSH_USERS=()
VERSION=""
DOMAIN=""
RELAY_IP=""
RELAY_USER=""

# --- Usage ---

usage() {
    cat <<'EOF'
Usage: update-network.sh [OPTIONS]

Stop all nodes, wipe state, install new binary, and restart.

Required:
  --ips USER@IP,...       Comma-separated USER@IP pairs (first = RPC node)
  --version VERSION       Release version to install (e.g. v1.1.0)

Optional:
  --relay-ip USER@IP      Git-relay server (stopped/wiped/restarted with validators)
  --domain DOMAIN         RPC domain for health check after restart
  --ssh-key PATH          SSH private key path
  -h, --help              Show this help

Example:
  ./update-network.sh \
    --ips debian@203.0.113.1,debian@203.0.113.2,admin@203.0.113.3 \
    --relay-ip debian@203.0.113.10 \
    --version v1.1.0 \
    --domain rpc.vastrum.org
EOF
    exit 0
}

# --- Argument Parsing ---

parse_args() {
    local raw_ips=()
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --ips)        IFS=',' read -ra raw_ips <<< "$2"; shift 2 ;;
            --relay-ip)   local relay_entry="$2"; RELAY_USER="${relay_entry%%@*}"; RELAY_IP="${relay_entry#*@}"; shift 2 ;;
            --version)    VERSION="$2"; shift 2 ;;
            --domain)     DOMAIN="$2"; shift 2 ;;
            --ssh-key)    SSH_KEY="$2"; shift 2 ;;
            -h|--help)    usage ;;
            *)            err "Unknown argument: $1"; exit 1 ;;
        esac
    done

    [[ ${#raw_ips[@]} -gt 0 ]] || { err "--ips is required"; exit 1; }

    for entry in "${raw_ips[@]}"; do
        if [[ "$entry" != *@* ]]; then
            err "Each --ips entry must be USER@IP, got: $entry"; exit 1
        fi
        SSH_USERS+=("${entry%%@*}")
        IPS+=("${entry#*@}")
    done

    [[ -n "$VERSION" ]] || { err "--version is required"; exit 1; }

    case "$VERSION" in
        v*) ;;
        *)  VERSION="v${VERSION}" ;;
    esac
}

# --- SSH Helper ---

remote_exec() {
    local user="$1" ip="$2"; shift 2
    ssh -o StrictHostKeyChecking=accept-new \
        -o ConnectTimeout=10 \
        -o ServerAliveInterval=30 \
        -o BatchMode=yes \
        ${SSH_KEY:+-i "$SSH_KEY"} \
        "${user}@${ip}" "$@"
}

# --- Phase 1: Stop all nodes ---

stop_all_nodes() {
    log "Phase 1: Stopping all nodes..."

    # Stop relay first to prevent stale state propagation
    if [[ -n "$RELAY_IP" ]]; then
        log "  Stopping git-relay on $RELAY_IP..."
        remote_exec "$RELAY_USER" "$RELAY_IP" "sudo systemctl stop vastrum-relay 2>/dev/null || true"
        log "  [$RELAY_IP] relay stopped"
    fi

    local pids=()
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        (
            remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "sudo systemctl stop vastrum-node"
            log "  [${IPS[$i]}] stopped"
        ) &
        pids+=($!)
    done

    local failed=false
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            failed=true
        fi
    done
    [[ "$failed" == false ]] || { err "Failed to stop some nodes"; exit 1; }

    # Verify all stopped — must confirm before proceeding
    log "  Verifying all nodes are stopped..."
    local all_stopped=true
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        local status
        status=$(remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "sudo systemctl is-active vastrum-node" 2>/dev/null || true)
        if [[ "$status" == "active" ]]; then
            err "  [${IPS[$i]}] still running!"
            all_stopped=false
        fi
    done
    [[ "$all_stopped" == true ]] || { err "Not all nodes stopped — aborting"; exit 1; }
    log "  All ${#IPS[@]} nodes confirmed stopped"
}

# --- Phase 2: Wipe DB and install new binary ---

update_all_nodes() {
    log "Phase 2: Wiping DB and installing $VERSION on all nodes..."

    local pids=()
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        update_single_node "$i" &
        pids+=($!)
    done

    local failed=false
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            failed=true
        fi
    done
    [[ "$failed" == false ]] || { err "Failed to update some nodes"; exit 1; }

    # Update relay if configured
    if [[ -n "$RELAY_IP" ]]; then
        log "  [$RELAY_IP] Wiping relay node DB..."
        remote_exec "$RELAY_USER" "$RELAY_IP" "sudo rm -rf /home/vastrum/.local/share/vastrum"

        log "  [$RELAY_IP] Installing vastrum-cli $VERSION..."
        remote_exec "$RELAY_USER" "$RELAY_IP" "VASTRUM_VERSION=$VERSION bash <(curl -fsSL https://raw.githubusercontent.com/vastrum/vastrum-monorepo/master/tooling/cli/install.sh)"

        remote_exec "$RELAY_USER" "$RELAY_IP" sudo bash <<'RELAY_COPY_EOF'
set -euo pipefail
CALLER_HOME=$(eval echo "~$SUDO_USER")
cp "$CALLER_HOME/.vastrum/bin/vastrum-cli" /home/vastrum/.vastrum/bin/vastrum-cli
chown vastrum:vastrum /home/vastrum/.vastrum/bin/vastrum-cli
RELAY_COPY_EOF
        log "  [$RELAY_IP] Relay updated"
    fi
}

update_single_node() {
    local idx="$1"
    local user="${SSH_USERS[$idx]}"
    local ip="${IPS[$idx]}"

    # Wipe DB (keep keystore.bin)
    log "  [$ip] Wiping DB..."
    remote_exec "$user" "$ip" sudo bash <<'WIPE_EOF'
set -euo pipefail
DATA_DIR="/home/vastrum/.local/share/vastrum"
find "$DATA_DIR" -maxdepth 1 -type f ! -name 'keystore.bin' -delete
rm -rf "$DATA_DIR/compiled_modules" "$DATA_DIR/logs"
WIPE_EOF

    # Install new binary
    log "  [$ip] Installing vastrum-cli $VERSION..."
    remote_exec "$user" "$ip" "VASTRUM_VERSION=$VERSION bash <(curl -fsSL https://raw.githubusercontent.com/vastrum/vastrum-monorepo/master/tooling/cli/install.sh)"

    # Copy binary to vastrum user
    remote_exec "$user" "$ip" sudo bash <<'COPY_EOF'
set -euo pipefail
CALLER_HOME=$(eval echo "~$SUDO_USER")
cp "$CALLER_HOME/.vastrum/bin/vastrum-cli" /home/vastrum/.vastrum/bin/vastrum-cli
chown vastrum:vastrum /home/vastrum/.vastrum/bin/vastrum-cli
COPY_EOF

    log "  [$ip] Updated"
}

# --- Phase 3: Start all nodes ---

start_all_nodes() {
    log "Phase 3: Starting all nodes..."

    local pids=()
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        (
            remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "sudo systemctl start vastrum-node"
            log "  [${IPS[$i]}] started"
        ) &
        pids+=($!)
    done

    local failed=false
    for pid in "${pids[@]}"; do
        if ! wait "$pid"; then
            failed=true
        fi
    done
    [[ "$failed" == false ]] || { err "Failed to start some nodes"; exit 1; }

    # Start relay after validators are up
    if [[ -n "$RELAY_IP" ]]; then
        log "  Starting git-relay on $RELAY_IP..."
        remote_exec "$RELAY_USER" "$RELAY_IP" "sudo systemctl start vastrum-relay"
        log "  [$RELAY_IP] relay started"
    fi
}

# --- Phase 4: Verify ---

verify() {
    log "Phase 4: Verifying deployment..."
    sleep 10

    local all_active=true
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        if remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "sudo systemctl is-active vastrum-node" &>/dev/null; then
            log "  [${IPS[$i]}] vastrum-node: active"
        else
            err "  [${IPS[$i]}] vastrum-node: NOT active"
            remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "sudo journalctl -u vastrum-node --no-pager -n 10 --output=cat" 2>/dev/null || true
            all_active=false
        fi
    done

    if [[ -n "$DOMAIN" ]]; then
        local height1 height2
        height1=$(curl -sf "https://${DOMAIN}/getlatestblockheight/" 2>/dev/null || echo "")
        if [[ -n "$height1" ]]; then
            sleep 5
            height2=$(curl -sf "https://${DOMAIN}/getlatestblockheight/" 2>/dev/null || echo "")
            if [[ -n "$height2" ]]; then
                log "  Block height: $height1 -> $height2"
            fi
        else
            warn "  Could not query block height (node may still be starting)"
        fi
    fi

    [[ "$all_active" == true ]] || { err "Some nodes are not active"; exit 1; }
}

# --- Main ---

main() {
    parse_args "$@"

    log "Updating ${#IPS[@]} nodes to $VERSION"
    log "  This will STOP all nodes and WIPE all state"
    log ""

    stop_all_nodes
    update_all_nodes
    start_all_nodes
    verify

    log ""
    log "Update complete! All ${#IPS[@]} nodes running $VERSION"
}

main "$@"
