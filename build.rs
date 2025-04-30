use spirv_builder::{MetadataPrintout, SpirvBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    SpirvBuilder::new("levenshtein_shader", "spirv-unknown-spv1.4")
        .print_metadata(MetadataPrintout::Full)
        .build()?;
    Ok(())
}
