import breeder from "./Breeder.lif"

const Patterns: { [key: string]: string } = {
  empty: `x = 1, y = 1
!
`,
  glider: `x = 3, y = 3
bo$2bo$3o!
`,
  r: `x = 3, y = 3
b2o$2o$bo!
`,
  breeder,
}

export default Patterns