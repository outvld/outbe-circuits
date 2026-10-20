# Groth16 Tribute measurements

Individual wall-clock observations, not a statistical benchmark. Fresh process per backend/case/sample; OS filesystem caches left intact. Verification occurs after proving, not in a standalone cold verifier. All proofs use fresh OS-entropy-seeded blinding. Groth16 setup is local single-party experimental setup, never production ceremony material. Production circuits and backend are unchanged.

## Core timings

Times in milliseconds. Witness means constrained synthesis/finalization/matrix materialization for Groth16 and initial-map construction/ACVM solving/finalization for Noir. Groth16 proving excludes repeated synthesis. N/A is inapplicable, never an unmeasured zero.

| Circuit | Backend | Sample | Witness ms | Witness encode ms | Prove ms | Verify ms |
|---|---|---:|---:|---:|---:|---:|
| demo_tribute | groth16 | 1 | 60.307 | N/A | 108.477 | 0.820 |
| demo_tribute | noir_bb | 1 | 1.667 | 0.767 | 157.089 | 1.162 |
| demo_tribute | noir_bb | 2 | 1.617 | 0.774 | 147.178 | 1.117 |
| demo_tribute | groth16 | 2 | 60.470 | N/A | 104.815 | 0.771 |
| demo_tribute | groth16 | 3 | 60.170 | N/A | 115.729 | 0.735 |
| demo_tribute | noir_bb | 3 | 1.692 | 0.792 | 163.365 | 1.085 |
| flat_aggregation_n1 | groth16 | 1 | 31.060 | N/A | 85.157 | 0.687 |
| flat_aggregation_n1 | noir_bb | 1 | 1.145 | 0.566 | 148.970 | 1.098 |
| flat_aggregation_n1 | noir_bb | 2 | 1.106 | 0.537 | 154.363 | 1.168 |
| flat_aggregation_n1 | groth16 | 2 | 30.778 | N/A | 78.201 | 0.702 |
| flat_aggregation_n1 | groth16 | 3 | 30.721 | N/A | 78.571 | 0.685 |
| flat_aggregation_n1 | noir_bb | 3 | 1.150 | 0.548 | 156.305 | 1.109 |
| flat_aggregation_n2 | groth16 | 1 | 59.574 | N/A | 130.800 | 0.795 |
| flat_aggregation_n2 | noir_bb | 1 | 1.879 | 0.979 | 163.528 | 1.093 |
| flat_aggregation_n2 | noir_bb | 2 | 1.847 | 0.968 | 159.086 | 1.124 |
| flat_aggregation_n2 | groth16 | 2 | 60.288 | N/A | 128.760 | 0.795 |
| flat_aggregation_n2 | groth16 | 3 | 60.858 | N/A | 119.865 | 0.834 |
| flat_aggregation_n2 | noir_bb | 3 | 1.862 | 0.972 | 153.186 | 1.150 |
| flat_aggregation_n4 | groth16 | 1 | 118.462 | N/A | 211.895 | 0.985 |
| flat_aggregation_n4 | noir_bb | 1 | 3.427 | 2.570 | 182.683 | 1.142 |
| flat_aggregation_n4 | noir_bb | 2 | 3.265 | 2.578 | 181.043 | 1.109 |
| flat_aggregation_n4 | groth16 | 2 | 118.582 | N/A | 196.840 | 1.025 |
| flat_aggregation_n4 | groth16 | 3 | 118.439 | N/A | 204.529 | 0.993 |
| flat_aggregation_n4 | noir_bb | 3 | 3.401 | 2.546 | 178.833 | 1.165 |
| flat_aggregation_n8 | groth16 | 1 | 236.116 | N/A | 365.497 | 1.443 |
| flat_aggregation_n8 | noir_bb | 1 | 6.395 | 6.361 | 219.243 | 1.172 |
| flat_aggregation_n8 | noir_bb | 2 | 7.067 | 6.324 | 224.619 | 1.147 |
| flat_aggregation_n8 | groth16 | 2 | 237.177 | N/A | 376.328 | 1.415 |
| flat_aggregation_n8 | groth16 | 3 | 236.295 | N/A | 370.537 | 1.403 |
| flat_aggregation_n8 | noir_bb | 3 | 6.241 | 6.165 | 230.795 | 1.419 |
| flat_aggregation_n16 | groth16 | 1 | 463.722 | N/A | 671.420 | 2.360 |
| flat_aggregation_n16 | noir_bb | 1 | 12.116 | 15.553 | 364.477 | 1.115 |
| flat_aggregation_n16 | noir_bb | 2 | 12.135 | 15.855 | 347.726 | 1.153 |
| flat_aggregation_n16 | groth16 | 2 | 465.986 | N/A | 652.323 | 2.399 |
| flat_aggregation_n16 | groth16 | 3 | 465.763 | N/A | 680.282 | 2.303 |
| flat_aggregation_n16 | noir_bb | 3 | 12.125 | 15.844 | 358.911 | 1.133 |
| flat_aggregation_n32 | groth16 | 1 | 933.837 | N/A | 1281.574 | 3.816 |
| flat_aggregation_n32 | noir_bb | 1 | 23.423 | 37.044 | 605.668 | 1.252 |
| flat_aggregation_n32 | noir_bb | 2 | 23.762 | 37.393 | 609.967 | 1.121 |
| flat_aggregation_n32 | groth16 | 2 | 937.284 | N/A | 1270.575 | 3.949 |
| flat_aggregation_n32 | groth16 | 3 | 940.005 | N/A | 1277.947 | 3.859 |
| flat_aggregation_n32 | noir_bb | 3 | 24.996 | 37.947 | 599.736 | 1.171 |
| flat_aggregation_n64 | groth16 | 1 | 1863.418 | N/A | 2401.959 | 7.026 |
| flat_aggregation_n64 | noir_bb | 1 | 47.239 | 83.450 | 1084.084 | 1.291 |
| flat_aggregation_n64 | noir_bb | 2 | 46.646 | 83.957 | 1076.255 | 1.172 |
| flat_aggregation_n64 | groth16 | 2 | 1857.494 | N/A | 2380.400 | 7.104 |
| flat_aggregation_n64 | groth16 | 3 | 1836.897 | N/A | 2330.501 | 7.046 |
| flat_aggregation_n64 | noir_bb | 3 | 45.635 | 83.151 | 1055.402 | 1.203 |

