import Breeder from "./Breeder.rle"
import ThreeEngineCordership from "./3enginecordership.rle"

const Patterns: { [key: string]: string } = {
  Glider: `x = 3, y = 3
bo$2bo$3o!
`,
  'R-Pentomino': `x = 3, y = 3
b2o$2o$bo!
`,
  Breeder,
  '3-engine Cordership': ThreeEngineCordership,
}

export default Patterns