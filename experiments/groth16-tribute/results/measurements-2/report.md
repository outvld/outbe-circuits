# Groth16 Tribute measurements

Individual wall-clock observations, not a statistical benchmark. Fresh process per backend/case/sample; OS filesystem caches left intact. Verification occurs after proving, not in a standalone cold verifier. All proofs use fresh OS-entropy-seeded blinding. Groth16 setup is local single-party experimental setup, never production ceremony material. Production circuits and backend are unchanged.

## Core timings

Times in milliseconds. Witness means constrained synthesis/finalization/matrix materialization for Groth16 and initial-map construction/ACVM solving/finalization for Noir. Groth16 proving excludes repeated synthesis. N/A is inapplicable, never an unmeasured zero.

| Circuit | Backend | Sample | Witness ms | Witness encode ms | Prove ms | Verify ms |
|---|---|---:|---:|---:|---:|---:|
| demo_tribute | groth16 | 1 | 59.410 | N/A | 103.617 | 0.740 |
| demo_tribute | noir_bb | 1 | 1.652 | 0.720 | 148.138 | 1.141 |
| demo_tribute | noir_bb | 2 | 1.592 | 0.760 | 150.329 | 1.162 |
| demo_tribute | groth16 | 2 | 59.108 | N/A | 103.756 | 0.740 |
| demo_tribute | groth16 | 3 | 59.223 | N/A | 101.354 | 0.733 |
| demo_tribute | noir_bb | 3 | 1.695 | 0.727 | 147.559 | 1.149 |
| flat_aggregation_n1 | groth16 | 1 | 30.514 | N/A | 79.552 | 0.683 |
| flat_aggregation_n1 | noir_bb | 1 | 1.113 | 0.547 | 138.966 | 1.104 |
| flat_aggregation_n1 | noir_bb | 2 | 1.051 | 0.529 | 140.330 | 1.065 |
| flat_aggregation_n1 | groth16 | 2 | 30.456 | N/A | 71.901 | 0.697 |
| flat_aggregation_n1 | groth16 | 3 | 30.986 | N/A | 73.295 | 0.685 |
| flat_aggregation_n1 | noir_bb | 3 | 1.120 | 0.554 | 145.416 | 1.085 |
| flat_aggregation_n2 | groth16 | 1 | 59.842 | N/A | 112.255 | 0.789 |
| flat_aggregation_n2 | noir_bb | 1 | 1.932 | 0.991 | 157.036 | 1.187 |
| flat_aggregation_n2 | noir_bb | 2 | 1.923 | 0.937 | 154.383 | 1.104 |
| flat_aggregation_n2 | groth16 | 2 | 59.282 | N/A | 111.749 | 0.799 |
| flat_aggregation_n2 | groth16 | 3 | 59.668 | N/A | 114.499 | 0.795 |
| flat_aggregation_n2 | noir_bb | 3 | 1.928 | 0.973 | 146.437 | 1.119 |
| flat_aggregation_n4 | groth16 | 1 | 120.256 | N/A | 188.808 | 0.995 |
| flat_aggregation_n4 | noir_bb | 1 | 3.333 | 2.518 | 171.136 | 1.092 |
| flat_aggregation_n4 | noir_bb | 2 | 3.292 | 2.586 | 170.126 | 1.068 |
| flat_aggregation_n4 | groth16 | 2 | 116.310 | N/A | 197.970 | 1.040 |
| flat_aggregation_n4 | groth16 | 3 | 116.431 | N/A | 191.996 | 0.982 |
| flat_aggregation_n4 | noir_bb | 3 | 3.340 | 2.621 | 169.808 | 1.062 |
| flat_aggregation_n8 | groth16 | 1 | 230.385 | N/A | 375.384 | 1.421 |
| flat_aggregation_n8 | noir_bb | 1 | 6.243 | 6.372 | 212.453 | 1.188 |
| flat_aggregation_n8 | noir_bb | 2 | 6.425 | 6.170 | 217.160 | 1.117 |
| flat_aggregation_n8 | groth16 | 2 | 234.123 | N/A | 356.973 | 1.416 |
| flat_aggregation_n8 | groth16 | 3 | 230.023 | N/A | 360.086 | 1.392 |
| flat_aggregation_n8 | noir_bb | 3 | 6.369 | 6.222 | 222.277 | 1.090 |
| flat_aggregation_n16 | groth16 | 1 | 458.395 | N/A | 629.258 | 2.219 |
| flat_aggregation_n16 | noir_bb | 1 | 12.081 | 15.596 | 345.008 | 1.119 |
| flat_aggregation_n16 | noir_bb | 2 | 12.347 | 15.686 | 343.382 | 1.093 |
| flat_aggregation_n16 | groth16 | 2 | 457.521 | N/A | 630.503 | 2.196 |
| flat_aggregation_n16 | groth16 | 3 | 457.445 | N/A | 640.812 | 2.195 |
| flat_aggregation_n16 | noir_bb | 3 | 12.293 | 15.620 | 347.628 | 1.145 |
| flat_aggregation_n32 | groth16 | 1 | 908.450 | N/A | 1235.987 | 3.829 |
| flat_aggregation_n32 | noir_bb | 1 | 23.107 | 36.489 | 594.346 | 1.262 |
| flat_aggregation_n32 | noir_bb | 2 | 23.373 | 36.597 | 596.412 | 1.165 |
| flat_aggregation_n32 | groth16 | 2 | 913.328 | N/A | 1219.640 | 3.835 |
| flat_aggregation_n32 | groth16 | 3 | 905.371 | N/A | 1220.051 | 3.905 |
| flat_aggregation_n32 | noir_bb | 3 | 23.189 | 36.453 | 589.444 | 1.152 |
| flat_aggregation_n64 | groth16 | 1 | 1811.341 | N/A | 2355.927 | 7.094 |
| flat_aggregation_n64 | noir_bb | 1 | 46.082 | 82.323 | 1066.429 | 1.235 |
| flat_aggregation_n64 | noir_bb | 2 | 45.318 | 82.395 | 1068.227 | 1.248 |
| flat_aggregation_n64 | groth16 | 2 | 1805.322 | N/A | 2465.801 | 7.058 |
| flat_aggregation_n64 | groth16 | 3 | 1799.540 | N/A | 2356.749 | 7.039 |
| flat_aggregation_n64 | noir_bb | 3 | 45.480 | 82.464 | 1071.829 | 1.258 |
| emit_mint | groth16 | 1 | 38.895 | N/A | 76.635 | 0.766 |
| emit_mint | noir_bb | 1 | 1.089 | 0.312 | 78.347 | 1.108 |
| emit_mint | noir_bb | 2 | 1.105 | 0.323 | 83.215 | 1.093 |
| emit_mint | groth16 | 2 | 39.837 | N/A | 75.152 | 0.775 |
| emit_mint | groth16 | 3 | 39.510 | N/A | 74.014 | 0.782 |
| emit_mint | noir_bb | 3 | 1.067 | 0.298 | 75.068 | 1.139 |

