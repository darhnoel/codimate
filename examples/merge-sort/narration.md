# Merge Sort Narration Script

Timing source: `examples/merge-sort/src/lib.rs`

- Overview: `1.40s`
- Merge step: `0.72s`
- Pass transition: `1.00s`
- Final hold: `1.60s`
- Total animation duration: `23.28s`

Use these timestamps as scene boundaries for voiceover. The wording is intentionally
short so each line can fit inside its visual segment.

| Time | Scene | Visual | Narration |
|---|---|---|---|
| 00:00.000-00:01.400 | overview | The unsorted input row appears above an empty output buffer. | Start with the unsorted values. Merge sort will repeatedly merge small sorted runs into larger sorted runs. |
| 00:01.400-00:02.120 | merge-step-00 | Compare `38` and `27`; move `27` into output slot `0`. | Compare the first two one-item runs. `27` is smaller, so it moves first. |
| 00:02.120-00:02.840 | merge-step-01 | `38` remains from the left run; move `38` into output slot `1`. | The right run is empty, so the remaining `38` follows. |
| 00:02.840-00:03.560 | merge-step-02 | Compare `43` and `3`; move `3` into output slot `2`. | Now merge the next pair. `3` is smaller than `43`. |
| 00:03.560-00:04.280 | merge-step-03 | `43` remains from the left run; move `43` into output slot `3`. | With `3` placed, `43` is the only value left in this pair. |
| 00:04.280-00:05.000 | merge-step-04 | Compare `9` and `82`; move `9` into output slot `4`. | Compare `9` and `82`. The smaller value, `9`, goes to the output buffer. |
| 00:05.000-00:05.720 | merge-step-05 | `82` remains from the right run; move `82` into output slot `5`. | `82` is left over, so it completes this two-value run. |
| 00:05.720-00:06.440 | merge-step-06 | Compare `10` and `15`; move `10` into output slot `6`. | Compare `10` and `15`. `10` is smaller. |
| 00:06.440-00:07.160 | merge-step-07 | `15` remains from the right run; move `15` into output slot `7`. | `15` is the remaining value, so it completes the first pass. |
| 00:07.160-00:08.160 | pass-1-copy-up | Completed pairs move from the output buffer back to the source row. | The first pass is done. Every run now has length two. |
| 00:08.160-00:08.880 | merge-step-08 | Compare `27` and `3`; move `3` into output slot `0`. | Pass two merges length-two runs. Between `27` and `3`, `3` moves first. |
| 00:08.880-00:09.600 | merge-step-09 | Compare `27` and `43`; move `27` into output slot `1`. | Continue comparing the front of each run. `27` comes next. |
| 00:09.600-00:10.320 | merge-step-10 | Compare `38` and `43`; move `38` into output slot `2`. | Now compare `38` and `43`. `38` is smaller. |
| 00:10.320-00:11.040 | merge-step-11 | `43` remains from the right run; move `43` into output slot `3`. | The left run is empty, so `43` finishes this merged run. |
| 00:11.040-00:11.760 | merge-step-12 | Compare `9` and `10`; move `9` into output slot `4`. | Merge the second pair of length-two runs. `9` is smaller than `10`. |
| 00:11.760-00:12.480 | merge-step-13 | Compare `82` and `10`; move `10` into output slot `5`. | Keep comparing the current heads. `10` comes before `82`. |
| 00:12.480-00:13.200 | merge-step-14 | Compare `82` and `15`; move `15` into output slot `6`. | `15` is still smaller than `82`, so it moves next. |
| 00:13.200-00:13.920 | merge-step-15 | `82` remains from the left run; move `82` into output slot `7`. | `82` is left over, completing the second length-four run. |
| 00:13.920-00:14.920 | pass-2-copy-up | Completed length-four runs move back to the source row. | The second pass is done. Now there are two sorted runs of length four. |
| 00:14.920-00:15.640 | merge-step-16 | Compare `3` and `9`; move `3` into output slot `0`. | The final pass merges the two length-four runs. `3` moves first. |
| 00:15.640-00:16.360 | merge-step-17 | Compare `27` and `9`; move `9` into output slot `1`. | Compare the current heads again. `9` is smaller than `27`. |
| 00:16.360-00:17.080 | merge-step-18 | Compare `27` and `10`; move `10` into output slot `2`. | `10` is next from the right run. |
| 00:17.080-00:17.800 | merge-step-19 | Compare `27` and `15`; move `15` into output slot `3`. | `15` also comes before `27`. |
| 00:17.800-00:18.520 | merge-step-20 | Compare `27` and `82`; move `27` into output slot `4`. | Now the left run catches up. `27` is smaller than `82`. |
| 00:18.520-00:19.240 | merge-step-21 | Compare `38` and `82`; move `38` into output slot `5`. | `38` comes next from the left run. |
| 00:19.240-00:19.960 | merge-step-22 | Compare `43` and `82`; move `43` into output slot `6`. | `43` is still smaller than `82`. |
| 00:19.960-00:20.680 | merge-step-23 | `82` remains from the right run; move `82` into output slot `7`. | Only `82` remains, so it completes the sorted output buffer. |
| 00:20.680-00:21.680 | final-output-to-sorted-array | The final output buffer moves into the sorted array row. | The final buffer becomes the sorted array. |
| 00:21.680-00:23.280 | sorted | The sorted row holds on screen: `3, 9, 10, 15, 27, 38, 43, 82`. | The result is fully sorted: `3, 9, 10, 15, 27, 38, 43, 82`. |

