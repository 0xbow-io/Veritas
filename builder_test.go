package veritas

import (
	"testing"

	"github.com/test-go/testify/require"
)

var test_template_B = Component{
	Type:   Template,
	Name:   "B",
	Params: []string{"ParamA", "ParamB"},
	Body: `
	    signal input in1;
	    signal input in2;
	    signal output out;

	    var x = in1 * ParamA;
	    var b = in2 * ParamB;

	    out <== x + b;
`,
	Requires: nil,
}

var test_template_A = Component{
	Type: Template,
	Name: "A",
	Params: []string{
		"ParamA",
		"ParamB",
	},
	Body: `
		signal input in1;
		signal input in2;
		signal output out;

		component b = B(ParamA, ParamB);
		b.in1 <== in1;
		b.in2 <== in2;

		out <== b.out;
`,
	Requires: []*Component{
		&test_template_B,
	},
}

func Test_Compose_Template(t *testing.T) {
	temp := test_template_A.Compose()
	require.NotEqual(t, "", temp)
}
