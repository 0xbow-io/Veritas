package examples

import (
	"fmt"
	"testing"

	. "github.com/0xBow-io/veritas"
	"github.com/test-go/testify/require"
)

/*
   Objective:
   Verify that the SUM template correctly computes the sum
   of two positive integers.
*/

const SUM_TEMPLATE = `
    pragma circom 2.0.0;

	template SUM() {
	    signal input a, b;
	    signal output out;

	    out <== a + b;
	}
`

// We compose a template which tests the SUM template
// which aseerts that difference between the output of SUM()
// and the expected value is equal to zero.
func Test_SUM_Template(t *testing.T) {
	// Compile the circuit
	circuit, err := NewCircomCompiler().Compile(&RunTimeCTX{
		Version: "2.0.0",
		Prime:   "bn128",
		Src: [][2]string{
			{
				"main",
				`
				pragma circom 2.0.0;

				include "sum";

				template Test_SUM(zero) {
				    assert(zero == 0);
				    signal input in[2];
					signal input expected;

				    component c = SUM();
				    c.a <== in[0];
					c.b <== in[1];

				    zero === c.out - expected;
				}

				component main {public[in]}= Test_SUM(0);
				`,
			},
			{
				"sum",
				SUM_TEMPLATE,
			},
		},
	})
	require.Nil(t, err)
	require.NotNil(t, circuit)

	// Execute the circuit
	// over 1000 test cases
	for i := 0; i < 1000; i++ {
		// form inputs
		inputs := fmt.Sprintf(`{"in": ["%d", "%d"], "expected": "%d"}`, i, i, i+i)
		// execute the circuit
		err = circuit.Exec([]byte(inputs))
		// assert that no errors were encountered
		require.Nil(t, err)
		// assert that the constraints are satisfied
	}

	// Execute the circuit
	// over 1 negative test case
	inputs := fmt.Sprintf(`{"in": ["%d", "%d"], "expected": "%d"}`, 10, 50, 0)
	err = circuit.Exec([]byte(inputs))
	// check for errors
	require.NotNil(t, err)
	// assert that the constraints are not satisfied

	// clean up
	circuit.Release()
}
