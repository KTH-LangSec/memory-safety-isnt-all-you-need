// SPDX-License-Identifier: MIT-0

package example

import (
	"errors"
	"net"
	"os"
	"syscall"
	"testing"
)

func TestIpv6ConnectEnabled(t *testing.T) {
	err := ipv6_connect_enabled()
	if err == nil {
		t.Fatal("Expected an error, got nil")
	}

	var opErr *net.OpError
	if !errors.As(err, &opErr) {
		t.Fatalf("Expected a net.OpError, got %v", err)
	}

	var sysErr *os.SyscallError
	if !errors.As(opErr.Err, &sysErr) {
		t.Fatalf("Expected an os.SyscallError, got %v", opErr.Err)
	}

	if !errors.Is(sysErr.Err, syscall.ECONNREFUSED) {
		t.Fatalf("Expected a syscall.ECONNREFUSED, got %v", sysErr.Err)
	}
}

func TestIpv6ConnectDisabled(t *testing.T) {
	err := ipv6_connect_disabled()
	if err == nil {
		t.Fatal("Expected an error, got nil")
	}

	var opErr *net.OpError
	if !errors.As(err, &opErr) {
		t.Fatalf("Expected a net.OpError, got %v", err)
	}

	var sysErr *os.SyscallError
	if !errors.As(opErr.Err, &sysErr) {
		t.Fatalf("Expected an os.SyscallError, got %v", opErr.Err)
	}

	if !errors.Is(sysErr.Err, syscall.EAFNOSUPPORT) {
		t.Fatalf("Expected a syscall.EAFNOSUPPORT, got %v", sysErr.Err)
	}
}
