package veritas

/*
#include <stddef.h>
#include <stdlib.h>
#include <stdint.h>

typedef void* FFIProgramArchive;
typedef void* FFICircomCircuit;
typedef void* FFIWitnessCalculator;

// program archive
extern void ffi_build_program_archive(uintptr_t ctx_handle, char* ctx_json);
extern void ffi_type_analysis(uintptr_t ctx_handle, FFIProgramArchive ffi_prog_arch);

// circom circuit
extern void ffi_compile_circom_circuit(uintptr_t ctx_handle, FFIProgramArchive ffi_prog_arch);

// witness generator
extern void ffi_generate_witness_calculator(uintptr_t ctx_handle, FFICircomCircuit ffi_circuit);
extern void ffi_calculate_witness(uintptr_t ctx_handle, FFICircomCircuit ffi_circuit, FFIWitnessCalculator ffi_wc, char* inputs_json);

// utils
extern void free_string(char* str);
extern void free_prog_arch(FFIProgramArchive ffi_prog_arch);
extern void free_circom_circuit(FFICircomCircuit ffi_circuit);
extern void free_witness_calculator(FFIWitnessCalculator ffi_wc);

#cgo circom_ffi_debug  -Wl LDFLAGS: -L./circom/target/debug   -lcircom_ffi
#cgo !circom_ffi_debug -Wl LDFLAGS: -L./bin/release -lcircom_ffi
*/
import "C"

import (
	"encoding/json"
	"fmt"
	"runtime/cgo"
	"sync"
	"unsafe"

	"github.com/pkg/errors"
)

type RunTimeCTX struct {
	Version string      `json:"version"`
	Prime   string      `json:"prime"`
	Src     [][2]string `json:"src"`
}

type CircomFFI struct {
	witness           chan json.RawMessage
	WitnessCalculator C.FFIWitnessCalculator
	CircomCircuit     C.FFICircomCircuit
	ProgramArchive    C.FFIProgramArchive
	Diagnostics       []CompilerDiagnostic `json:"diagnostics"`
	Err               error                `json:"err"`
}

func (c *CircomFFI) FreeWitnessCalculator() { C.free_witness_calculator(c.WitnessCalculator) }
func (c *CircomFFI) FreeCircomCircuit()     { C.free_circom_circuit(c.CircomCircuit) }
func (c *CircomFFI) FreeProgramArchive()    { C.free_prog_arch(c.ProgramArchive) }
func (c *CircomFFI) Free() {
	c.FreeWitnessCalculator()
	c.FreeCircomCircuit()
	c.FreeProgramArchive()
}

func unwrapCtx(ctx_handle C.uintptr_t) *CircomFFI {
	ctx, ok := cgo.Handle(ctx_handle).Value().(*CircomFFI)
	if !ok {
		panic("cannot cast ctx")
	}

	return ctx
}

//export VeritasIncludeProgArch
func VeritasIncludeProgArch(ctx_handle C.uintptr_t, arch C.FFIProgramArchive) {
	ctx := unwrapCtx(ctx_handle)
	ctx.ProgramArchive = arch
}

//export VeritasIncludeCircomCircuit
func VeritasIncludeCircomCircuit(ctx_handle C.uintptr_t, circuit C.FFICircomCircuit) {
	ctx := unwrapCtx(ctx_handle)
	ctx.CircomCircuit = circuit
}

//export VeritasIncludeWC
func VeritasIncludeWC(ctx_handle C.uintptr_t, wc C.FFIWitnessCalculator) {
	ctx := unwrapCtx(ctx_handle)
	ctx.WitnessCalculator = wc
}

//export VeritasIncludeWitness
func VeritasIncludeWitness(ctx_handle C.uintptr_t, witness_json *C.char) {
	unwrapCtx(ctx_handle)
	witness_json_str := C.GoString(witness_json)
	fmt.Printf("Witness: %s\n", witness_json_str)
}

