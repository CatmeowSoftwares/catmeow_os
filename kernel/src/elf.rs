use bitfields::bitfield;

struct Elf {}
#[bitfield(u64)]
struct ElfHeader {
    magic_number: bool,
    #[bits(3)]
    elf: u8,
    system: bool,
    endian: bool,
    version: bool,
    abi: bool,
    _padding: u8,
    #[bits(2)]
    elf_type: u8,
    #[bits(2)]
    instruction_set: u8,
    #[bits(4)]
    elf_version: u8,
    program_entry_offset: u8,
    program_header_table_offset: u8,
    section_header_table_offset: u8,
    #[bits(4)]
    flags: u8,
    #[bits(2)]
    elf_header_size: u8,
    #[bits(2)]
    program_header_table_entry_size: u8,
    #[bits(2)]
    program_header_table_entry_number: u8,
    #[bits(2)]
    section_header_table_entry_size: u8,
    #[bits(2)]
    section_header_table_entry_number: u8,
    #[bits(2)]
    section_header_string_table_section_index: u8,
}
