export prefix := "/usr"
BINDIR := prefix / "bin"

BINARY := "electrum"
TARGET := "debug"

default: setup extract-vendor build

setup: clean
  @echo "Setting up git hooks..."
  @cp -f hooks/pre-commit.hook .git/hooks/pre-commit

clean:
  @cargo clean
  @rm -rf .cargo vendor vendor.tar target

vendor:
  @mkdir -p .cargo
  @cargo vendor | head -n -1 > .cargo/config
  @echo 'directory = "vendor"' >> .cargo/config
  @tar pcf vendor.tar vendor
  @rm -rf vendor

extract-vendor: vendor
  @rm -rf vendor; tar pxf vendor.tar

build +ARGS="":
  @cargo build {{ARGS}} 

install:
  @install -Dm0755 "target"/{{TARGET}}/{{BINARY}} {{BINDIR}}/{{BINARY}}
  # TODO: Install Systemd

uninstall:
  @rm {{BINDIR}}/{{BINARY}}
  #TODO: Uninstall Systemd
