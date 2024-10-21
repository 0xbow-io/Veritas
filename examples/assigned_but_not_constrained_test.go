package example

import (
	"strings"
	"testing"

	. "github.com/0xBow-io/veritas"
	"github.com/google/go-cmp/cmp"
	"github.com/test-go/testify/require"
)

// Taken from: Assigned but not Constrained
// https://github.com/0xPARC/zk-bug-tracker?tab=readme-ov-file#8-assigned-but-not-constrained
var (
	isZero_v1 = Program{
		Identity: "IsZero",
		Src: `
        template IsZero() {
            signal input in;
            signal output out;
            signal inv;
            inv <-- in!=0 ? 1/in : 0;
            out <== -in*inv +1;
            in*out === 0;
        }
		`,
	}
	isZero_v2 = Program{
		Identity: "IsZero",
		Src: `
		template IsZero() {
		    signal input in;
            signal input out;

            signal temp;
            temp <-- in != 0 ? 0 : 1;

            out === temp;
        }`,
	}
)

func Test_IsZero_V1(t *testing.T) {
	lib := NewEmptyLibrary()
	defer lib.Burn()

	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs: []Program{
			{
				Identity: "main",
				Src:      `component main {public [in]}= IsZero();`,
			},
			isZero_v1,
		},
	})
	require.Nil(t, err)
	require.Len(t, reports, 0)

	evaluation, err := lib.Evaluate([]byte(`{"in": 0}`))
	// Execute the circuit
	require.Nil(t, err)
	require.NotNil(t, evaluation)

	// Check for any reports
	reports, err = lib.GetReports()
	require.Nil(t, err)
	require.Len(t, reports, 0)

	// Check that constraints are satisfied
	require.True(t, len(evaluation.SatisfiedConstraints()) > 0)
	require.Len(t, evaluation.UnSatisfiedConstraints(), 0)

	//ensure that all symbols are assigned to the correct witness value
	evaluation.AssignWitToSym()
	// lets get a list of constrained symbols
	// which should be: main.out, main.in, main.inv
	constrainedSyms := evaluation.ConstrainedSyms()
	println("Constrained Symbols: ", strings.Join(constrainedSyms, ", "))
	require.True(t, cmp.Equal(constrainedSyms, []string{"main.out", "main.in", "main.inv"}))

	// meanwhile there shouldn't be any uyconstrained symbols
	unconstrainedSyms := evaluation.UnConstrainedSyms()
	println("Unconstrained Symbols: ", strings.Join(unconstrainedSyms, ", "))
	require.True(t, len(unconstrainedSyms) == 0)
}

func Test_IsZero_V2(t *testing.T) {
	lib := NewEmptyLibrary()
	p := []Program{
		{
			Identity: "main",
			Src:      `component main {public [in]}= IsZero();`,
		},
		isZero_v2,
	}
	defer lib.Burn()

	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs:      p,
	})
	require.Nil(t, err)

	// will receive this Report
	// " In template "IsZero()": Local signal in does not appear in any constraint"
	if reports != nil {
		println(reports.String())
	}
}
