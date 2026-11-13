// SPDX-License-Identifier: MIT-0

package example

import (
	"testing"
)

func TestCryptoRandReaderBad(t *testing.T) {
	got := crypto_rand_reader_bad()
	if want := "AAAAAAAAAAAAAAAAAAAAAAAAAA"; got != want {
		t.Errorf("Exploit failed (got %q, want %q)", got, want)
	}
}

func TestCryptoRandReaderGood(t *testing.T) {
	got := crypto_rand_reader_good()
	if want := "AAAAAAAAAAAAAAAAAAAAAAAAAA"; got == want {
		t.Errorf("Unexpected result (wanted %q != %q)", got, want)
	}
}
