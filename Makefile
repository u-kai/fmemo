SHELL := /bin/bash

# Source cargo environment if available
ifneq (,$(wildcard ~/.cargo/env))
    CARGO_ENV := . ~/.cargo/env &&
else
    CARGO_ENV :=
endif

# Configurable vars
ROOT ?= .
PORT ?= 3030
BIN  ?= target/release/fmemo
FEATURES ?= embed_frontend
INSTALL_DIR ?= /usr/local/bin

.PHONY: all install-frontend build-frontend build package run serve start-bg stop verify verify-api verify-ui install uninstall clean

all: package

install-frontend:
	cd frontend && npm ci

build-frontend: install-frontend
	cd frontend && npm run build

# Build Rust binary with embedded frontend
package: build-frontend
	$(CARGO_ENV) cargo build --release --features $(FEATURES)

build:
	$(CARGO_ENV) cargo build --release --features $(FEATURES)

run: package
	$(BIN) -r $(ROOT) -p $(PORT)

serve: run

# Start server in background and wait until ready
start-bg: package
	@echo "Starting fmemo on port $(PORT) with root $(ROOT) ..."
	@$(BIN) -r $(ROOT) -p $(PORT) > server.log 2>&1 & echo $$! > server.pid
	@echo "PID: $$(cat server.pid)"
	@echo -n "Waiting for server to be ready"
	@for i in {1..60}; do \
		if curl -fsS http://localhost:$(PORT)/api/root >/dev/null 2>&1; then echo " - OK"; break; fi; \
		echo -n "."; sleep 0.5; \
	done

stop:
	@if [ -f server.pid ]; then \
		PID=$$(cat server.pid); \
		echo "Stopping fmemo (PID $$PID)"; \
		kill $$PID || true; \
		rm -f server.pid; \
	else \
		echo "No server.pid found"; \
	fi

verify-api:
	@echo "Verifying API /api/root ..."
	@curl -fsS http://localhost:$(PORT)/api/root | grep -q '"files"' && echo "API root OK" || (echo "API root FAILED" && exit 1)

verify-ui:
	@echo "Verifying UI / (index.html) ..."
	@curl -fsS http://localhost:$(PORT)/ | grep -q 'id=\"root\"' && echo "UI index OK" || (echo "UI index FAILED" && exit 1)

verify: start-bg
	@$(MAKE) verify-api PORT=$(PORT)
	@$(MAKE) verify-ui PORT=$(PORT)
	@$(MAKE) stop || true
	@echo "All checks passed."

install: package
	@echo "Installing fmemo to $(INSTALL_DIR)..."
	@install -m 755 $(BIN) $(INSTALL_DIR)/fmemo
	@echo "Installation complete. You can now run 'fmemo' from anywhere."

uninstall:
	@echo "Removing fmemo from $(INSTALL_DIR)..."
	@rm -f $(INSTALL_DIR)/fmemo
	@echo "Uninstall complete."

clean:
	$(CARGO_ENV) cargo clean
	rm -f server.pid server.log
	rm -rf frontend/node_modules frontend/dist

