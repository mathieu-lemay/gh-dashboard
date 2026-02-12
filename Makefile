MAKEFLAGS    += --always-make
SHELL        := /usr/bin/env bash
.SHELLFLAGS  := -e -o pipefail -c
.NOTPARALLEL :

PROJECT_ROOT ?= $(shell git rev-parse --show-toplevel)
PROJECT_NAME ?= pricing-service

DOCKER_CONTEXT ?= ${PROJECT_ROOT}
DOCKER_REPO    ?= ${PROJECT_NAME}
DOCKER_TAG     ?= dev

format:
	cargo +nightly fmt

lint:
	cargo +nightly fmt -- --check
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

coverage:
	cargo tarpaulin --skip-clean --include-tests --out html

docker.build:
	docker build \
		--ssh default \
		--tag "${DOCKER_REPO}:${DOCKER_TAG}" \
		${DOCKER_CONTEXT}

docker.run: docker.build
	docker run --rm -it \
		--name "${PROJECT_NAME}" \
		--env "GITHUB_TOKEN=$$(gh auth token)" \
		-v "${PROJECT_ROOT}/config.toml:/etc/gh-dashboard.toml:ro" \
		"${DOCKER_REPO}:${DOCKER_TAG}"
