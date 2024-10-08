package veritas

import (
	"fmt"
	"testing"

	"github.com/test-go/testify/require"
)

func Test_CircomCompiler_Compile(t *testing.T) {
	circom, err := NewCircomCompiler().Compile(&RunTimeCTX{
		Version: "2.0.0",
		Prime:   "bn128",
		Src: [][2]string{
			{
				"test.circom",
				`
				pragma circom 2.0.0;

				template B(ParamA, ParamB){
					signal input in1;
					signal input in2;
					signal output out;

					var x = in1 * ParamA;
					var b = in2 * ParamB;

					out <== x + b;
				}

				template A(ParamA, ParamB){
					signal input in1;
					signal input in2;
					signal output out;

					component b = B(ParamA, ParamB);
					b.in1 <== in1;
					b.in2 <== in2;

					out <== b.out;
				}

				component main {public [in1]}= A(9, 10);
			`,
			},
		},
	})
	require.Nil(t, err)
	require.NotNil(t, circom)
	circom.Release()
}

func Test_CircomCompiler_Compile_Undeclared_Symbol(t *testing.T) {
	circom, err := NewCircomCompiler().Compile(&RunTimeCTX{
		Version: "2.0.0",
		Prime:   "bn128",
		Src: [][2]string{
			{
				"test.circom",
				`
				pragma circom 2.0.0;

				template A(){
					signal input in1;
					signal input in2;
					signal output out;
					out <== in1 * in3;
				}

				component main {public [in1]}= A();
			`,
			},
		},
	})
	require.Nil(t, err)
	require.Nil(t, circom)
}

func Test_CircomCompiler_Exec(t *testing.T) {
	// Compile the circuit
	circom, err := NewCircomCompiler().Compile(&RunTimeCTX{
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
	require.NotNil(t, circom)

	// Execute the circuit
	for i := 0; i < 1000; i++ {
		inputs := fmt.Sprintf(`{"in": ["%d", "%d"]}`, i, i+1)
		err = circom.Exec([]byte(inputs))
	}

	circom.Release()
}
