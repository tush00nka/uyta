all: linux windows

linux:
	cargo build --release
	cp ./target/release/uyta .
	zip -r uyta-linux.zip static uyta
	rm ./uyta

windows:
	cross build --release --target=x86_64-pc-windows-gnu
	cp ./target/x86_64-pc-windows-gnu/release/uyta.exe .
	zip -r uyta-win.zip static uyta.exe
	rm ./uyta.exe
