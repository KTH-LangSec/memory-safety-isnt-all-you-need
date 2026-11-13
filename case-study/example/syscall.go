// SPDX-License-Identifier: MIT-0

package example

import (
	"net"
	"syscall"
)

func ipv6_connect_enabled() error {
	var old = syscall.SocketDisableIPv6
	defer func() { syscall.SocketDisableIPv6 = old }()

	syscall.SocketDisableIPv6 = false

	_, err := net.Dial("tcp", "[::1]:80")
	return err
}

func ipv6_connect_disabled() error {
	var old = syscall.SocketDisableIPv6
	defer func() { syscall.SocketDisableIPv6 = old }()

	syscall.SocketDisableIPv6 = true

	_, err := net.Dial("tcp", "[::1]:80")
	return err
}
