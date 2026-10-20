//go:build darwin || linux

// SPDX-License-Identifier: Apache-2.0
package main

import (
	"runtime"
	"syscall"
)

func rssBytes(usage *syscall.Rusage) uint64 {
	rss := uint64(usage.Maxrss)
	if runtime.GOOS == "linux" {
		rss *= 1024 // Linux reports KiB; Darwin reports bytes.
	}
	return rss
}

func peakRSSBytes() uint64 {
	var usage syscall.Rusage
	if err := syscall.Getrusage(syscall.RUSAGE_SELF, &usage); err != nil {
		panic(err)
	}
	return rssBytes(&usage)
}
