package veritas

import (
	"github.com/iden3/go-rapidsnark/witness/v2"
)

type CircomWitness interface {
	GenerateWitness(inputs map[string]interface{}) (map[string]interface{}, error)
}

type _CircomWitness struct {
	calc witness.Calculator
}

func NewCircomWitness(wasm []byte) (w CircomWitness, err error) {
	/*
		w = &_CircomWitness{}
		// If the inputs are empty or bad formatted, it raises a panic. To avoid it,
		// catch the panic and return an error instead.
		defer func() {
			if p := recover(); p != nil {
				err, _ := p.(error)
				panicErr = fmt.Errorf("%w: %w", ErrParsingWitness, err)
			}
		}()
		var ops []witness.Option
		ops = append(ops, witness.WithWasmEngine(wazero.NewCircom2WZWitnessCalculator))

		w.calc, err = witness.NewCalculator(wasm, ops...)
		if err != nil {
			return nil, fmt.Errorf("%w: %w", ErrInitWitnessCalc, err)
		}

		return witCalc
	*/
	return nil, nil
}

func (w *_CircomWitness) GenerateWitness(inputs map[string]interface{}) (map[string]interface{}, error) {
	return nil, nil
}
