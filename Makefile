# Single install recipe, used by the PKGBUILD and by `sudo make install` on any distribution.
PREFIX ?= /usr/local
DESTDIR ?=

all:
	cargo build --release --locked

install:
	install -Dm755 target/release/debarras $(DESTDIR)$(PREFIX)/bin/debarras
	install -Dm644 dev.andrea.Debarras.desktop $(DESTDIR)$(PREFIX)/share/applications/dev.andrea.Debarras.desktop
	install -Dm644 dev.andrea.Debarras.svg $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/dev.andrea.Debarras.svg
	install -Dm644 LICENSE $(DESTDIR)$(PREFIX)/share/licenses/debarras/LICENSE
	@# The desktop rereads an icon theme only when the theme's folder changes, not its subfolders.
	touch $(DESTDIR)$(PREFIX)/share/icons/hicolor

uninstall:
	rm -f $(DESTDIR)$(PREFIX)/bin/debarras \
	  $(DESTDIR)$(PREFIX)/share/applications/dev.andrea.Debarras.desktop \
	  $(DESTDIR)$(PREFIX)/share/icons/hicolor/scalable/apps/dev.andrea.Debarras.svg
	rm -rf $(DESTDIR)$(PREFIX)/share/licenses/debarras
	touch $(DESTDIR)$(PREFIX)/share/icons/hicolor

test:
	cargo test --locked

# The user guide: the shortcuts page is written from the help panel, then the book is built
# into book/book/ (needs mdbook: `cargo install mdbook`).
docs:
	DEBARRAS_WRITE_DOCS=1 cargo test --locked shortcuts_page_is_current
	mdbook build book

.PHONY: all install uninstall test docs
