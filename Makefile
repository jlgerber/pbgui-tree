build:
	cargo build --example pbgui-tree --release

install:
	cp ./target/release/examples/pbgui-tree ~/bin/.

rcc:
	rcc -binary ./resources/pbgui_tree.qrc -o ./resources/pbgui_tree.rcc

install-rcc:
	cp ./resources/pbgui-tree.rcc ~/bin/. && rm ./resources/pbgui-tree.rcc

all: build install rcc