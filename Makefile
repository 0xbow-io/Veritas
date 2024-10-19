.DEFAULT_GOAL := help

.PHONY: vm

ifeq ($(FFI_DEBUG),true)
    GO_TAGS = -tags FFI_DEBUG
    FFI_TARGET = debug
else
    GO_TAGS =
    FFI_TARGET = all
endif

ARCH = $(shell uname -m)
OS = $(shell uname -s | tr '[:upper:]' '[:lower:]')
LIB_NAME = libcircom_$(OS)_$(ARCH).a

ifeq ($(shell uname -s),darwin)
	export CGO_LDFLAGS=-framework Foundation -framework SystemConfiguration

	# Set macOS deployment target in order to avoid linker warnings linke
	# "ld: warning: object file XX was built for newer macOS version (14.4) than being linked (14.0)"
	export MACOSX_DEPLOYMENT_TARGET=$(shell sw_vers --productVersion)

	# for test-race we need to pass -ldflags to fix linker warnings on macOS
	# see https://github.com/golang/go/issues/61229#issuecomment-1988965927
	TEST_RACE_LDFLAGS=-ldflags=-extldflags=-Wl,-ld_classic

	# Number of processes
	NPROCS = $(shell sysctl hw.ncpu  | grep -o '[0-9]\+')
else
	export CGO_LDFLAGS=-ldl -lm
	TEST_RACE_LDFLAGS=
	NPROCS = $(shell grep -c 'processor' /proc/cpuinfo)
endif

MAKEFLAGS += -j$(NPROCS)

ffi: circom ### compile circom-ffi bindings

circom:
	$(MAKE) -C circom_ffi/circom $(FFI_TARGET)
	mkdir -p include/$(OS)_$(ARCH)
	cp circom_ffi/target/release/libcircom.a include/$(OS)_$(ARCH)/$(LIB_NAME)

generate: ## generate
	mkdir -p mocks
	go generate ./...

clean-testcache:
	go clean -testcache

test: clean-testcache ffi ## tests
	go test $(GO_TAGS) ./...

test-cached: rustdeps ## tests with existing cache
	go test $(GO_TAGS) ./...

test-race: clean-testcache rustdeps
	go test $(GO_TAGS) ./... -race $(TEST_RACE_LDFLAGS)

benchmarks: rustdeps ## benchmarking
	go test $(GO_TAGS) ./... -run=^# -bench=. -benchmem

test-cover: clean-testcache rustdeps ## tests with coverage
	mkdir -p coverage
	go test $(GO_TAGS) -coverpkg=./... -coverprofile=coverage/coverage.out -covermode=atomic ./...
	go tool cover -html=coverage/coverage.out -o coverage/coverage.html

install-deps: | install-gofumpt install-mockgen install-golangci-lint## install some project dependencies

install-gofumpt:
	go install mvdan.cc/gofumpt@latest

install-mockgen:
	go install go.uber.org/mock/mockgen@latest

install-golangci-lint:
	@which golangci-lint || go install github.com/golangci/golangci-lint/cmd/golangci-lint@v1.61.0

lint: install-golangci-lint
	golangci-lint run

tidy: ## add missing and remove unused modules
	 go mod tidy

format: ## run go & rust formatters
	$(MAKE) -C circom/circom format
	gofumpt -l -w .

clean: ## clean project builds
	$(MAKE) -C circom_ffi/circom clean
	@rm -rf ./build
	@rm -rf ./include

help: ## show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
