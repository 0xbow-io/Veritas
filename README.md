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

## Rationale:

> Close the gap between circom and golang.
>
> Rapidly iterate on your circuit and test your constraints.
>
> Secure, optimise and captilise with Circom.

We built this out of necessity for the development of Privacy Pool,
a **compliant** privacy protocol for the EVM.

Our intention is to promote the use of Circom DSL
for the development & innovation of zk-SNARKs circuits and to provide
Developers with a seamless workflow for engineering secure circuits.


## How to Use:

See the examples found in the `examples` directory.

```Go
func Test_Circuit(t *testing.T) {
	// Compile the circuit
	circuit, err := NewCircomCompiler().Compile(&RunTimeCTX{
		Version: "2.0.0",
		Prime:   "bn128",
		Src: [][2]string{
			{
				"main",
				`
				pragma circom 2.0.0;

				template Internal() {
				    signal input in[2];
				    signal output out;
				    out <== in[0]*in[1];
				}

				template Test() {
				    signal input in[2];
				    signal output out;
				    component c = Internal ();
				    c.in[0] <== in[0];
				    c.in[1] <== in[1]+2*in[0]+1;
				    c.out ==> out;
				}

				component main {public[in]}= Test();
				`,
			},
		},
	})
	require.Nil(t, err)
	require.NotNil(t, circuit)

	// Execute the circuit
	for i := 0; i < 1000; i++ {
		// form inputs
		inputs := fmt.Sprintf(`{"in": ["%d", "%d"]}`, i, i+1)
		// execute the circuit
		err = circuit.Exec([]byte(inputs))
	}
	// clean up
	circuit.Release()
}
```
## TODO:

- [] Refactor & Clean up the code.
- [] Add more tests / examples.
- [] Build for different target Archs.
- [] Add Constraint Analysis & Optimisation.
- [] Support Formal Verification Tools.
- [] Support for Folding Schemes Applications (i.e Sonobe integration).

## How to Contribute:




## Acknolwedgements:

Wouldn't be possible without the achievements of the following projects:

- [Iden3 Circom]()
- [Costa Group UCM Civer]()
- [Circomspect]()
- [Arkwork]()
