ifndef EXE
	EXE := Vilhelm
endif

openbench:
	@echo Compiling $(EXE) for OpenBench
	cargo rustc --release --bin uci -- -C target-cpu=native --emit link=$(EXE)
