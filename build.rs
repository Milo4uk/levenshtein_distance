use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("levenshtein_shader", "spirv-unknown-vulkan1.2")
        .capability(spirv_builder::Capability::Int8)
        // .extension("SPV_KHR_8bit_storage")
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}
