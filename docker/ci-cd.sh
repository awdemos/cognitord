#!/bin/bash
set -e

# CI/CD Pipeline Script for DSRs Daemon Docker
# This script is designed for automated CI/CD pipelines

echo "🚀 DSRs Daemon Docker CI/CD Pipeline"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="${IMAGE_NAME:-dsrs-daemon}"
IMAGE_TAG="${IMAGE_TAG:-$(git rev-parse --short HEAD)}"
REGISTRY="${REGISTRY:-}"
PLATFORMS="${PLATFORMS:-linux/amd64,linux/arm64}"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

build_and_test() {
    log_info "Building and testing Docker image..."
    
    # Run the comprehensive test suite
    if ./docker/test-docker.sh; then
        log_success "All tests passed"
    else
        log_error "Tests failed"
        exit 1
    fi
}

build_multi_platform() {
    log_info "Building multi-platform images for: $PLATFORMS"
    
    # Enable Docker BuildKit
    export DOCKER_BUILDKIT=1
    
    # Build multi-platform image
    if docker buildx build \
        --platform "$PLATFORMS" \
        --tag "${REGISTRY}${IMAGE_NAME}:${IMAGE_TAG}" \
        --tag "${REGISTRY}${IMAGE_NAME}:latest" \
        --push \
        .; then
        log_success "Multi-platform build successful"
    else
        log_error "Multi-platform build failed"
        exit 1
    fi
}

build_single_platform() {
    log_info "Building single-platform image..."
    
    # Build single platform image
    if docker build \
        --tag "${REGISTRY}${IMAGE_NAME}:${IMAGE_TAG}" \
        --tag "${REGISTRY}${IMAGE_NAME}:latest" \
        .; then
        log_success "Single-platform build successful"
    else
        log_error "Single-platform build failed"
        exit 1
    fi
}

push_image() {
    if [ -n "$REGISTRY" ]; then
        log_info "Pushing image to registry: $REGISTRY"
        
        if docker push "${REGISTRY}${IMAGE_NAME}:${IMAGE_TAG}"; then
            log_success "Image pushed successfully"
        else
            log_error "Failed to push image"
            exit 1
        fi
        
        if docker push "${REGISTRY}${IMAGE_NAME}:latest"; then
            log_success "Latest tag pushed successfully"
        else
            log_error "Failed to push latest tag"
            exit 1
        fi
    else
        log_warning "No registry specified, skipping push"
    fi
}

scan_vulnerabilities() {
    log_info "Scanning for vulnerabilities..."
    
    # Install trivy if not present
    if ! command -v trivy &> /dev/null; then
        log_info "Installing Trivy..."
        curl -sfL https://github.com/aquasecurity/trivy/releases/download/v0.42.1/trivy_0.42.1_Linux-64bit.tar.gz | tar -xzv
        sudo mv trivy /usr/local/bin/
    fi
    
    # Scan image
    if trivy image --exit-code 1 --severity CRITICAL,HIGH "${REGISTRY}${IMAGE_NAME}:${IMAGE_TAG}"; then
        log_success "No critical or high vulnerabilities found"
    else
        log_error "Critical or high vulnerabilities found"
        exit 1
    fi
}

generate_sbom() {
    log_info "Generating SBOM..."
    
    # Install syft if not present
    if ! command -v syft &> /dev/null; then
        log_info "Installing Syft..."
        curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin
    fi
    
    # Generate SBOM
    if syft "${REGISTRY}${IMAGE_NAME}:${IMAGE_TAG}" --output cyclonedx-json > sbom.json; then
        log_success "SBOM generated successfully"
    else
        log_error "Failed to generate SBOM"
        exit 1
    fi
}

sign_image() {
    if [ -n "$COSIGN_KEY" ]; then
        log_info "Signing image with Cosign..."
        
        # Install cosign if not present
        if ! command -v cosign &> /dev/null; then
            log_info "Installing Cosign..."
            curl -sSfL https://github.com/sigstore/cosign/releases/download/v2.2.0/cosign-linux-amd64 > cosign
            chmod +x cosign
            sudo mv cosign /usr/local/bin/
        fi
        
        # Sign image
        if cosign sign --key "$COSIGN_KEY" "${REGISTRY}${IMAGE_NAME}:${IMAGE_TAG}"; then
            log_success "Image signed successfully"
        else
            log_error "Failed to sign image"
            exit 1
        fi
    else
        log_warning "No cosign key provided, skipping image signing"
    fi
}

cleanup() {
    log_info "Cleaning up..."
    docker system prune -f > /dev/null 2>&1 || true
}

# Main execution
main() {
    log_info "Starting CI/CD pipeline for $IMAGE_NAME:$IMAGE_TAG"
    
    # Parse command line arguments
    case "${1:-build}" in
        build)
            build_and_test
            build_single_platform
            push_image
            ;;
        build-multi)
            build_and_test
            build_multi_platform
            push_image
            ;;
        scan)
            scan_vulnerabilities
            ;;
        sbom)
            generate_sbom
            ;;
        sign)
            sign_image
            ;;
        full)
            build_and_test
            build_multi_platform
            push_image
            scan_vulnerabilities
            generate_sbom
            sign_image
            ;;
        *)
            echo "Usage: $0 {build|build-multi|scan|sbom|sign|full}"
            echo "  build: Build and test single platform (default)"
            echo "  build-multi: Build and test multi-platform"
            echo "  scan: Scan for vulnerabilities"
            echo "  sbom: Generate SBOM"
            echo "  sign: Sign image with cosign"
            echo "  full: Run full pipeline"
            exit 1
            ;;
    esac
    
    log_success "CI/CD pipeline completed successfully!"
}

# Set trap for cleanup
trap cleanup EXIT

# Run main function
main "$@"