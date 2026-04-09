#!/usr/bin/env bash
set -euo pipefail

# Vastrum network deployment script
# Deploys validator nodes to VPS machines via SSH
#
# Prerequisites:
#   1. Run genesis tool and commit genesis.json
#   2. Tag + push to trigger release workflow (or local build + gh release create)
#   3. Keystores available locally (genesis/validator-{i}/keystore.bin)
#   4. Domain DNS A record pointing to the RPC node IP

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Defaults
SSH_KEY=""
EMAIL=""
IPS=()
SSH_USERS=()
KEYSTORE_DIR=""
DOMAIN=""
SITE_DOMAIN=""
DNS_TOKEN=""
VERSION=""

# --- Logging ---

log() { echo "[$(date +%H:%M:%S)] $*"; }
err() { echo "[$(date +%H:%M:%S)] ERROR: $*" >&2; }
warn() { echo "[$(date +%H:%M:%S)] WARN: $*" >&2; }

# --- Usage ---

usage() {
    cat <<'EOF'
Usage: deploy-network.sh [OPTIONS]

Deploy Vastrum validator nodes to VPS machines.

Required:
  --ips USER@IP,...       Comma-separated USER@IP pairs (first = bootstrap+RPC)
  --keystore-dir DIR      Directory with validator-{i}/keystore.bin files
  --domain DOMAIN         Domain for HTTPS on the RPC node (e.g. rpc.vastrum.org)
  --version VERSION       Release version to install (e.g. v0.2.0)

Optional:
  --site-domain DOMAIN    Site-serving domain (e.g. vastrum.net). Enables *.DOMAIN
                          with wildcard cert via DNS-01/deSEC.
  --dns-token TOKEN       deSEC API token (required if --site-domain is set)
  --ssh-key PATH          SSH private key path
  --email EMAIL           Email for Caddy ACME (default: admin@DOMAIN)
  -h, --help              Show this help

Example:
  ./deploy-network.sh \
    --ips admin@203.0.113.1,debian@203.0.113.2,root@203.0.113.3 \
    --keystore-dir ./genesis \
    --domain rpc.vastrum.org \
    --site-domain vastrum.net \
    --dns-token deSEC-api-token-here \
    --version v0.2.0
EOF
    exit 0
}

# --- Argument Parsing ---

parse_args() {
    local raw_ips=()
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --ips)        IFS=',' read -ra raw_ips <<< "$2"; shift 2 ;;
            --keystore-dir) KEYSTORE_DIR="$2"; shift 2 ;;
            --domain)     DOMAIN="$2"; shift 2 ;;
            --site-domain) SITE_DOMAIN="$2"; shift 2 ;;
            --dns-token)  DNS_TOKEN="$2"; shift 2 ;;
            --version)    VERSION="$2"; shift 2 ;;
            --ssh-key)    SSH_KEY="$2"; shift 2 ;;
            --email)      EMAIL="$2"; shift 2 ;;
            -h|--help)    usage ;;
            *)            err "Unknown argument: $1"; exit 1 ;;
        esac
    done

    [[ ${#raw_ips[@]} -gt 0 ]]   || { err "--ips is required"; exit 1; }

    # Parse user@ip entries
    for entry in "${raw_ips[@]}"; do
        if [[ "$entry" != *@* ]]; then
            err "Each --ips entry must be USER@IP, got: $entry"; exit 1
        fi
        SSH_USERS+=("${entry%%@*}")
        IPS+=("${entry#*@}")
    done
    [[ -n "$KEYSTORE_DIR" ]] || { err "--keystore-dir is required"; exit 1; }
    [[ -n "$DOMAIN" ]]       || { err "--domain is required"; exit 1; }
    [[ -n "$VERSION" ]]      || { err "--version is required"; exit 1; }

    # Ensure version has v prefix
    case "$VERSION" in
        v*) ;;
        *)  VERSION="v${VERSION}" ;;
    esac

    # --dns-token required if --site-domain is set
    if [[ -n "$SITE_DOMAIN" ]] && [[ -z "$DNS_TOKEN" ]]; then
        err "--dns-token is required when --site-domain is set"; exit 1
    fi

    [[ -n "$EMAIL" ]] || EMAIL="admin@${DOMAIN}"
}

