// Integration test for SigmaOS-calculator
// Verifies the crate builds and links as a dependency.
#![allow(unused_imports)]

use sigmaos_calculator as _;

#[test]
fn crate_links_and_runs() {
    assert_eq!(2 + 2, 4);
}
