package veritas

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"runtime/cgo"
	"strconv"
	"sync"
	"unsafe"
)

/*
#include <stddef.h>
#include <stdlib.h>
#include <stdint.h>

typedef void* FFICircom;

// ffi_compile_library will parse the pkg
// and compile the circuit components (i.e. witness generator)
// that are required for witness generation
extern void ffi_compile_library(uintptr_t ctx_handle, char* pkg_json_raw);

// ffi_circuit_execution will generate witness for the given inputs
extern void ffi_circuit_execution(uintptr_t ctx_handle, FFICircom ffi_circom, char* pkg_json_raw);

// utils
extern void free_string(char* str);
extern void free_circom(FFICircom ptr);

#cgo FFI_DEBUG -Wl LDFLAGS: -L./circom/target/debug   -lcircom
*/
import "C"

//export share_evaluations
func share_evaluations(ctx_handle C.uintptr_t, jsonBytes *C.void, bytesLen C.size_t) {
	unwrapCtx(ctx_handle).CacheEval(toJsonRaw(jsonBytes, bytesLen))
}

//export share_report
func share_report(ctx_handle C.uintptr_t, jsonBytes *C.void, bytesLen C.size_t) {
	unwrapCtx(ctx_handle).StoreReport(toJsonRaw(jsonBytes, bytesLen))
}

//export share_circom_ptr
func share_circom_ptr(ctx_handle C.uintptr_t, circom C.FFICircom) { unwrapCtx(ctx_handle).ptr = circom }

type _CtxFFI struct {
	ptr     C.FFICircom
	reports ReportCollection
	// cache for the last evaluation result
	last_eval *evaluation
}

func (f *_CtxFFI) free() {
	if f.ptr != nil {
		C.free_circom(f.ptr)
	}
}

func (f *_CtxFFI) CacheEval(e json.RawMessage) {
	f.last_eval = &evaluation{}
	if err := json.Unmarshal(e, f.last_eval); err != nil {
		return
	}
}
func (f *_CtxFFI) StoreReport(r json.RawMessage) {
	var report Report
	if err := json.Unmarshal(r, &report); err != nil {
		return
	}
	f.reports = append(f.reports, report)
}

func unwrapCtx(ctx_handle C.uintptr_t) *_CtxFFI {
	ctx, ok := cgo.Handle(ctx_handle).Value().(*_CtxFFI)
	if !ok {
		// this shouldn never happen
		panic("invalid context handle")
	}
	return ctx
}

type CircuitLibrary interface {
	Evaluate(inputs []byte) (Evaluation, error)
	Compile(pkg CircuitPkg) (ReportCollection, error)
	GetReports() (ReportCollection, error)

	Burn()
}
type _CircuitLibrary struct {
	ctx *_CtxFFI
	mtx *sync.Mutex
}

func NewEmptyLibrary() CircuitLibrary {
	return &_CircuitLibrary{mtx: &sync.Mutex{}}
}

func (lib *_CircuitLibrary) Compile(pkg CircuitPkg) (ReportCollection, error) {
	if lib.ctx != nil {
		return nil, errors.New("FFI Bindings exists, make sure to free them before compiling again")
	}
	defer lib.mtx.Unlock()
	var (
		ctx = &_CtxFFI{
			ptr:       nil,
			reports:   make(ReportCollection, 0),
			last_eval: nil,
		}
		ctx_handle = cgo.NewHandle(ctx)
	)
	defer ctx_handle.Delete()

	pkgJson, err := json.Marshal(pkg)
	if err != nil {
		return nil, err
	}
	pkgJSONStr := cstring(pkgJson)
	lib.mtx.Lock()

	// compile the circuit
	C.ffi_compile_library(C.uintptr_t(ctx_handle), pkgJSONStr)
	// Release the json string from memory
	C.free_string(pkgJSONStr)
	// store the context
	lib.ctx = ctx
	// return the reports
	return lib.GetReports()
}

