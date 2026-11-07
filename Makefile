RELAYLOG ?= relaylog
CODEX ?= codex
HOOK_SCOPE ?= user
CAPTURE ?= metadata
SESSION_DIR ?=
OUT ?= session.jsonl
AUTHORIZE_SHARING ?= no

.PHONY: hooks-install hooks-uninstall record status attach export investigate

hooks-install:
	"$(RELAYLOG)" agent hooks install --provider codex --scope "$(HOOK_SCOPE)"

hooks-uninstall:
	"$(RELAYLOG)" agent hooks uninstall --provider codex --scope "$(HOOK_SCOPE)"

# Starts Relaylog's interactive shell, then prints the new session directory.
record:
	@set -eu; \
	spool_root="$${XDG_STATE_HOME:-$$HOME/.local/state}/relaylog"; \
	scratch=$$(mktemp -d); \
	trap 'rm -rf "$$scratch"' EXIT HUP INT TERM; \
	find "$$spool_root" -mindepth 1 -maxdepth 1 -type d -name 'session-*' -print 2>/dev/null | LC_ALL=C sort > "$$scratch/before"; \
	status=0; "$(RELAYLOG)" start --agent-capture "$(CAPTURE)" || status=$$?; \
	find "$$spool_root" -mindepth 1 -maxdepth 1 -type d -name 'session-*' -print 2>/dev/null | LC_ALL=C sort > "$$scratch/after"; \
	comm -13 "$$scratch/before" "$$scratch/after" > "$$scratch/new"; \
	if [ -s "$$scratch/new" ]; then \
		while IFS= read -r session_dir; do printf 'Relaylog session directory: %s\n' "$$session_dir"; done < "$$scratch/new"; \
	else \
		printf 'No new session directory detected; use the path printed by Relaylog at startup.\n' >&2; \
	fi; \
	exit "$$status"

# Run this target from inside the Relaylog-managed shell.
status:
	"$(RELAYLOG)" agent status --provider codex

attach:
	@test -n "$(SESSION_DIR)" || { echo "Set SESSION_DIR=/path/to/active-session" >&2; exit 2; }
	"$(RELAYLOG)" agent attach --session-directory "$(SESSION_DIR)" -- "$(CODEX)"

export:
	@test -n "$(SESSION_DIR)" || { echo "Set SESSION_DIR=/path/to/session" >&2; exit 2; }
	"$(RELAYLOG)" export "$(SESSION_DIR)" > "$(OUT)"
	@echo "Exported JSONL to $(OUT)"

investigate:
	@test -n "$(SESSION_DIR)" || { echo "Set SESSION_DIR=/path/to/session" >&2; exit 2; }
	@test "$(AUTHORIZE_SHARING)" = yes || { echo "Set AUTHORIZE_SHARING=yes to prepare session metadata for the agent" >&2; exit 2; }
	cargo run -p incident-room -- investigate --session "$(SESSION_DIR)" --relaylog-bin "$(RELAYLOG)" --authorize-sharing
