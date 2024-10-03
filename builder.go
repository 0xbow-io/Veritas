package veritas

import (
	"fmt"
	"strings"
)

type compType uint

const (
	Template compType = iota
	Function
)

func (c compType) Identifier() string {
	return [...]string{"template", "function"}[c]
}

func AggregateComponents(comps ...*Component) string {
	agg := make([]string, len(comps))
	for i, comp := range comps {
		agg[i] = comp.Compose()
	}
	return strings.Join(agg[:], "\n")
}

type Component struct {
	Type     compType
	Name     string   `json:"name"`
	Params   []string `json:"params"`
	Body     string   `json:"body"`
	Requires []*Component
}

func (c *Component) includes() []string {
	in := make([]string, len(c.Requires))
	for i, req := range c.Requires {
		in[i] = fmt.Sprintf(`include "%s.circom";`, req.Name)
	}
	return in
}

func (c *Component) Compose() string {
	return fmt.Sprintf(
		"%s\n%s %s(%s){\n%s\n}\n",
		strings.Join(c.includes()[:], "\n"),
		c.Type.Identifier(),
		c.Name,
		strings.Join(c.Params[:], ","),
		c.Body,
	)
}
func (c *Component) AsMain(
	pubIn []string,
	params []string,
) string {
	if c.Type != Template {
		return ""
	}
	return fmt.Sprintf(
		"\ncomponent main {public [%s]} = %s(%s)\n",
		strings.Join(pubIn[:], ","),
		c.Name,
		strings.Join(params[:], ","),
	)
}

// TO-DO Decompose a stringified component into a Component struct
// To make it easier for one to work with existing .cirom files
func (c *Component) Decompose(obj string) *Component { return nil }
