```go
/*  _____                                                       _____  */
/* ( ___ )-----------------------------------------------------( ___ ) */
/*  |   |                                                       |   |  */
/*  |   | ██╗   ██╗███████╗██████╗ ██╗████████╗ █████╗ ███████╗ |   |  */
/*  |   | ██║   ██║██╔════╝██╔══██╗██║╚══██╔══╝██╔══██╗██╔════╝ |   |  */
/*  |   | ██║   ██║█████╗  ██████╔╝██║   ██║   ███████║███████╗ |   |  */
/*  |   | ╚██╗ ██╔╝██╔══╝  ██╔══██╗██║   ██║   ██╔══██║╚════██║ |   |  */
/*  |   |  ╚████╔╝ ███████╗██║  ██║██║   ██║   ██║  ██║███████║ |   |  */
/*  |   |   ╚═══╝  ╚══════╝╚═╝  ╚═╝╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝ |   |  */
/*  |___|                                                       |___|  */
/* (_____)-----------------------------------------------------(_____) */

" The physical manifestation of Veritas took the form of a silvery mist,
contained in pressurized canisters reminiscent of Ubik spray. When released,
Veritas Mist enveloped its subjects in a shimmering cloud, allowing them to
prove the veracity of their statements without revealing
any underlying information. " - PKD.

"In a universe of Veritas, the only certainty is doubt"
	- Joe Chip, Ubik
```

> [!Important]
> This project is in active development.
>
> We are actively seeking contributors to help us improve this project.
>
> If you are interested in contributing, please reach out to us.

---

# Rationale:

Veritas was built out of necessity for the development of Privacy Pool
a **compliant** privacy protocol for the EVM. In order to rapidly
prototype and test our circuits, we needed a more robust toolchain.

Current dev tools for Circom are not as robust as we would like them to be,
and we believe that there is a lot of room for improvement.

We also believe Golang perfectly complements the Circom DSL but is
underutilised in the common Circom Dev-Toolchain.

Therefore, we've built Veritas, which is an opinionated Go front-end
for Circom that enables Circom Circuits to be embedded within Go projects.

