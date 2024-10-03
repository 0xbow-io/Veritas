package examples

import (
	"fmt"
	"testing"

	. "github.com/0xBow-io/veritas"
	"github.com/test-go/testify/require"

	_ "embed"
)

/*
   Objective:
   Verify the Poseidon Hash Template from iden3 circomlib.

   Here the templates are implemented in .circom files and can be found
    in the extern/poseidon directory.
*/

//go:embed extern/poseidon/ark.circom
var poseidon_ark string

//go:embed extern/poseidon/mix.circom
var poseidon_mix string

//go:embed extern/poseidon/sigma.circom
var poseidon_sigma string

//go:embed extern/poseidon/sigma.circom
var poseidon_constants string

//go:embed extern/poseidon/poseidon.circom
var poseidon_poseidon string

func Test_POSEIDON_HASH(t *testing.T) {
	// Compile the circuit
	circuit, err := NewCircomCompiler().Compile(&RunTimeCTX{
		Version: "2.0.0",
		Prime:   "bn128",
		Src: [][2]string{
			{
				"main",
				`
				pragma circom 2.0.0;

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
			{"poseidon_poseidon", poseidon_poseidon},
			{"poseidon_ark", poseidon_ark},
			{"poseidon_mix", poseidon_mix},
			{"poseidon_sigma", poseidon_sigma},
			{"poseidon_constants", poseidon_constants},
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