# --- SSH Helpers ---

remote_exec() {
    local user="$1" ip="$2"; shift 2
    ssh -o StrictHostKeyChecking=accept-new \
        -o ConnectTimeout=10 \
        -o ServerAliveInterval=30 \
        -o BatchMode=yes \
        ${SSH_KEY:+-i "$SSH_KEY"} \
        "${user}@${ip}" "$@"
}

remote_copy() {
    local user="$1" src="$2" ip="$3" dst="$4"
    scp -o StrictHostKeyChecking=accept-new \
        -o ConnectTimeout=10 \
        -o BatchMode=yes \
        ${SSH_KEY:+-i "$SSH_KEY"} \
        "$src" "${user}@${ip}:${dst}"
}

# --- Phase 0: Validation ---

validate() {
    log "Phase 0: Validating configuration..."

    # Check keystore files exist
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        local ks="$KEYSTORE_DIR/validator-${i}/keystore.bin"
        [[ -f "$ks" ]] || { err "Keystore not found: $ks"; exit 1; }
    done
    log "  Keystores: OK (${#IPS[@]} found)"

    # Clear stale host keys and pre-populate fresh ones (avoids race in parallel Phase 2)
    for ip in "${IPS[@]}"; do
        ssh-keygen -f "$HOME/.ssh/known_hosts" -R "$ip" 2>/dev/null || true
    done
    for ip in "${IPS[@]}"; do
        ssh-keyscan -H "$ip" >> "$HOME/.ssh/known_hosts" 2>/dev/null || true
    done

    # Check SSH connectivity
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        if ! remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "exit" 2>/dev/null; then
            err "Cannot SSH to ${SSH_USERS[$i]}@${IPS[$i]}"
            exit 1
        fi
    done
    log "  SSH connectivity: OK"

    # DNS check (warning only)
    if command -v dig &>/dev/null; then
        local resolved
        resolved=$(dig +short "$DOMAIN" | head -1)
        if [[ "$resolved" != "${IPS[0]}" ]]; then
            warn "$DOMAIN resolves to '$resolved', expected '${IPS[0]}'"
            warn "  Caddy may fail to obtain certificates if DNS is not yet propagated"
        else
            log "  DNS: OK ($DOMAIN -> ${IPS[0]})"
        fi
    fi

    # Site domain DNS check
    if [[ -n "$SITE_DOMAIN" ]] && command -v dig &>/dev/null; then
        local site_resolved
        site_resolved=$(dig +short "$SITE_DOMAIN" | head -1)
        if [[ "$site_resolved" != "${IPS[0]}" ]]; then
            warn "$SITE_DOMAIN resolves to '$site_resolved', expected '${IPS[0]}'"
        else
            log "  DNS: OK ($SITE_DOMAIN -> ${IPS[0]})"
        fi
    fi

    # Bootstrap domain DNS check
    if command -v dig &>/dev/null; then
        local genesis_file="$SCRIPT_DIR/../shared-types/genesis.json"
        if [[ -f "$genesis_file" ]]; then
            local bootstrap_host
            bootstrap_host=$(grep -o '"host": *"[^"]*"' "$genesis_file" | head -1 | sed 's/.*"host": *"//;s/"//')
            if [[ -n "$bootstrap_host" ]]; then
                local bootstrap_resolved
                bootstrap_resolved=$(dig +short "$bootstrap_host" | head -1)
                if [[ "$bootstrap_resolved" != "${IPS[0]}" ]]; then
                    warn "$bootstrap_host resolves to '$bootstrap_resolved', expected '${IPS[0]}'"
                    warn "  Nodes will panic at startup if bootstrap domain is unreachable"
                else
                    log "  DNS: OK ($bootstrap_host -> ${IPS[0]})"
                fi
            fi
        fi
    fi

    log "  Config: ${#IPS[@]} validators, RPC=${IPS[0]}, domain=$DOMAIN${SITE_DOMAIN:+, sites=*.$SITE_DOMAIN}, version=$VERSION"
}

