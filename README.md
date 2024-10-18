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
> We are actively seeking contributors to help us improve this project.
> If you are interested in contributing, please reach out to us.

## Rationale:

> Close the gap between circom and golang.
>
> Rapidly iterate on your circuit and test your constraints.
>
> Secure, optimise and captilise with Circom.

We built this out of necessity for the development of Privacy Pool,
a **compliant** privacy protocol for the EVM.

Our intention is to promote further innovations for Circom DSL by providing
Developers with a seamless workflow for engineering secure circuits.

Veritas does not replace other Circom DevTools.

Instead we wish to provide a complementary tool that can be used in conjunction
with other tools to streamline the development & testing of Circom circuits.

## How to Use:

### Packing Programs into a Circuit Library:

Veritas considers circom files (.circom) as **Programs** which may include more than 1
templates / functions, etc.

These programs are packaged (CircuitPkg) together to be compiled as a **_Circuit Library._**

A program needs nothing more than it's **Identity** and **Src** to be defined.

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

In the snippet below, we are creating a Circuit Library hat will compile a
Circuit Package containing 2 programs:

-   Program 1: The main definition of the circuit.
-   Program 2: The template that will assigned to main.

> [!Tip]
> You do not need to specify the version pragma or the includes in the program src.
> Circom will merge templates together as long as they're all within the same package.

```Go
const test_template = `
    template SUM() {
        signal input a, ex;
        signal output out;

        0 === a * ex - 3;
        out <== 1;
    }
`
reports, err := NewEmptyLibrary().Compile(CircuitPkg{
        TargetVersion: "2.0.0",
        Field:         "bn128",
        Programs: []Program{
           	{
          		Identity: "main",
          		Src:      `component main {public[a, ex]}= Test();`,
           	},
           	{
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

You can evaluate your circuits by simply calling the `Evaluate` method on the circuit object
with a json strig that contains the input values for the circuit.

The evaluation will output an interface to a data structure which will contain the witness assignments,
linear constraints, and constrained & unconstrained symbols.

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
```

With it you can verify the correctness of your circuit and that constraints are satisfied.

```Go
// 100 iterations
for i := 0; i < 100 i++ {
    // evaluate the circuit per iteration
    // with a different input value
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

We will be including more examples soon to demonstrate the variety of use cases for Veritas!

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
