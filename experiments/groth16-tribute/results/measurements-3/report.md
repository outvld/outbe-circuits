# Groth16 Tribute measurements

Individual wall-clock observations, not a statistical benchmark. Fresh process per backend/case/sample; OS filesystem caches left intact. Verification occurs after proving, not in a standalone cold verifier. All proofs use fresh OS-entropy-seeded blinding. Groth16 setup is local single-party experimental setup, never production ceremony material. Production circuits and backend are unchanged.

## Core timings

Times in milliseconds. Witness means constrained synthesis/finalization/matrix materialization for Groth16 and initial-map construction/ACVM solving/finalization for Noir. Groth16 proving excludes repeated synthesis. N/A is inapplicable, never an unmeasured zero.

| Circuit | Backend | Sample | Witness ms | Witness encode ms | Prove ms | Verify ms |
|---|---|---:|---:|---:|---:|---:|
| demo_tribute | groth16 | 1 | 58.768 | N/A | 104.349 | 0.747 |
| demo_tribute | noir_bb | 1 | 1.555 | 0.730 | 154.134 | 1.088 |
| demo_tribute | noir_bb | 2 | 1.611 | 0.716 | 167.251 | 1.243 |
| demo_tribute | groth16 | 2 | 59.403 | N/A | 106.360 | 0.759 |
| demo_tribute | groth16 | 3 | 59.761 | N/A | 105.801 | 0.759 |
| demo_tribute | noir_bb | 3 | 1.622 | 0.723 | 155.663 | 1.206 |
| flat_aggregation_n1 | groth16 | 1 | 30.173 | N/A | 76.986 | 0.693 |
| flat_aggregation_n1 | noir_bb | 1 | 1.117 | 0.542 | 147.414 | 1.132 |
| flat_aggregation_n1 | noir_bb | 2 | 1.129 | 0.508 | 154.587 | 1.125 |
| flat_aggregation_n1 | groth16 | 2 | 30.620 | N/A | 83.443 | 0.715 |
| flat_aggregation_n1 | groth16 | 3 | 30.453 | N/A | 73.707 | 0.716 |
| flat_aggregation_n1 | noir_bb | 3 | 1.064 | 0.542 | 148.493 | 1.251 |
| flat_aggregation_n2 | groth16 | 1 | 65.833 | N/A | 140.891 | 0.815 |
| flat_aggregation_n2 | noir_bb | 1 | 1.973 | 0.954 | 182.074 | 1.234 |
| flat_aggregation_n2 | noir_bb | 2 | 1.973 | 1.006 | 169.139 | 1.233 |
| flat_aggregation_n2 | groth16 | 2 | 60.541 | N/A | 124.721 | 0.863 |
| flat_aggregation_n2 | groth16 | 3 | 63.423 | N/A | 131.759 | 0.841 |
| flat_aggregation_n2 | noir_bb | 3 | 1.900 | 0.956 | 155.094 | 1.236 |
| flat_aggregation_n4 | groth16 | 1 | 117.128 | N/A | 206.043 | 1.021 |
| flat_aggregation_n4 | noir_bb | 1 | 3.451 | 2.559 | 172.585 | 1.147 |
| flat_aggregation_n4 | noir_bb | 2 | 3.300 | 2.485 | 170.845 | 1.147 |
| flat_aggregation_n4 | groth16 | 2 | 117.398 | N/A | 208.067 | 1.026 |
| flat_aggregation_n4 | groth16 | 3 | 115.961 | N/A | 214.112 | 1.003 |
| flat_aggregation_n4 | noir_bb | 3 | 3.336 | 2.624 | 187.681 | 1.190 |
| flat_aggregation_n8 | groth16 | 1 | 238.763 | N/A | 420.802 | 1.430 |
| flat_aggregation_n8 | noir_bb | 1 | 6.332 | 6.443 | 226.901 | 1.126 |
| flat_aggregation_n8 | noir_bb | 2 | 6.134 | 6.129 | 224.412 | 1.217 |
| flat_aggregation_n8 | groth16 | 2 | 240.230 | N/A | 378.666 | 1.417 |
| flat_aggregation_n8 | groth16 | 3 | 237.761 | N/A | 376.440 | 1.510 |
| flat_aggregation_n8 | noir_bb | 3 | 6.947 | 6.405 | 230.679 | 1.197 |
| flat_aggregation_n16 | groth16 | 1 | 458.716 | N/A | 708.499 | 2.210 |
| flat_aggregation_n16 | noir_bb | 1 | 11.959 | 15.601 | 361.345 | 1.106 |
| flat_aggregation_n16 | noir_bb | 2 | 11.858 | 15.763 | 369.174 | 1.186 |
| flat_aggregation_n16 | groth16 | 2 | 500.383 | N/A | 735.241 | 2.268 |
| flat_aggregation_n16 | groth16 | 3 | 467.264 | N/A | 676.102 | 2.293 |
| flat_aggregation_n16 | noir_bb | 3 | 12.742 | 15.758 | 367.015 | 1.118 |
| flat_aggregation_n32 | groth16 | 1 | 927.021 | N/A | 1432.050 | 3.930 |
| flat_aggregation_n32 | noir_bb | 1 | 23.908 | 36.783 | 660.048 | 1.169 |
| flat_aggregation_n32 | noir_bb | 2 | 23.909 | 37.990 | 633.878 | 1.213 |
| flat_aggregation_n32 | groth16 | 2 | 931.389 | N/A | 1273.109 | 4.001 |
| flat_aggregation_n32 | groth16 | 3 | 927.953 | N/A | 1308.873 | 3.813 |
| flat_aggregation_n32 | noir_bb | 3 | 23.994 | 37.247 | 617.321 | 1.191 |
| flat_aggregation_n64 | groth16 | 1 | 1871.556 | N/A | 2541.927 | 7.098 |
| flat_aggregation_n64 | noir_bb | 1 | 48.452 | 83.290 | 1078.013 | 1.166 |
| flat_aggregation_n64 | noir_bb | 2 | 46.291 | 82.905 | 1115.842 | 1.229 |
| flat_aggregation_n64 | groth16 | 2 | 1836.005 | N/A | 2648.991 | 7.098 |
| flat_aggregation_n64 | groth16 | 3 | 1827.398 | N/A | 2539.404 | 7.134 |
| flat_aggregation_n64 | noir_bb | 3 | 47.166 | 83.369 | 1112.360 | 1.286 |
| emit_mint | groth16 | 1 | 39.552 | N/A | 86.579 | 0.776 |
| emit_mint | noir_bb | 1 | 1.076 | 0.315 | 87.267 | 1.129 |
| emit_mint | noir_bb | 2 | 1.074 | 0.326 | 86.681 | 1.073 |
| emit_mint | groth16 | 2 | 39.608 | N/A | 95.332 | 0.800 |
| emit_mint | groth16 | 3 | 39.435 | N/A | 87.721 | 0.780 |
| emit_mint | noir_bb | 3 | 1.082 | 0.308 | 89.544 | 1.135 |

Emit rows use frozen `emit_mint@1.5.0` and partial-mint fixtures with wide amounts, cross-limb borrowing, and a real index-5 note inclusion path. Full mints and amount/address boundaries are checked separately. The port mirrors the pinned `noir-bignum v0.10.0` statement, including its `[120,120,17]`-bit ABI limb bounds and subtraction-flag constraints; it does not silently add a stricter uint256 gate or repair the production circuit.

## Technical timings

One row per observation, in milliseconds. Sample 0 is one-time preparation (one setup per shape), not a per-proof sample. Compilation and VK derivation include subprocess overhead. Groth16 PK/VK writes include compressed serialization plus disk write; checked key decoding is separate from disk reads. bb has no stored proving key or separate VK preparation API: PK read/decode and VK decode/prepare are N/A; native VK deserialization/preprocessing stays inside FFI verify. `srs_init` loads/checksums and initializes the maximum 1,048,577-point prefix, not a circuit-minimal SRS. Requested thread counts are in metadata, not a claim of measured effective utilization.