# --- Shared Deployment Helpers ---

install_binary() {
    local user="$1" ip="$2"
    log "  [$ip] Installing vastrum-cli $VERSION..."
    remote_exec "$user" "$ip" "VASTRUM_VERSION=$VERSION bash <(curl -fsSL https://raw.githubusercontent.com/vastrum/vastrum-monorepo/master/tooling/cli/install.sh)"
}

setup_user_and_files() {
    local user="$1" ip="$2" idx="$3"
    log "  [$ip] Setting up vastrum user and files..."

    # Upload keystore
    remote_copy "$user" "$KEYSTORE_DIR/validator-${idx}/keystore.bin" "$ip" "/tmp/vastrum-keystore.bin"

    # Create user and directory structure, move files
    remote_exec "$user" "$ip" sudo bash <<'SETUP_EOF'
set -euo pipefail
CALLER_HOME=$(eval echo "~$SUDO_USER")
id -u vastrum &>/dev/null || useradd --system --home-dir /home/vastrum --create-home --shell /usr/sbin/nologin vastrum
mkdir -p /home/vastrum/.vastrum/bin /home/vastrum/.local/share/vastrum
cp "$CALLER_HOME/.vastrum/bin/vastrum-cli" /home/vastrum/.vastrum/bin/vastrum-cli
mv /tmp/vastrum-keystore.bin /home/vastrum/.local/share/vastrum/keystore.bin
chown -R vastrum:vastrum /home/vastrum
chmod 600 /home/vastrum/.local/share/vastrum/keystore.bin
SETUP_EOF
}

install_systemd_service() {
    local user="$1" ip="$2" rpc_flag="$3"
    log "  [$ip] Installing systemd service..."

    local exec_start="/home/vastrum/.vastrum/bin/vastrum-cli start-node"
    [[ -n "$rpc_flag" ]] && exec_start="$exec_start --rpc"

    remote_exec "$user" "$ip" sudo bash <<SERVICE_EOF
set -euo pipefail

cat > /etc/systemd/system/vastrum-node.service <<EOF
[Unit]
Description=Vastrum Validator Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=vastrum
Group=vastrum
ExecStart=$exec_start
Restart=always
RestartSec=5
LimitNOFILE=65536
ProtectSystem=full
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable vastrum-node
SERVICE_EOF
}

start_node() {
    local user="$1" ip="$2"
    log "  [$ip] Starting vastrum-node..."
    remote_exec "$user" "$ip" "sudo systemctl restart vastrum-node"
}

harden_server() {
    local user="$1" ip="$2"
    log "  [$ip] Hardening server..."
    remote_exec "$user" "$ip" sudo bash <<'HARDEN_EOF'
set -euo pipefail

# Disable SSH password authentication (key-only)
sed -i 's/^#*PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/^#*KbdInteractiveAuthentication.*/KbdInteractiveAuthentication no/' /etc/ssh/sshd_config
systemctl restart sshd

# Enable automatic security updates
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq unattended-upgrades >/dev/null
cat > /etc/apt/apt.conf.d/20auto-upgrades <<EOF
APT::Periodic::Update-Package-Lists "1";
APT::Periodic::Unattended-Upgrade "1";
EOF
HARDEN_EOF
}

# --- Phase 1: Deploy Bootstrap+RPC Node ---

deploy_rpc_node() {
    local user="${SSH_USERS[0]}" ip="${IPS[0]}"
    log "Phase 1: Deploying bootstrap+RPC node ($ip)..."

    harden_server "$user" "$ip"
    install_binary "$user" "$ip"
    setup_user_and_files "$user" "$ip" 0
    install_systemd_service "$user" "$ip" "--rpc"

    # Firewall
    log "  [$ip] Configuring firewall..."
    remote_exec "$user" "$ip" sudo bash <<'FW_EOF'
set -euo pipefail
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq ufw >/dev/null
ufw allow 22/tcp comment "SSH" >/dev/null
ufw allow 15555/tcp comment "Vastrum P2P" >/dev/null
ufw allow 15556/tcp comment "Vastrum RPC" >/dev/null
ufw allow 3478/udp comment "Vastrum WebRTC" >/dev/null
ufw allow 80/tcp comment "HTTP" >/dev/null
ufw allow 443/tcp comment "HTTPS" >/dev/null
ufw allow 443/udp comment "HTTPS QUIC/HTTP3" >/dev/null
ufw --force enable >/dev/null
FW_EOF

    start_node "$user" "$ip"
    setup_caddy "$user" "$ip"
}