## Technical timings

One row per observation, in milliseconds. Sample 0 is one-time preparation (one setup per shape), not a per-proof sample. Compilation and VK derivation include subprocess overhead. Groth16 PK/VK writes include compressed serialization plus disk write; checked key decoding is separate from disk reads. bb has no stored proving key or separate VK preparation API: PK read/decode and VK decode/prepare are N/A; native VK deserialization/preprocessing stays inside FFI verify. `srs_init` loads/checksums and initializes the maximum 1,048,577-point prefix, not a circuit-minimal SRS. Requested thread counts are in metadata, not a claim of measured effective utilization.

| Circuit | Backend | Sample | Stage | Elapsed ms |
|---|---|---:|---|---:|
| demo_tribute | noir_bb | 0 | compile_subprocess | 123.820 |
| demo_tribute | noir_bb | 0 | vk_derive_subprocess | 82.088 |
| demo_tribute | noir_bb | 0 | circuit_stats | 20.709 |
| flat_aggregation_n1 | noir_bb | 0 | compile_subprocess | 106.907 |
| flat_aggregation_n1 | noir_bb | 0 | vk_derive_subprocess | 70.348 |
| flat_aggregation_n1 | noir_bb | 0 | circuit_stats | 4.439 |
| flat_aggregation_n2 | noir_bb | 0 | compile_subprocess | 118.048 |
| flat_aggregation_n2 | noir_bb | 0 | vk_derive_subprocess | 77.911 |
| flat_aggregation_n2 | noir_bb | 0 | circuit_stats | 7.710 |
| flat_aggregation_n4 | noir_bb | 0 | compile_subprocess | 136.818 |
| flat_aggregation_n4 | noir_bb | 0 | vk_derive_subprocess | 104.077 |
| flat_aggregation_n4 | noir_bb | 0 | circuit_stats | 13.124 |
| flat_aggregation_n8 | noir_bb | 0 | compile_subprocess | 181.549 |
| flat_aggregation_n8 | noir_bb | 0 | vk_derive_subprocess | 131.135 |
| flat_aggregation_n8 | noir_bb | 0 | circuit_stats | 23.552 |
| flat_aggregation_n16 | noir_bb | 0 | compile_subprocess | 275.080 |
| flat_aggregation_n16 | noir_bb | 0 | vk_derive_subprocess | 188.758 |
| flat_aggregation_n16 | noir_bb | 0 | circuit_stats | 44.970 |
| flat_aggregation_n32 | noir_bb | 0 | compile_subprocess | 475.499 |
| flat_aggregation_n32 | noir_bb | 0 | vk_derive_subprocess | 312.711 |
| flat_aggregation_n32 | noir_bb | 0 | circuit_stats | 88.901 |
| flat_aggregation_n64 | noir_bb | 0 | compile_subprocess | 956.599 |
| flat_aggregation_n64 | noir_bb | 0 | vk_derive_subprocess | 550.008 |
| flat_aggregation_n64 | noir_bb | 0 | circuit_stats | 183.370 |
| demo_tribute | groth16 | 0 | setup | 174.944 |
| demo_tribute | groth16 | 0 | pk_write | 7.720 |
| demo_tribute | groth16 | 0 | vk_write | 0.116 |
| flat_aggregation_n1 | groth16 | 0 | setup | 113.971 |
| flat_aggregation_n1 | groth16 | 0 | pk_write | 7.428 |
| flat_aggregation_n1 | groth16 | 0 | vk_write | 0.102 |
| flat_aggregation_n2 | groth16 | 0 | setup | 210.429 |
| flat_aggregation_n2 | groth16 | 0 | pk_write | 11.286 |
| flat_aggregation_n2 | groth16 | 0 | vk_write | 0.122 |
| flat_aggregation_n4 | groth16 | 0 | setup | 411.169 |
| flat_aggregation_n4 | groth16 | 0 | pk_write | 23.852 |
| flat_aggregation_n4 | groth16 | 0 | vk_write | 0.131 |
| flat_aggregation_n8 | groth16 | 0 | setup | 779.967 |
| flat_aggregation_n8 | groth16 | 0 | pk_write | 42.756 |
| flat_aggregation_n8 | groth16 | 0 | vk_write | 0.127 |
| flat_aggregation_n16 | groth16 | 0 | setup | 1518.825 |
| flat_aggregation_n16 | groth16 | 0 | pk_write | 100.143 |
| flat_aggregation_n16 | groth16 | 0 | vk_write | 0.123 |
| flat_aggregation_n32 | groth16 | 0 | setup | 3001.943 |
| flat_aggregation_n32 | groth16 | 0 | pk_write | 194.547 |
| flat_aggregation_n32 | groth16 | 0 | vk_write | 0.135 |
| flat_aggregation_n64 | groth16 | 0 | setup | 6047.534 |
| flat_aggregation_n64 | groth16 | 0 | pk_write | 423.864 |
| flat_aggregation_n64 | groth16 | 0 | vk_write | 0.198 |
| demo_tribute | groth16 | 1 | fixture | 7.435 |
| demo_tribute | groth16 | 1 | pk_read | 0.928 |
| demo_tribute | groth16 | 1 | pk_decode | 1565.765 |
| demo_tribute | groth16 | 1 | vk_read | 0.063 |
| demo_tribute | groth16 | 1 | vk_decode | 0.446 |
| demo_tribute | groth16 | 1 | vk_prepare | 0.550 |
| demo_tribute | groth16 | 1 | proof_encode | 0.002 |
| demo_tribute | groth16 | 1 | proof_decode | 0.104 |
| demo_tribute | noir_bb | 1 | fixture | 7.674 |
| demo_tribute | noir_bb | 1 | artifact_read | 0.059 |
| demo_tribute | noir_bb | 1 | artifact_base64 | 0.010 |
| demo_tribute | noir_bb | 1 | program_decode | 0.769 |
| demo_tribute | noir_bb | 1 | acir_gunzip | 0.100 |
| demo_tribute | noir_bb | 1 | vk_read | 0.021 |
| demo_tribute | noir_bb | 1 | backend_init | 0.000 |
| demo_tribute | noir_bb | 1 | srs_init | 149.472 |
| demo_tribute | noir_bb | 1 | proof_encode | 0.001 |
| demo_tribute | noir_bb | 1 | proof_decode | 0.005 |
| demo_tribute | noir_bb | 2 | fixture | 7.661 |
| demo_tribute | noir_bb | 2 | artifact_read | 0.041 |
| demo_tribute | noir_bb | 2 | artifact_base64 | 0.010 |
| demo_tribute | noir_bb | 2 | program_decode | 0.785 |
| demo_tribute | noir_bb | 2 | acir_gunzip | 0.095 |
| demo_tribute | noir_bb | 2 | vk_read | 0.024 |
| demo_tribute | noir_bb | 2 | backend_init | 0.000 |
| demo_tribute | noir_bb | 2 | srs_init | 147.698 |
| demo_tribute | noir_bb | 2 | proof_encode | 0.001 |
| demo_tribute | noir_bb | 2 | proof_decode | 0.005 |
| demo_tribute | groth16 | 2 | fixture | 7.665 |
| demo_tribute | groth16 | 2 | pk_read | 0.948 |
| demo_tribute | groth16 | 2 | pk_decode | 1584.370 |
| demo_tribute | groth16 | 2 | vk_read | 0.081 |
| demo_tribute | groth16 | 2 | vk_decode | 0.463 |
| demo_tribute | groth16 | 2 | vk_prepare | 0.572 |
| demo_tribute | groth16 | 2 | proof_encode | 0.003 |
| demo_tribute | groth16 | 2 | proof_decode | 0.120 |
| demo_tribute | groth16 | 3 | fixture | 7.811 |
| demo_tribute | groth16 | 3 | pk_read | 1.076 |
| demo_tribute | groth16 | 3 | pk_decode | 1576.269 |
| demo_tribute | groth16 | 3 | vk_read | 0.070 |
| demo_tribute | groth16 | 3 | vk_decode | 0.455 |
| demo_tribute | groth16 | 3 | vk_prepare | 0.540 |
| demo_tribute | groth16 | 3 | proof_encode | 0.002 |
| demo_tribute | groth16 | 3 | proof_decode | 0.109 |
| demo_tribute | noir_bb | 3 | fixture | 7.663 |
| demo_tribute | noir_bb | 3 | artifact_read | 0.059 |
| demo_tribute | noir_bb | 3 | artifact_base64 | 0.009 |
| demo_tribute | noir_bb | 3 | program_decode | 0.701 |
| demo_tribute | noir_bb | 3 | acir_gunzip | 0.101 |
| demo_tribute | noir_bb | 3 | vk_read | 0.021 |
| demo_tribute | noir_bb | 3 | backend_init | 0.000 |
| demo_tribute | noir_bb | 3 | srs_init | 149.654 |
| demo_tribute | noir_bb | 3 | proof_encode | 0.003 |
| demo_tribute | noir_bb | 3 | proof_decode | 0.005 |
| flat_aggregation_n1 | groth16 | 1 | fixture | 0.390 |
| flat_aggregation_n1 | groth16 | 1 | pk_read | 0.669 |
| flat_aggregation_n1 | groth16 | 1 | pk_decode | 1137.659 |
| flat_aggregation_n1 | groth16 | 1 | vk_read | 0.058 |
| flat_aggregation_n1 | groth16 | 1 | vk_decode | 0.432 |
| flat_aggregation_n1 | groth16 | 1 | vk_prepare | 0.551 |
| flat_aggregation_n1 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 1 | proof_decode | 0.114 |
| flat_aggregation_n1 | noir_bb | 1 | fixture | 0.407 |
| flat_aggregation_n1 | noir_bb | 1 | artifact_read | 0.028 |
| flat_aggregation_n1 | noir_bb | 1 | artifact_base64 | 0.013 |
| flat_aggregation_n1 | noir_bb | 1 | program_decode | 0.658 |
| flat_aggregation_n1 | noir_bb | 1 | acir_gunzip | 0.081 |
| flat_aggregation_n1 | noir_bb | 1 | vk_read | 0.015 |
| flat_aggregation_n1 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 1 | srs_init | 149.693 |
| flat_aggregation_n1 | noir_bb | 1 | proof_encode | 0.002 |
| flat_aggregation_n1 | noir_bb | 1 | proof_decode | 0.004 |
| flat_aggregation_n1 | noir_bb | 2 | fixture | 0.382 |
| flat_aggregation_n1 | noir_bb | 2 | artifact_read | 0.031 |
| flat_aggregation_n1 | noir_bb | 2 | artifact_base64 | 0.008 |
| flat_aggregation_n1 | noir_bb | 2 | program_decode | 0.634 |
| flat_aggregation_n1 | noir_bb | 2 | acir_gunzip | 0.085 |
| flat_aggregation_n1 | noir_bb | 2 | vk_read | 0.017 |
| flat_aggregation_n1 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 2 | srs_init | 148.515 |
| flat_aggregation_n1 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n1 | groth16 | 2 | fixture | 0.424 |
| flat_aggregation_n1 | groth16 | 2 | pk_read | 0.772 |
| flat_aggregation_n1 | groth16 | 2 | pk_decode | 1137.584 |
| flat_aggregation_n1 | groth16 | 2 | vk_read | 0.077 |
| flat_aggregation_n1 | groth16 | 2 | vk_decode | 0.470 |
| flat_aggregation_n1 | groth16 | 2 | vk_prepare | 0.530 |
| flat_aggregation_n1 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 2 | proof_decode | 0.104 |
| flat_aggregation_n1 | groth16 | 3 | fixture | 0.388 |
| flat_aggregation_n1 | groth16 | 3 | pk_read | 0.668 |
| flat_aggregation_n1 | groth16 | 3 | pk_decode | 1138.924 |
| flat_aggregation_n1 | groth16 | 3 | vk_read | 0.072 |
| flat_aggregation_n1 | groth16 | 3 | vk_decode | 0.435 |
| flat_aggregation_n1 | groth16 | 3 | vk_prepare | 0.549 |
| flat_aggregation_n1 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n1 | groth16 | 3 | proof_decode | 0.104 |
| flat_aggregation_n1 | noir_bb | 3 | fixture | 0.385 |
| flat_aggregation_n1 | noir_bb | 3 | artifact_read | 0.034 |
| flat_aggregation_n1 | noir_bb | 3 | artifact_base64 | 0.009 |
| flat_aggregation_n1 | noir_bb | 3 | program_decode | 0.629 |
| flat_aggregation_n1 | noir_bb | 3 | acir_gunzip | 0.084 |
| flat_aggregation_n1 | noir_bb | 3 | vk_read | 0.021 |
| flat_aggregation_n1 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n1 | noir_bb | 3 | srs_init | 150.556 |
| flat_aggregation_n1 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n1 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n2 | groth16 | 1 | fixture | 0.742 |
| flat_aggregation_n2 | groth16 | 1 | pk_read | 1.296 |
| flat_aggregation_n2 | groth16 | 1 | pk_decode | 2268.443 |
| flat_aggregation_n2 | groth16 | 1 | vk_read | 0.058 |
| flat_aggregation_n2 | groth16 | 1 | vk_decode | 0.457 |
| flat_aggregation_n2 | groth16 | 1 | vk_prepare | 0.560 |
| flat_aggregation_n2 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 1 | proof_decode | 0.104 |
| flat_aggregation_n2 | noir_bb | 1 | fixture | 0.752 |
| flat_aggregation_n2 | noir_bb | 1 | artifact_read | 0.037 |
| flat_aggregation_n2 | noir_bb | 1 | artifact_base64 | 0.010 |
| flat_aggregation_n2 | noir_bb | 1 | program_decode | 1.089 |
| flat_aggregation_n2 | noir_bb | 1 | acir_gunzip | 0.140 |
| flat_aggregation_n2 | noir_bb | 1 | vk_read | 0.022 |
| flat_aggregation_n2 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 1 | srs_init | 149.987 |
| flat_aggregation_n2 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n2 | noir_bb | 2 | fixture | 0.736 |
| flat_aggregation_n2 | noir_bb | 2 | artifact_read | 0.028 |
| flat_aggregation_n2 | noir_bb | 2 | artifact_base64 | 0.010 |
| flat_aggregation_n2 | noir_bb | 2 | program_decode | 1.080 |
| flat_aggregation_n2 | noir_bb | 2 | acir_gunzip | 0.140 |
| flat_aggregation_n2 | noir_bb | 2 | vk_read | 0.018 |
| flat_aggregation_n2 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 2 | srs_init | 149.242 |
| flat_aggregation_n2 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 2 | proof_decode | 0.004 |
| flat_aggregation_n2 | groth16 | 2 | fixture | 0.741 |
| flat_aggregation_n2 | groth16 | 2 | pk_read | 1.338 |
| flat_aggregation_n2 | groth16 | 2 | pk_decode | 2278.810 |
| flat_aggregation_n2 | groth16 | 2 | vk_read | 0.076 |
| flat_aggregation_n2 | groth16 | 2 | vk_decode | 0.480 |
| flat_aggregation_n2 | groth16 | 2 | vk_prepare | 0.538 |
| flat_aggregation_n2 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n2 | groth16 | 3 | fixture | 0.725 |
| flat_aggregation_n2 | groth16 | 3 | pk_read | 1.342 |
| flat_aggregation_n2 | groth16 | 3 | pk_decode | 2275.474 |
| flat_aggregation_n2 | groth16 | 3 | vk_read | 0.083 |
| flat_aggregation_n2 | groth16 | 3 | vk_decode | 0.457 |
| flat_aggregation_n2 | groth16 | 3 | vk_prepare | 0.529 |
| flat_aggregation_n2 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n2 | groth16 | 3 | proof_decode | 0.106 |
| flat_aggregation_n2 | noir_bb | 3 | fixture | 0.736 |
| flat_aggregation_n2 | noir_bb | 3 | artifact_read | 0.038 |
| flat_aggregation_n2 | noir_bb | 3 | artifact_base64 | 0.010 |
| flat_aggregation_n2 | noir_bb | 3 | program_decode | 1.142 |
| flat_aggregation_n2 | noir_bb | 3 | acir_gunzip | 0.144 |
| flat_aggregation_n2 | noir_bb | 3 | vk_read | 0.025 |
| flat_aggregation_n2 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n2 | noir_bb | 3 | srs_init | 151.010 |
| flat_aggregation_n2 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n2 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n4 | groth16 | 1 | fixture | 1.431 |
| flat_aggregation_n4 | groth16 | 1 | pk_read | 2.637 |
| flat_aggregation_n4 | groth16 | 1 | pk_decode | 4558.747 |
| flat_aggregation_n4 | groth16 | 1 | vk_read | 0.064 |
| flat_aggregation_n4 | groth16 | 1 | vk_decode | 0.489 |
| flat_aggregation_n4 | groth16 | 1 | vk_prepare | 0.538 |
| flat_aggregation_n4 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 1 | proof_decode | 0.105 |
| flat_aggregation_n4 | noir_bb | 1 | fixture | 1.424 |
| flat_aggregation_n4 | noir_bb | 1 | artifact_read | 0.035 |
| flat_aggregation_n4 | noir_bb | 1 | artifact_base64 | 0.016 |
| flat_aggregation_n4 | noir_bb | 1 | program_decode | 2.035 |
| flat_aggregation_n4 | noir_bb | 1 | acir_gunzip | 0.252 |
| flat_aggregation_n4 | noir_bb | 1 | vk_read | 0.025 |
| flat_aggregation_n4 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 1 | srs_init | 148.901 |
| flat_aggregation_n4 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n4 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n4 | noir_bb | 2 | fixture | 1.442 |
| flat_aggregation_n4 | noir_bb | 2 | artifact_read | 0.030 |
| flat_aggregation_n4 | noir_bb | 2 | artifact_base64 | 0.016 |
| flat_aggregation_n4 | noir_bb | 2 | program_decode | 1.950 |
| flat_aggregation_n4 | noir_bb | 2 | acir_gunzip | 0.260 |
| flat_aggregation_n4 | noir_bb | 2 | vk_read | 0.021 |
| flat_aggregation_n4 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 2 | srs_init | 148.964 |
| flat_aggregation_n4 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n4 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n4 | groth16 | 2 | fixture | 1.450 |
| flat_aggregation_n4 | groth16 | 2 | pk_read | 2.720 |
| flat_aggregation_n4 | groth16 | 2 | pk_decode | 4570.803 |
| flat_aggregation_n4 | groth16 | 2 | vk_read | 0.075 |
| flat_aggregation_n4 | groth16 | 2 | vk_decode | 0.507 |
| flat_aggregation_n4 | groth16 | 2 | vk_prepare | 0.547 |
| flat_aggregation_n4 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 2 | proof_decode | 0.123 |
| flat_aggregation_n4 | groth16 | 3 | fixture | 1.566 |
| flat_aggregation_n4 | groth16 | 3 | pk_read | 2.789 |
| flat_aggregation_n4 | groth16 | 3 | pk_decode | 4567.022 |
| flat_aggregation_n4 | groth16 | 3 | vk_read | 0.070 |
| flat_aggregation_n4 | groth16 | 3 | vk_decode | 0.494 |
| flat_aggregation_n4 | groth16 | 3 | vk_prepare | 0.569 |
| flat_aggregation_n4 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n4 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n4 | noir_bb | 3 | fixture | 1.449 |
| flat_aggregation_n4 | noir_bb | 3 | artifact_read | 0.035 |
| flat_aggregation_n4 | noir_bb | 3 | artifact_base64 | 0.016 |
| flat_aggregation_n4 | noir_bb | 3 | program_decode | 2.049 |
| flat_aggregation_n4 | noir_bb | 3 | acir_gunzip | 0.252 |
| flat_aggregation_n4 | noir_bb | 3 | vk_read | 0.025 |
| flat_aggregation_n4 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n4 | noir_bb | 3 | srs_init | 150.102 |
| flat_aggregation_n4 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n4 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n8 | groth16 | 1 | fixture | 2.833 |
| flat_aggregation_n8 | groth16 | 1 | pk_read | 5.249 |
| flat_aggregation_n8 | groth16 | 1 | pk_decode | 9155.590 |
| flat_aggregation_n8 | groth16 | 1 | vk_read | 0.088 |
| flat_aggregation_n8 | groth16 | 1 | vk_decode | 0.487 |
| flat_aggregation_n8 | groth16 | 1 | vk_prepare | 0.566 |
| flat_aggregation_n8 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 1 | proof_decode | 0.106 |
| flat_aggregation_n8 | noir_bb | 1 | fixture | 2.851 |
| flat_aggregation_n8 | noir_bb | 1 | artifact_read | 0.039 |
| flat_aggregation_n8 | noir_bb | 1 | artifact_base64 | 0.028 |
| flat_aggregation_n8 | noir_bb | 1 | program_decode | 3.826 |
| flat_aggregation_n8 | noir_bb | 1 | acir_gunzip | 0.635 |
| flat_aggregation_n8 | noir_bb | 1 | vk_read | 0.057 |
| flat_aggregation_n8 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 1 | srs_init | 150.537 |
| flat_aggregation_n8 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n8 | noir_bb | 2 | fixture | 3.076 |
| flat_aggregation_n8 | noir_bb | 2 | artifact_read | 0.040 |
| flat_aggregation_n8 | noir_bb | 2 | artifact_base64 | 0.031 |
| flat_aggregation_n8 | noir_bb | 2 | program_decode | 4.076 |
| flat_aggregation_n8 | noir_bb | 2 | acir_gunzip | 0.699 |
| flat_aggregation_n8 | noir_bb | 2 | vk_read | 0.035 |
| flat_aggregation_n8 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 2 | srs_init | 151.007 |
| flat_aggregation_n8 | noir_bb | 2 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n8 | groth16 | 2 | fixture | 2.829 |
| flat_aggregation_n8 | groth16 | 2 | pk_read | 5.218 |
| flat_aggregation_n8 | groth16 | 2 | pk_decode | 9171.930 |
| flat_aggregation_n8 | groth16 | 2 | vk_read | 0.083 |
| flat_aggregation_n8 | groth16 | 2 | vk_decode | 0.504 |
| flat_aggregation_n8 | groth16 | 2 | vk_prepare | 0.553 |
| flat_aggregation_n8 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 2 | proof_decode | 0.108 |
| flat_aggregation_n8 | groth16 | 3 | fixture | 2.822 |
| flat_aggregation_n8 | groth16 | 3 | pk_read | 5.541 |
| flat_aggregation_n8 | groth16 | 3 | pk_decode | 9186.262 |
| flat_aggregation_n8 | groth16 | 3 | vk_read | 0.080 |
| flat_aggregation_n8 | groth16 | 3 | vk_decode | 0.477 |
| flat_aggregation_n8 | groth16 | 3 | vk_prepare | 0.536 |
| flat_aggregation_n8 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n8 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n8 | noir_bb | 3 | fixture | 2.847 |
| flat_aggregation_n8 | noir_bb | 3 | artifact_read | 0.050 |
| flat_aggregation_n8 | noir_bb | 3 | artifact_base64 | 0.029 |
| flat_aggregation_n8 | noir_bb | 3 | program_decode | 3.893 |
| flat_aggregation_n8 | noir_bb | 3 | acir_gunzip | 0.611 |
| flat_aggregation_n8 | noir_bb | 3 | vk_read | 0.075 |
| flat_aggregation_n8 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n8 | noir_bb | 3 | srs_init | 151.530 |
| flat_aggregation_n8 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n8 | noir_bb | 3 | proof_decode | 0.002 |
| flat_aggregation_n16 | groth16 | 1 | fixture | 5.716 |
| flat_aggregation_n16 | groth16 | 1 | pk_read | 10.405 |
| flat_aggregation_n16 | groth16 | 1 | pk_decode | 18330.301 |
| flat_aggregation_n16 | groth16 | 1 | vk_read | 0.082 |
| flat_aggregation_n16 | groth16 | 1 | vk_decode | 0.567 |
| flat_aggregation_n16 | groth16 | 1 | vk_prepare | 0.566 |
| flat_aggregation_n16 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 1 | proof_decode | 0.109 |
| flat_aggregation_n16 | noir_bb | 1 | fixture | 5.643 |
| flat_aggregation_n16 | noir_bb | 1 | artifact_read | 0.071 |
| flat_aggregation_n16 | noir_bb | 1 | artifact_base64 | 0.052 |
| flat_aggregation_n16 | noir_bb | 1 | program_decode | 7.349 |
| flat_aggregation_n16 | noir_bb | 1 | acir_gunzip | 1.010 |
| flat_aggregation_n16 | noir_bb | 1 | vk_read | 0.059 |
| flat_aggregation_n16 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 1 | srs_init | 150.966 |
| flat_aggregation_n16 | noir_bb | 1 | proof_encode | 0.005 |
| flat_aggregation_n16 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n16 | noir_bb | 2 | fixture | 5.666 |
| flat_aggregation_n16 | noir_bb | 2 | artifact_read | 0.050 |
| flat_aggregation_n16 | noir_bb | 2 | artifact_base64 | 0.051 |
| flat_aggregation_n16 | noir_bb | 2 | program_decode | 7.319 |
| flat_aggregation_n16 | noir_bb | 2 | acir_gunzip | 1.074 |
| flat_aggregation_n16 | noir_bb | 2 | vk_read | 0.049 |
| flat_aggregation_n16 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 2 | srs_init | 149.272 |
| flat_aggregation_n16 | noir_bb | 2 | proof_encode | 0.004 |
| flat_aggregation_n16 | noir_bb | 2 | proof_decode | 0.002 |
| flat_aggregation_n16 | groth16 | 2 | fixture | 5.804 |
| flat_aggregation_n16 | groth16 | 2 | pk_read | 10.773 |
| flat_aggregation_n16 | groth16 | 2 | pk_decode | 18342.159 |
| flat_aggregation_n16 | groth16 | 2 | vk_read | 0.072 |
| flat_aggregation_n16 | groth16 | 2 | vk_decode | 0.613 |
| flat_aggregation_n16 | groth16 | 2 | vk_prepare | 0.533 |
| flat_aggregation_n16 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n16 | groth16 | 2 | proof_decode | 0.113 |
| flat_aggregation_n16 | groth16 | 3 | fixture | 5.642 |
| flat_aggregation_n16 | groth16 | 3 | pk_read | 10.480 |
| flat_aggregation_n16 | groth16 | 3 | pk_decode | 18309.971 |
| flat_aggregation_n16 | groth16 | 3 | vk_read | 0.070 |
| flat_aggregation_n16 | groth16 | 3 | vk_decode | 0.572 |
| flat_aggregation_n16 | groth16 | 3 | vk_prepare | 0.567 |
| flat_aggregation_n16 | groth16 | 3 | proof_encode | 0.006 |
| flat_aggregation_n16 | groth16 | 3 | proof_decode | 0.108 |
| flat_aggregation_n16 | noir_bb | 3 | fixture | 5.681 |
| flat_aggregation_n16 | noir_bb | 3 | artifact_read | 0.076 |
| flat_aggregation_n16 | noir_bb | 3 | artifact_base64 | 0.054 |
| flat_aggregation_n16 | noir_bb | 3 | program_decode | 7.476 |
| flat_aggregation_n16 | noir_bb | 3 | acir_gunzip | 1.092 |
| flat_aggregation_n16 | noir_bb | 3 | vk_read | 0.057 |
| flat_aggregation_n16 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n16 | noir_bb | 3 | srs_init | 149.904 |
| flat_aggregation_n16 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n16 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n32 | groth16 | 1 | fixture | 11.300 |
| flat_aggregation_n32 | groth16 | 1 | pk_read | 20.818 |
| flat_aggregation_n32 | groth16 | 1 | pk_decode | 36580.277 |
| flat_aggregation_n32 | groth16 | 1 | vk_read | 0.056 |
| flat_aggregation_n32 | groth16 | 1 | vk_decode | 0.696 |
| flat_aggregation_n32 | groth16 | 1 | vk_prepare | 0.548 |
| flat_aggregation_n32 | groth16 | 1 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 1 | proof_decode | 0.104 |
| flat_aggregation_n32 | noir_bb | 1 | fixture | 11.237 |
| flat_aggregation_n32 | noir_bb | 1 | artifact_read | 0.123 |
| flat_aggregation_n32 | noir_bb | 1 | artifact_base64 | 0.119 |
| flat_aggregation_n32 | noir_bb | 1 | program_decode | 13.941 |
| flat_aggregation_n32 | noir_bb | 1 | acir_gunzip | 2.372 |
| flat_aggregation_n32 | noir_bb | 1 | vk_read | 0.064 |
| flat_aggregation_n32 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 1 | srs_init | 149.910 |
| flat_aggregation_n32 | noir_bb | 1 | proof_encode | 0.001 |
| flat_aggregation_n32 | noir_bb | 1 | proof_decode | 0.003 |
| flat_aggregation_n32 | noir_bb | 2 | fixture | 11.328 |
| flat_aggregation_n32 | noir_bb | 2 | artifact_read | 0.121 |
| flat_aggregation_n32 | noir_bb | 2 | artifact_base64 | 0.099 |
| flat_aggregation_n32 | noir_bb | 2 | program_decode | 13.915 |
| flat_aggregation_n32 | noir_bb | 2 | acir_gunzip | 2.422 |
| flat_aggregation_n32 | noir_bb | 2 | vk_read | 0.056 |
| flat_aggregation_n32 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 2 | srs_init | 150.738 |
| flat_aggregation_n32 | noir_bb | 2 | proof_encode | 0.002 |
| flat_aggregation_n32 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n32 | groth16 | 2 | fixture | 11.264 |
| flat_aggregation_n32 | groth16 | 2 | pk_read | 21.215 |
| flat_aggregation_n32 | groth16 | 2 | pk_decode | 37517.689 |
| flat_aggregation_n32 | groth16 | 2 | vk_read | 0.081 |
| flat_aggregation_n32 | groth16 | 2 | vk_decode | 0.698 |
| flat_aggregation_n32 | groth16 | 2 | vk_prepare | 0.545 |
| flat_aggregation_n32 | groth16 | 2 | proof_encode | 0.004 |
| flat_aggregation_n32 | groth16 | 2 | proof_decode | 0.107 |
| flat_aggregation_n32 | groth16 | 3 | fixture | 11.319 |
| flat_aggregation_n32 | groth16 | 3 | pk_read | 20.991 |
| flat_aggregation_n32 | groth16 | 3 | pk_decode | 36720.375 |
| flat_aggregation_n32 | groth16 | 3 | vk_read | 0.067 |
| flat_aggregation_n32 | groth16 | 3 | vk_decode | 0.709 |
| flat_aggregation_n32 | groth16 | 3 | vk_prepare | 0.603 |
| flat_aggregation_n32 | groth16 | 3 | proof_encode | 0.002 |
| flat_aggregation_n32 | groth16 | 3 | proof_decode | 0.105 |
| flat_aggregation_n32 | noir_bb | 3 | fixture | 11.440 |
| flat_aggregation_n32 | noir_bb | 3 | artifact_read | 0.131 |
| flat_aggregation_n32 | noir_bb | 3 | artifact_base64 | 0.098 |
| flat_aggregation_n32 | noir_bb | 3 | program_decode | 13.895 |
| flat_aggregation_n32 | noir_bb | 3 | acir_gunzip | 2.316 |
| flat_aggregation_n32 | noir_bb | 3 | vk_read | 0.074 |
| flat_aggregation_n32 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n32 | noir_bb | 3 | srs_init | 149.972 |
| flat_aggregation_n32 | noir_bb | 3 | proof_encode | 0.001 |
| flat_aggregation_n32 | noir_bb | 3 | proof_decode | 0.003 |
| flat_aggregation_n64 | groth16 | 1 | fixture | 22.543 |
| flat_aggregation_n64 | groth16 | 1 | pk_read | 40.740 |
| flat_aggregation_n64 | groth16 | 1 | pk_decode | 73061.122 |
| flat_aggregation_n64 | groth16 | 1 | vk_read | 0.074 |
| flat_aggregation_n64 | groth16 | 1 | vk_decode | 1.056 |
| flat_aggregation_n64 | groth16 | 1 | vk_prepare | 0.568 |
| flat_aggregation_n64 | groth16 | 1 | proof_encode | 0.004 |
| flat_aggregation_n64 | groth16 | 1 | proof_decode | 0.105 |
| flat_aggregation_n64 | noir_bb | 1 | fixture | 22.566 |
| flat_aggregation_n64 | noir_bb | 1 | artifact_read | 0.200 |
| flat_aggregation_n64 | noir_bb | 1 | artifact_base64 | 0.199 |
| flat_aggregation_n64 | noir_bb | 1 | program_decode | 27.148 |
| flat_aggregation_n64 | noir_bb | 1 | acir_gunzip | 4.523 |
| flat_aggregation_n64 | noir_bb | 1 | vk_read | 0.070 |
| flat_aggregation_n64 | noir_bb | 1 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 1 | srs_init | 149.730 |
| flat_aggregation_n64 | noir_bb | 1 | proof_encode | 0.004 |
| flat_aggregation_n64 | noir_bb | 1 | proof_decode | 0.002 |
| flat_aggregation_n64 | noir_bb | 2 | fixture | 22.692 |
| flat_aggregation_n64 | noir_bb | 2 | artifact_read | 0.187 |
| flat_aggregation_n64 | noir_bb | 2 | artifact_base64 | 0.193 |
| flat_aggregation_n64 | noir_bb | 2 | program_decode | 27.775 |
| flat_aggregation_n64 | noir_bb | 2 | acir_gunzip | 4.513 |
| flat_aggregation_n64 | noir_bb | 2 | vk_read | 0.060 |
| flat_aggregation_n64 | noir_bb | 2 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 2 | srs_init | 149.217 |
| flat_aggregation_n64 | noir_bb | 2 | proof_encode | 0.004 |
| flat_aggregation_n64 | noir_bb | 2 | proof_decode | 0.003 |
| flat_aggregation_n64 | groth16 | 2 | fixture | 22.634 |
| flat_aggregation_n64 | groth16 | 2 | pk_read | 40.413 |
| flat_aggregation_n64 | groth16 | 2 | pk_decode | 73115.517 |
| flat_aggregation_n64 | groth16 | 2 | vk_read | 0.080 |
| flat_aggregation_n64 | groth16 | 2 | vk_decode | 1.053 |
| flat_aggregation_n64 | groth16 | 2 | vk_prepare | 0.555 |
| flat_aggregation_n64 | groth16 | 2 | proof_encode | 0.002 |
| flat_aggregation_n64 | groth16 | 2 | proof_decode | 0.105 |
| flat_aggregation_n64 | groth16 | 3 | fixture | 22.553 |
| flat_aggregation_n64 | groth16 | 3 | pk_read | 39.693 |
| flat_aggregation_n64 | groth16 | 3 | pk_decode | 72692.042 |
| flat_aggregation_n64 | groth16 | 3 | vk_read | 0.081 |
| flat_aggregation_n64 | groth16 | 3 | vk_decode | 1.046 |
| flat_aggregation_n64 | groth16 | 3 | vk_prepare | 0.541 |
| flat_aggregation_n64 | groth16 | 3 | proof_encode | 0.003 |
| flat_aggregation_n64 | groth16 | 3 | proof_decode | 0.106 |
| flat_aggregation_n64 | noir_bb | 3 | fixture | 22.308 |
| flat_aggregation_n64 | noir_bb | 3 | artifact_read | 0.168 |
| flat_aggregation_n64 | noir_bb | 3 | artifact_base64 | 0.206 |
| flat_aggregation_n64 | noir_bb | 3 | program_decode | 26.728 |
| flat_aggregation_n64 | noir_bb | 3 | acir_gunzip | 4.430 |
| flat_aggregation_n64 | noir_bb | 3 | vk_read | 0.076 |
| flat_aggregation_n64 | noir_bb | 3 | backend_init | 0.000 |
| flat_aggregation_n64 | noir_bb | 3 | srs_init | 147.601 |
| flat_aggregation_n64 | noir_bb | 3 | proof_encode | 0.004 |
| flat_aggregation_n64 | noir_bb | 3 | proof_decode | 0.004 |

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

Exact nanoseconds are in `timings.csv`; full per-sample fixture/public digests and numeric sizes are in `observations.json`. Hardware, pins, encodings, settings, provenance hashes and stage definitions are in `metadata.json`. No averages, selected best runs, or subtracted cost estimates are reported.