| Circuit | Backend | Sample | Stage | Elapsed ms |
|---|---|---:|---|---:|
| demo_tribute | noir_bb | 0 | compile_subprocess | 102.397 |
| demo_tribute | noir_bb | 0 | vk_derive_subprocess | 79.366 |
| demo_tribute | noir_bb | 0 | circuit_stats | 19.761 |
| flat_aggregation_n1 | noir_bb | 0 | compile_subprocess | 106.329 |
| flat_aggregation_n1 | noir_bb | 0 | vk_derive_subprocess | 68.571 |
| flat_aggregation_n1 | noir_bb | 0 | circuit_stats | 4.580 |
| flat_aggregation_n2 | noir_bb | 0 | compile_subprocess | 110.893 |
| flat_aggregation_n2 | noir_bb | 0 | vk_derive_subprocess | 77.766 |
| flat_aggregation_n2 | noir_bb | 0 | circuit_stats | 6.857 |
| flat_aggregation_n4 | noir_bb | 0 | compile_subprocess | 130.100 |
| flat_aggregation_n4 | noir_bb | 0 | vk_derive_subprocess | 103.456 |
| flat_aggregation_n4 | noir_bb | 0 | circuit_stats | 12.396 |
| flat_aggregation_n8 | noir_bb | 0 | compile_subprocess | 170.590 |
| flat_aggregation_n8 | noir_bb | 0 | vk_derive_subprocess | 129.041 |
| flat_aggregation_n8 | noir_bb | 0 | circuit_stats | 22.825 |
| flat_aggregation_n16 | noir_bb | 0 | compile_subprocess | 256.726 |
| flat_aggregation_n16 | noir_bb | 0 | vk_derive_subprocess | 184.108 |
| flat_aggregation_n16 | noir_bb | 0 | circuit_stats | 42.852 |
| flat_aggregation_n32 | noir_bb | 0 | compile_subprocess | 443.788 |
| flat_aggregation_n32 | noir_bb | 0 | vk_derive_subprocess | 348.364 |
| flat_aggregation_n32 | noir_bb | 0 | circuit_stats | 92.391 |
| flat_aggregation_n64 | noir_bb | 0 | compile_subprocess | 1055.441 |
| flat_aggregation_n64 | noir_bb | 0 | vk_derive_subprocess | 548.873 |
| flat_aggregation_n64 | noir_bb | 0 | circuit_stats | 179.421 |
| emit_mint | noir_bb | 0 | compile_subprocess | 264.608 |
| emit_mint | noir_bb | 0 | vk_derive_subprocess | 27.785 |
| emit_mint | noir_bb | 0 | circuit_stats | 1.719 |
| demo_tribute | groth16 | 0 | setup | 176.952 |
| demo_tribute | groth16 | 0 | pk_write | 7.528 |
| demo_tribute | groth16 | 0 | vk_write | 0.105 |
| flat_aggregation_n1 | groth16 | 0 | setup | 110.119 |
| flat_aggregation_n1 | groth16 | 0 | pk_write | 5.365 |
| flat_aggregation_n1 | groth16 | 0 | vk_write | 0.069 |
| flat_aggregation_n2 | groth16 | 0 | setup | 212.777 |
| flat_aggregation_n2 | groth16 | 0 | pk_write | 10.459 |
| flat_aggregation_n2 | groth16 | 0 | vk_write | 0.105 |
| flat_aggregation_n4 | groth16 | 0 | setup | 391.151 |
| flat_aggregation_n4 | groth16 | 0 | pk_write | 23.981 |
| flat_aggregation_n4 | groth16 | 0 | vk_write | 0.105 |
| flat_aggregation_n8 | groth16 | 0 | setup | 785.361 |
| flat_aggregation_n8 | groth16 | 0 | pk_write | 76.140 |
| flat_aggregation_n8 | groth16 | 0 | vk_write | 0.155 |
| flat_aggregation_n16 | groth16 | 0 | setup | 1620.671 |
| flat_aggregation_n16 | groth16 | 0 | pk_write | 158.254 |
| flat_aggregation_n16 | groth16 | 0 | vk_write | 0.197 |
| flat_aggregation_n32 | groth16 | 0 | setup | 3030.887 |
| flat_aggregation_n32 | groth16 | 0 | pk_write | 318.980 |
| flat_aggregation_n32 | groth16 | 0 | vk_write | 0.265 |
| flat_aggregation_n64 | groth16 | 0 | setup | 6165.125 |
| flat_aggregation_n64 | groth16 | 0 | pk_write | 707.885 |
| flat_aggregation_n64 | groth16 | 0 | vk_write | 0.191 |
| emit_mint | groth16 | 0 | setup | 108.374 |
| emit_mint | groth16 | 0 | pk_write | 7.160 |
| emit_mint | groth16 | 0 | vk_write | 0.096 |
| demo_tribute | groth16 | 1 | fixture | 7.357 |
| demo_tribute | groth16 | 1 | pk_read | 0.919 |
| demo_tribute | groth16 | 1 | pk_decode | 1535.145 |
| demo_tribute | groth16 | 1 | vk_read | 0.059 |
| demo_tribute | groth16 | 1 | vk_decode | 0.467 |
| demo_tribute | groth16 | 1 | vk_prepare | 0.538 |
| demo_tribute | groth16 | 1 | proof_encode | 0.002 |
| demo_tribute | groth16 | 1 | proof_decode | 0.106 |
| demo_tribute | noir_bb | 1 | fixture | 7.642 |
| demo_tribute | noir_bb | 1 | artifact_read | 0.045 |
| demo_tribute | noir_bb | 1 | artifact_base64 | 0.008 |
| demo_tribute | noir_bb | 1 | program_decode | 0.761 |
| demo_tribute | noir_bb | 1 | acir_gunzip | 0.102 |
| demo_tribute | noir_bb | 1 | vk_read | 0.027 |
| demo_tribute | noir_bb | 1 | backend_init | 0.000 |
| demo_tribute | noir_bb | 1 | srs_init | 151.171 |
| demo_tribute | noir_bb | 1 | proof_encode | 0.001 |
| demo_tribute | noir_bb | 1 | proof_decode | 0.005 |
| demo_tribute | noir_bb | 2 | fixture | 7.478 |
| demo_tribute | noir_bb | 2 | artifact_read | 0.031 |
| demo_tribute | noir_bb | 2 | artifact_base64 | 0.007 |
| demo_tribute | noir_bb | 2 | program_decode | 0.736 |
| demo_tribute | noir_bb | 2 | acir_gunzip | 0.116 |
| demo_tribute | noir_bb | 2 | vk_read | 0.021 |
| demo_tribute | noir_bb | 2 | backend_init | 0.000 |
| demo_tribute | noir_bb | 2 | srs_init | 147.954 |
| demo_tribute | noir_bb | 2 | proof_encode | 0.001 |
| demo_tribute | noir_bb | 2 | proof_decode | 0.004 |
| demo_tribute | groth16 | 2 | fixture | 7.974 |
| demo_tribute | groth16 | 2 | pk_read | 1.036 |
| demo_tribute | groth16 | 2 | pk_decode | 1561.050 |
| demo_tribute | groth16 | 2 | vk_read | 0.090 |
| demo_tribute | groth16 | 2 | vk_decode | 0.470 |
| demo_tribute | groth16 | 2 | vk_prepare | 0.574 |
| demo_tribute | groth16 | 2 | proof_encode | 0.002 |
| demo_tribute | groth16 | 2 | proof_decode | 0.110 |
| demo_tribute | groth16 | 3 | fixture | 7.652 |
| demo_tribute | groth16 | 3 | pk_read | 0.972 |
| demo_tribute | groth16 | 3 | pk_decode | 1550.416 |
| demo_tribute | groth16 | 3 | vk_read | 0.059 |
| demo_tribute | groth16 | 3 | vk_decode | 0.491 |
| demo_tribute | groth16 | 3 | vk_prepare | 0.547 |
| demo_tribute | groth16 | 3 | proof_encode | 0.002 |
| demo_tribute | groth16 | 3 | proof_decode | 0.104 |
| demo_tribute | noir_bb | 3 | fixture | 7.643 |
| demo_tribute | noir_bb | 3 | artifact_read | 0.041 |
| demo_tribute | noir_bb | 3 | artifact_base64 | 0.008 |
| demo_tribute | noir_bb | 3 | program_decode | 0.785 |
| demo_tribute | noir_bb | 3 | acir_gunzip | 0.127 |
| demo_tribute | noir_bb | 3 | vk_read | 0.031 |
| demo_tribute | noir_bb | 3 | backend_init | 0.000 |
| demo_tribute | noir_bb | 3 | srs_init | 152.893 |
| demo_tribute | noir_bb | 3 | proof_encode | 0.001 |
| demo_tribute | noir_bb | 3 | proof_decode | 0.005 |
| flat_aggregation_n1 | groth16 | 1 | fixture | 0.365 |
| flat_aggregation_n1 | groth16 | 1 | pk_read | 0.633 |
| flat_aggregation_n1 | groth16 | 1 | pk_decode | 1102.488 |
| flat_aggregation_n1 | groth16 | 1 | vk_read | 0.067 |
| flat_aggregation_n1 | groth16 | 1 | vk_decode | 0.421 |
| flat_aggregation_n1 | groth16 | 1 | vk_prepare | 0.541 |
| flat_aggregation_n1 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 1 | proof_decode | 0.102 |
| flat_aggregation_n1 | noir_bb | 1 | fixture | 0.363 |
| flat_aggregation_n1 | noir_bb | 1 | artifact_read | 0.024 |
| flat_aggregation_n1 | noir_bb | 1 | artifact_base64 | 0.006 |
| flat_aggregation_n1 | noir_bb | 1 | program_decode | 0.615 |
| flat_aggregation_n1 | noir_bb | 1 | acir_gunzip | 0.081 |
| flat_aggregation_n1 | noir_bb | 1 | vk_read | 0.014 |
| flat_aggregation_n1 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 1 | srs_init | 156.003 |
| flat_aggregation_n1 | noir_bb | 1 | proof_encode | 0.002 |
| flat_aggregation_n1 | noir_bb | 1 | proof_decode | 0.006 |
| flat_aggregation_n1 | noir_bb | 2 | fixture | 0.391 |
| flat_aggregation_n1 | noir_bb | 2 | artifact_read | 0.030 |
| flat_aggregation_n1 | noir_bb | 2 | artifact_base64 | 0.007 |
| flat_aggregation_n1 | noir_bb | 2 | program_decode | 0.628 |
| flat_aggregation_n1 | noir_bb | 2 | acir_gunzip | 0.087 |
| flat_aggregation_n1 | noir_bb | 2 | vk_read | 0.021 |
| flat_aggregation_n1 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 2 | srs_init | 154.396 |
| flat_aggregation_n1 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 2 | proof_decode | 0.006 |
| flat_aggregation_n1 | groth16 | 2 | fixture | 0.358 |
| flat_aggregation_n1 | groth16 | 2 | pk_read | 0.721 |
| flat_aggregation_n1 | groth16 | 2 | pk_decode | 1114.714 |
| flat_aggregation_n1 | groth16 | 2 | vk_read | 0.072 |
| flat_aggregation_n1 | groth16 | 2 | vk_decode | 0.423 |
| flat_aggregation_n1 | groth16 | 2 | vk_prepare | 0.531 |
| flat_aggregation_n1 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 2 | proof_decode | 0.103 |
| flat_aggregation_n1 | groth16 | 3 | fixture | 0.390 |
| flat_aggregation_n1 | groth16 | 3 | pk_read | 0.699 |
| flat_aggregation_n1 | groth16 | 3 | pk_decode | 1117.040 |
| flat_aggregation_n1 | groth16 | 3 | vk_read | 0.075 |
| flat_aggregation_n1 | groth16 | 3 | vk_decode | 0.420 |
| flat_aggregation_n1 | groth16 | 3 | vk_prepare | 0.533 |
| flat_aggregation_n1 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 3 | proof_decode | 0.104 |
| flat_aggregation_n1 | noir_bb | 3 | fixture | 0.405 |
| flat_aggregation_n1 | noir_bb | 3 | artifact_read | 0.026 |
| flat_aggregation_n1 | noir_bb | 3 | artifact_base64 | 0.006 |
| flat_aggregation_n1 | noir_bb | 3 | program_decode | 0.690 |
| flat_aggregation_n1 | noir_bb | 3 | acir_gunzip | 0.081 |
| flat_aggregation_n1 | noir_bb | 3 | vk_read | 0.021 |
| flat_aggregation_n1 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 3 | srs_init | 150.176 |
| flat_aggregation_n1 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 3 | proof_decode | 0.005 |
| flat_aggregation_n2 | groth16 | 1 | fixture | 0.759 |
| flat_aggregation_n2 | groth16 | 1 | pk_read | 1.450 |
| flat_aggregation_n2 | groth16 | 1 | pk_decode | 2250.142 |
| flat_aggregation_n2 | groth16 | 1 | vk_read | 0.089 |
| flat_aggregation_n2 | groth16 | 1 | vk_decode | 0.518 |
| flat_aggregation_n2 | groth16 | 1 | vk_prepare | 0.575 |
| flat_aggregation_n2 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 1 | proof_decode | 0.105 |
| flat_aggregation_n2 | noir_bb | 1 | fixture | 0.758 |
| flat_aggregation_n2 | noir_bb | 1 | artifact_read | 0.041 |
| flat_aggregation_n2 | noir_bb | 1 | artifact_base64 | 0.010 |
| flat_aggregation_n2 | noir_bb | 1 | program_decode | 1.150 |
| flat_aggregation_n2 | noir_bb | 1 | acir_gunzip | 0.142 |
| flat_aggregation_n2 | noir_bb | 1 | vk_read | 0.022 |
| flat_aggregation_n2 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 1 | srs_init | 159.079 |
| flat_aggregation_n2 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 1 | proof_decode | 0.007 |
| flat_aggregation_n2 | noir_bb | 2 | fixture | 0.866 |
| flat_aggregation_n2 | noir_bb | 2 | artifact_read | 0.061 |
| flat_aggregation_n2 | noir_bb | 2 | artifact_base64 | 0.012 |
| flat_aggregation_n2 | noir_bb | 2 | program_decode | 1.174 |
| flat_aggregation_n2 | noir_bb | 2 | acir_gunzip | 0.142 |
| flat_aggregation_n2 | noir_bb | 2 | vk_read | 0.032 |
| flat_aggregation_n2 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 2 | srs_init | 161.150 |
| flat_aggregation_n2 | noir_bb | 2 | proof_encode | 0.002 |
| flat_aggregation_n2 | noir_bb | 2 | proof_decode | 0.005 |
| flat_aggregation_n2 | groth16 | 2 | fixture | 0.745 |
| flat_aggregation_n2 | groth16 | 2 | pk_read | 1.478 |
| flat_aggregation_n2 | groth16 | 2 | pk_decode | 2312.818 |
| flat_aggregation_n2 | groth16 | 2 | vk_read | 0.080 |
| flat_aggregation_n2 | groth16 | 2 | vk_decode | 0.471 |
| flat_aggregation_n2 | groth16 | 2 | vk_prepare | 0.560 |
| flat_aggregation_n2 | groth16 | 2 | proof_encode | 0.003 |
| flat_aggregation_n2 | groth16 | 2 | proof_decode | 0.107 |
| flat_aggregation_n2 | groth16 | 3 | fixture | 0.735 |
| flat_aggregation_n2 | groth16 | 3 | pk_read | 1.275 |
| flat_aggregation_n2 | groth16 | 3 | pk_decode | 2271.978 |
| flat_aggregation_n2 | groth16 | 3 | vk_read | 0.081 |
| flat_aggregation_n2 | groth16 | 3 | vk_decode | 0.454 |
| flat_aggregation_n2 | groth16 | 3 | vk_prepare | 0.559 |
| flat_aggregation_n2 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 3 | proof_decode | 0.104 |
| flat_aggregation_n2 | noir_bb | 3 | fixture | 0.743 |
| flat_aggregation_n2 | noir_bb | 3 | artifact_read | 0.042 |
| flat_aggregation_n2 | noir_bb | 3 | artifact_base64 | 0.010 |
| flat_aggregation_n2 | noir_bb | 3 | program_decode | 1.120 |
| flat_aggregation_n2 | noir_bb | 3 | acir_gunzip | 0.150 |
| flat_aggregation_n2 | noir_bb | 3 | vk_read | 0.028 |
| flat_aggregation_n2 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 3 | srs_init | 151.076 |
| flat_aggregation_n2 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n4 | groth16 | 1 | fixture | 1.429 |
| flat_aggregation_n4 | groth16 | 1 | pk_read | 2.559 |
| flat_aggregation_n4 | groth16 | 1 | pk_decode | 4553.627 |
| flat_aggregation_n4 | groth16 | 1 | vk_read | 0.064 |
| flat_aggregation_n4 | groth16 | 1 | vk_decode | 0.479 |
| flat_aggregation_n4 | groth16 | 1 | vk_prepare | 0.556 |
| flat_aggregation_n4 | groth16 | 1 | proof_encode | 0.003 |
| flat_aggregation_n4 | groth16 | 1 | proof_decode | 0.106 |
| flat_aggregation_n4 | noir_bb | 1 | fixture | 1.422 |
| flat_aggregation_n4 | noir_bb | 1 | artifact_read | 0.035 |
| flat_aggregation_n4 | noir_bb | 1 | artifact_base64 | 0.014 |
| flat_aggregation_n4 | noir_bb | 1 | program_decode | 2.006 |
| flat_aggregation_n4 | noir_bb | 1 | acir_gunzip | 0.256 |
| flat_aggregation_n4 | noir_bb | 1 | vk_read | 0.021 |
| flat_aggregation_n4 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 1 | srs_init | 154.205 |
| flat_aggregation_n4 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n4 | noir_bb | 1 | proof_decode | 0.005 |
| flat_aggregation_n4 | noir_bb | 2 | fixture | 1.465 |
| flat_aggregation_n4 | noir_bb | 2 | artifact_read | 0.045 |
| flat_aggregation_n4 | noir_bb | 2 | artifact_base64 | 0.014 |
| flat_aggregation_n4 | noir_bb | 2 | program_decode | 2.005 |
| flat_aggregation_n4 | noir_bb | 2 | acir_gunzip | 0.249 |
| flat_aggregation_n4 | noir_bb | 2 | vk_read | 0.021 |
| flat_aggregation_n4 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 2 | srs_init | 149.104 |
| flat_aggregation_n4 | noir_bb | 2 | proof_encode | 0.002 |
| flat_aggregation_n4 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n4 | groth16 | 2 | fixture | 1.442 |
| flat_aggregation_n4 | groth16 | 2 | pk_read | 2.561 |
| flat_aggregation_n4 | groth16 | 2 | pk_decode | 4486.618 |
| flat_aggregation_n4 | groth16 | 2 | vk_read | 0.077 |
| flat_aggregation_n4 | groth16 | 2 | vk_decode | 0.450 |
| flat_aggregation_n4 | groth16 | 2 | vk_prepare | 0.539 |
| flat_aggregation_n4 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 2 | proof_decode | 0.107 |
| flat_aggregation_n4 | groth16 | 3 | fixture | 1.471 |
| flat_aggregation_n4 | groth16 | 3 | pk_read | 2.697 |
| flat_aggregation_n4 | groth16 | 3 | pk_decode | 4519.423 |
| flat_aggregation_n4 | groth16 | 3 | vk_read | 0.066 |
| flat_aggregation_n4 | groth16 | 3 | vk_decode | 0.460 |
| flat_aggregation_n4 | groth16 | 3 | vk_prepare | 0.545 |
| flat_aggregation_n4 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n4 | noir_bb | 3 | fixture | 1.424 |
| flat_aggregation_n4 | noir_bb | 3 | artifact_read | 0.027 |
| flat_aggregation_n4 | noir_bb | 3 | artifact_base64 | 0.014 |
| flat_aggregation_n4 | noir_bb | 3 | program_decode | 2.005 |
| flat_aggregation_n4 | noir_bb | 3 | acir_gunzip | 0.250 |
| flat_aggregation_n4 | noir_bb | 3 | vk_read | 0.026 |
| flat_aggregation_n4 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 3 | srs_init | 148.971 |
| flat_aggregation_n4 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n4 | noir_bb | 3 | proof_decode | 0.004 |
| flat_aggregation_n8 | groth16 | 1 | fixture | 2.828 |
| flat_aggregation_n8 | groth16 | 1 | pk_read | 5.190 |
| flat_aggregation_n8 | groth16 | 1 | pk_decode | 9085.809 |
| flat_aggregation_n8 | groth16 | 1 | vk_read | 0.068 |
| flat_aggregation_n8 | groth16 | 1 | vk_decode | 0.485 |
| flat_aggregation_n8 | groth16 | 1 | vk_prepare | 0.532 |
| flat_aggregation_n8 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 1 | proof_decode | 0.106 |
| flat_aggregation_n8 | noir_bb | 1 | fixture | 2.919 |
| flat_aggregation_n8 | noir_bb | 1 | artifact_read | 0.079 |
| flat_aggregation_n8 | noir_bb | 1 | artifact_base64 | 0.032 |
| flat_aggregation_n8 | noir_bb | 1 | program_decode | 3.925 |
| flat_aggregation_n8 | noir_bb | 1 | acir_gunzip | 0.574 |
| flat_aggregation_n8 | noir_bb | 1 | vk_read | 0.048 |
| flat_aggregation_n8 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 1 | srs_init | 150.858 |
| flat_aggregation_n8 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n8 | noir_bb | 2 | fixture | 2.855 |
| flat_aggregation_n8 | noir_bb | 2 | artifact_read | 0.053 |
| flat_aggregation_n8 | noir_bb | 2 | artifact_base64 | 0.027 |
| flat_aggregation_n8 | noir_bb | 2 | program_decode | 3.915 |
| flat_aggregation_n8 | noir_bb | 2 | acir_gunzip | 0.582 |
| flat_aggregation_n8 | noir_bb | 2 | vk_read | 0.035 |
| flat_aggregation_n8 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 2 | srs_init | 150.504 |
| flat_aggregation_n8 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n8 | groth16 | 2 | fixture | 2.838 |
| flat_aggregation_n8 | groth16 | 2 | pk_read | 5.470 |
| flat_aggregation_n8 | groth16 | 2 | pk_decode | 9092.335 |
| flat_aggregation_n8 | groth16 | 2 | vk_read | 0.072 |
| flat_aggregation_n8 | groth16 | 2 | vk_decode | 0.498 |
| flat_aggregation_n8 | groth16 | 2 | vk_prepare | 0.553 |
| flat_aggregation_n8 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n8 | groth16 | 3 | fixture | 2.832 |
| flat_aggregation_n8 | groth16 | 3 | pk_read | 5.177 |
| flat_aggregation_n8 | groth16 | 3 | pk_decode | 9179.724 |
| flat_aggregation_n8 | groth16 | 3 | vk_read | 0.073 |
| flat_aggregation_n8 | groth16 | 3 | vk_decode | 0.600 |
| flat_aggregation_n8 | groth16 | 3 | vk_prepare | 0.589 |
| flat_aggregation_n8 | groth16 | 3 | proof_encode | 0.003 |
| flat_aggregation_n8 | groth16 | 3 | proof_decode | 0.108 |
| flat_aggregation_n8 | noir_bb | 3 | fixture | 2.887 |
| flat_aggregation_n8 | noir_bb | 3 | artifact_read | 0.044 |
| flat_aggregation_n8 | noir_bb | 3 | artifact_base64 | 0.028 |
| flat_aggregation_n8 | noir_bb | 3 | program_decode | 4.116 |
| flat_aggregation_n8 | noir_bb | 3 | acir_gunzip | 0.702 |
| flat_aggregation_n8 | noir_bb | 3 | vk_read | 0.067 |
| flat_aggregation_n8 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 3 | srs_init | 159.021 |
| flat_aggregation_n8 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n16 | groth16 | 1 | fixture | 5.658 |
| flat_aggregation_n16 | groth16 | 1 | pk_read | 10.226 |
| flat_aggregation_n16 | groth16 | 1 | pk_decode | 18294.321 |
| flat_aggregation_n16 | groth16 | 1 | vk_read | 0.081 |
| flat_aggregation_n16 | groth16 | 1 | vk_decode | 0.544 |
| flat_aggregation_n16 | groth16 | 1 | vk_prepare | 0.541 |
| flat_aggregation_n16 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 1 | proof_decode | 0.103 |
| flat_aggregation_n16 | noir_bb | 1 | fixture | 5.630 |
| flat_aggregation_n16 | noir_bb | 1 | artifact_read | 0.290 |
| flat_aggregation_n16 | noir_bb | 1 | artifact_base64 | 0.051 |
| flat_aggregation_n16 | noir_bb | 1 | program_decode | 7.240 |
| flat_aggregation_n16 | noir_bb | 1 | acir_gunzip | 0.990 |
| flat_aggregation_n16 | noir_bb | 1 | vk_read | 0.216 |
| flat_aggregation_n16 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 1 | srs_init | 152.014 |
| flat_aggregation_n16 | noir_bb | 1 | proof_encode | 0.002 |
| flat_aggregation_n16 | noir_bb | 1 | proof_decode | 0.002 |
| flat_aggregation_n16 | noir_bb | 2 | fixture | 5.704 |
| flat_aggregation_n16 | noir_bb | 2 | artifact_read | 0.071 |
| flat_aggregation_n16 | noir_bb | 2 | artifact_base64 | 0.051 |
| flat_aggregation_n16 | noir_bb | 2 | program_decode | 7.349 |
| flat_aggregation_n16 | noir_bb | 2 | acir_gunzip | 1.096 |
| flat_aggregation_n16 | noir_bb | 2 | vk_read | 0.047 |
| flat_aggregation_n16 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 2 | srs_init | 153.519 |
| flat_aggregation_n16 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n16 | noir_bb | 2 | proof_decode | 0.002 |
| flat_aggregation_n16 | groth16 | 2 | fixture | 5.787 |
| flat_aggregation_n16 | groth16 | 2 | pk_read | 10.589 |
| flat_aggregation_n16 | groth16 | 2 | pk_decode | 18301.752 |
| flat_aggregation_n16 | groth16 | 2 | vk_read | 0.082 |
| flat_aggregation_n16 | groth16 | 2 | vk_decode | 0.603 |
| flat_aggregation_n16 | groth16 | 2 | vk_prepare | 0.603 |
| flat_aggregation_n16 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n16 | groth16 | 3 | fixture | 5.676 |
| flat_aggregation_n16 | groth16 | 3 | pk_read | 15.652 |
| flat_aggregation_n16 | groth16 | 3 | pk_decode | 18358.473 |
| flat_aggregation_n16 | groth16 | 3 | vk_read | 0.081 |
| flat_aggregation_n16 | groth16 | 3 | vk_decode | 0.613 |
| flat_aggregation_n16 | groth16 | 3 | vk_prepare | 0.610 |
| flat_aggregation_n16 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 3 | proof_decode | 0.108 |
| flat_aggregation_n16 | noir_bb | 3 | fixture | 5.697 |
| flat_aggregation_n16 | noir_bb | 3 | artifact_read | 0.394 |
| flat_aggregation_n16 | noir_bb | 3 | artifact_base64 | 0.052 |
| flat_aggregation_n16 | noir_bb | 3 | program_decode | 7.414 |
| flat_aggregation_n16 | noir_bb | 3 | acir_gunzip | 1.047 |
| flat_aggregation_n16 | noir_bb | 3 | vk_read | 0.237 |
| flat_aggregation_n16 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 3 | srs_init | 154.137 |
| flat_aggregation_n16 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n16 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n32 | groth16 | 1 | fixture | 11.379 |
| flat_aggregation_n32 | groth16 | 1 | pk_read | 30.658 |
| flat_aggregation_n32 | groth16 | 1 | pk_decode | 36610.066 |
| flat_aggregation_n32 | groth16 | 1 | vk_read | 0.287 |
| flat_aggregation_n32 | groth16 | 1 | vk_decode | 0.761 |
| flat_aggregation_n32 | groth16 | 1 | vk_prepare | 0.543 |
| flat_aggregation_n32 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 1 | proof_decode | 0.113 |
| flat_aggregation_n32 | noir_bb | 1 | fixture | 11.311 |
| flat_aggregation_n32 | noir_bb | 1 | artifact_read | 0.404 |
| flat_aggregation_n32 | noir_bb | 1 | artifact_base64 | 0.095 |
| flat_aggregation_n32 | noir_bb | 1 | program_decode | 14.124 |
| flat_aggregation_n32 | noir_bb | 1 | acir_gunzip | 2.287 |
| flat_aggregation_n32 | noir_bb | 1 | vk_read | 0.275 |
| flat_aggregation_n32 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 1 | srs_init | 160.652 |
| flat_aggregation_n32 | noir_bb | 1 | proof_encode | 0.003 |
| flat_aggregation_n32 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n32 | noir_bb | 2 | fixture | 11.388 |
| flat_aggregation_n32 | noir_bb | 2 | artifact_read | 0.127 |
| flat_aggregation_n32 | noir_bb | 2 | artifact_base64 | 0.099 |
| flat_aggregation_n32 | noir_bb | 2 | program_decode | 13.600 |
| flat_aggregation_n32 | noir_bb | 2 | acir_gunzip | 2.181 |
| flat_aggregation_n32 | noir_bb | 2 | vk_read | 0.061 |
| flat_aggregation_n32 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 2 | srs_init | 152.425 |
| flat_aggregation_n32 | noir_bb | 2 | proof_encode | 0.002 |
| flat_aggregation_n32 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n32 | groth16 | 2 | fixture | 11.268 |
| flat_aggregation_n32 | groth16 | 2 | pk_read | 30.670 |
| flat_aggregation_n32 | groth16 | 2 | pk_decode | 36631.907 |
| flat_aggregation_n32 | groth16 | 2 | vk_read | 0.086 |
| flat_aggregation_n32 | groth16 | 2 | vk_decode | 0.685 |
| flat_aggregation_n32 | groth16 | 2 | vk_prepare | 0.538 |
| flat_aggregation_n32 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 2 | proof_decode | 0.110 |
| flat_aggregation_n32 | groth16 | 3 | fixture | 12.934 |
| flat_aggregation_n32 | groth16 | 3 | pk_read | 31.088 |
| flat_aggregation_n32 | groth16 | 3 | pk_decode | 36634.501 |
| flat_aggregation_n32 | groth16 | 3 | vk_read | 0.082 |
| flat_aggregation_n32 | groth16 | 3 | vk_decode | 0.720 |
| flat_aggregation_n32 | groth16 | 3 | vk_prepare | 0.542 |
| flat_aggregation_n32 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 3 | proof_decode | 0.103 |
| flat_aggregation_n32 | noir_bb | 3 | fixture | 11.347 |
| flat_aggregation_n32 | noir_bb | 3 | artifact_read | 0.383 |
| flat_aggregation_n32 | noir_bb | 3 | artifact_base64 | 0.102 |
| flat_aggregation_n32 | noir_bb | 3 | program_decode | 14.169 |
| flat_aggregation_n32 | noir_bb | 3 | acir_gunzip | 2.338 |
| flat_aggregation_n32 | noir_bb | 3 | vk_read | 0.295 |
| flat_aggregation_n32 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 3 | srs_init | 157.821 |
| flat_aggregation_n32 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n32 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n64 | groth16 | 1 | fixture | 22.572 |
| flat_aggregation_n64 | groth16 | 1 | pk_read | 60.867 |
| flat_aggregation_n64 | groth16 | 1 | pk_decode | 73314.015 |
| flat_aggregation_n64 | groth16 | 1 | vk_read | 0.313 |
| flat_aggregation_n64 | groth16 | 1 | vk_decode | 1.069 |
| flat_aggregation_n64 | groth16 | 1 | vk_prepare | 0.589 |
| flat_aggregation_n64 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n64 | groth16 | 1 | proof_decode | 0.105 |
| flat_aggregation_n64 | noir_bb | 1 | fixture | 22.461 |
| flat_aggregation_n64 | noir_bb | 1 | artifact_read | 0.487 |
| flat_aggregation_n64 | noir_bb | 1 | artifact_base64 | 0.190 |
| flat_aggregation_n64 | noir_bb | 1 | program_decode | 27.072 |
| flat_aggregation_n64 | noir_bb | 1 | acir_gunzip | 4.537 |
| flat_aggregation_n64 | noir_bb | 1 | vk_read | 0.277 |
| flat_aggregation_n64 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 1 | srs_init | 156.675 |
| flat_aggregation_n64 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n64 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n64 | noir_bb | 2 | fixture | 22.809 |
| flat_aggregation_n64 | noir_bb | 2 | artifact_read | 0.179 |
| flat_aggregation_n64 | noir_bb | 2 | artifact_base64 | 0.192 |
| flat_aggregation_n64 | noir_bb | 2 | program_decode | 27.697 |
| flat_aggregation_n64 | noir_bb | 2 | acir_gunzip | 4.478 |
| flat_aggregation_n64 | noir_bb | 2 | vk_read | 0.070 |
| flat_aggregation_n64 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 2 | srs_init | 155.616 |
| flat_aggregation_n64 | noir_bb | 2 | proof_encode | 0.005 |
| flat_aggregation_n64 | noir_bb | 2 | proof_decode | 0.002 |
| flat_aggregation_n64 | groth16 | 2 | fixture | 22.671 |
| flat_aggregation_n64 | groth16 | 2 | pk_read | 62.957 |
| flat_aggregation_n64 | groth16 | 2 | pk_decode | 72873.738 |
| flat_aggregation_n64 | groth16 | 2 | vk_read | 0.082 |
| flat_aggregation_n64 | groth16 | 2 | vk_decode | 1.015 |
| flat_aggregation_n64 | groth16 | 2 | vk_prepare | 0.542 |
| flat_aggregation_n64 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n64 | groth16 | 2 | proof_decode | 0.107 |
| flat_aggregation_n64 | groth16 | 3 | fixture | 22.501 |
| flat_aggregation_n64 | groth16 | 3 | pk_read | 60.373 |
| flat_aggregation_n64 | groth16 | 3 | pk_decode | 73082.998 |
| flat_aggregation_n64 | groth16 | 3 | vk_read | 0.070 |
| flat_aggregation_n64 | groth16 | 3 | vk_decode | 1.082 |
| flat_aggregation_n64 | groth16 | 3 | vk_prepare | 0.572 |
| flat_aggregation_n64 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n64 | groth16 | 3 | proof_decode | 0.103 |
| flat_aggregation_n64 | noir_bb | 3 | fixture | 22.368 |
| flat_aggregation_n64 | noir_bb | 3 | artifact_read | 0.442 |
| flat_aggregation_n64 | noir_bb | 3 | artifact_base64 | 0.197 |
| flat_aggregation_n64 | noir_bb | 3 | program_decode | 27.001 |
| flat_aggregation_n64 | noir_bb | 3 | acir_gunzip | 4.508 |
| flat_aggregation_n64 | noir_bb | 3 | vk_read | 0.244 |
| flat_aggregation_n64 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 3 | srs_init | 159.382 |
| flat_aggregation_n64 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n64 | noir_bb | 3 | proof_decode | 0.008 |
| emit_mint | groth16 | 1 | fixture | 5.582 |
| emit_mint | groth16 | 1 | pk_read | 0.765 |
| emit_mint | groth16 | 1 | pk_decode | 803.906 |
| emit_mint | groth16 | 1 | vk_read | 2.238 |
| emit_mint | groth16 | 1 | vk_decode | 0.527 |
| emit_mint | groth16 | 1 | vk_prepare | 0.535 |
| emit_mint | groth16 | 1 | proof_encode | 0.002 |
| emit_mint | groth16 | 1 | proof_decode | 0.103 |
| emit_mint | noir_bb | 1 | fixture | 5.663 |
| emit_mint | noir_bb | 1 | artifact_read | 0.281 |
| emit_mint | noir_bb | 1 | artifact_base64 | 0.008 |
| emit_mint | noir_bb | 1 | program_decode | 0.304 |
| emit_mint | noir_bb | 1 | acir_gunzip | 0.048 |
| emit_mint | noir_bb | 1 | vk_read | 0.207 |
| emit_mint | noir_bb | 1 | backend_init | 0.000 |
| emit_mint | noir_bb | 1 | srs_init | 154.807 |
| emit_mint | noir_bb | 1 | proof_encode | 0.001 |
| emit_mint | noir_bb | 1 | proof_decode | 0.004 |
| emit_mint | noir_bb | 2 | fixture | 5.605 |
| emit_mint | noir_bb | 2 | artifact_read | 0.027 |
| emit_mint | noir_bb | 2 | artifact_base64 | 0.007 |
| emit_mint | noir_bb | 2 | program_decode | 0.293 |
| emit_mint | noir_bb | 2 | acir_gunzip | 0.050 |
| emit_mint | noir_bb | 2 | vk_read | 0.017 |
| emit_mint | noir_bb | 2 | backend_init | 0.000 |
| emit_mint | noir_bb | 2 | srs_init | 157.142 |
| emit_mint | noir_bb | 2 | proof_encode | 0.001 |
| emit_mint | noir_bb | 2 | proof_decode | 0.004 |
| emit_mint | groth16 | 2 | fixture | 5.664 |
| emit_mint | groth16 | 2 | pk_read | 0.515 |
| emit_mint | groth16 | 2 | pk_decode | 803.725 |
| emit_mint | groth16 | 2 | vk_read | 0.061 |
| emit_mint | groth16 | 2 | vk_decode | 0.440 |
| emit_mint | groth16 | 2 | vk_prepare | 0.561 |
| emit_mint | groth16 | 2 | proof_encode | 0.002 |
| emit_mint | groth16 | 2 | proof_decode | 0.105 |
| emit_mint | groth16 | 3 | fixture | 5.768 |
| emit_mint | groth16 | 3 | pk_read | 0.482 |
| emit_mint | groth16 | 3 | pk_decode | 801.085 |
| emit_mint | groth16 | 3 | vk_read | 0.073 |
| emit_mint | groth16 | 3 | vk_decode | 0.479 |
| emit_mint | groth16 | 3 | vk_prepare | 0.550 |
| emit_mint | groth16 | 3 | proof_encode | 0.003 |
| emit_mint | groth16 | 3 | proof_decode | 0.104 |
| emit_mint | noir_bb | 3 | fixture | 5.599 |
| emit_mint | noir_bb | 3 | artifact_read | 0.032 |
| emit_mint | noir_bb | 3 | artifact_base64 | 0.005 |
| emit_mint | noir_bb | 3 | program_decode | 0.292 |
| emit_mint | noir_bb | 3 | acir_gunzip | 0.049 |
| emit_mint | noir_bb | 3 | vk_read | 0.025 |
| emit_mint | noir_bb | 3 | backend_init | 0.000 |
| emit_mint | noir_bb | 3 | srs_init | 158.353 |
| emit_mint | noir_bb | 3 | proof_encode | 0.001 |
| emit_mint | noir_bb | 3 | proof_decode | 0.004 |