setup_caddy() {
    local user="$1" ip="$2"
    log "  [$ip] Installing Caddy with deSEC DNS plugin..."

    remote_exec "$user" "$ip" sudo bash <<'CADDY_INSTALL_EOF'
set -euo pipefail

# Install Go
DEBIAN_FRONTEND=noninteractive apt-get update -qq
apt-get install -y -qq curl golang >/dev/null

# Install xcaddy via go install and build Caddy with deSEC DNS plugin
GOBIN=/usr/local/bin go install github.com/caddyserver/xcaddy/cmd/xcaddy@latest
/usr/local/bin/xcaddy build --with github.com/caddy-dns/desec --output /usr/bin/caddy

# Create caddy user and directories
id -u caddy &>/dev/null || useradd --system --home-dir /var/lib/caddy --create-home --shell /usr/sbin/nologin caddy
mkdir -p /etc/caddy /var/lib/caddy /var/log/caddy
chown caddy:caddy /var/lib/caddy /var/log/caddy
CADDY_INSTALL_EOF

    # Write Caddyfile
    log "  [$ip] Writing Caddyfile..."
    local caddyfile
    if [[ -n "$SITE_DOMAIN" ]]; then
        caddyfile="$(cat <<CADDYEOF
{
    email $EMAIL
}

$DOMAIN {
    reverse_proxy localhost:15556
}

*.$SITE_DOMAIN, $SITE_DOMAIN {
    tls {
        issuer acme {
            email $EMAIL
            dns desec {
                token {\$DESEC_TOKEN}
            }
            propagation_delay 300s
            propagation_timeout 600s
        }
    }
    reverse_proxy localhost:15556
}
CADDYEOF
)"
    else
        caddyfile="$(cat <<CADDYEOF
{
    email $EMAIL
}

$DOMAIN {
    reverse_proxy localhost:15556
}
CADDYEOF
)"
    fi

    printf '%s\n' "$caddyfile" | remote_exec "$user" "$ip" "sudo tee /etc/caddy/Caddyfile >/dev/null"

    # Write deSEC token to env file (piped via stdin to avoid ps exposure)
    if [[ -n "$DNS_TOKEN" ]]; then
        log "  [$ip] Configuring deSEC token..."
        printf 'DESEC_TOKEN=%s\n' "$DNS_TOKEN" | remote_exec "$user" "$ip" "sudo tee /etc/caddy/env >/dev/null && sudo chmod 600 /etc/caddy/env && sudo chown caddy:caddy /etc/caddy/env"
    fi

    # Install systemd service
    log "  [$ip] Installing Caddy systemd service..."
    remote_exec "$user" "$ip" sudo bash <<'CADDY_SERVICE_EOF'
set -euo pipefail

cat > /etc/systemd/system/caddy.service <<EOF
[Unit]
Description=Caddy web server
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
User=caddy
Group=caddy
ExecStart=/usr/bin/caddy run --config /etc/caddy/Caddyfile
ExecReload=/usr/bin/caddy reload --config /etc/caddy/Caddyfile --force
TimeoutStopSec=5s
LimitNOFILE=1048576
PrivateTmp=true
ProtectSystem=full
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE
EnvironmentFile=-/etc/caddy/env
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now caddy
CADDY_SERVICE_EOF

    # Wait for wildcard cert (don't reload Caddy — reloads cancel in-flight DNS-01 challenges)
    if [[ -n "$SITE_DOMAIN" ]]; then
        log "  [$ip] Waiting for wildcard cert (*.${SITE_DOMAIN})..."
        local max_attempts=10
        local attempt=0
        while (( attempt < max_attempts )); do
            sleep 60
            if curl -sf --max-time 10 "https://${SITE_DOMAIN}" &>/dev/null; then
                log "  [$ip] Wildcard cert: OK"
                break
            fi
            attempt=$((attempt + 1))
            log "  [$ip] Wildcard cert not ready (attempt ${attempt}/${max_attempts}), waiting..."
        done
        if (( attempt >= max_attempts )); then
            warn "  [$ip] Wildcard cert not obtained after ${max_attempts} attempts — check Caddy logs"
        fi
    fi

    log "  [$ip] Caddy configured with HTTPS"
}

