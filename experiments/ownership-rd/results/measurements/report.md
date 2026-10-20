# Ownership proving R&D

Public deterministic fixtures only. The Poseidon2 signature is a new unaudited authorization protocol, not compatible with production signatures. Groth16 setup is local single-party test material. Nova uses one ownership per step and Spartan IPA compression; this is not a Groth16 final proof, EVM verifier, or a deployed stateful rollup. Production cryptographic statements, canonical artifacts and prior measurements are unchanged. Cargo features preserve native parallel math while selecting serial arkworks for browser WASM.

Cold: new process and checked key load per proof, with OS filesystem caches left intact. Warm: one checked load followed by independent fresh blinded proofs in the same process; no warmup is discarded. A warm process's peak RSS covers the whole series, not an individual proof. No statistical aggregates or subtracted cost estimates.

## Individual core observations

Milliseconds. Groth16 witness includes constrained synthesis/finalization/matrices; prove excludes repeated synthesis. Nova folding includes step synthesis and IVC work; compress is actual Spartan compression. The two stage decompositions are not equivalent. All proofs were encoded, decoded and verified against expected claims.

| Engine | Signature | N | Mode | Sample | Fixture ms | Witness ms | Folding ms | Prove ms | Compress ms | Verify ms |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| groth16 | canonical | 1 | cold | 1 | 0.401 | 30.306 | N/A | 89.206 | N/A | 0.642 |
| nova | canonical | 1 | cold | 1 | 0.358 | N/A | 164.717 | N/A | 1525.358 | 108.094 |
| nova | canonical | 1 | cold | 2 | 0.367 | N/A | 164.910 | N/A | 1582.134 | 115.176 |
| groth16 | canonical | 1 | cold | 2 | 0.347 | 29.698 | N/A | 79.168 | N/A | 0.660 |
| groth16 | canonical | 1 | cold | 3 | 0.353 | 29.376 | N/A | 81.271 | N/A | 0.695 |
| nova | canonical | 1 | cold | 3 | 0.438 | N/A | 171.046 | N/A | 1552.444 | 118.544 |
| groth16 | canonical | 1 | warm | 1 | 0.369 | 29.639 | N/A | 73.325 | N/A | 0.710 |
| groth16 | canonical | 1 | warm | 2 | 0.380 | 27.240 | N/A | 76.096 | N/A | 0.697 |
| groth16 | canonical | 1 | warm | 3 | 0.370 | 26.951 | N/A | 83.781 | N/A | 0.684 |
| nova | canonical | 1 | warm | 1 | 0.365 | N/A | 161.209 | N/A | 1514.847 | 113.792 |
| nova | canonical | 1 | warm | 2 | 0.370 | N/A | 68.100 | N/A | 1598.140 | 33.883 |
| nova | canonical | 1 | warm | 3 | 0.384 | N/A | 72.243 | N/A | 1586.312 | 39.520 |
| groth16 | poseidon2 | 1 | cold | 1 | 0.260 | 17.833 | N/A | 44.554 | N/A | 0.678 |
| nova | poseidon2 | 1 | cold | 1 | 0.289 | N/A | 128.151 | N/A | 940.248 | 80.033 |
| nova | poseidon2 | 1 | cold | 2 | 0.267 | N/A | 127.529 | N/A | 921.583 | 81.285 |
| groth16 | poseidon2 | 1 | cold | 2 | 0.245 | 17.805 | N/A | 44.590 | N/A | 0.686 |
| groth16 | poseidon2 | 1 | cold | 3 | 0.266 | 18.246 | N/A | 42.798 | N/A | 0.678 |
| nova | poseidon2 | 1 | cold | 3 | 0.267 | N/A | 127.667 | N/A | 933.985 | 86.665 |
| groth16 | poseidon2 | 1 | warm | 1 | 0.266 | 17.984 | N/A | 43.729 | N/A | 0.709 |
| groth16 | poseidon2 | 1 | warm | 2 | 0.267 | 16.611 | N/A | 42.876 | N/A | 0.684 |
| groth16 | poseidon2 | 1 | warm | 3 | 0.254 | 15.865 | N/A | 40.990 | N/A | 0.703 |
| nova | poseidon2 | 1 | warm | 1 | 0.274 | N/A | 128.444 | N/A | 919.460 | 79.412 |
| nova | poseidon2 | 1 | warm | 2 | 0.253 | N/A | 47.174 | N/A | 914.273 | 24.885 |
| nova | poseidon2 | 1 | warm | 3 | 0.256 | N/A | 46.034 | N/A | 931.099 | 25.392 |
| groth16 | canonical | 8 | cold | 1 | 2.723 | 226.089 | N/A | 369.689 | N/A | 1.426 |
| nova | canonical | 8 | cold | 1 | 2.825 | N/A | 825.024 | N/A | 1555.132 | 109.477 |
| nova | canonical | 8 | cold | 2 | 2.827 | N/A | 785.393 | N/A | 1531.805 | 108.364 |
| groth16 | canonical | 8 | cold | 2 | 2.822 | 227.577 | N/A | 356.503 | N/A | 1.414 |
| groth16 | canonical | 8 | cold | 3 | 2.819 | 227.764 | N/A | 382.966 | N/A | 1.389 |
| nova | canonical | 8 | cold | 3 | 2.871 | N/A | 783.470 | N/A | 1636.662 | 110.195 |
| groth16 | canonical | 8 | warm | 1 | 2.863 | 230.804 | N/A | 391.434 | N/A | 1.404 |
| groth16 | canonical | 8 | warm | 2 | 3.138 | 230.265 | N/A | 418.814 | N/A | 1.409 |
| groth16 | canonical | 8 | warm | 3 | 2.839 | 234.516 | N/A | 419.133 | N/A | 1.420 |
| nova | canonical | 8 | warm | 1 | 2.869 | N/A | 833.266 | N/A | 1518.666 | 143.850 |
| nova | canonical | 8 | warm | 2 | 2.826 | N/A | 699.023 | N/A | 1569.192 | 43.815 |
| nova | canonical | 8 | warm | 3 | 2.821 | N/A | 693.627 | N/A | 1552.617 | 40.173 |
| groth16 | poseidon2 | 8 | cold | 1 | 1.973 | 136.122 | N/A | 170.810 | N/A | 1.376 |
| nova | poseidon2 | 8 | cold | 1 | 1.959 | N/A | 580.319 | N/A | 946.444 | 79.938 |
| nova | poseidon2 | 8 | cold | 2 | 1.994 | N/A | 582.230 | N/A | 945.638 | 79.888 |
| groth16 | poseidon2 | 8 | cold | 2 | 1.973 | 137.287 | N/A | 213.459 | N/A | 1.407 |
| groth16 | poseidon2 | 8 | cold | 3 | 2.080 | 136.938 | N/A | 190.617 | N/A | 1.434 |
| nova | poseidon2 | 8 | cold | 3 | 2.281 | N/A | 630.779 | N/A | 1061.766 | 91.999 |
| groth16 | poseidon2 | 8 | warm | 1 | 1.966 | 136.946 | N/A | 190.819 | N/A | 1.421 |
| groth16 | poseidon2 | 8 | warm | 2 | 2.033 | 136.517 | N/A | 189.960 | N/A | 1.388 |
| groth16 | poseidon2 | 8 | warm | 3 | 1.992 | 127.088 | N/A | 194.203 | N/A | 1.440 |
| nova | poseidon2 | 8 | warm | 1 | 1.989 | N/A | 599.035 | N/A | 1015.150 | 85.573 |
| nova | poseidon2 | 8 | warm | 2 | 1.970 | N/A | 503.198 | N/A | 1013.856 | 29.214 |
| nova | poseidon2 | 8 | warm | 3 | 2.331 | N/A | 505.683 | N/A | 993.408 | 28.751 |
| groth16 | canonical | 64 | cold | 1 | 22.537 | 1855.204 | N/A | 2693.987 | N/A | 7.060 |
| nova | canonical | 64 | cold | 1 | 22.433 | N/A | 5770.390 | N/A | 1586.232 | 111.583 |
| nova | canonical | 64 | cold | 2 | 23.100 | N/A | 5750.060 | N/A | 1617.202 | 118.441 |
| groth16 | canonical | 64 | cold | 2 | 22.599 | 1815.888 | N/A | 2623.960 | N/A | 7.145 |
| groth16 | canonical | 64 | cold | 3 | 22.553 | 1866.369 | N/A | 2746.549 | N/A | 7.061 |
| nova | canonical | 64 | cold | 3 | 22.753 | N/A | 5793.675 | N/A | 1673.014 | 114.585 |
| groth16 | canonical | 64 | warm | 1 | 22.547 | 1789.720 | N/A | 2602.362 | N/A | 7.090 |
| groth16 | canonical | 64 | warm | 2 | 22.381 | 1776.533 | N/A | 2688.856 | N/A | 7.076 |
| groth16 | canonical | 64 | warm | 3 | 22.487 | 1798.307 | N/A | 2396.306 | N/A | 7.047 |
| nova | canonical | 64 | warm | 1 | 22.417 | N/A | 5564.695 | N/A | 1491.331 | 107.530 |
| nova | canonical | 64 | warm | 2 | 22.373 | N/A | 5490.975 | N/A | 1496.981 | 33.662 |
| nova | canonical | 64 | warm | 3 | 22.320 | N/A | 5442.314 | N/A | 1519.997 | 41.979 |
| groth16 | poseidon2 | 64 | cold | 1 | 16.212 | 1175.103 | N/A | 1252.062 | N/A | 7.300 |
| nova | poseidon2 | 64 | cold | 1 | 15.905 | N/A | 4186.589 | N/A | 1010.676 | 88.397 |
| nova | poseidon2 | 64 | cold | 2 | 16.050 | N/A | 4207.821 | N/A | 1001.783 | 87.214 |
| groth16 | poseidon2 | 64 | cold | 2 | 16.767 | 1157.428 | N/A | 1241.720 | N/A | 7.136 |
| groth16 | poseidon2 | 64 | cold | 3 | 15.998 | 1125.608 | N/A | 1143.814 | N/A | 7.460 |
| nova | poseidon2 | 64 | cold | 3 | 16.610 | N/A | 4273.820 | N/A | 1051.364 | 90.615 |
| groth16 | poseidon2 | 64 | warm | 1 | 15.647 | 1098.684 | N/A | 1131.954 | N/A | 7.223 |
| groth16 | poseidon2 | 64 | warm | 2 | 15.807 | 1072.537 | N/A | 1316.969 | N/A | 7.435 |
| groth16 | poseidon2 | 64 | warm | 3 | 16.012 | 1097.741 | N/A | 1127.786 | N/A | 7.281 |
| nova | poseidon2 | 64 | warm | 1 | 15.828 | N/A | 4370.243 | N/A | 1048.576 | 92.892 |
| nova | poseidon2 | 64 | warm | 2 | 16.023 | N/A | 4462.539 | N/A | 1049.060 | 26.005 |
| nova | poseidon2 | 64 | warm | 3 | 16.100 | N/A | 4184.590 | N/A | 1158.799 | 35.339 |

