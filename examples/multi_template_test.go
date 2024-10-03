package examples

import (
	"fmt"
	"testing"

	. "github.com/0xBow-io/veritas"
	"github.com/test-go/testify/require"
)

const PRODUCT_TEMPLATE = `
    pragma circom 2.0.0;

	template PRODUCT() {
	   signal input x, multiplier;
	   signal output out;

	    out <== x * multiplier;
	}
`

// We compose a template which tests the Product template
// where the multiplier is 2 and the expected value is the product of x and 2
// or x + x
func Test_PRODUCT_Template(t *testing.T) {
	// Compile the circuit
	circuit, err := NewCircomCompiler().Compile(&RunTimeCTX{
		Version: "2.0.0",
		Prime:   "bn128",
		Src: [][2]string{
			{
				"main",
				`
				pragma circom 2.0.0;
				template Test_PRODUCT(zero) {
				    assert(zero == 0);
				    signal input x, multiplier;

				    component c = PRODUCT();
				    c.x <== x;
				    c.multiplier <== multiplier;

					component s = SUM();
					s.a <== x;
					s.b <== x;

				    zero === c.out - s.out;
				}
				component main {public[x]}= Test_PRODUCT(0);
				`,
			},
			{"sum", SUM_TEMPLATE},
			{"product", PRODUCT_TEMPLATE},
		},
	})
	require.Nil(t, err)
	require.NotNil(t, circuit)

	// Execute the circuit
	for i := 0; i < 100; i++ {
		// form inputs
		inputs := fmt.Sprintf(`{"x": %d, "multiplier":%d}`, i, 2)
		// execute the circuit
		err = circuit.Exec([]byte(inputs))
		// assert that no errors were encountered
		require.Nil(t, err)
		// assert that the constraints are satisfied
	}

	// Execute the circuit
	// over 1 negative test case
	inputs := fmt.Sprintf(`{"x": %d, "multiplier":%d}`, 10, 50)
	err = circuit.Exec([]byte(inputs))
	// check for errors
	require.NotNil(t, err)
	// assert that the constraints are not satisfied

	// clean up
	circuit.Release()
}
