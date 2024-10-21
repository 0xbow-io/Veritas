package veritas

import (
	"fmt"
	"math/big"
	"testing"

	"github.com/test-go/testify/require"
)

var (
	testProgA = Program{
		Identity: "A",
		Src: `
		template A(ParamA, ParamB){
			signal input in1;
			signal input in2;
			signal output out;

			component b = B(ParamA, ParamB);
			b.in1 <== in1;
			b.in2 <== in2;

			out <== b.out;
		}
       `,
	}
	testProgB = Program{
		Identity: "B",
		Src: `
       	template B(ParamA, ParamB){
       		signal input in1;
       		signal input in2;
       		signal output out;

       		var x = in1 * ParamA;
       		var b = in2 * ParamB;

       		out <== x + b;
       	}
       `,
	}
	testProgC = Program{
		Identity: "Undeclared_Symbol",
		Src: `
        template A(){
			signal input in1;
			signal input in2;
			signal output out;
			out <== in1 * in3;
		}`,
	}
	testProgD = Program{
		Identity: "UnderConstrained",
		Src: `
        template A(){
			signal input in1;
			signal input in2;
			signal output out;
			out <== 1;
		}`,
	}
)

func Test_Compile(t *testing.T) {
	var (
		public_inputs = "in1"
		params        = "5, 9"
		lib           = NewEmptyLibrary()
	)
	defer lib.Burn()

	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs: []Program{
			{
				Identity: "main",
				Src: fmt.Sprintf("component main {public[%s]}= A(%s);",
					public_inputs, params),
			},
			testProgA, testProgB},
	})
	require.Nil(t, err)
	require.Len(t, reports, 0)
}

func Test_Compile_Anon(t *testing.T) {
	var (
		lib = NewEmptyLibrary()
	)
	defer lib.Burn()
	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.2.0",
		Field:         "bn128",
		Programs: []Program{
			{
				Identity: "main",
				Src:      "component main {public[in1]}= A();",
			},
			{
				Identity: "A",
				Src: `
                template A(){
                    signal input in1;
                    signal output out;
                    out <== Anon()(in1);
          		}`,
			},
		},
	})
	require.Nil(t, err)
	require.True(t, len(reports) > 0)
	print(reports.String())
}

func Test_Compile_UndeclaredSymbol(t *testing.T) {
	var (
		public_inputs = "in1"
		params        = ""
		lib           = NewEmptyLibrary()
	)
	lib.Burn()

	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs: []Program{
			{
				Identity: "main",
				Src: fmt.Sprintf("component main {public[%s]}= A(%s);",
					public_inputs, params),
			},
			testProgC},
	})
	require.Nil(t, err)
	require.True(t, len(reports) > 0)
	print(reports.String())
}

func Test_Compile_UnderConstrained(t *testing.T) {
	var (
		public_inputs = "in1"
		params        = ""
		lib           = NewEmptyLibrary()
	)
	defer lib.Burn()

	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs: []Program{
			{
				Identity: "main",
				Src: fmt.Sprintf("component main {public[%s]}= A(%s);",
					public_inputs, params),
			},
			testProgD},
	})
	require.Nil(t, err)
	require.True(t, len(reports) > 0)
	print(reports.String())
}

func Test_Evaluation(t *testing.T) {
	var (
		lib = NewEmptyLibrary()
		// A template
		// that injects the expected value of x into the circuit
		// and checks that the constraint is satisfied
		progA = Program{
			Identity: "Test",
			Src: `
		     template Test(){
				signal input a;
				signal input ex;

                signal output b;

                var x = a*a;
                x += 3;

                0 === ex - x;

                b <== 1;
            }
			`}
		evaluator = func(a *big.Int) *big.Int {
			const i = 2
			var x = new(big.Int).Mul(a, a)
			x.Add(x, big.NewInt(3))
			return x
		}
		expectedSyms = []Symbol{
			{
				Symbol:     "main.b",
				Original:   "1",
				Witness:    "1",
				NodeID:     "0",
				Assignment: big.NewInt(1),
			},
			{
				Symbol:   "main.a",
				Original: "2",
				Witness:  "2",
				NodeID:   "0",
			},
			{
				Symbol:   "main.ex",
				Original: "3",
				Witness:  "3",
				NodeID:   "0",
			},
		}
	)
	defer lib.Burn()

	reports, err := lib.Compile(CircuitPkg{
		TargetVersion: "2.0.0",
		Field:         "bn128",
		Programs: []Program{
			{
				Identity: "main",
				Src:      `component main {public[a, ex]}= Test();`,
			},
			progA},
	})
	require.Nil(t, err)
	require.Len(t, reports, 0)

	for i := 0; i < 10; i++ {

		expectedSyms[2].Assignment = big.NewInt(0).Set(evaluator(big.NewInt(int64(i))))
		expectedSyms[1].Assignment = big.NewInt(int64(i))
		input := fmt.Sprintf(`{"a":%d, "ex":%s}`, i, expectedSyms[2].Assignment.String())

		evaluation, err := lib.Evaluate([]byte(input))
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

		for _, sym := range expectedSyms {
			val := evaluation.GetSymbolAssignment(&sym)
			require.NotNil(t, val)
			require.Equal(t, sym.Assignment, val)
		}
	}

}