To achieve this, we've decided to fork Circom at v2.2.0 to support the
[Bus implementation](https://docta.ucm.es/rest/api/core/bitstreams/bab72e69-b6c9-42cc-8ac3-63407eb2a6b6/content).
and implemented [FFI bindings](https://github.com/0xbow-io/Veritas/tree/main/circom_ffi/circom/src)
for a continuous pipeline for circuit compilation (from AST to WASM) & evaluation.

---

# How to Use:

> [!Note]
>
> **Veritas does not replace other Circom DevTools.**
>
> It is a complementary tool that can be used in conjunction
> with other tools (i.e., Circomkit, Circomspect) to
> streamline the development and testing of Circom circuits.

## Circuit Package & Library:

Veritas considers circom artifacts (i.e. .circom file) as **Programs** w
hich may include more than 1 templates / functions, etc.

A program is defined by it's **Identity** and **Src** (Circom Code Block).
These programs are linked when packaged together to be compiled as a _Circuit Library_.

```Go
type Program struct {
	Identity string `json:"identity"`
	Src      string `json:"src"`
}

type CircuitPkg struct {
	TargetVersion string    `json:"target_version"`
	Field         string    `json:"field"`
	Programs      []Program `json:"programs"`
}
```

You may either write your circom templates within the Go test or import it from a file
via go:embed directive like so:

```Go
//go:embed extern/poseidon/ark.circom
var poseidon_ark string
```

In the snippet below, we are creating a Circuit Library that will compile a
Circuit Package, which contains 2 programs:

-   Program 1: The main definition of the circuit.
-   Program 2: The template that is linked to main.

> [!Tip]
> You do not need to specify the version pragma or the includes in the program src.
> Circom will merge templates together as long as they're all within the same package.

```Go

// Circom Template BLock
const test_template = `
    template SUM() {
        signal input a, ex;
        signal output out;

        0 === a * ex - 3;
        out <== 1;
    }
`
// NewEmptyLibrary() creates a new empty Circuit Library
// Compile() will compile the Circuit Package
// and return a collection of warning / error reports
// if any.
reports, err := NewEmptyLibrary().Compile(CircuitPkg{
        TargetVersion: "2.0.0",
        Field:         "bn128",
        Programs: []Program{
           	{
          		Identity: "main",
                // Main Circuit Definition
          		Src:      `component main {public[a, ex]}= Test();`,
           	},
           	{
                // Contains Code referenced in main
               	Identity: "Test",
                Src: test_template
            },
       	}
    },
})
```

The output of the compilation will be a collection of warning / error reports (if any), i.e:

```Rust
Pkg has been unpacked into Circuit Library .. 2 Programs Available
**	Severity: Error
**	Code: T2021
**	Message: Undeclared symbol
**		Style: Primary
**		FileId: 1
**		Range: 104-107
**		Message: Using unknown symbol
Pkg has been unpacked into Circuit Library .. 2 Programs Available
**	Severity: Warning
**	Code: CA01
**	Message: In template "A()": Local signal in2 does not appear in any constraint
**	Severity: Warning
**	Code: CA01
**	Message: In template "A()": Local signal in1 does not appear in any constraint
```

### Evaluating the Circuit:

You can evaluate your circuits by simply calling the `Evaluate` method with a JSON input.

```Go
// 100 iterations
for i := 0; i < 100 i++ {
    // evaluate the circuit per iteration
    // with different input values
    evaluation, err := lib.Evaluate([]byte(fmt.Sprintf(`{"a":%d, "ex":%s}`, i, i*3)))
    require.Nil(t, err)
    require.NotNil(t, evaluation)

    // Check that constraints are satisfied
    require.True(t, len(evaluation.SatisfiedConstraints()) > 0)
    require.Len(t, evaluation.UnSatisfiedConstraints(), 0)

    // check that the assignments to the symbols are correct & as expected
   	for _, sym := range expectedSyms {
		val := evaluation.GetSymbolAssignment(&sym)
		require.NotNil(t, val)
		require.Equal(t, sym.Assignment, val)
	}
}
```

The evaluation will output an interface to a data structure that will contain the witness assignments,
linear constraints, and constrained & unconstrained symbols.

With it, you can verify the correctness of your circuit logic, that the right symbols were constrained
and confirm that all constraints were satisfied.

```Go
type evaluation struct {
	Field       string   `json:"field"`
	Assignments []string `json:"assignments"`
	Constraints lcs      `json:"constraints"`
	Symbols     struct {
		Constrained   []Symbol `json:"constrained"`
		Unconstrained []Symbol `json:"unconstrained"`
	} `json:"symbols"`
}

// linear constraints
type lc struct {
	// witness to coefficient mapping
	A               [][2]string `json:"a_constraints"`
	B               [][2]string `json:"b_constraints"`
	C               [][2]string `json:"c_constraints"`
	// a * b - c
	Arithmetization [4]string   `json:"arithmetization"`
	IsSatisfied     string      `json:"satisfied"`
}
```

We will be including more [examples](https://github.com/0xbow-io/Veritas/tree/main/examples).
soon to demonstrate the variety of use cases for Veritas!

---

## Build Locally:

> Checkout the Makefile for more details.

To compile the circom ffi bindings, run:

```bash
make ffi
```

**_To clean artifacts, run:_**

```bash
make clean
```

## TODO:

-   [ ] Refactor & Clean up the code.
-   [ ] Add more tests / examples.
-   [ ] Add Further Constraint Analysis & Optimisations.
-   [ ] Support Formal Verification Tools.
-   [ ] Support for Folding Schemes Applications (i.e Sonobe integration).

## How to Contribute:

Submit a PR with your changes.
We will prioritise those that align with the TODO list or is a critical bug fix.
See CONTRIBUTING.md for more details.

All contributions are welcome and contributers will be credited & recognised.

## Acknolwedgements:

Wouldn't be possible without the achievements of the following projects:

-   [Iden3 Circom](https://github.com/iden3/circom)
-   [Costa Group UCM Civer](https://github.com/costa-group/circom_civer)
-   [Circomspect](https://github.com/trailofbits/circomspect)
-   [Arkwork](https://github.com/arkworks-rs)