## Process totals and memory

Measured by Darwin `/usr/bin/time -l` for each fresh worker process; RSS is bytes from the OS, displayed as MiB (1 MiB = 1,048,576 bytes). This is whole-process peak resident memory, including native bb allocations, keys, SRS, witness/matrices and prover/verifier—not serialized sizes, virtual memory, allocator-only usage, or a per-stage peak. It does not identify which phase caused the peak. Sample 0 covers Groth16 setup plus key serialization/writes; samples 1+ cover fixture creation through proof verification and worker exit. Compiler and CLI VK-generation memory is not measured. Process wall time is independently measured with Instant around the parent invocation, including process/timing-wrapper startup, log setup and shutdown; it is not a sum or subtraction of stage timings. The resource collector runs outside the worker's named stage timers. Raw bytes and process times are in `memory.csv`; original OS resource logs are retained under the diagnostics path in metadata.

| Circuit | Backend | Sample | Process wall ms | Peak RSS MiB |
|---|---|---:|---:|---:|
| demo_tribute | groth16 | 0 | 199.826 | 122.141 |
| flat_aggregation_n1 | groth16 | 0 | 121.997 | 74.953 |
| flat_aggregation_n2 | groth16 | 0 | 231.712 | 153.562 |
| flat_aggregation_n4 | groth16 | 0 | 424.766 | 282.062 |
| flat_aggregation_n8 | groth16 | 0 | 877.316 | 557.125 |
| flat_aggregation_n16 | groth16 | 0 | 1815.688 | 1134.062 |
| flat_aggregation_n32 | groth16 | 0 | 3412.702 | 2148.141 |
| flat_aggregation_n64 | groth16 | 0 | 6991.220 | 4203.438 |
| emit_mint | groth16 | 0 | 130.557 | 82.531 |
| demo_tribute | groth16 | 1 | 1717.493 | 155.172 |
| demo_tribute | noir_bb | 1 | 328.525 | 462.281 |
| demo_tribute | noir_bb | 2 | 338.826 | 461.812 |
| demo_tribute | groth16 | 2 | 1747.641 | 153.797 |
| demo_tribute | groth16 | 3 | 1736.004 | 158.062 |
| demo_tribute | noir_bb | 3 | 330.647 | 462.719 |
| flat_aggregation_n1 | groth16 | 1 | 1219.380 | 103.016 |
| flat_aggregation_n1 | noir_bb | 1 | 317.645 | 456.469 |
| flat_aggregation_n1 | noir_bb | 2 | 323.777 | 456.656 |
| flat_aggregation_n1 | groth16 | 2 | 1239.053 | 104.906 |
| flat_aggregation_n1 | groth16 | 3 | 1231.160 | 105.656 |
| flat_aggregation_n1 | noir_bb | 3 | 312.765 | 456.250 |
| flat_aggregation_n2 | groth16 | 1 | 2471.112 | 179.812 |
| flat_aggregation_n2 | noir_bb | 1 | 359.501 | 463.656 |
| flat_aggregation_n2 | noir_bb | 2 | 348.623 | 461.906 |
| flat_aggregation_n2 | groth16 | 2 | 2513.127 | 181.438 |
| flat_aggregation_n2 | groth16 | 3 | 2481.583 | 184.125 |
| flat_aggregation_n2 | noir_bb | 3 | 324.723 | 462.234 |
| flat_aggregation_n4 | groth16 | 1 | 4896.624 | 350.438 |
| flat_aggregation_n4 | noir_bb | 1 | 348.290 | 479.375 |
| flat_aggregation_n4 | noir_bb | 2 | 341.890 | 479.422 |
| flat_aggregation_n4 | groth16 | 2 | 4831.130 | 342.016 |
| flat_aggregation_n4 | groth16 | 3 | 4869.353 | 345.109 |
| flat_aggregation_n4 | noir_bb | 3 | 358.149 | 478.641 |
| flat_aggregation_n8 | groth16 | 1 | 9778.334 | 679.656 |
| flat_aggregation_n8 | noir_bb | 1 | 411.413 | 523.812 |
| flat_aggregation_n8 | noir_bb | 2 | 408.517 | 524.125 |
| flat_aggregation_n8 | groth16 | 2 | 9745.195 | 672.625 |
| flat_aggregation_n8 | groth16 | 3 | 9828.186 | 664.750 |
| flat_aggregation_n8 | noir_bb | 3 | 424.709 | 524.047 |
| flat_aggregation_n16 | groth16 | 1 | 19521.014 | 1321.312 |
| flat_aggregation_n16 | noir_bb | 1 | 567.024 | 586.062 |
| flat_aggregation_n16 | noir_bb | 2 | 574.493 | 629.938 |
| flat_aggregation_n16 | groth16 | 2 | 19612.700 | 1324.078 |
| flat_aggregation_n16 | groth16 | 3 | 19572.662 | 1323.344 |
| flat_aggregation_n16 | noir_bb | 3 | 580.129 | 629.391 |
| flat_aggregation_n32 | groth16 | 1 | 39113.342 | 2499.031 |
| flat_aggregation_n32 | noir_bb | 1 | 927.748 | 739.344 |
| flat_aggregation_n32 | noir_bb | 2 | 889.108 | 734.141 |
| flat_aggregation_n32 | groth16 | 2 | 38964.729 | 2517.016 |
| flat_aggregation_n32 | groth16 | 3 | 39005.940 | 2519.188 |
| flat_aggregation_n32 | noir_bb | 3 | 879.567 | 734.344 |
| flat_aggregation_n64 | groth16 | 1 | 77987.676 | 4366.516 |
| flat_aggregation_n64 | noir_bb | 1 | 1442.835 | 984.469 |
| flat_aggregation_n64 | noir_bb | 2 | 1474.998 | 979.516 |
| flat_aggregation_n64 | groth16 | 2 | 77637.338 | 4736.312 |
| flat_aggregation_n64 | groth16 | 3 | 77727.834 | 4750.016 |
| flat_aggregation_n64 | noir_bb | 3 | 1479.133 | 982.422 |
| emit_mint | groth16 | 1 | 948.647 | 100.188 |
| emit_mint | noir_bb | 1 | 260.654 | 415.875 |
| emit_mint | noir_bb | 2 | 262.615 | 415.828 |
| emit_mint | groth16 | 2 | 955.280 | 95.609 |
| emit_mint | groth16 | 3 | 944.654 | 97.578 |
| emit_mint | noir_bb | 3 | 265.575 | 416.312 |