func (lib *_CircuitLibrary) Evaluate(inputs []byte) (Evaluation, error) {
	if lib.ctx == nil || lib.ctx.ptr == nil {
		return nil, errors.New("FFI Bindings has not been initialized")
	}

	defer lib.mtx.Unlock()
	lib.mtx.Lock()

	ctx_handle := cgo.NewHandle(lib.ctx)
	defer ctx_handle.Delete()

	inputsJSONCStr := cstring(inputs)
	C.ffi_circuit_execution(C.uintptr_t(ctx_handle), lib.ctx.ptr, inputsJSONCStr)
	C.free_string(inputsJSONCStr)
	return lib.GetEvaluation()
}

func (lib *_CircuitLibrary) GetReports() (ReportCollection, error) {
	if lib.ctx == nil {
		return nil, errors.New("FFI Bindings does not exist")
	}
	return lib.ctx.reports, nil
}

func (lib *_CircuitLibrary) GetEvaluation() (Evaluation, error) {
	if lib.ctx == nil {
		return nil, errors.New("FFI Bindings does not exist")
	}
	if lib.ctx.last_eval == nil {
		return nil, errors.New("No evaluation has been performed")
	}
	return lib.ctx.last_eval, nil
}
func (lib *_CircuitLibrary) Burn() {
	if lib.ctx != nil {
		lib.ctx.free()
	}
	lib.ctx = nil
}

type ReportCollection []Report

func (c ReportCollection) String() (s string) {
	for _, r := range c {
		s += r.String()
	}
	return
}

func (c ReportCollection) Print(programs []Program) {
	for _, r := range c {
		r.Print(programs)
	}
	return
}

