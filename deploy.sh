#!/bin/bash
#
# NetSentinel - Deployment Script for Embedded Debian
# Usage: ./deploy.sh [install|start|stop|restart|status|logs|update|uninstall]
#

set -e

# Configuration
INSTALL_DIR="/opt/netsentinel"
COMPOSE_FILE="docker-compose.yml"
ENV_FILE=".env"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (sudo)"
        exit 1
    fi
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."

    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
        echo "  curl -fsSL https://get.docker.com | sh"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed."
        exit 1
    fi

    log_success "All dependencies found"
}

# Get docker compose command
get_compose_cmd() {
    if docker compose version &> /dev/null 2>&1; then
        echo "docker compose"
    else
        echo "docker-compose"
    fi
}

# Install NetSentinel
install() {
    check_root
    check_dependencies

    log_info "Installing NetSentinel to ${INSTALL_DIR}..."

    # Create installation directory
    mkdir -p ${INSTALL_DIR}

    # Copy files
    cp -r . ${INSTALL_DIR}/
    cd ${INSTALL_DIR}

    # Create .env file if not exists
    if [[ ! -f ${ENV_FILE} ]]; then
        log_info "Creating environment file..."
        cp .env.example ${ENV_FILE}

        # Generate secure secrets
        SECRET_KEY=$(openssl rand -hex 32)
        DB_PASSWORD=$(openssl rand -hex 16)

        sed -i "s/SECRET_KEY=.*/SECRET_KEY=${SECRET_KEY}/" ${ENV_FILE}
        sed -i "s/DB_PASSWORD=.*/DB_PASSWORD=${DB_PASSWORD}/" ${ENV_FILE}

        log_success "Generated secure passwords"
    fi

    # Create log directory
    mkdir -p /var/log/netsentinel

    # Build images
    log_info "Building Docker images (this may take a while)..."
    $(get_compose_cmd) build

    # Create systemd service
    create_systemd_service

    log_success "NetSentinel installed successfully!"
    echo ""
    echo "Next steps:"
    echo "  1. Edit ${INSTALL_DIR}/.env to customize settings"
    echo "  2. Edit ${INSTALL_DIR}/config/capture.docker.toml to set network interfaces"
    echo "  3. Start with: systemctl start netsentinel"
    echo "  4. Access web interface at: http://<your-ip>:8080"
}

# Create systemd service
create_systemd_service() {
    log_info "Creating systemd service..."

    cat > /etc/systemd/system/netsentinel.service << EOF
[Unit]
Description=NetSentinel Network Monitoring
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=${INSTALL_DIR}
ExecStart=$(which docker) compose up -d
ExecStop=$(which docker) compose down
ExecReload=$(which docker) compose restart

[Install]
WantedBy=multi-user.target
EOF

    systemctl daemon-reload
    systemctl enable netsentinel

    log_success "Systemd service created and enabled"
}

# Start services
start() {
    cd ${INSTALL_DIR}
    log_info "Starting NetSentinel..."
    $(get_compose_cmd) up -d
    log_success "NetSentinel started"
    echo ""
    echo "Web interface: http://localhost:$(grep WEB_PORT .env 2>/dev/null | cut -d= -f2 || echo 8080)"
}

# Stop services
stop() {
    cd ${INSTALL_DIR}
    log_info "Stopping NetSentinel..."
    $(get_compose_cmd) down
    log_success "NetSentinel stopped"
}

# Restart services
restart() {
    cd ${INSTALL_DIR}
    log_info "Restarting NetSentinel..."
    $(get_compose_cmd) restart
    log_success "NetSentinel restarted"
}

# Show status
status() {
    cd ${INSTALL_DIR}
    echo ""
    echo "=== NetSentinel Status ==="
    echo ""
    $(get_compose_cmd) ps
    echo ""
    echo "=== Resource Usage ==="
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" \
        netsentinel-capture netsentinel-aggregator netsentinel-api netsentinel-web \
        netsentinel-db netsentinel-redis 2>/dev/null || true
}

# Show logs
logs() {
    cd ${INSTALL_DIR}
    SERVICE=${1:-}
    if [[ -n "$SERVICE" ]]; then
        $(get_compose_cmd) logs -f $SERVICE
    else
        $(get_compose_cmd) logs -f
    fi
}

# Update NetSentinel
update() {
    check_root
    cd ${INSTALL_DIR}

    log_info "Updating NetSentinel..."

    # Pull latest changes if git repo
    if [[ -d .git ]]; then
        git pull origin main
    fi

    # Rebuild images
    log_info "Rebuilding Docker images..."
    $(get_compose_cmd) build --no-cache

    # Restart services
    log_info "Restarting services..."
    $(get_compose_cmd) up -d

    log_success "NetSentinel updated successfully!"
}

# Uninstall NetSentinel
uninstall() {
    check_root

    log_warn "This will remove NetSentinel and all data!"
    read -p "Are you sure? (yes/no): " confirm

    if [[ "$confirm" != "yes" ]]; then
        log_info "Uninstall cancelled"
        exit 0
    fi

    cd ${INSTALL_DIR} 2>/dev/null || true

    # Stop and remove containers
    log_info "Stopping containers..."
    $(get_compose_cmd) down -v 2>/dev/null || true

    # Remove systemd service
    log_info "Removing systemd service..."
    systemctl stop netsentinel 2>/dev/null || true
    systemctl disable netsentinel 2>/dev/null || true
    rm -f /etc/systemd/system/netsentinel.service
    systemctl daemon-reload

    # Remove installation directory
    log_info "Removing installation directory..."
    rm -rf ${INSTALL_DIR}

    # Remove logs
    rm -rf /var/log/netsentinel

    log_success "NetSentinel uninstalled"
}

# Show help
show_help() {
    echo "NetSentinel Deployment Script"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  install     Install NetSentinel"
    echo "  start       Start all services"
    echo "  stop        Stop all services"
    echo "  restart     Restart all services"
    echo "  status      Show service status"
    echo "  logs [svc]  Show logs (optionally for specific service)"
    echo "  update      Update and rebuild"
    echo "  uninstall   Remove NetSentinel"
    echo ""
    echo "Services: capture, aggregator, api, web, postgres, redis"
}

# Main
case "${1:-}" in
    install)
        install
        ;;
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        restart
        ;;
    status)
        status
        ;;
    logs)
        logs $2
        ;;
    update)
        update
        ;;
    uninstall)
        uninstall
        ;;
    *)
        show_help
        ;;
esac