## Key loading and process memory

One row per process. Cold load sample is the proof sample; warm load sample 0 denotes the single load shared by all proofs in that series. Native RSS includes keys, matrices/witnesses, proof work and native allocations. Browser WASM linear-memory capacity is a different metric, not RSS. Requested thread counts and hardware are in metadata.

| Engine | Signature | N | Mode | Load sample | PK decode ms | VK decode ms | VK prepare ms | Params decode ms | Process wall ms | Peak RSS MiB |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| groth16 | canonical | 1 | cold | 1 | 1102.105 | 0.414 | 3.307 | N/A | 1247.359 | 104.531 |
| nova | canonical | 1 | cold | 1 | N/A | N/A | N/A | 1847.604 | 3772.206 | 360.219 |
| nova | canonical | 1 | cold | 2 | N/A | N/A | N/A | 1886.651 | 3876.524 | 351.719 |
| groth16 | canonical | 1 | cold | 2 | 1163.480 | 0.435 | 0.532 | N/A | 1294.278 | 102.125 |
| groth16 | canonical | 1 | cold | 3 | 1108.444 | 0.438 | 0.541 | N/A | 1240.303 | 103.797 |
| nova | canonical | 1 | cold | 3 | N/A | N/A | N/A | 1947.270 | 3920.088 | 354.344 |
| groth16 | canonical | 1 | warm | 0 | 1109.029 | 0.473 | 0.538 | N/A | 1451.767 | 113.875 |
| nova | canonical | 1 | warm | 0 | N/A | N/A | N/A | 1883.690 | 7233.382 | 409.812 |
| groth16 | poseidon2 | 1 | cold | 1 | 300.105 | 0.451 | 0.530 | N/A | 373.911 | 51.141 |
| nova | poseidon2 | 1 | cold | 1 | N/A | N/A | N/A | 1406.150 | 2666.934 | 272.031 |
| nova | poseidon2 | 1 | cold | 2 | N/A | N/A | N/A | 1421.884 | 2663.771 | 268.234 |
| groth16 | poseidon2 | 1 | cold | 2 | 298.703 | 0.418 | 0.536 | N/A | 372.124 | 52.062 |
| groth16 | poseidon2 | 1 | cold | 3 | 299.756 | 0.461 | 0.567 | N/A | 371.783 | 51.641 |
| nova | poseidon2 | 1 | cold | 3 | N/A | N/A | N/A | 1493.950 | 2755.253 | 269.422 |
| groth16 | poseidon2 | 1 | warm | 0 | 299.414 | 0.450 | 0.563 | N/A | 491.415 | 59.297 |
| nova | poseidon2 | 1 | warm | 0 | N/A | N/A | N/A | 1400.827 | 4653.892 | 313.922 |
| groth16 | canonical | 8 | cold | 1 | 9012.544 | 0.486 | 0.519 | N/A | 9743.302 | 669.359 |
| nova | canonical | 8 | cold | 1 | N/A | N/A | N/A | 1929.469 | 4590.685 | 409.188 |
| nova | canonical | 8 | cold | 2 | N/A | N/A | N/A | 1936.002 | 4525.076 | 407.500 |
| groth16 | canonical | 8 | cold | 2 | 9188.553 | 0.503 | 0.534 | N/A | 9903.966 | 665.016 |
| groth16 | canonical | 8 | cold | 3 | 9280.288 | 0.528 | 0.525 | N/A | 10021.462 | 670.391 |
| nova | canonical | 8 | cold | 3 | N/A | N/A | N/A | 1933.683 | 4630.064 | 407.625 |
| groth16 | canonical | 8 | warm | 0 | 9337.880 | 0.572 | 0.532 | N/A | 11434.362 | 664.500 |
| nova | canonical | 8 | warm | 0 | N/A | N/A | N/A | 2048.765 | 9396.792 | 584.391 |
| groth16 | poseidon2 | 8 | cold | 1 | 2413.682 | 0.508 | 0.537 | N/A | 2766.732 | 313.797 |
| nova | poseidon2 | 8 | cold | 1 | N/A | N/A | N/A | 1424.837 | 3159.386 | 309.875 |
| nova | poseidon2 | 8 | cold | 2 | N/A | N/A | N/A | 1426.912 | 3164.361 | 313.953 |
| groth16 | poseidon2 | 8 | cold | 2 | 2410.847 | 0.539 | 0.548 | N/A | 2807.105 | 322.078 |
| groth16 | poseidon2 | 8 | cold | 3 | 2438.207 | 0.572 | 0.581 | N/A | 2811.501 | 317.984 |
| nova | poseidon2 | 8 | cold | 3 | N/A | N/A | N/A | 1459.306 | 3373.924 | 310.047 |
| groth16 | poseidon2 | 8 | warm | 0 | 2432.600 | 0.544 | 0.564 | N/A | 3470.492 | 359.844 |
| nova | poseidon2 | 8 | warm | 0 | N/A | N/A | N/A | 1427.792 | 6397.464 | 459.297 |
| groth16 | canonical | 64 | cold | 1 | 74164.556 | 1.039 | 0.538 | N/A | 79804.424 | 4627.312 |
| nova | canonical | 64 | cold | 1 | N/A | N/A | N/A | 1950.022 | 9606.029 | 525.703 |
| nova | canonical | 64 | cold | 2 | N/A | N/A | N/A | 1956.693 | 9632.114 | 521.016 |
| groth16 | canonical | 64 | cold | 2 | 73578.390 | 1.108 | 0.562 | N/A | 79048.652 | 4737.781 |
| groth16 | canonical | 64 | cold | 3 | 74683.766 | 1.054 | 0.566 | N/A | 80344.511 | 4690.766 |
| nova | canonical | 64 | cold | 3 | N/A | N/A | N/A | 1958.198 | 9725.956 | 523.406 |
| groth16 | canonical | 64 | warm | 0 | 74241.637 | 1.063 | 0.562 | N/A | 88601.097 | 4733.906 |
| nova | canonical | 64 | warm | 0 | N/A | N/A | N/A | 1959.232 | 23451.372 | 545.453 |
| groth16 | poseidon2 | 64 | cold | 1 | 19843.303 | 1.064 | 0.599 | N/A | 22589.103 | 2410.469 |
| nova | poseidon2 | 64 | cold | 1 | N/A | N/A | N/A | 1548.624 | 6980.332 | 449.547 |
| nova | poseidon2 | 64 | cold | 2 | N/A | N/A | N/A | 1452.350 | 6897.604 | 451.016 |
| groth16 | poseidon2 | 64 | cold | 2 | 20040.865 | 1.044 | 0.710 | N/A | 22767.414 | 2419.047 |
| groth16 | poseidon2 | 64 | cold | 3 | 20026.650 | 1.051 | 0.564 | N/A | 22618.577 | 2419.234 |
| nova | poseidon2 | 64 | cold | 3 | N/A | N/A | N/A | 1467.048 | 7040.759 | 450.766 |
| groth16 | poseidon2 | 64 | warm | 0 | 19767.094 | 1.051 | 0.573 | N/A | 27074.208 | 2413.641 |
| nova | poseidon2 | 64 | warm | 0 | N/A | N/A | N/A | 1459.411 | 18134.811 | 456.469 |

