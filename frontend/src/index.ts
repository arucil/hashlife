import { Universe } from "hashlife-wasm"

let universe: Universe | null = null

const universePattern = document.querySelector('#universe-pattern') as HTMLSelectElement

const Patterns: { [key: string]: string } = {
  empty: `x = 1, y = 1
!`,
  glider: `
`,
}

function loadPattern(patternRle: string) {
  if (universe !== null) {
    universe.free()
  }

  universe = Universe.read(patternRle)
}

universePattern.addEventListener('change', _event => {
  loadPattern(Patterns[universePattern.value])
})