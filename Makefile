SRC_DIR		:= src

TARGET 		:= i386-unknown-none.json
RUST_SRC	:= $(shell find $(SRC_DIR)/rust -type f -name '*.rs')
ASM_SRC 	:= $(shell find $(SRC_DIR)/asm -type f -name '*.asm')

ASM_OBJ 	:= $(ASM_SRC:.asm=.o)

# Build profile selection (default: release). Set DEBUG=1 to build debug.
DEBUG ?=
BUILD_PROFILE := $(if $(DEBUG),debug,release)
CARGO_FLAGS   := $(if $(DEBUG),,--release)

RUST_LIB	:= target/i386-unknown-none/$(BUILD_PROFILE)/libkfs.a

KERNEL_BIN  := kernel.bin
ISO_DIR     := iso
ISO_OUT     := kfs.iso

# --- Top-level targets -------------------------------------------------------

all: $(KERNEL_BIN)

iso: $(ISO_OUT)

run: $(ISO_OUT)
	kvm -cpu host -cdrom $(ISO_OUT)

clean:
	rm -rf *.o $(KERNEL_BIN) $(ISO_OUT)

fclean: clean
	rm -rf iso/boot/kernel.bin
	cargo clean

re: fclean all

# --- Build objects -----------------------------------------------------------

%.o: %.asm
	nasm -f elf32 $< -o $@

$(RUST_LIB): $(RUST_SRC) Cargo.toml $(TARGET)
	@echo "Building Rust crate (profile: $(BUILD_PROFILE))"
	cargo +nightly build --target $(TARGET) $(CARGO_FLAGS)

# --- Link kernel -------------------------------------------------------------

$(KERNEL_BIN): $(ASM_OBJ) $(RUST_LIB) linker.ld
	# Link the assembled objects with the rust static library which
	# contains libcore and the panic implementation
	ld -m elf_i386 -T linker.ld -o $(KERNEL_BIN) $(ASM_OBJ) $(RUST_LIB)

# --- ISO with GRUB -----------------------------------------------------------

$(ISO_OUT): $(KERNEL_BIN) iso/boot/grub/grub.cfg
	@cp $(KERNEL_BIN) $(ISO_DIR)/boot/kernel.bin
	@grub-mkrescue -o $(ISO_OUT) $(ISO_DIR)

define compile_from_source
    @rm -rf source_dir source.tar.gz
	@wget -O source.tar.gz $(1)
    @mkdir source_dir && tar xvf source.tar.gz -C source_dir --strip-components=1
    @cd source_dir && ./configure --prefix=$$HOME/.local && make -j && make install
    @rm -rf source_dir source.tar.gz
endef

install_requirements: uninstall_requirements
	$(call compile_from_source,https://mirror.cyberbits.eu/gnu/bison/bison-3.8.tar.xz)
	$(call compile_from_source,https://github.com/westes/flex/files/981163/flex-2.6.4.tar.gz)
	$(call compile_from_source,ftp://ftp.gnu.org/gnu/grub/grub-2.06.tar.xz)
	$(call compile_from_source,https://www.gnu.org/software/xorriso/xorriso-1.5.4.tar.gz)

uninstall_requirements:
	@rm -rf source_dir source.tar.gz
	@rm -rf $$HOME/.local/bin/grub-*
	@rm -rf $$HOME/.local/bin/xorriso*
	@rm -rf $$HOME/.local/bin/osirrox
	@rm -rf $$HOME/.local/bin/xorrecord
	@rm -rf $$HOME/.local/etc/grub.d
	@rm -rf $$HOME/.local/share/grub

.PHONY: all iso run clean fclean re install_requirements uninstall_requirements