Emit rows use frozen `emit_mint@1.5.0` and partial-mint fixtures with wide amounts, cross-limb borrowing, and a real index-5 note inclusion path. Full mints and amount/address boundaries are checked separately. The port mirrors the pinned `noir-bignum v0.10.0` statement, including its `[120,120,17]`-bit ABI limb bounds and subtraction-flag constraints; it does not silently add a stricter uint256 gate or repair the production circuit.

## Technical timings

One row per observation, in milliseconds. Sample 0 is one-time preparation (one setup per shape), not a per-proof sample. Compilation and VK derivation include subprocess overhead. Groth16 PK/VK writes include compressed serialization plus disk write; checked key decoding is separate from disk reads. bb has no stored proving key or separate VK preparation API: PK read/decode and VK decode/prepare are N/A; native VK deserialization/preprocessing stays inside FFI verify. `srs_init` loads/checksums and initializes the maximum 1,048,577-point prefix, not a circuit-minimal SRS. Requested thread counts are in metadata, not a claim of measured effective utilization.

| Circuit | Backend | Sample | Stage | Elapsed ms |
|---|---|---:|---|---:|
| demo_tribute | noir_bb | 0 | compile_subprocess | 106.659 |
| demo_tribute | noir_bb | 0 | vk_derive_subprocess | 81.428 |
| demo_tribute | noir_bb | 0 | circuit_stats | 20.452 |
| flat_aggregation_n1 | noir_bb | 0 | compile_subprocess | 104.962 |
| flat_aggregation_n1 | noir_bb | 0 | vk_derive_subprocess | 73.954 |
| flat_aggregation_n1 | noir_bb | 0 | circuit_stats | 5.135 |
| flat_aggregation_n2 | noir_bb | 0 | compile_subprocess | 115.314 |
| flat_aggregation_n2 | noir_bb | 0 | vk_derive_subprocess | 83.247 |
| flat_aggregation_n2 | noir_bb | 0 | circuit_stats | 7.219 |
| flat_aggregation_n4 | noir_bb | 0 | compile_subprocess | 139.237 |
| flat_aggregation_n4 | noir_bb | 0 | vk_derive_subprocess | 108.379 |
| flat_aggregation_n4 | noir_bb | 0 | circuit_stats | 13.245 |
| flat_aggregation_n8 | noir_bb | 0 | compile_subprocess | 181.094 |
| flat_aggregation_n8 | noir_bb | 0 | vk_derive_subprocess | 127.594 |
| flat_aggregation_n8 | noir_bb | 0 | circuit_stats | 23.190 |
| flat_aggregation_n16 | noir_bb | 0 | compile_subprocess | 292.448 |
| flat_aggregation_n16 | noir_bb | 0 | vk_derive_subprocess | 185.756 |
| flat_aggregation_n16 | noir_bb | 0 | circuit_stats | 44.298 |
| flat_aggregation_n32 | noir_bb | 0 | compile_subprocess | 451.882 |
| flat_aggregation_n32 | noir_bb | 0 | vk_derive_subprocess | 307.213 |
| flat_aggregation_n32 | noir_bb | 0 | circuit_stats | 87.737 |
| flat_aggregation_n64 | noir_bb | 0 | compile_subprocess | 861.792 |
| flat_aggregation_n64 | noir_bb | 0 | vk_derive_subprocess | 542.090 |
| flat_aggregation_n64 | noir_bb | 0 | circuit_stats | 181.019 |
| emit_mint | noir_bb | 0 | compile_subprocess | 261.283 |
| emit_mint | noir_bb | 0 | vk_derive_subprocess | 29.130 |
| emit_mint | noir_bb | 0 | circuit_stats | 1.731 |
| demo_tribute | groth16 | 0 | setup | 177.829 |
| demo_tribute | groth16 | 0 | pk_write | 7.309 |
| demo_tribute | groth16 | 0 | vk_write | 0.095 |
| flat_aggregation_n1 | groth16 | 0 | setup | 106.200 |
| flat_aggregation_n1 | groth16 | 0 | pk_write | 5.205 |
| flat_aggregation_n1 | groth16 | 0 | vk_write | 0.095 |
| flat_aggregation_n2 | groth16 | 0 | setup | 204.511 |
| flat_aggregation_n2 | groth16 | 0 | pk_write | 10.330 |
| flat_aggregation_n2 | groth16 | 0 | vk_write | 0.118 |
| flat_aggregation_n4 | groth16 | 0 | setup | 378.515 |
| flat_aggregation_n4 | groth16 | 0 | pk_write | 20.599 |
| flat_aggregation_n4 | groth16 | 0 | vk_write | 0.127 |
| flat_aggregation_n8 | groth16 | 0 | setup | 744.295 |
| flat_aggregation_n8 | groth16 | 0 | pk_write | 42.948 |
| flat_aggregation_n8 | groth16 | 0 | vk_write | 0.121 |
| flat_aggregation_n16 | groth16 | 0 | setup | 1457.874 |
| flat_aggregation_n16 | groth16 | 0 | pk_write | 99.387 |
| flat_aggregation_n16 | groth16 | 0 | vk_write | 0.142 |
| flat_aggregation_n32 | groth16 | 0 | setup | 2891.671 |
| flat_aggregation_n32 | groth16 | 0 | pk_write | 224.077 |
| flat_aggregation_n32 | groth16 | 0 | vk_write | 0.185 |
| flat_aggregation_n64 | groth16 | 0 | setup | 5869.431 |
| flat_aggregation_n64 | groth16 | 0 | pk_write | 698.059 |
| flat_aggregation_n64 | groth16 | 0 | vk_write | 0.150 |
| emit_mint | groth16 | 0 | setup | 103.939 |
| emit_mint | groth16 | 0 | pk_write | 9.079 |
| emit_mint | groth16 | 0 | vk_write | 0.087 |
| demo_tribute | groth16 | 1 | fixture | 7.545 |
| demo_tribute | groth16 | 1 | pk_read | 0.847 |
| demo_tribute | groth16 | 1 | pk_decode | 1546.585 |
| demo_tribute | groth16 | 1 | vk_read | 0.058 |
| demo_tribute | groth16 | 1 | vk_decode | 0.451 |
| demo_tribute | groth16 | 1 | vk_prepare | 0.531 |
| demo_tribute | groth16 | 1 | proof_encode | 0.002 |
| demo_tribute | groth16 | 1 | proof_decode | 0.105 |
| demo_tribute | noir_bb | 1 | fixture | 7.591 |
| demo_tribute | noir_bb | 1 | artifact_read | 0.034 |
| demo_tribute | noir_bb | 1 | artifact_base64 | 0.009 |
| demo_tribute | noir_bb | 1 | program_decode | 0.734 |
| demo_tribute | noir_bb | 1 | acir_gunzip | 0.099 |
| demo_tribute | noir_bb | 1 | vk_read | 0.018 |
| demo_tribute | noir_bb | 1 | backend_init | 0.000 |
| demo_tribute | noir_bb | 1 | srs_init | 146.160 |
| demo_tribute | noir_bb | 1 | proof_encode | 0.001 |
| demo_tribute | noir_bb | 1 | proof_decode | 0.005 |
| demo_tribute | noir_bb | 2 | fixture | 7.624 |
| demo_tribute | noir_bb | 2 | artifact_read | 0.035 |
| demo_tribute | noir_bb | 2 | artifact_base64 | 0.009 |
| demo_tribute | noir_bb | 2 | program_decode | 0.720 |
| demo_tribute | noir_bb | 2 | acir_gunzip | 0.089 |
| demo_tribute | noir_bb | 2 | vk_read | 0.017 |
| demo_tribute | noir_bb | 2 | backend_init | 0.000 |
| demo_tribute | noir_bb | 2 | srs_init | 145.185 |
| demo_tribute | noir_bb | 2 | proof_encode | 0.002 |
| demo_tribute | noir_bb | 2 | proof_decode | 0.004 |
| demo_tribute | groth16 | 2 | fixture | 7.641 |
| demo_tribute | groth16 | 2 | pk_read | 0.885 |
| demo_tribute | groth16 | 2 | pk_decode | 1562.132 |
| demo_tribute | groth16 | 2 | vk_read | 0.058 |
| demo_tribute | groth16 | 2 | vk_decode | 0.430 |
| demo_tribute | groth16 | 2 | vk_prepare | 0.527 |
| demo_tribute | groth16 | 2 | proof_encode | 0.002 |
| demo_tribute | groth16 | 2 | proof_decode | 0.108 |
| demo_tribute | groth16 | 3 | fixture | 7.624 |
| demo_tribute | groth16 | 3 | pk_read | 0.979 |
| demo_tribute | groth16 | 3 | pk_decode | 1558.047 |
| demo_tribute | groth16 | 3 | vk_read | 0.069 |
| demo_tribute | groth16 | 3 | vk_decode | 0.434 |
| demo_tribute | groth16 | 3 | vk_prepare | 0.544 |
| demo_tribute | groth16 | 3 | proof_encode | 0.002 |
| demo_tribute | groth16 | 3 | proof_decode | 0.121 |
| demo_tribute | noir_bb | 3 | fixture | 7.624 |
| demo_tribute | noir_bb | 3 | artifact_read | 0.042 |
| demo_tribute | noir_bb | 3 | artifact_base64 | 0.009 |
| demo_tribute | noir_bb | 3 | program_decode | 0.737 |
| demo_tribute | noir_bb | 3 | acir_gunzip | 0.107 |
| demo_tribute | noir_bb | 3 | vk_read | 0.024 |
| demo_tribute | noir_bb | 3 | backend_init | 0.000 |
| demo_tribute | noir_bb | 3 | srs_init | 145.629 |
| demo_tribute | noir_bb | 3 | proof_encode | 0.003 |
| demo_tribute | noir_bb | 3 | proof_decode | 0.007 |
| flat_aggregation_n1 | groth16 | 1 | fixture | 0.386 |
| flat_aggregation_n1 | groth16 | 1 | pk_read | 0.645 |
| flat_aggregation_n1 | groth16 | 1 | pk_decode | 1118.993 |
| flat_aggregation_n1 | groth16 | 1 | vk_read | 0.061 |
| flat_aggregation_n1 | groth16 | 1 | vk_decode | 0.431 |
| flat_aggregation_n1 | groth16 | 1 | vk_prepare | 0.539 |
| flat_aggregation_n1 | groth16 | 1 | proof_encode | 0.001 |
| flat_aggregation_n1 | groth16 | 1 | proof_decode | 0.104 |
| flat_aggregation_n1 | noir_bb | 1 | fixture | 0.386 |
| flat_aggregation_n1 | noir_bb | 1 | artifact_read | 0.027 |
| flat_aggregation_n1 | noir_bb | 1 | artifact_base64 | 0.008 |
| flat_aggregation_n1 | noir_bb | 1 | program_decode | 0.629 |
| flat_aggregation_n1 | noir_bb | 1 | acir_gunzip | 0.082 |
| flat_aggregation_n1 | noir_bb | 1 | vk_read | 0.018 |
| flat_aggregation_n1 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 1 | srs_init | 147.852 |
| flat_aggregation_n1 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 1 | proof_decode | 0.006 |
| flat_aggregation_n1 | noir_bb | 2 | fixture | 0.380 |
| flat_aggregation_n1 | noir_bb | 2 | artifact_read | 0.029 |
| flat_aggregation_n1 | noir_bb | 2 | artifact_base64 | 0.008 |
| flat_aggregation_n1 | noir_bb | 2 | program_decode | 0.615 |
| flat_aggregation_n1 | noir_bb | 2 | acir_gunzip | 0.083 |
| flat_aggregation_n1 | noir_bb | 2 | vk_read | 0.018 |
| flat_aggregation_n1 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 2 | srs_init | 146.853 |
| flat_aggregation_n1 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 2 | proof_decode | 0.004 |
| flat_aggregation_n1 | groth16 | 2 | fixture | 0.386 |
| flat_aggregation_n1 | groth16 | 2 | pk_read | 0.631 |
| flat_aggregation_n1 | groth16 | 2 | pk_decode | 1120.195 |
| flat_aggregation_n1 | groth16 | 2 | vk_read | 0.062 |
| flat_aggregation_n1 | groth16 | 2 | vk_decode | 0.444 |
| flat_aggregation_n1 | groth16 | 2 | vk_prepare | 0.543 |
| flat_aggregation_n1 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 2 | proof_decode | 0.104 |
| flat_aggregation_n1 | groth16 | 3 | fixture | 0.388 |
| flat_aggregation_n1 | groth16 | 3 | pk_read | 0.671 |
| flat_aggregation_n1 | groth16 | 3 | pk_decode | 1119.989 |
| flat_aggregation_n1 | groth16 | 3 | vk_read | 0.063 |
| flat_aggregation_n1 | groth16 | 3 | vk_decode | 0.408 |
| flat_aggregation_n1 | groth16 | 3 | vk_prepare | 0.530 |
| flat_aggregation_n1 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 3 | proof_decode | 0.103 |
| flat_aggregation_n1 | noir_bb | 3 | fixture | 0.384 |
| flat_aggregation_n1 | noir_bb | 3 | artifact_read | 0.030 |
| flat_aggregation_n1 | noir_bb | 3 | artifact_base64 | 0.008 |
| flat_aggregation_n1 | noir_bb | 3 | program_decode | 0.623 |
| flat_aggregation_n1 | noir_bb | 3 | acir_gunzip | 0.082 |
| flat_aggregation_n1 | noir_bb | 3 | vk_read | 0.017 |
| flat_aggregation_n1 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 3 | srs_init | 146.395 |
| flat_aggregation_n1 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n2 | groth16 | 1 | fixture | 0.741 |
| flat_aggregation_n2 | groth16 | 1 | pk_read | 1.301 |
| flat_aggregation_n2 | groth16 | 1 | pk_decode | 2231.247 |
| flat_aggregation_n2 | groth16 | 1 | vk_read | 0.067 |
| flat_aggregation_n2 | groth16 | 1 | vk_decode | 0.461 |
| flat_aggregation_n2 | groth16 | 1 | vk_prepare | 0.541 |
| flat_aggregation_n2 | groth16 | 1 | proof_encode | 0.001 |
| flat_aggregation_n2 | groth16 | 1 | proof_decode | 0.106 |
| flat_aggregation_n2 | noir_bb | 1 | fixture | 0.742 |
| flat_aggregation_n2 | noir_bb | 1 | artifact_read | 0.032 |
| flat_aggregation_n2 | noir_bb | 1 | artifact_base64 | 0.011 |
| flat_aggregation_n2 | noir_bb | 1 | program_decode | 1.115 |
| flat_aggregation_n2 | noir_bb | 1 | acir_gunzip | 0.137 |
| flat_aggregation_n2 | noir_bb | 1 | vk_read | 0.015 |
| flat_aggregation_n2 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 1 | srs_init | 148.446 |
| flat_aggregation_n2 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 1 | proof_decode | 0.008 |
| flat_aggregation_n2 | noir_bb | 2 | fixture | 0.738 |
| flat_aggregation_n2 | noir_bb | 2 | artifact_read | 0.024 |
| flat_aggregation_n2 | noir_bb | 2 | artifact_base64 | 0.009 |
| flat_aggregation_n2 | noir_bb | 2 | program_decode | 1.083 |
| flat_aggregation_n2 | noir_bb | 2 | acir_gunzip | 0.137 |
| flat_aggregation_n2 | noir_bb | 2 | vk_read | 0.017 |
| flat_aggregation_n2 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 2 | srs_init | 146.421 |
| flat_aggregation_n2 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 2 | proof_decode | 0.007 |
| flat_aggregation_n2 | groth16 | 2 | fixture | 0.741 |
| flat_aggregation_n2 | groth16 | 2 | pk_read | 1.295 |
| flat_aggregation_n2 | groth16 | 2 | pk_decode | 2237.987 |
| flat_aggregation_n2 | groth16 | 2 | vk_read | 0.059 |
| flat_aggregation_n2 | groth16 | 2 | vk_decode | 0.490 |
| flat_aggregation_n2 | groth16 | 2 | vk_prepare | 0.543 |
| flat_aggregation_n2 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n2 | groth16 | 3 | fixture | 0.726 |
| flat_aggregation_n2 | groth16 | 3 | pk_read | 1.348 |
| flat_aggregation_n2 | groth16 | 3 | pk_decode | 2240.355 |
| flat_aggregation_n2 | groth16 | 3 | vk_read | 0.065 |
| flat_aggregation_n2 | groth16 | 3 | vk_decode | 0.425 |
| flat_aggregation_n2 | groth16 | 3 | vk_prepare | 0.530 |
| flat_aggregation_n2 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n2 | noir_bb | 3 | fixture | 0.731 |
| flat_aggregation_n2 | noir_bb | 3 | artifact_read | 0.025 |
| flat_aggregation_n2 | noir_bb | 3 | artifact_base64 | 0.010 |
| flat_aggregation_n2 | noir_bb | 3 | program_decode | 1.142 |
| flat_aggregation_n2 | noir_bb | 3 | acir_gunzip | 0.147 |
| flat_aggregation_n2 | noir_bb | 3 | vk_read | 0.019 |
| flat_aggregation_n2 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 3 | srs_init | 146.364 |
| flat_aggregation_n2 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 3 | proof_decode | 0.004 |
| flat_aggregation_n4 | groth16 | 1 | fixture | 1.425 |
| flat_aggregation_n4 | groth16 | 1 | pk_read | 2.727 |
| flat_aggregation_n4 | groth16 | 1 | pk_decode | 4486.899 |
| flat_aggregation_n4 | groth16 | 1 | vk_read | 0.068 |
| flat_aggregation_n4 | groth16 | 1 | vk_decode | 0.484 |
| flat_aggregation_n4 | groth16 | 1 | vk_prepare | 0.535 |
| flat_aggregation_n4 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 1 | proof_decode | 0.105 |
| flat_aggregation_n4 | noir_bb | 1 | fixture | 1.429 |
| flat_aggregation_n4 | noir_bb | 1 | artifact_read | 0.026 |
| flat_aggregation_n4 | noir_bb | 1 | artifact_base64 | 0.022 |
| flat_aggregation_n4 | noir_bb | 1 | program_decode | 1.953 |
| flat_aggregation_n4 | noir_bb | 1 | acir_gunzip | 0.248 |
| flat_aggregation_n4 | noir_bb | 1 | vk_read | 0.015 |
| flat_aggregation_n4 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 1 | srs_init | 148.582 |
| flat_aggregation_n4 | noir_bb | 1 | proof_encode | 0.003 |
| flat_aggregation_n4 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n4 | noir_bb | 2 | fixture | 1.440 |
| flat_aggregation_n4 | noir_bb | 2 | artifact_read | 0.027 |
| flat_aggregation_n4 | noir_bb | 2 | artifact_base64 | 0.016 |
| flat_aggregation_n4 | noir_bb | 2 | program_decode | 1.976 |
| flat_aggregation_n4 | noir_bb | 2 | acir_gunzip | 0.258 |
| flat_aggregation_n4 | noir_bb | 2 | vk_read | 0.024 |
| flat_aggregation_n4 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 2 | srs_init | 147.477 |
| flat_aggregation_n4 | noir_bb | 2 | proof_encode | 0.003 |
| flat_aggregation_n4 | noir_bb | 2 | proof_decode | 0.004 |
| flat_aggregation_n4 | groth16 | 2 | fixture | 1.434 |
| flat_aggregation_n4 | groth16 | 2 | pk_read | 2.815 |
| flat_aggregation_n4 | groth16 | 2 | pk_decode | 4486.494 |
| flat_aggregation_n4 | groth16 | 2 | vk_read | 0.068 |
| flat_aggregation_n4 | groth16 | 2 | vk_decode | 0.523 |
| flat_aggregation_n4 | groth16 | 2 | vk_prepare | 0.542 |
| flat_aggregation_n4 | groth16 | 2 | proof_encode | 0.003 |
| flat_aggregation_n4 | groth16 | 2 | proof_decode | 0.110 |
| flat_aggregation_n4 | groth16 | 3 | fixture | 1.428 |
| flat_aggregation_n4 | groth16 | 3 | pk_read | 2.688 |
| flat_aggregation_n4 | groth16 | 3 | pk_decode | 4499.979 |
| flat_aggregation_n4 | groth16 | 3 | vk_read | 0.057 |
| flat_aggregation_n4 | groth16 | 3 | vk_decode | 0.494 |
| flat_aggregation_n4 | groth16 | 3 | vk_prepare | 0.547 |
| flat_aggregation_n4 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n4 | noir_bb | 3 | fixture | 1.436 |
| flat_aggregation_n4 | noir_bb | 3 | artifact_read | 0.029 |
| flat_aggregation_n4 | noir_bb | 3 | artifact_base64 | 0.016 |
| flat_aggregation_n4 | noir_bb | 3 | program_decode | 2.018 |
| flat_aggregation_n4 | noir_bb | 3 | acir_gunzip | 0.248 |
| flat_aggregation_n4 | noir_bb | 3 | vk_read | 0.017 |
| flat_aggregation_n4 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 3 | srs_init | 147.758 |
| flat_aggregation_n4 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n4 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n8 | groth16 | 1 | fixture | 2.841 |
| flat_aggregation_n8 | groth16 | 1 | pk_read | 5.259 |
| flat_aggregation_n8 | groth16 | 1 | pk_decode | 8970.977 |
| flat_aggregation_n8 | groth16 | 1 | vk_read | 0.074 |
| flat_aggregation_n8 | groth16 | 1 | vk_decode | 0.492 |
| flat_aggregation_n8 | groth16 | 1 | vk_prepare | 0.536 |
| flat_aggregation_n8 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 1 | proof_decode | 0.108 |
| flat_aggregation_n8 | noir_bb | 1 | fixture | 2.835 |
| flat_aggregation_n8 | noir_bb | 1 | artifact_read | 0.031 |
| flat_aggregation_n8 | noir_bb | 1 | artifact_base64 | 0.028 |
| flat_aggregation_n8 | noir_bb | 1 | program_decode | 3.888 |
| flat_aggregation_n8 | noir_bb | 1 | acir_gunzip | 0.585 |
| flat_aggregation_n8 | noir_bb | 1 | vk_read | 0.041 |
| flat_aggregation_n8 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 1 | srs_init | 148.556 |
| flat_aggregation_n8 | noir_bb | 1 | proof_encode | 0.002 |
| flat_aggregation_n8 | noir_bb | 1 | proof_decode | 0.002 |
| flat_aggregation_n8 | noir_bb | 2 | fixture | 2.824 |
| flat_aggregation_n8 | noir_bb | 2 | artifact_read | 0.038 |
| flat_aggregation_n8 | noir_bb | 2 | artifact_base64 | 0.029 |
| flat_aggregation_n8 | noir_bb | 2 | program_decode | 3.788 |
| flat_aggregation_n8 | noir_bb | 2 | acir_gunzip | 0.600 |
| flat_aggregation_n8 | noir_bb | 2 | vk_read | 0.018 |
| flat_aggregation_n8 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 2 | srs_init | 147.229 |
| flat_aggregation_n8 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n8 | groth16 | 2 | fixture | 2.840 |
| flat_aggregation_n8 | groth16 | 2 | pk_read | 5.242 |
| flat_aggregation_n8 | groth16 | 2 | pk_decode | 8991.657 |
| flat_aggregation_n8 | groth16 | 2 | vk_read | 0.066 |
| flat_aggregation_n8 | groth16 | 2 | vk_decode | 0.512 |
| flat_aggregation_n8 | groth16 | 2 | vk_prepare | 0.540 |
| flat_aggregation_n8 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 2 | proof_decode | 0.108 |
| flat_aggregation_n8 | groth16 | 3 | fixture | 2.841 |
| flat_aggregation_n8 | groth16 | 3 | pk_read | 5.054 |
| flat_aggregation_n8 | groth16 | 3 | pk_decode | 8976.457 |
| flat_aggregation_n8 | groth16 | 3 | vk_read | 0.076 |
| flat_aggregation_n8 | groth16 | 3 | vk_decode | 0.473 |
| flat_aggregation_n8 | groth16 | 3 | vk_prepare | 0.544 |
| flat_aggregation_n8 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 3 | proof_decode | 0.106 |
| flat_aggregation_n8 | noir_bb | 3 | fixture | 2.835 |
| flat_aggregation_n8 | noir_bb | 3 | artifact_read | 0.031 |
| flat_aggregation_n8 | noir_bb | 3 | artifact_base64 | 0.028 |
| flat_aggregation_n8 | noir_bb | 3 | program_decode | 3.881 |
| flat_aggregation_n8 | noir_bb | 3 | acir_gunzip | 0.577 |
| flat_aggregation_n8 | noir_bb | 3 | vk_read | 0.023 |
| flat_aggregation_n8 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 3 | srs_init | 147.927 |
| flat_aggregation_n8 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n16 | groth16 | 1 | fixture | 5.642 |
| flat_aggregation_n16 | groth16 | 1 | pk_read | 10.261 |
| flat_aggregation_n16 | groth16 | 1 | pk_decode | 17926.005 |
| flat_aggregation_n16 | groth16 | 1 | vk_read | 0.066 |
| flat_aggregation_n16 | groth16 | 1 | vk_decode | 0.624 |
| flat_aggregation_n16 | groth16 | 1 | vk_prepare | 0.554 |
| flat_aggregation_n16 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 1 | proof_decode | 0.118 |
| flat_aggregation_n16 | noir_bb | 1 | fixture | 5.669 |
| flat_aggregation_n16 | noir_bb | 1 | artifact_read | 0.059 |
| flat_aggregation_n16 | noir_bb | 1 | artifact_base64 | 0.052 |
| flat_aggregation_n16 | noir_bb | 1 | program_decode | 7.227 |
| flat_aggregation_n16 | noir_bb | 1 | acir_gunzip | 1.135 |
| flat_aggregation_n16 | noir_bb | 1 | vk_read | 0.031 |
| flat_aggregation_n16 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 1 | srs_init | 148.211 |
| flat_aggregation_n16 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n16 | noir_bb | 1 | proof_decode | 0.006 |
| flat_aggregation_n16 | noir_bb | 2 | fixture | 5.690 |
| flat_aggregation_n16 | noir_bb | 2 | artifact_read | 0.052 |
| flat_aggregation_n16 | noir_bb | 2 | artifact_base64 | 0.054 |
| flat_aggregation_n16 | noir_bb | 2 | program_decode | 7.314 |
| flat_aggregation_n16 | noir_bb | 2 | acir_gunzip | 1.138 |
| flat_aggregation_n16 | noir_bb | 2 | vk_read | 0.060 |
| flat_aggregation_n16 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 2 | srs_init | 151.196 |
| flat_aggregation_n16 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n16 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n16 | groth16 | 2 | fixture | 5.738 |
| flat_aggregation_n16 | groth16 | 2 | pk_read | 10.160 |
| flat_aggregation_n16 | groth16 | 2 | pk_decode | 18006.706 |
| flat_aggregation_n16 | groth16 | 2 | vk_read | 0.066 |
| flat_aggregation_n16 | groth16 | 2 | vk_decode | 0.535 |
| flat_aggregation_n16 | groth16 | 2 | vk_prepare | 0.531 |
| flat_aggregation_n16 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 2 | proof_decode | 0.108 |
| flat_aggregation_n16 | groth16 | 3 | fixture | 5.631 |
| flat_aggregation_n16 | groth16 | 3 | pk_read | 9.997 |
| flat_aggregation_n16 | groth16 | 3 | pk_decode | 17914.706 |
| flat_aggregation_n16 | groth16 | 3 | vk_read | 0.079 |
| flat_aggregation_n16 | groth16 | 3 | vk_decode | 0.593 |
| flat_aggregation_n16 | groth16 | 3 | vk_prepare | 0.555 |
| flat_aggregation_n16 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n16 | noir_bb | 3 | fixture | 5.637 |
| flat_aggregation_n16 | noir_bb | 3 | artifact_read | 0.070 |
| flat_aggregation_n16 | noir_bb | 3 | artifact_base64 | 0.066 |
| flat_aggregation_n16 | noir_bb | 3 | program_decode | 7.158 |
| flat_aggregation_n16 | noir_bb | 3 | acir_gunzip | 1.072 |
| flat_aggregation_n16 | noir_bb | 3 | vk_read | 0.042 |
| flat_aggregation_n16 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 3 | srs_init | 149.735 |
| flat_aggregation_n16 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n16 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n32 | groth16 | 1 | fixture | 11.261 |
| flat_aggregation_n32 | groth16 | 1 | pk_read | 19.968 |
| flat_aggregation_n32 | groth16 | 1 | pk_decode | 35768.635 |
| flat_aggregation_n32 | groth16 | 1 | vk_read | 0.075 |
| flat_aggregation_n32 | groth16 | 1 | vk_decode | 0.739 |
| flat_aggregation_n32 | groth16 | 1 | vk_prepare | 0.545 |
| flat_aggregation_n32 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 1 | proof_decode | 0.108 |
| flat_aggregation_n32 | noir_bb | 1 | fixture | 11.225 |
| flat_aggregation_n32 | noir_bb | 1 | artifact_read | 0.099 |
| flat_aggregation_n32 | noir_bb | 1 | artifact_base64 | 0.101 |
| flat_aggregation_n32 | noir_bb | 1 | program_decode | 13.765 |
| flat_aggregation_n32 | noir_bb | 1 | acir_gunzip | 2.299 |
| flat_aggregation_n32 | noir_bb | 1 | vk_read | 0.065 |
| flat_aggregation_n32 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 1 | srs_init | 147.297 |
| flat_aggregation_n32 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n32 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n32 | noir_bb | 2 | fixture | 11.248 |
| flat_aggregation_n32 | noir_bb | 2 | artifact_read | 0.089 |
| flat_aggregation_n32 | noir_bb | 2 | artifact_base64 | 0.098 |
| flat_aggregation_n32 | noir_bb | 2 | program_decode | 13.930 |
| flat_aggregation_n32 | noir_bb | 2 | acir_gunzip | 2.284 |
| flat_aggregation_n32 | noir_bb | 2 | vk_read | 0.066 |
| flat_aggregation_n32 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 2 | srs_init | 146.833 |
| flat_aggregation_n32 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n32 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n32 | groth16 | 2 | fixture | 11.270 |
| flat_aggregation_n32 | groth16 | 2 | pk_read | 20.347 |
| flat_aggregation_n32 | groth16 | 2 | pk_decode | 35771.192 |
| flat_aggregation_n32 | groth16 | 2 | vk_read | 0.068 |
| flat_aggregation_n32 | groth16 | 2 | vk_decode | 0.729 |
| flat_aggregation_n32 | groth16 | 2 | vk_prepare | 0.534 |
| flat_aggregation_n32 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n32 | groth16 | 3 | fixture | 11.286 |
| flat_aggregation_n32 | groth16 | 3 | pk_read | 19.742 |
| flat_aggregation_n32 | groth16 | 3 | pk_decode | 35740.929 |
| flat_aggregation_n32 | groth16 | 3 | vk_read | 0.069 |
| flat_aggregation_n32 | groth16 | 3 | vk_decode | 0.710 |
| flat_aggregation_n32 | groth16 | 3 | vk_prepare | 0.548 |
| flat_aggregation_n32 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 3 | proof_decode | 0.110 |
| flat_aggregation_n32 | noir_bb | 3 | fixture | 11.285 |
| flat_aggregation_n32 | noir_bb | 3 | artifact_read | 0.124 |
| flat_aggregation_n32 | noir_bb | 3 | artifact_base64 | 0.163 |
| flat_aggregation_n32 | noir_bb | 3 | program_decode | 13.892 |
| flat_aggregation_n32 | noir_bb | 3 | acir_gunzip | 2.314 |
| flat_aggregation_n32 | noir_bb | 3 | vk_read | 0.056 |
| flat_aggregation_n32 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 3 | srs_init | 146.947 |
| flat_aggregation_n32 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n32 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n64 | groth16 | 1 | fixture | 22.544 |
| flat_aggregation_n64 | groth16 | 1 | pk_read | 40.635 |
| flat_aggregation_n64 | groth16 | 1 | pk_decode | 71172.493 |
| flat_aggregation_n64 | groth16 | 1 | vk_read | 0.072 |
| flat_aggregation_n64 | groth16 | 1 | vk_decode | 1.038 |
| flat_aggregation_n64 | groth16 | 1 | vk_prepare | 0.531 |
| flat_aggregation_n64 | groth16 | 1 | proof_encode | 0.005 |
| flat_aggregation_n64 | groth16 | 1 | proof_decode | 0.105 |
| flat_aggregation_n64 | noir_bb | 1 | fixture | 22.383 |
| flat_aggregation_n64 | noir_bb | 1 | artifact_read | 0.472 |
| flat_aggregation_n64 | noir_bb | 1 | artifact_base64 | 0.187 |
| flat_aggregation_n64 | noir_bb | 1 | program_decode | 27.087 |
| flat_aggregation_n64 | noir_bb | 1 | acir_gunzip | 4.844 |
| flat_aggregation_n64 | noir_bb | 1 | vk_read | 0.255 |
| flat_aggregation_n64 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 1 | srs_init | 155.230 |
| flat_aggregation_n64 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n64 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n64 | noir_bb | 2 | fixture | 22.522 |
| flat_aggregation_n64 | noir_bb | 2 | artifact_read | 0.193 |
| flat_aggregation_n64 | noir_bb | 2 | artifact_base64 | 0.199 |
| flat_aggregation_n64 | noir_bb | 2 | program_decode | 27.574 |
| flat_aggregation_n64 | noir_bb | 2 | acir_gunzip | 4.506 |
| flat_aggregation_n64 | noir_bb | 2 | vk_read | 0.063 |
| flat_aggregation_n64 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 2 | srs_init | 153.473 |
| flat_aggregation_n64 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n64 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n64 | groth16 | 2 | fixture | 22.479 |
| flat_aggregation_n64 | groth16 | 2 | pk_read | 47.075 |
| flat_aggregation_n64 | groth16 | 2 | pk_decode | 71600.521 |
| flat_aggregation_n64 | groth16 | 2 | vk_read | 0.063 |
| flat_aggregation_n64 | groth16 | 2 | vk_decode | 1.002 |
| flat_aggregation_n64 | groth16 | 2 | vk_prepare | 0.529 |
| flat_aggregation_n64 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n64 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n64 | groth16 | 3 | fixture | 22.530 |
| flat_aggregation_n64 | groth16 | 3 | pk_read | 49.956 |
| flat_aggregation_n64 | groth16 | 3 | pk_decode | 71371.732 |
| flat_aggregation_n64 | groth16 | 3 | vk_read | 0.079 |
| flat_aggregation_n64 | groth16 | 3 | vk_decode | 0.978 |
| flat_aggregation_n64 | groth16 | 3 | vk_prepare | 0.552 |
| flat_aggregation_n64 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n64 | groth16 | 3 | proof_decode | 0.107 |
| flat_aggregation_n64 | noir_bb | 3 | fixture | 22.406 |
| flat_aggregation_n64 | noir_bb | 3 | artifact_read | 0.494 |
| flat_aggregation_n64 | noir_bb | 3 | artifact_base64 | 0.203 |
| flat_aggregation_n64 | noir_bb | 3 | program_decode | 27.003 |
| flat_aggregation_n64 | noir_bb | 3 | acir_gunzip | 4.503 |
| flat_aggregation_n64 | noir_bb | 3 | vk_read | 0.283 |
| flat_aggregation_n64 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 3 | srs_init | 156.655 |
| flat_aggregation_n64 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n64 | noir_bb | 3 | proof_decode | 0.003 |
| emit_mint | groth16 | 1 | fixture | 5.621 |
| emit_mint | groth16 | 1 | pk_read | 0.807 |
| emit_mint | groth16 | 1 | pk_decode | 797.232 |
| emit_mint | groth16 | 1 | vk_read | 2.302 |
| emit_mint | groth16 | 1 | vk_decode | 0.516 |
| emit_mint | groth16 | 1 | vk_prepare | 0.528 |
| emit_mint | groth16 | 1 | proof_encode | 0.002 |
| emit_mint | groth16 | 1 | proof_decode | 0.105 |
| emit_mint | noir_bb | 1 | fixture | 5.616 |
| emit_mint | noir_bb | 1 | artifact_read | 0.283 |
| emit_mint | noir_bb | 1 | artifact_base64 | 0.009 |
| emit_mint | noir_bb | 1 | program_decode | 0.307 |
| emit_mint | noir_bb | 1 | acir_gunzip | 0.049 |
| emit_mint | noir_bb | 1 | vk_read | 0.163 |
| emit_mint | noir_bb | 1 | backend_init | 0.000 |
| emit_mint | noir_bb | 1 | srs_init | 151.693 |
| emit_mint | noir_bb | 1 | proof_encode | 0.001 |
| emit_mint | noir_bb | 1 | proof_decode | 0.004 |
| emit_mint | noir_bb | 2 | fixture | 5.644 |
| emit_mint | noir_bb | 2 | artifact_read | 0.030 |
| emit_mint | noir_bb | 2 | artifact_base64 | 0.006 |
| emit_mint | noir_bb | 2 | program_decode | 0.296 |
| emit_mint | noir_bb | 2 | acir_gunzip | 0.048 |
| emit_mint | noir_bb | 2 | vk_read | 0.016 |
| emit_mint | noir_bb | 2 | backend_init | 0.000 |
| emit_mint | noir_bb | 2 | srs_init | 150.470 |
| emit_mint | noir_bb | 2 | proof_encode | 0.001 |
| emit_mint | noir_bb | 2 | proof_decode | 0.008 |
| emit_mint | groth16 | 2 | fixture | 5.621 |
| emit_mint | groth16 | 2 | pk_read | 0.502 |
| emit_mint | groth16 | 2 | pk_decode | 796.372 |
| emit_mint | groth16 | 2 | vk_read | 0.043 |
| emit_mint | groth16 | 2 | vk_decode | 0.498 |
| emit_mint | groth16 | 2 | vk_prepare | 0.647 |
| emit_mint | groth16 | 2 | proof_encode | 0.003 |
| emit_mint | groth16 | 2 | proof_decode | 0.107 |
| emit_mint | groth16 | 3 | fixture | 5.666 |
| emit_mint | groth16 | 3 | pk_read | 0.497 |
| emit_mint | groth16 | 3 | pk_decode | 794.063 |
| emit_mint | groth16 | 3 | vk_read | 0.074 |
| emit_mint | groth16 | 3 | vk_decode | 0.460 |
| emit_mint | groth16 | 3 | vk_prepare | 0.541 |
| emit_mint | groth16 | 3 | proof_encode | 0.002 |
| emit_mint | groth16 | 3 | proof_decode | 0.102 |
| emit_mint | noir_bb | 3 | fixture | 5.643 |
| emit_mint | noir_bb | 3 | artifact_read | 0.026 |
| emit_mint | noir_bb | 3 | artifact_base64 | 0.007 |
| emit_mint | noir_bb | 3 | program_decode | 0.299 |
| emit_mint | noir_bb | 3 | acir_gunzip | 0.048 |
| emit_mint | noir_bb | 3 | vk_read | 0.015 |
| emit_mint | noir_bb | 3 | backend_init | 0.000 |
| emit_mint | noir_bb | 3 | srs_init | 158.766 |
| emit_mint | noir_bb | 3 | proof_encode | 0.001 |
| emit_mint | noir_bb | 3 | proof_decode | 0.005 |

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