type CompilerDiagnostic struct {
	Severity string `json:"severity"`
	Code     string `json:"code"`
	Message  string `json:"message"`
	Labels   []struct {
		Style  string `json:"style"`
		FileId int    `json:"file_id"`
		Range  struct {
			Start int `json:"start"`
			End   int `json:"end"`
		}
		Message string `json:"message"`
	} `json:"labels"`
	Notes []string `json:"notes"`
}

func (diag *CompilerDiagnostic) Print() (s string) {
	s += fmt.Sprintf("**\tSeverity: %s\n", diag.Severity)
	s += fmt.Sprintf("**\tCode: %s\n", diag.Code)
	s += fmt.Sprintf("**\tMessage: %s\n", diag.Message)
	for _, label := range diag.Labels {
		s += fmt.Sprintf("**\t\tStyle: %s\n", label.Style)
		s += fmt.Sprintf("**\t\tFileId: %d\n", label.FileId)
		s += fmt.Sprintf("**\t\tRange: %d-%d\n", label.Range.Start, label.Range.End)
		s += fmt.Sprintf("**\t\tMessage: %s\n", label.Message)
	}
	for _, note := range diag.Notes {
		s += fmt.Sprintf("**\t\tNote: %s\n", note)
	}
	return
}

//export VeritasIncludeDiagnostic
func VeritasIncludeDiagnostic(ctx_handle C.uintptr_t, jsonBytes *C.void, bytesLen C.size_t) {
	ctx := unwrapCtx(ctx_handle)

	jsonRaw := json.RawMessage(C.GoBytes(unsafe.Pointer(jsonBytes), C.int(bytesLen)))
	var diag CompilerDiagnostic
	err := json.Unmarshal(jsonRaw, &diag)
	if err == nil {
		ctx.Diagnostics = append(ctx.Diagnostics, diag)
	} else {
		ctx.Err = errors.Wrap(ctx.Err, err.Error())
	}
}

//export VeritasIncludeError
func VeritasIncludeError(ctx_handle C.uintptr_t, msg *C.char) {
	ctx := unwrapCtx(ctx_handle)
	ctx.Err = errors.Wrap(ctx.Err, C.GoString(msg))
}

type CircomCompiler interface {
	Compile(ctx *RunTimeCTX) (CircomCompiler, error)
	Exec(inputs []byte) error
	Release() bool
}

type _CircomCompiler struct {
	ctx *CircomFFI
	mtx *sync.Mutex
}

func NewCircomCompiler() CircomCompiler {
	return &_CircomCompiler{
		mtx: new(sync.Mutex),
	}
}

func CheckDiagnostics(ctx *CircomFFI) bool {
	if ctx.Err != nil {
		fmt.Printf("\n_____Error_____\n\n")
		fmt.Println(ctx.Err)
		return false
	}
	if len(ctx.Diagnostics) > 0 {
		fmt.Printf("\n_____Diagnostics_____\n\n")
		for _, diag := range ctx.Diagnostics {
			fmt.Println(diag.Print())
		}
		return false
	}
	return true
}

func (c *_CircomCompiler) Exec(inputs []byte) error {
	if c.ctx == nil {
		return errors.New("FFI Bindings not initialized")
	}

	if c.ctx.WitnessCalculator == nil || c.ctx.CircomCircuit == nil {
		return errors.New("WitnessCalculator or CircomCircuit not initialized")
	}

	defer c.mtx.Unlock()
	c.mtx.Lock()

	ctx_handle := cgo.NewHandle(c.ctx)
	defer ctx_handle.Delete()

	inputsJSONCStr := cstring(inputs)

	C.ffi_calculate_witness(C.uintptr_t(ctx_handle), c.ctx.CircomCircuit, c.ctx.WitnessCalculator, inputsJSONCStr)
	C.free_string(inputsJSONCStr)

	if !CheckDiagnostics(c.ctx) {
		c.ctx.Free()
		return errors.Wrap(c.ctx.Err, "CircomCompiler generate witness calculator needs attention")
	}
	return nil
}

