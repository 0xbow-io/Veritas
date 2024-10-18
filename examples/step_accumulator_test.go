package example

import (
	_ "embed"
	"fmt"
	"testing"

	. "github.com/0xBow-io/veritas"
	"github.com/test-go/testify/require"
)

// This example will illustrate at very basic level
// how one can emulate a folding scheme like nova
// where we simply feed the witness at round n into the circuit at round n+1

var (
	// A template
	// that injects the expected value of x into the circuit
	// and checks that the constraint is satisfied
	progA = Program{
		Identity: "Test",
		Src: `
		    template Test(C){
    			signal input stepIn[2];
                signal input ex;
                signal output stepOut[2];

                var y = stepIn[0] * 2;
                signal c <== y * C;

                signal x <== ex * stepIn[1];
                c - x ===  0;

                stepOut[0] <== x;
                stepOut[1] <== c;
            }
	`}
)

func Test_StepAccumulator(t *testing.T) {
	var (
		lib      = NewEmptyLibrary()
		programs = []Program{
			{
				Identity: "main",
				Src:      fmt.Sprintf("component main {public [stepIn]} = Test(3);"),
			},
			progA,
		}
	)
	defer lib.Burn()
	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs:      programs,
	})
	require.Nil(t, err)
	if len(reports) > 0 {
		reports.Print(programs)
		t.FailNow()
	}

	inputs := `{"stepIn": [1, 1], "ex": 6}`
	for i := 0; i < 10; i++ {
		fmt.Printf("\nAt Round %d inputs: %s\n", i, inputs)
		evaluation, err := lib.Evaluate([]byte(inputs))
		// Execute the circuit
		require.Nil(t, err)
		require.NotNil(t, evaluation)

		// Check for any reportss
		reports, err = lib.GetReports()
		require.Nil(t, err)
		require.Len(t, reports, 0)

		// Check that constraints are satisfied
		constraints_str := fmt.Sprintf("round %d, unsatisfied constraint: %s", i, evaluation.String())
		require.True(t, len(evaluation.SatisfiedConstraints()) > 0, constraints_str)
		require.Len(t, evaluation.UnSatisfiedConstraints(), 0)

		//ensure that all symbols are assigned to the correct witness value
		evaluation.AssignWitToSym()

		// get the output of sym b
		// and use it as the input for the next round into input ex
		a := evaluation.GetSymbolAssignment(&Symbol{
			Symbol:   "main.stepOut[0]",
			Original: "1",
			Witness:  "1",
			NodeID:   "0",
		})
		require.NotNil(t, a)

		b := evaluation.GetSymbolAssignment(&Symbol{
			Symbol:   "main.stepOut[1]",
			Original: "2",
			Witness:  "2",
			NodeID:   "0",
		})
		require.NotNil(t, b)

		// forward the output of the circuit to the next round
		inputs = fmt.Sprintf(`{"stepIn": [%s, %s], "ex": 6}`, a.String(), b.String())
	}
}
