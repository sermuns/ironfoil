use std::error::Error;
use vergen_gitcl::{Emitter, GitclBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    let git = GitclBuilder::default().describe(true, true, None).build()?;
    Emitter::default().add_instructions(&git)?.emit()?;
    Ok(())
}