## Sizes and circuit structure

All sizes are bytes; N/A means inapplicable, not zero. Groth16 R1CS constraints and Nova augmented primary/secondary constraints per step are different metrics. Nova parameters are for a fixed one-ownership step, independent of N. Groth16 proof payload excludes its expected public fields. Nova's serialized compressed proof already includes the final state: its 96-byte expected final-state value is NOT additional wire overhead and must not be added to proof_bytes. Nova verification also takes the initial state and step count. Neither format is claimed to be EVM calldata.

| Engine | Signature | N | Mode | Sample | PK B | VK B | Params B | Proof B | Expected public/final-state B | R1CS constraints | Nova primary/step | Nova secondary/step |
|---|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| groth16 | canonical | 1 | cold | 1 | 7339376 | 360 | N/A | 128 | 96 | 34033 | N/A | N/A |
| nova | canonical | 1 | cold | 1 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 1 | cold | 2 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | canonical | 1 | cold | 2 | 7339376 | 360 | N/A | 128 | 96 | 34033 | N/A | N/A |
| groth16 | canonical | 1 | cold | 3 | 7339376 | 360 | N/A | 128 | 96 | 34033 | N/A | N/A |
| nova | canonical | 1 | cold | 3 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | canonical | 1 | warm | 1 | 7339376 | 360 | N/A | 128 | 96 | 34033 | N/A | N/A |
| groth16 | canonical | 1 | warm | 2 | 7339376 | 360 | N/A | 128 | 96 | 34033 | N/A | N/A |
| groth16 | canonical | 1 | warm | 3 | 7339376 | 360 | N/A | 128 | 96 | 34033 | N/A | N/A |
| nova | canonical | 1 | warm | 1 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 1 | warm | 2 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 1 | warm | 3 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | poseidon2 | 1 | cold | 1 | 1773072 | 360 | N/A | 128 | 96 | 8289 | N/A | N/A |
| nova | poseidon2 | 1 | cold | 1 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 1 | cold | 2 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | poseidon2 | 1 | cold | 2 | 1773072 | 360 | N/A | 128 | 96 | 8289 | N/A | N/A |
| groth16 | poseidon2 | 1 | cold | 3 | 1773072 | 360 | N/A | 128 | 96 | 8289 | N/A | N/A |
| nova | poseidon2 | 1 | cold | 3 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | poseidon2 | 1 | warm | 1 | 1773072 | 360 | N/A | 128 | 96 | 8289 | N/A | N/A |
| groth16 | poseidon2 | 1 | warm | 2 | 1773072 | 360 | N/A | 128 | 96 | 8289 | N/A | N/A |
| groth16 | poseidon2 | 1 | warm | 3 | 1773072 | 360 | N/A | 128 | 96 | 8289 | N/A | N/A |
| nova | poseidon2 | 1 | warm | 1 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 1 | warm | 2 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 1 | warm | 3 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | canonical | 8 | cold | 1 | 58710640 | 808 | N/A | 128 | 544 | 272264 | N/A | N/A |
| nova | canonical | 8 | cold | 1 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 8 | cold | 2 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | canonical | 8 | cold | 2 | 58710640 | 808 | N/A | 128 | 544 | 272264 | N/A | N/A |
| groth16 | canonical | 8 | cold | 3 | 58710640 | 808 | N/A | 128 | 544 | 272264 | N/A | N/A |
| nova | canonical | 8 | cold | 3 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | canonical | 8 | warm | 1 | 58710640 | 808 | N/A | 128 | 544 | 272264 | N/A | N/A |
| groth16 | canonical | 8 | warm | 2 | 58710640 | 808 | N/A | 128 | 544 | 272264 | N/A | N/A |
| groth16 | canonical | 8 | warm | 3 | 58710640 | 808 | N/A | 128 | 544 | 272264 | N/A | N/A |
| nova | canonical | 8 | warm | 1 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 8 | warm | 2 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 8 | warm | 3 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | poseidon2 | 8 | cold | 1 | 14180208 | 808 | N/A | 128 | 544 | 66312 | N/A | N/A |
| nova | poseidon2 | 8 | cold | 1 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 8 | cold | 2 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | poseidon2 | 8 | cold | 2 | 14180208 | 808 | N/A | 128 | 544 | 66312 | N/A | N/A |
| groth16 | poseidon2 | 8 | cold | 3 | 14180208 | 808 | N/A | 128 | 544 | 66312 | N/A | N/A |
| nova | poseidon2 | 8 | cold | 3 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | poseidon2 | 8 | warm | 1 | 14180208 | 808 | N/A | 128 | 544 | 66312 | N/A | N/A |
| groth16 | poseidon2 | 8 | warm | 2 | 14180208 | 808 | N/A | 128 | 544 | 66312 | N/A | N/A |
| groth16 | poseidon2 | 8 | warm | 3 | 14180208 | 808 | N/A | 128 | 544 | 66312 | N/A | N/A |
| nova | poseidon2 | 8 | warm | 1 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 8 | warm | 2 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 8 | warm | 3 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | canonical | 64 | cold | 1 | 469680752 | 4392 | N/A | 128 | 4128 | 2178112 | N/A | N/A |
| nova | canonical | 64 | cold | 1 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 64 | cold | 2 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | canonical | 64 | cold | 2 | 469680752 | 4392 | N/A | 128 | 4128 | 2178112 | N/A | N/A |
| groth16 | canonical | 64 | cold | 3 | 469680752 | 4392 | N/A | 128 | 4128 | 2178112 | N/A | N/A |
| nova | canonical | 64 | cold | 3 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | canonical | 64 | warm | 1 | 469680752 | 4392 | N/A | 128 | 4128 | 2178112 | N/A | N/A |
| groth16 | canonical | 64 | warm | 2 | 469680752 | 4392 | N/A | 128 | 4128 | 2178112 | N/A | N/A |
| groth16 | canonical | 64 | warm | 3 | 469680752 | 4392 | N/A | 128 | 4128 | 2178112 | N/A | N/A |
| nova | canonical | 64 | warm | 1 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 64 | warm | 2 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| nova | canonical | 64 | warm | 3 | N/A | N/A | 60427349 | 10510 | 96 | N/A | 45080 | 10538 |
| groth16 | poseidon2 | 64 | cold | 1 | 113437296 | 4392 | N/A | 128 | 4128 | 530496 | N/A | N/A |
| nova | poseidon2 | 64 | cold | 1 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 64 | cold | 2 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | poseidon2 | 64 | cold | 2 | 113437296 | 4392 | N/A | 128 | 4128 | 530496 | N/A | N/A |
| groth16 | poseidon2 | 64 | cold | 3 | 113437296 | 4392 | N/A | 128 | 4128 | 530496 | N/A | N/A |
| nova | poseidon2 | 64 | cold | 3 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| groth16 | poseidon2 | 64 | warm | 1 | 113437296 | 4392 | N/A | 128 | 4128 | 530496 | N/A | N/A |
| groth16 | poseidon2 | 64 | warm | 2 | 113437296 | 4392 | N/A | 128 | 4128 | 530496 | N/A | N/A |
| groth16 | poseidon2 | 64 | warm | 3 | 113437296 | 4392 | N/A | 128 | 4128 | 530496 | N/A | N/A |
| nova | poseidon2 | 64 | warm | 1 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 64 | warm | 2 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |
| nova | poseidon2 | 64 | warm | 3 | N/A | N/A | 50157355 | 10219 | 96 | N/A | 19336 | 10538 |

