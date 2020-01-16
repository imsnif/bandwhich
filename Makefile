prefix ?= /usr/local

TARGET=debug
DEBUG ?= 0
ifeq ($(DEBUG),0)
	TARGET = release
	ARGS = --release
endif

VENDOR ?= 0
ifeq ($(VENDOR),1)
	ARGS += --frozen
endif

APP=bandwhich
BIN=target/$(TARGET)/$(APP)
BIN_DST=$(DESTDIR)$(prefix)/bin/$(APP)
DOC_DST=$(DESTDIR)$(prefix)/share/man/man1/bandwhich.1
LIC_DST=$(DESTDIR)$(prefix)/share/licenses/$(APP)
SRC = Makefile Cargo.lock Cargo.toml $(shell find src -type f -wholename 'src/*.rs')

.PHONY: all clean distclean install uninstall vendor

all: $(BIN)

clean:
	cargo clean

distclean:
	rm -rf .cargo vendor vendor.tar

$(BIN): $(SRC)
ifeq ($(VENDOR),1)
	tar pxf vendor.tar
endif
	cargo build $(ARGS)

install:
	install -Dm755 $(BIN) $(BIN_DST)
	install -Dm644 docs/bandwhich.1 $(DOC_DST)
	install -Dm644 LICENSE.md $(LIC_DST)/LICENSE

uninstall:
	rm -rf $(BIN_DST) $(DOC_DST) $(LIC_DST)

vendor:
	mkdir -p .cargo
	cargo vendor | head -n -1 > .cargo/config
	echo 'directory = "vendor"' >> .cargo/config
	tar pcf vendor.tar vendor
