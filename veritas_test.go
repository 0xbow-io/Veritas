package veritas

/*
var template_b = CircomTemplate{
	Pragma: "2.0.0",
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
}

var template_a = CircomTemplate{
	Pragma: "2.0.0",
	Deps:   []*CircomTemplate{&template_b},
	Name:   "A",
	Params: []string{"ParamA", "ParamB"},
	Body: `
      signal input in1;
      signal input in2;
      signal output out;

      component b = B(ParamA, ParamB);
      b.in1 <== in1;
      b.in2 <== in2;

      out <== b.out;
  `,
}

func Test_Template_A(t *testing.T) {
	circom := NewCircom()
	// Compile the template
	// And return pointer to witness generator
	wg, err := circom.Compile(&CompilationConfig{
		Main:         &template_a,
		PublicInputs: []string{"in1"},
		ParamValues:  []string{"2", "3"},
	})
	require.NoError(t, err)
	require.NotNil(t, wg)

	// Generate witness
	witness, err := wg.GenerateWitness(
		map[string]interface{}{
			"in1": 2,
			"in2": 3,
		})
	require.NoError(t, err)
	require.NotNil(t, witness)

	// Check the output
	// The output should be 6
	require.Equal(t, 6, witness["out"])
}
*/