func (c *_CircomCompiler) Release() bool {
	if c.ctx != nil {
		c.ctx.Free()
		c.ctx = nil
		return true
	}
	return false
}

// Compile compiles the CircomCompiler code
// By executing the ffi function ffi_compile_CircomCompiler
// Prints out diagnostics if any
// Returns an error if any
func (c *_CircomCompiler) Compile(rtCtx *RunTimeCTX) (CircomCompiler, error) {
	if c.ctx != nil {
		return nil, errors.New("FFI Bindings exists, make sure to free them before compiling again")
	}

	defer c.mtx.Unlock()
	var (
		ctx = &CircomFFI{
			Diagnostics: make([]CompilerDiagnostic, 0),
		}
		ctx_handle = cgo.NewHandle(ctx)
	)
	defer ctx_handle.Delete()

	rtCtxJSON, err := json.Marshal(rtCtx)
	if err != nil {
		return nil, errors.Wrap(err, "unable to marshal runtime ctx")
	}
	rtCtxJSONCstr := cstring(rtCtxJSON)
	c.mtx.Lock()

	// (1) Build Program Archive
	C.ffi_build_program_archive(C.uintptr_t(ctx_handle), rtCtxJSONCstr)
	C.free_string(rtCtxJSONCstr)

	if !CheckDiagnostics(ctx) {
		ctx.Free()
		return nil, errors.Wrap(ctx.Err, "CircomCompiler build program archive needs attention")
	}

	// (2) Compile Circuit
	// This includes type checking
	C.ffi_compile_circom_circuit(C.uintptr_t(ctx_handle), ctx.ProgramArchive)
	if !CheckDiagnostics(ctx) {
		ctx.Free()
		return nil, errors.Wrap(ctx.Err, "CircomCompiler compile circuit needs attention")
	}

	// (3) Generate Witness Calculator
	C.ffi_generate_witness_calculator(C.uintptr_t(ctx_handle), ctx.CircomCircuit)
	if !CheckDiagnostics(ctx) {
		ctx.Free()
		return nil, errors.Wrap(ctx.Err, "CircomCompiler generate witness calculator needs attention")
	}
	c.ctx = ctx
	return c, nil
}

/*
func (c *_CircomCompiler) CalcWitness(rtCtx *RunTimeCTX, circIn []byte) error {
	defer c.mtx.Unlock()
	var (
		ctx = &CircomFFI{
			Diagnostics: make([]CompilerDiagnostic, 0),
		}
		ctx_handle = cgo.NewHandle(ctx)
	)
	defer ctx_handle.Delete()

	rtCtxJSON, err := json.Marshal(rtCtx)
	if err != nil {
		return errors.Wrap(err, "unable to marshal runtime ctx")
	}
	rtCtxJSONCstr := cstring(rtCtxJSON)
	circInCstr := cstring(circIn)

	c.mtx.Lock()
	C.ffi_calculate_witness(C.uintptr_t(ctx_handle), rtCtxJSONCstr, circInCstr)
	C.free(unsafe.Pointer(rtCtxJSONCstr))
	C.free(unsafe.Pointer(circInCstr))

	if len(ctx.Diagnostics) > 0 {
		fmt.Printf("\n_____Diagnostics_____\n\n")
		for _, diag := range ctx.Diagnostics {
			fmt.Println(diag.Print())
		}
	}

	fmt.Printf("Witnesss: %+v", string(ctx.Witness))

	return errors.Wrap(ctx.Err, "CircomCompiler compile error")

	}
*/

// cstring creates a null-terminated C string from the given byte slice.
// the caller is responsible for freeing the underlying memory
func cstring(data []byte) *C.char {
	str := unsafe.String(unsafe.SliceData(data), len(data))
	return C.CString(str)
}
