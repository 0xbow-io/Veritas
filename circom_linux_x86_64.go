//go:build linux && amd64

package veritas

/*
#cgo LDFLAGS: -L./include/linux_x86_64 -lcircom_linux_x86_64
*/
import "C"
