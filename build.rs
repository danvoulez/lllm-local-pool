fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/vvtv/llmpool/v1/llmpool.proto")?;
    Ok(())
}
