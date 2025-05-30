.PHONY: test dev-db test-db clean-test-db

test: clean-test-db test-db
	@printf "\nRunning unittest...\n"
	DATABASE_URL=postgres://postgres:password@localhost:5432/test_biominer_indexd cargo test

test-db: clean-test-db
	@printf "\nLaunch postgres database...(default password: password)\n"
	@docker run --name biominer-indexd -e POSTGRES_PASSWORD=password -e POSTGRES_USER=postgres -p 5432:5432 -d postgres:10.0
	@sleep 3
	@echo "Create database: test_biominer_indexd"
	@bash build/create-db.sh test_biominer_indexd 5432

clean-test-db:
	@printf "Stop "
	@-docker stop biominer-indexd
	@printf "Clean "
	@-docker rm biominer-indexd

build-studio:
	@cd studio && yarn && yarn openapi && yarn build:embed && cd ..

build-indexd:
	@cargo build --release

build-indexd-linux:
	@cargo build --release --target=x86_64-unknown-linux-musl

build-mac: build-studio build-indexd
	@printf "\nBuilding...\n"

build-linux: build-studio build-indexd-linux
	@printf "\nBuilding...\n"

build-linux-on-mac: build-studio build-indexd-cross-compile-musl

# You must run `make build-service` to build new api spec for studio when you change the api spec
build-service:
	@printf "Building service based on openapi...\n"
	@curl -H "Accept: application/json" http://localhost:3000/spec -o studio/config/biominer-api.json
	@cd studio && yarn && yarn openapi && cd ..

build-indexd-cross-compile-musl:
	@docker run --rm -it -v "$(CURDIR)":/home/rust/src messense/rust-musl-cross:x86_64-musl cargo build --release
	@rsync -avP -e 'ssh -i ~/.ssh/biominer-system-ssh-token.pem' target/x86_64-unknown-linux-musl/release/biominer-indexd target/x86_64-unknown-linux-musl/release/biominer-indexd-cli root@api.3steps.cn:/opt/biominer-indexd/
