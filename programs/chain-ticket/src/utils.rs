// Utility so we don't have to add the `solana_sdk` crate as a dependency
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;

pub fn sol_to_lamports(sol: f64) -> u64 {
    (sol * LAMPORTS_PER_SOL as f64) as u64
}