## Preparation observations

Single-party Groth16 parameter generation and transparent Nova IPA parameter/compression-key preparation happen before samples. Each has its own fresh preparation process.

| Engine | Signature | Shape capacity | Stage | Elapsed ms |
|---|---|---:|---|---:|
| groth16 | canonical | 1 | fixture | 1.282 |
| groth16 | canonical | 1 | setup | 114.576 |
| groth16 | canonical | 1 | pk_encode | 4.775 |
| groth16 | canonical | 1 | vk_encode | 0.001 |
| groth16 | canonical | 1 | pk_write | 0.682 |
| groth16 | canonical | 1 | vk_write | 0.133 |
| groth16 | canonical | 2 | fixture | 0.693 |
| groth16 | canonical | 2 | setup | 206.667 |
| groth16 | canonical | 2 | pk_encode | 8.972 |
| groth16 | canonical | 2 | vk_encode | 0.001 |
| groth16 | canonical | 2 | pk_write | 1.370 |
| groth16 | canonical | 2 | vk_write | 0.119 |
| groth16 | canonical | 4 | fixture | 1.374 |
| groth16 | canonical | 4 | setup | 387.493 |
| groth16 | canonical | 4 | pk_encode | 18.396 |
| groth16 | canonical | 4 | vk_encode | 0.002 |
| groth16 | canonical | 4 | pk_write | 2.553 |
| groth16 | canonical | 4 | vk_write | 0.144 |
| groth16 | canonical | 8 | fixture | 2.743 |
| groth16 | canonical | 8 | setup | 746.677 |
| groth16 | canonical | 8 | pk_encode | 36.381 |
| groth16 | canonical | 8 | vk_encode | 0.002 |
| groth16 | canonical | 8 | pk_write | 5.053 |
| groth16 | canonical | 8 | vk_write | 0.142 |
| groth16 | canonical | 16 | fixture | 5.532 |
| groth16 | canonical | 16 | setup | 1490.481 |
| groth16 | canonical | 16 | pk_encode | 71.157 |
| groth16 | canonical | 16 | vk_encode | 0.019 |
| groth16 | canonical | 16 | pk_write | 9.403 |
| groth16 | canonical | 16 | vk_write | 0.171 |
| groth16 | canonical | 32 | fixture | 11.807 |
| groth16 | canonical | 32 | setup | 3221.957 |
| groth16 | canonical | 32 | pk_encode | 147.060 |
| groth16 | canonical | 32 | vk_encode | 0.050 |
| groth16 | canonical | 32 | pk_write | 28.947 |
| groth16 | canonical | 32 | vk_write | 0.183 |
| groth16 | canonical | 64 | fixture | 22.721 |
| groth16 | canonical | 64 | setup | 5809.348 |
| groth16 | canonical | 64 | pk_encode | 301.206 |
| groth16 | canonical | 64 | vk_encode | 0.005 |
| groth16 | canonical | 64 | pk_write | 65.982 |
| groth16 | canonical | 64 | vk_write | 0.181 |
| nova | canonical | 1 | step_synthesis | 52.522 |
| nova | canonical | 1 | pp_setup | 751.254 |
| nova | canonical | 1 | compression_setup | 100.165 |
| nova | canonical | 1 | params_encode | 80.806 |
| nova | canonical | 1 | params_decode | 1926.488 |
| nova | canonical | 1 | params_write | 4.560 |
| groth16 | poseidon2 | 1 | fixture | 0.262 |
| groth16 | poseidon2 | 1 | setup | 48.956 |
| groth16 | poseidon2 | 1 | pk_encode | 1.035 |
| groth16 | poseidon2 | 1 | vk_encode | 0.001 |
| groth16 | poseidon2 | 1 | pk_write | 0.284 |
| groth16 | poseidon2 | 1 | vk_write | 0.066 |
| groth16 | poseidon2 | 2 | fixture | 0.490 |
| groth16 | poseidon2 | 2 | setup | 86.739 |
| groth16 | poseidon2 | 2 | pk_encode | 2.269 |
| groth16 | poseidon2 | 2 | vk_encode | 0.002 |
| groth16 | poseidon2 | 2 | pk_write | 0.375 |
| groth16 | poseidon2 | 2 | vk_write | 0.089 |
| groth16 | poseidon2 | 4 | fixture | 0.989 |
| groth16 | poseidon2 | 4 | setup | 160.907 |
| groth16 | poseidon2 | 4 | pk_encode | 4.508 |
| groth16 | poseidon2 | 4 | vk_encode | 0.001 |
| groth16 | poseidon2 | 4 | pk_write | 0.647 |
| groth16 | poseidon2 | 4 | vk_write | 0.099 |
| groth16 | poseidon2 | 8 | fixture | 1.953 |
| groth16 | poseidon2 | 8 | setup | 308.479 |
| groth16 | poseidon2 | 8 | pk_encode | 9.169 |
| groth16 | poseidon2 | 8 | vk_encode | 0.007 |
| groth16 | poseidon2 | 8 | pk_write | 1.365 |
| groth16 | poseidon2 | 8 | vk_write | 0.131 |
| groth16 | poseidon2 | 16 | fixture | 3.889 |
| groth16 | poseidon2 | 16 | setup | 594.967 |
| groth16 | poseidon2 | 16 | pk_encode | 18.010 |
| groth16 | poseidon2 | 16 | vk_encode | 0.002 |
| groth16 | poseidon2 | 16 | pk_write | 3.466 |
| groth16 | poseidon2 | 16 | vk_write | 0.144 |
| groth16 | poseidon2 | 32 | fixture | 7.700 |
| groth16 | poseidon2 | 32 | setup | 1165.481 |
| groth16 | poseidon2 | 32 | pk_encode | 37.106 |
| groth16 | poseidon2 | 32 | vk_encode | 0.003 |
| groth16 | poseidon2 | 32 | pk_write | 7.076 |
| groth16 | poseidon2 | 32 | vk_write | 0.157 |
| groth16 | poseidon2 | 64 | fixture | 15.421 |
| groth16 | poseidon2 | 64 | setup | 2297.086 |
| groth16 | poseidon2 | 64 | pk_encode | 72.386 |
| groth16 | poseidon2 | 64 | vk_encode | 0.049 |
| groth16 | poseidon2 | 64 | pk_write | 9.147 |
| groth16 | poseidon2 | 64 | vk_write | 0.176 |
| nova | poseidon2 | 1 | step_synthesis | 34.231 |
| nova | poseidon2 | 1 | pp_setup | 645.739 |
| nova | poseidon2 | 1 | compression_setup | 83.186 |
| nova | poseidon2 | 1 | params_encode | 64.940 |
| nova | poseidon2 | 1 | params_decode | 1419.175 |
| nova | poseidon2 | 1 | params_write | 3.669 |

Every raw stage, including file reads, asset hashes, encoding/decoding and any recursive verification, is retained in `timings.csv`. `observations.json` retains all load reports, proofs' verification outcomes, claim/witness digests and per-process memory. Metadata includes source/lockfile hashes, tool versions, hardware and original preparation reports. Browser results must be collected on the device; native times are never substituted for them.
