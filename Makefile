WINDOWS = x86_64-pc-windows-gnu
OUTDIR = builds

PROJ = autotabber
CLI = autotabber
GUI = autotabber-gui
SOURCE_ZIP = autotabber_source_code.zip

build: windows linux source pack

windows:
	cargo build --release -p $(CLI) --target $(WINDOWS)
	cargo build --release -p $(GUI) --target $(WINDOWS)

linux:
	cargo build --release -p $(CLI)
	cargo build --release -p $(GUI)

source:
	git archive --prefix=autotabber_source_code/ -o $(SOURCE_ZIP) HEAD

pack:
	mkdir -p $(OUTDIR)
	cp target/release/$(CLI) $(OUTDIR)
	cp target/release/$(GUI) $(OUTDIR)
	cp target/$(WINDOWS)/release/$(CLI).exe $(OUTDIR)
	cp target/$(WINDOWS)/release/$(GUI).exe $(OUTDIR)
	mv $(SOURCE_ZIP) $(OUTDIR)

itch:
	butler push $(OUTDIR)/$(GUI) seebass22/$(PROJ):linux
	butler push $(OUTDIR)/$(CLI) seebass22/$(PROJ):linux-cli
	butler push $(OUTDIR)/$(GUI).exe seebass22/$(PROJ):windows
	butler push $(OUTDIR)/$(CLI).exe seebass22/$(PROJ):windows-cli
	butler push $(OUTDIR)/$(SOURCE_ZIP) seebass22/$(PROJ):source
