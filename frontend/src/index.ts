import { Universe } from "hashlife-wasm"
import Patterns from "./patterns"
import render from "./render"

const univCanvas = document.querySelector('#universe') as HTMLCanvasElement
const univPattern = document.querySelector('#universe-pattern') as HTMLSelectElement

univCanvas.width = 600
univCanvas.height = 480

let universe: Universe | null = null
let scale = 1

function loadPattern(patternRle: string) {

  try {
    const newUniverse = Universe.read(patternRle)
    if (universe !== null) {
      universe.free()
    }
    universe = newUniverse
  } catch (e) {
    console.error(e)
    alert('invalid RLE format!')
    return
  }

  render(universe, univCanvas.getContext('2d') as any, univCanvas.width, univCanvas.height)
}

function setScale(newScale: number) {
  scale = newScale
  univCanvas.width = univCanvas.scrollWidth / scale
  univCanvas.height = univCanvas.scrollHeight / scale
}

univPattern.addEventListener('change', _event => {
  loadPattern(Patterns[univPattern.value])
})

setScale(1)
loadPattern(Patterns[univPattern.value])

