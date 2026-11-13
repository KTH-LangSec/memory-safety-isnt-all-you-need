// SPDX-License-Identifier: MIT-0

// See also <https://github.com/golang/go/issues/70942>

package example

import (
	"crypto/rand"
)

type zero struct{}

func (z zero) Read(p []byte) (int, error) {
	for i := range p {
		p[i] = 0
	}
	return len(p), nil
}

func crypto_rand_reader_bad() string {
	var old = rand.Reader
	defer func() { rand.Reader = old }()

	rand.Reader = zero{}

	return rand.Text()
}

func crypto_rand_reader_good() string {
	return rand.Text()
}