type Report struct {
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

func (*Report) Default() Report {
	return Report{
		Severity: "error",
		Code:     "default",
		Message:  "default",
		Labels:   nil,
		Notes:    nil,
	}
}

func (r *Report) String() (s string) {
	s += fmt.Sprintf("Severity: %s\n", r.Severity)
	s += fmt.Sprintf("Code: %s\n", r.Code)
	s += fmt.Sprintf("Message: \n\t[%s]\n", r.Message)
	for _, label := range r.Labels {
		s += fmt.Sprintf("**\t\tStyle: %s\n", label.Style)
		s += fmt.Sprintf("**\t\tFileId: %d\n", label.FileId)
		s += fmt.Sprintf("**\t\tRange: %d-%d\n", label.Range.Start, label.Range.End)
		s += fmt.Sprintf("**\t\tMessage: %s\n", label.Message)
	}
	for _, note := range r.Notes {
		s += fmt.Sprintf("**\t\tNote: %s\n", note)
	}
	return
}

func (r *Report) Print(programs []Program) {
	src := programs[r.Labels[0].FileId].Src[r.Labels[0].Range.Start:r.Labels[0].Range.End]
	fmt.Printf("%s source: %s", r.String(), src)
}

type Program struct {
	Identity string `json:"identity"`
	Src      string `json:"src"`
}

type CircuitPkg struct {
	TargetVersion string    `json:"target_version"`
	Field         string    `json:"field"`
	Programs      []Program `json:"programs"`
}

type Evaluation interface {
	ConstrainedSyms() []string
	UnConstrainedSyms() []string
	WitnessAssignment() []*big.Int
	GetSymbolAssignment(sym *Symbol) *big.Int
	SatisfiedConstraints() []uint
	UnSatisfiedConstraints() []uint
	AssignWitToSym()
	String() string
}

type evaluation struct {
	Field       string   `json:"field"`
	Assignments []string `json:"assignments"`
	Constraints lcs      `json:"constraints"`
	Symbols     struct {
		Constrained   []Symbol `json:"constrained"`
		Unconstrained []Symbol `json:"unconstrained"`
	} `json:"symbols"`
}

// Keeping fields as string for now
type lcs []lc
type lc struct {
	// witness to coefficient mapping
	A               [][2]string `json:"a_constraints"`
	B               [][2]string `json:"b_constraints"`
	C               [][2]string `json:"c_constraints"`
	Arithmetization [4]string   `json:"arithmetization"`
	IsSatisfied     string      `json:"satisfied"`
}

func (e *evaluation) ConstrainedSyms() []string {
	var res []string
	for i := 1; i < len(e.Symbols.Constrained); i++ {
		res = append(res, e.Symbols.Constrained[i].Symbol)
	}
	return res
}

func (e *evaluation) UnConstrainedSyms() []string {
	var res []string
	for _, sym := range e.Symbols.Unconstrained {
		res = append(res, sym.Symbol)
	}
	return res
}

func (e *evaluation) WitnessAssignment() []*big.Int {
	var assignments = make([]*big.Int, len(e.Assignments))
	for i, assignment := range e.Assignments {
		assignments[i], _ = new(big.Int).SetString(assignment, 10)
	}
	return assignments
}

func (e *evaluation) AssignWitToSym() {
	for i := 0; i < len(e.Symbols.Constrained); i++ {
		e.Symbols.Constrained[i].Assignment, _ = new(big.Int).SetString(e.Assignments[i], 10)
	}
}
func (e *evaluation) GetSymbolAssignment(sym *Symbol) *big.Int {
	// if sym.witness != "-1" then it is a constrained symbol
	// constrained symbols are arranged by witness index
	idx, err := strconv.Atoi(sym.Witness)
	if idx < 0 || err != nil || idx > len(e.Assignments) {
		return nil
	}
	if diff := sym.SameSym(&e.Symbols.Constrained[idx]); diff == "" {
		return e.Symbols.Constrained[idx].Assignment
	}
	return nil
}
func (e *evaluation) SatisfiedConstraints() []uint {
	var res []uint
	for i, lc := range e.Constraints {
		if lc.IsSatisfied == "true" {
			res = append(res, uint(i))
		}
	}
	return res
}
func (e *evaluation) UnSatisfiedConstraints() []uint {
	var res []uint
	for i, lc := range e.Constraints {
		if lc.IsSatisfied == "false" {
			res = append(res, uint(i))
		}
	}
	return res
}

func (e *evaluation) String() string {
	linear_a_string := ""
	linear_b_string := ""
	linear_c_string := ""
	out := ""
	for _, lc := range e.Constraints {
		for _, a := range lc.A {
			witness, _ := strconv.Atoi(a[0])
			assignment := e.Assignments[witness]

			linear_a_string += fmt.Sprintf("[%s](%s * %s) + ",
				e.Symbols.Constrained[witness].Symbol, assignment, a[1])
		}
		for _, b := range lc.B {
			witness, _ := strconv.Atoi(b[0])
			assignment := e.Assignments[witness]

			linear_b_string += fmt.Sprintf("[%s](%s * %s) + ",
				e.Symbols.Constrained[witness].Symbol, assignment, b[1])
		}
		for _, c := range lc.C {
			witness, _ := strconv.Atoi(c[0])
			assignment := e.Assignments[witness]

			linear_c_string += fmt.Sprintf("[%s](%s * %s) + ",
				e.Symbols.Constrained[witness].Symbol, assignment, c[1])
		}
		out += fmt.Sprintf("\nA: %s = %s\nB: %s = %s\nC: %s = %s\n",
			linear_a_string, lc.Arithmetization[0],
			linear_b_string, lc.Arithmetization[1],
			linear_c_string, lc.Arithmetization[2])
	}
	return out
}

type Symbol struct {
	Symbol     string `json:"symbol"`
	NodeID     string `json:"node_id"`
	Original   string `json:"original"`
	Witness    string `json:"witness"`
	Assignment *big.Int
}

func (s *Symbol) String() string {
	return fmt.Sprintf("Sym: %s (%s %s %s) --> %s",
		s.Symbol,
		s.NodeID,
		s.Original,
		s.Witness,
		s.Assignment.String())
}

func (s *Symbol) SameSym(other *Symbol) string {
	if other == nil {
		return ""
	}
	if s.Symbol != other.Symbol {
		return "!Symbol"
	}
	if s.NodeID != other.NodeID {
		return "!NodeID"
	}
	if s.Original != other.Original {
		return "!Original"
	}
	if s.Witness != other.Witness {
		return "!Witness"
	}
	return ""
}

func toJsonRaw(jsonBytes *C.void, bytesLen C.size_t) json.RawMessage {
	return json.RawMessage(C.GoBytes(unsafe.Pointer(jsonBytes), C.int(bytesLen)))
}

// cstring creates a null-terminated C string from the given byte slice.
// the caller is responsible for freeing the underlying memory
func cstring(data []byte) *C.char {
	str := unsafe.String(unsafe.SliceData(data), len(data))
	return C.CString(str)
}