## Sizes and structural counts

All sizes are bytes. Proof payload excludes public inputs (32 bytes each). Groth16 proof/key encodings are checked compressed arkworks canonical encodings, not EVM calldata. bb proofs are concatenated 32-byte fields and do not include a second copy of public inputs. Groth16 assignment footprint is 32 bytes per field including constant-one/public slots, not a persisted witness-file format. Noir witness size is raw msgpack. R1CS constraints/variables and bb gates/dyadic sizes are different metrics, not equivalent work units.

| Circuit | Backend | Sample | Proof B | Public B | PK B | VK B | Witness msgpack B | Assignment fields | Assignment B | R1CS constraints | Instance vars incl. one | Witness vars | bb gates | bb dyadic |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| demo_tribute | groth16 | 1 | 128 | 128 | 10063376 | 392 | N/A | 49787 | 1593184 | 51026 | 5 | 49782 | N/A | N/A |
| demo_tribute | noir_bb | 1 | 8768 | 128 | N/A | 1888 | 48760 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| demo_tribute | noir_bb | 2 | 8768 | 128 | N/A | 1888 | 48760 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| demo_tribute | groth16 | 2 | 128 | 128 | 10063376 | 392 | N/A | 49787 | 1593184 | 51026 | 5 | 49782 | N/A | N/A |
| demo_tribute | groth16 | 3 | 128 | 128 | 10063376 | 392 | N/A | 49787 | 1593184 | 51026 | 5 | 49782 | N/A | N/A |
| demo_tribute | noir_bb | 3 | 8768 | 128 | N/A | 1888 | 48760 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n1 | groth16 | 1 | 128 | 96 | 7339376 | 360 | N/A | 32762 | 1048384 | 34033 | 4 | 32758 | N/A | N/A |
| flat_aggregation_n1 | noir_bb | 1 | 8768 | 96 | N/A | 1888 | 32110 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n1 | noir_bb | 2 | 8768 | 96 | N/A | 1888 | 32110 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n1 | groth16 | 2 | 128 | 96 | 7339376 | 360 | N/A | 32762 | 1048384 | 34033 | 4 | 32758 | N/A | N/A |
| flat_aggregation_n1 | groth16 | 3 | 128 | 96 | 7339376 | 360 | N/A | 32762 | 1048384 | 34033 | 4 | 32758 | N/A | N/A |
| flat_aggregation_n1 | noir_bb | 3 | 8768 | 96 | N/A | 1888 | 32110 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n2 | groth16 | 1 | 128 | 160 | 14678128 | 424 | N/A | 65522 | 2096704 | 68066 | 6 | 65516 | N/A | N/A |
| flat_aggregation_n2 | noir_bb | 1 | 8768 | 160 | N/A | 1888 | 64115 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n2 | noir_bb | 2 | 8768 | 160 | N/A | 1888 | 64115 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n2 | groth16 | 2 | 128 | 160 | 14678128 | 424 | N/A | 65522 | 2096704 | 68066 | 6 | 65516 | N/A | N/A |
| flat_aggregation_n2 | groth16 | 3 | 128 | 160 | 14678128 | 424 | N/A | 65522 | 2096704 | 68066 | 6 | 65516 | N/A | N/A |
| flat_aggregation_n2 | noir_bb | 3 | 8768 | 160 | N/A | 1888 | 64115 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n4 | groth16 | 1 | 128 | 288 | 29355632 | 552 | N/A | 131042 | 4193344 | 136132 | 10 | 131032 | N/A | N/A |
| flat_aggregation_n4 | noir_bb | 1 | 8768 | 288 | N/A | 1888 | 128125 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n4 | noir_bb | 2 | 8768 | 288 | N/A | 1888 | 128125 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n4 | groth16 | 2 | 128 | 288 | 29355632 | 552 | N/A | 131042 | 4193344 | 136132 | 10 | 131032 | N/A | N/A |
| flat_aggregation_n4 | groth16 | 3 | 128 | 288 | 29355632 | 552 | N/A | 131042 | 4193344 | 136132 | 10 | 131032 | N/A | N/A |
| flat_aggregation_n4 | noir_bb | 3 | 8768 | 288 | N/A | 1888 | 128125 | N/A | N/A | N/A | N/A | N/A | 46088 | 65536 |
| flat_aggregation_n8 | groth16 | 1 | 128 | 544 | 58710640 | 808 | N/A | 262082 | 8386624 | 272264 | 18 | 262064 | N/A | N/A |
| flat_aggregation_n8 | noir_bb | 1 | 8768 | 544 | N/A | 1888 | 256145 | N/A | N/A | N/A | N/A | N/A | 55260 | 65536 |
| flat_aggregation_n8 | noir_bb | 2 | 8768 | 544 | N/A | 1888 | 256145 | N/A | N/A | N/A | N/A | N/A | 55260 | 65536 |
| flat_aggregation_n8 | groth16 | 2 | 128 | 544 | 58710640 | 808 | N/A | 262082 | 8386624 | 272264 | 18 | 262064 | N/A | N/A |
| flat_aggregation_n8 | groth16 | 3 | 128 | 544 | 58710640 | 808 | N/A | 262082 | 8386624 | 272264 | 18 | 262064 | N/A | N/A |
| flat_aggregation_n8 | noir_bb | 3 | 8768 | 544 | N/A | 1888 | 256145 | N/A | N/A | N/A | N/A | N/A | 55260 | 65536 |
| flat_aggregation_n16 | groth16 | 1 | 128 | 1056 | 117420656 | 1320 | N/A | 524162 | 16773184 | 544528 | 34 | 524128 | N/A | N/A |
| flat_aggregation_n16 | noir_bb | 1 | 9152 | 1056 | N/A | 1888 | 512185 | N/A | N/A | N/A | N/A | N/A | 107624 | 131072 |
| flat_aggregation_n16 | noir_bb | 2 | 9152 | 1056 | N/A | 1888 | 512185 | N/A | N/A | N/A | N/A | N/A | 107624 | 131072 |
| flat_aggregation_n16 | groth16 | 2 | 128 | 1056 | 117420656 | 1320 | N/A | 524162 | 16773184 | 544528 | 34 | 524128 | N/A | N/A |
| flat_aggregation_n16 | groth16 | 3 | 128 | 1056 | 117420656 | 1320 | N/A | 524162 | 16773184 | 544528 | 34 | 524128 | N/A | N/A |
| flat_aggregation_n16 | noir_bb | 3 | 9152 | 1056 | N/A | 1888 | 512185 | N/A | N/A | N/A | N/A | N/A | 107624 | 131072 |
| flat_aggregation_n32 | groth16 | 1 | 128 | 2080 | 234840688 | 2344 | N/A | 1048322 | 33546304 | 1089056 | 66 | 1048256 | N/A | N/A |
| flat_aggregation_n32 | noir_bb | 1 | 9536 | 2080 | N/A | 1888 | 1024265 | N/A | N/A | N/A | N/A | N/A | 212352 | 262144 |
| flat_aggregation_n32 | noir_bb | 2 | 9536 | 2080 | N/A | 1888 | 1024265 | N/A | N/A | N/A | N/A | N/A | 212352 | 262144 |
| flat_aggregation_n32 | groth16 | 2 | 128 | 2080 | 234840688 | 2344 | N/A | 1048322 | 33546304 | 1089056 | 66 | 1048256 | N/A | N/A |
| flat_aggregation_n32 | groth16 | 3 | 128 | 2080 | 234840688 | 2344 | N/A | 1048322 | 33546304 | 1089056 | 66 | 1048256 | N/A | N/A |
| flat_aggregation_n32 | noir_bb | 3 | 9536 | 2080 | N/A | 1888 | 1024265 | N/A | N/A | N/A | N/A | N/A | 212352 | 262144 |
| flat_aggregation_n64 | groth16 | 1 | 128 | 4128 | 469680752 | 4392 | N/A | 2096642 | 67092544 | 2178112 | 130 | 2096512 | N/A | N/A |
| flat_aggregation_n64 | noir_bb | 1 | 9920 | 4128 | N/A | 1888 | 2048425 | N/A | N/A | N/A | N/A | N/A | 421808 | 524288 |
| flat_aggregation_n64 | noir_bb | 2 | 9920 | 4128 | N/A | 1888 | 2048425 | N/A | N/A | N/A | N/A | N/A | 421808 | 524288 |
| flat_aggregation_n64 | groth16 | 2 | 128 | 4128 | 469680752 | 4392 | N/A | 2096642 | 67092544 | 2178112 | 130 | 2096512 | N/A | N/A |
| flat_aggregation_n64 | groth16 | 3 | 128 | 4128 | 469680752 | 4392 | N/A | 2096642 | 67092544 | 2178112 | 130 | 2096512 | N/A | N/A |
| flat_aggregation_n64 | noir_bb | 3 | 9920 | 4128 | N/A | 1888 | 2048425 | N/A | N/A | N/A | N/A | N/A | 421808 | 524288 |
| emit_mint | groth16 | 1 | 128 | 256 | 4823280 | 520 | N/A | 23590 | 754880 | 23563 | 9 | 23581 | N/A | N/A |
| emit_mint | noir_bb | 1 | 8000 | 256 | N/A | 1888 | 21380 | N/A | N/A | N/A | N/A | N/A | 9123 | 16384 |
| emit_mint | noir_bb | 2 | 8000 | 256 | N/A | 1888 | 21380 | N/A | N/A | N/A | N/A | N/A | 9123 | 16384 |
| emit_mint | groth16 | 2 | 128 | 256 | 4823280 | 520 | N/A | 23590 | 754880 | 23563 | 9 | 23581 | N/A | N/A |
| emit_mint | groth16 | 3 | 128 | 256 | 4823280 | 520 | N/A | 23590 | 754880 | 23563 | 9 | 23581 | N/A | N/A |
| emit_mint | noir_bb | 3 | 8000 | 256 | N/A | 1888 | 21380 | N/A | N/A | N/A | N/A | N/A | 9123 | 16384 |

Exact nanoseconds are in `timings.csv`; full per-sample fixture/public digests and numeric sizes are in `observations.json`. Hardware, pins, encodings, settings, provenance hashes and stage definitions are in `metadata.json`. No averages, selected best runs, or subtracted cost estimates are reported.