# --- Phase 2: Deploy Validator Nodes ---

deploy_validators() {
    local num_validators=${#IPS[@]}

    if [[ $num_validators -le 1 ]]; then
        log "Phase 2: No additional validators to deploy"
        return
    fi

    log "Phase 2: Deploying $((num_validators - 1)) validator node(s) in parallel..."

    local pids=()
    local failed_nodes=()

    for i in $(seq 1 $((num_validators - 1))); do
        deploy_single_validator "$i" "${IPS[$i]}" &
        pids+=($!)
    done

    for j in "${!pids[@]}"; do
        if ! wait "${pids[$j]}"; then
            failed_nodes+=("${IPS[$((j + 1))]}")
        fi
    done

    if [[ ${#failed_nodes[@]} -gt 0 ]]; then
        err "Failed to deploy validators: ${failed_nodes[*]}"
        exit 1
    fi
}

deploy_single_validator() {
    local idx="$1" ip="$2" user="${SSH_USERS[$1]}"
    log "  [$ip] Deploying validator-$idx..."

    harden_server "$user" "$ip"
    install_binary "$user" "$ip"
    setup_user_and_files "$user" "$ip" "$idx"
    install_systemd_service "$user" "$ip" ""

    # Firewall: P2P only
    remote_exec "$user" "$ip" sudo bash <<'FW_EOF'
set -euo pipefail
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq ufw >/dev/null
ufw allow 22/tcp comment "SSH" >/dev/null
ufw allow 15555/tcp comment "Vastrum P2P" >/dev/null
ufw --force enable >/dev/null
FW_EOF

    start_node "$user" "$ip"
    log "  [$ip] Validator-$idx deployed"
}

# --- Phase 3: Verification ---

verify_network() {
    log "Phase 3: Verifying deployment..."

    sleep 10

    # Check systemd status on all nodes
    local all_active=true
    for i in $(seq 0 $((${#IPS[@]} - 1))); do
        if remote_exec "${SSH_USERS[$i]}" "${IPS[$i]}" "sudo systemctl is-active vastrum-node" &>/dev/null; then
            log "  [${IPS[$i]}] vastrum-node: active"
        else
            err "  [${IPS[$i]}] vastrum-node: NOT active"
            all_active=false
        fi
    done

    # Check HTTPS health endpoint
    if curl -sf "https://${DOMAIN}/health" &>/dev/null; then
        log "  HTTPS health check: OK"
    else
        warn "  HTTPS health check: failed (node may still be starting)"
    fi

    # Check site domain HTTPS
    if [[ -n "$SITE_DOMAIN" ]]; then
        if curl -sf --max-time 10 "https://${SITE_DOMAIN}" &>/dev/null; then
            log "  HTTPS site domain check: OK (${SITE_DOMAIN})"
        else
            warn "  HTTPS site domain check: FAILED (${SITE_DOMAIN})"
        fi
    fi

    # Check block height advancing
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

    [[ "$all_active" == true ]] || { err "Some nodes are not active"; exit 1; }
}

# --- Main ---

main() {
    parse_args "$@"
    validate
    deploy_rpc_node
    deploy_validators
    verify_network

    log ""
    log "Deployment complete!"
    log "  RPC endpoint:  https://${DOMAIN}"
    [[ -n "$SITE_DOMAIN" ]] && log "  Sites:         https://*.$SITE_DOMAIN"
    log "  Validators:    ${#IPS[@]}"
    log "  Bootstrap:     ${IPS[0]}"
}

main "$@"
