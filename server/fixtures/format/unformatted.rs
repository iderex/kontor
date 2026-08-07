// The file the server formatter has to refuse, and it is not part of any crate:
// server/Cargo.toml names crates/* and fixtures/lint as its members, so nothing
// compiles this and `cargo fmt --all` never reaches it. Only ./prove-quality
// points rustfmt at it, and it does so with --config-path server/rustfmt.toml
// so the shipped configuration is what judges it rather than a copy.
//
// The wrongness here is ordinary rather than exotic: the indentation somebody
// gets from a fast edit in an editor that was not set up. formatted.rs is the
// same code after rustfmt has been over it.

pub fn total( values : &[u32] ) -> u32
{
      let mut sum=0;
    for value in values { sum += value; }
  sum
}
